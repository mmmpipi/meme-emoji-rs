//! 可选的 GPU 后端。
//!
//! 运行时 dlopen `libEGL`，建一个无窗口（surfaceless）上下文，把 Skia 的渲染交给
//! Ganesh GL。没显卡、没驱动、上下文建不起来——任何一步失败都静默退回 CPU 光栅，
//! 不会让出图失败。环境变量 `MEME_GPU=0` 可以强制关掉。

use std::{cell::RefCell, ffi::c_void, ptr, sync::OnceLock};

use khronos_egl as egl;
use meme_generator_utils::tools::new_surface;
use skia_safe::{
    gpu::{self, direct_contexts, gl::Interface, Budgeted, DirectContext, SurfaceOrigin},
    image::CachingHint,
    ISize, Image, ImageFilter, ImageInfo, Paint, Surface,
};

/// Mesa 的无窗口平台：服务器上没有 X11/Wayland 时靠它拿 display。
const PLATFORM_SURFACELESS_MESA: egl::Enum = 0x31DD;
/// NVIDIA 的 EGLDevice 平台。
const PLATFORM_DEVICE_EXT: egl::Enum = 0x313F;
/// GPU 资源缓存上限，按线程算。
const RESOURCE_CACHE_BYTES: usize = 128 * 1024 * 1024;

type Instance = egl::DynamicInstance<egl::EGL1_5>;

struct Ready {
    context: DirectContext,
    /// GL_RENDERER 字符串，用来分辨真显卡还是 llvmpipe 软件光栅。
    #[allow(dead_code, reason = "诊断用，只有开发期夹具会读")]
    renderer: String,
}

enum State {
    Uninit,
    Unavailable,
    Ready(Ready),
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State::Uninit) };
}

fn disabled_by_env() -> bool {
    matches!(
        std::env::var("MEME_GPU").as_deref(),
        Ok("0") | Ok("off") | Ok("false") | Ok("no")
    )
}

fn instance() -> Option<&'static Instance> {
    static INSTANCE: OnceLock<Option<&'static Instance>> = OnceLock::new();
    *INSTANCE.get_or_init(|| {
        let loaded = unsafe { Instance::load_required() }.ok()?;
        Some(&*Box::leak(Box::new(loaded)))
    })
}

fn display(egl: &'static Instance) -> Option<egl::Display> {
    for platform in [PLATFORM_SURFACELESS_MESA, PLATFORM_DEVICE_EXT] {
        if let Ok(display) =
            unsafe { egl.get_platform_display(platform, egl::DEFAULT_DISPLAY, &[egl::ATTRIB_NONE]) }
        {
            return Some(display);
        }
    }
    unsafe { egl.get_display(egl::DEFAULT_DISPLAY) }
}

const GL_RENDERER: u32 = 0x1F01;

fn create_context() -> Option<Ready> {
    if disabled_by_env() {
        return None;
    }
    let egl = instance()?;
    let display = display(egl)?;
    egl.initialize(display).ok()?;
    egl.bind_api(egl::OPENGL_API).ok()?;
    let config = egl
        .choose_first_config(
            display,
            &[
                egl::SURFACE_TYPE,
                egl::PBUFFER_BIT,
                egl::RENDERABLE_TYPE,
                egl::OPENGL_BIT,
                egl::RED_SIZE,
                8,
                egl::GREEN_SIZE,
                8,
                egl::BLUE_SIZE,
                8,
                egl::ALPHA_SIZE,
                8,
                egl::NONE,
            ],
        )
        .ok()??;
    let context = egl
        .create_context(
            display,
            config,
            None,
            &[egl::CONTEXT_MAJOR_VERSION, 3, egl::NONE],
        )
        .ok()?;
    // 不建 surface，直接 make_current（EGL_KHR_surfaceless_context）。
    egl.make_current(display, None, None, Some(context)).ok()?;

    let interface = Interface::new_load_with(|name| {
        egl.get_proc_address(name)
            .map(|address| address as *const c_void)
            .unwrap_or(ptr::null())
    })?;
    let renderer = gl_string(egl, GL_RENDERER);
    // 软件光栅（llvmpipe/swrast）比 Skia 自己的 CPU 光栅还慢——实测 7.2MP 图
    // llvmpipe 1.6s、Skia CPU 0.93s——这种情况下宁可不要 GPU。
    if is_software_renderer(&renderer) {
        return None;
    }

    let mut context = direct_contexts::make_gl(interface, None)?;
    context.set_resource_cache_limit(RESOURCE_CACHE_BYTES);
    Some(Ready { context, renderer })
}

fn is_software_renderer(renderer: &str) -> bool {
    if matches!(std::env::var("MEME_GPU").as_deref(), Ok("force")) {
        return false;
    }
    let renderer = renderer.to_ascii_lowercase();
    ["llvmpipe", "softpipe", "swrast", "software rasterizer", "lavapipe"]
        .iter()
        .any(|name| renderer.contains(name))
}

/// 直接问 GL 要 GL_RENDERER/GL_VERSION，用来判断到底落在哪个驱动上。
fn gl_string(egl: &'static Instance, name: u32) -> String {
    type GetString = unsafe extern "system" fn(u32) -> *const std::ffi::c_char;
    let Some(address) = egl.get_proc_address("glGetString") else {
        return String::new();
    };
    let get_string: GetString = unsafe { std::mem::transmute(address) };
    let value = unsafe { get_string(name) };
    if value.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

fn with_context<R>(f: impl FnOnce(Option<&mut DirectContext>) -> R) -> R {
    STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if matches!(*state, State::Uninit) {
            *state = match create_context() {
                Some(ready) => State::Ready(ready),
                None => State::Unavailable,
            };
        }
        match &mut *state {
            State::Ready(ready) => f(Some(&mut ready.context)),
            _ => f(None),
        }
    })
}

/// 当前线程用的后端描述，给诊断/日志用：GPU 时是 GL_RENDERER，否则是 "cpu"。
#[allow(dead_code, reason = "诊断用，只有开发期夹具会调")]
pub(crate) fn backend_name() -> String {
    STATE.with(|cell| {
        {
            let state = cell.borrow();
            if let State::Ready(ready) = &*state {
                return ready.renderer.clone();
            }
            if matches!(*state, State::Unavailable) {
                return "cpu".to_string();
            }
        }
        with_context(|_| ());
        let state = cell.borrow();
        match &*state {
            State::Ready(ready) => ready.renderer.clone(),
            _ => "cpu".to_string(),
        }
    })
}

/// 取一张绘制目标：有 GPU 就用 GPU render target，否则退回 CPU 光栅。
pub(crate) fn surface(size: ISize) -> Surface {
    with_context(|context| {
        let context = context?;
        let info = ImageInfo::new_n32_premul(size, None);
        gpu::surfaces::render_target(
            context,
            Budgeted::Yes,
            &info,
            None,
            SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        )
    })
    .unwrap_or_else(|| new_surface(size))
}

/// 在当前后端上套一个 image filter（utils 里的同名方法写死了 CPU 光栅）。
pub(crate) fn filtered(image: &Image, filter: ImageFilter) -> Image {
    let mut surface = surface(image.dimensions());
    let mut paint = Paint::default();
    paint.set_image_filter(filter);
    surface.canvas().draw_image(image, (0, 0), Some(&paint));
    surface.image_snapshot()
}

/// GPU 上的图要回读成光栅图，编码器和后续 CPU 步骤才拿得到像素。
pub(crate) fn to_raster(image: &Image) -> Image {
    with_context(|context| {
        let context = context?;
        context.flush_and_submit();
        image.make_raster_image(context, CachingHint::Disallow)
    })
    .unwrap_or_else(|| image.clone())
}
