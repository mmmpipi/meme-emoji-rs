use std::thread::LocalKey;

use skia_safe::{
    gradient_shader::{self, GradientShaderColors},
    high_contrast_config::InvertStyle,
    high_contrast_filter, image_filters,
    runtime_effect::ChildPtr,
    scalar, BlendMode, Color, Data, HighContrastConfig, IPoint, IRect, ISize, Image, ImageFilter,
    Paint, Point, Rect, RuntimeEffect, SamplingOptions, Shader, TileMode,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{local_date, new_paint},
};

use crate::{gpu_backend, options::Louvre, register_meme};

/// 开启「缩放」后处理前限制的长边像素。
const FAST_LONG_SIDE: i32 = 1600;
/// 线稿主通道两级高斯的尺度比，1.6 是 DoG 逼近 LoG 的经典取值。
const DOG_RATIO: f32 = 1.6;
/// 线宽随图像尺寸归一化的参考短边。
const REFERENCE_SHORT_SIDE: f32 = 1000.0;

/// 线稿主通道：两级高斯做差（DoG），只取「变暗的一侧」当墨色。
///
/// 逐通道取差值的最大值而不是只看亮度，蓝底上的气泡、同亮度不同色相的
/// 交界才不会整片丢失；再按局部亮度补偿，暗部（深色衣服、头发）的细节
/// 才不会被绝对阈值一刀切掉。输出是「rgb=0，alpha=墨量」的预乘图，
/// 后面铺渐变的 SrcIn 才有意义。
const LINE_SHADER_SOURCE: &str = r#"
    uniform shader u_fine;
    uniform shader u_coarse;
    uniform float u_low;
    uniform float u_high;
    uniform float u_dark;

    vec4 main(vec2 uv) {
        vec3 fine = u_fine.eval(uv).rgb;
        vec3 coarse = u_coarse.eval(uv).rgb;
        vec3 d = coarse - fine;
        float e = max(d.r, max(d.g, d.b));
        float luma = clamp(dot(coarse, vec3(0.299, 0.587, 0.114)), 0.0, 1.0);
        e = e * (1.0 + u_dark * (1.0 - luma));
        return vec4(0.0, 0.0, 0.0, smoothstep(u_low, u_high, e));
    }
"#;

/// 经典模式：卷积模糊图与原图求差，做成墨色 alpha。
const DIFF_SHADER_SOURCE: &str = r#"
    uniform shader u_texture1;
    uniform shader u_texture2;
    uniform float u_light_cut;
    uniform float u_dark_cut;

    vec4 main(vec2 uv) {
        float color1 = u_texture1.eval(uv).r;
        float color2 = u_texture2.eval(uv).r;
        float r = 0.5 + color2 - color1; // 128/255 ≈ 0.5
        float scale = 255.0 / max(255.0 - u_light_cut - u_dark_cut, 1.0);
        r = clamp((r * 255.0 - u_dark_cut) * scale, 0.0, 255.0) / 255.0;
        return vec4(0.0, 0.0, 0.0, 1.0 - r);
    }
"#;

/// 把亮度转成墨色 alpha，供「线稿 / 取消差异化」分支使用。
const INK_SHADER_SOURCE: &str = r#"
    uniform shader u_image;
    uniform float u_gain;

    vec4 main(vec2 uv) {
        vec4 color = u_image.eval(uv);
        float luma = clamp(dot(color.rgb, vec3(0.299, 0.587, 0.114)), 0.0, 1.0);
        return vec4(0.0, 0.0, 0.0, clamp((1.0 - luma) * u_gain, 0.0, 1.0));
    }
"#;

thread_local! {
    static LINE_EFFECT: Result<RuntimeEffect, String> =
        RuntimeEffect::make_for_shader(LINE_SHADER_SOURCE, None);
    static DIFF_EFFECT: Result<RuntimeEffect, String> =
        RuntimeEffect::make_for_shader(DIFF_SHADER_SOURCE, None);
    static INK_EFFECT: Result<RuntimeEffect, String> =
        RuntimeEffect::make_for_shader(INK_SHADER_SOURCE, None);
}

fn filter_error(msg: &str) -> Error {
    Error::MemeFeedback(msg.to_string())
}

fn effect(key: &'static LocalKey<Result<RuntimeEffect, String>>) -> Result<RuntimeEffect, Error> {
    key.with(|effect| {
        effect
            .clone()
            .map_err(|err| filter_error(&format!("着色器编译失败：{err}")))
    })
}

fn image_shader(image: &Image) -> Result<Shader, Error> {
    image
        .to_shader(
            Some((TileMode::Clamp, TileMode::Clamp)),
            SamplingOptions::default(),
            None,
        )
        .ok_or_else(|| filter_error("图像转 shader 失败"))
}

fn draw_shader(size: ISize, shader: Shader) -> Image {
    let mut surface = gpu_backend::surface(size);
    surface.canvas().draw_rect(
        Rect::from_isize(size),
        Paint::default()
            .set_shader(shader)
            .set_blend_mode(BlendMode::Src),
    );
    surface.image_snapshot()
}

/// 两级模糊都渲在 N32 上：实测半浮点缓冲会让整条流水线慢约 9 倍
/// （2269×3173 的图 0.7s → 6.2s），而输出差异只落在线条边缘的墨量过渡上，
/// 肉眼分辨不出来。
///
/// 另外记一笔实测：Skia 的 CPU 高斯以 sigma = 2 为界，小于 2 走精确卷积
/// （7.2MP 图 sigma=1.9 要 398ms），大于等于 2 切到常数代价的盒式近似（84ms）。
/// 想省这一段可以先把图缩一半再模糊，但线条会变粗变毛躁，所以没有默认启用；
/// 需要提速请用「缩放」选项，直接把长边限制到 1600。
fn blur_image(image: &Image, sigma: f32) -> Result<Image, Error> {
    let sigma = sigma.max(0.05);
    let filter = image_filters::blur((sigma, sigma), TileMode::Clamp, None, None)
        .ok_or_else(|| filter_error("高斯模糊创建失败"))?;
    let mut surface = gpu_backend::surface(image.dimensions());
    let mut paint = Paint::default();
    paint.set_image_filter(filter);
    surface.canvas().draw_image(image, (0, 0), Some(&paint));
    Ok(surface.image_snapshot())
}

fn line_art_ink(
    image: &Image,
    sigma: f32,
    low: f32,
    high: f32,
    dark: f32,
    thickness: f32,
) -> Result<Image, Error> {
    let fine = blur_image(image, sigma)?;
    let coarse = blur_image(image, sigma * DOG_RATIO)?;

    let mut uniforms = Vec::with_capacity(12);
    uniforms.extend_from_slice(&low.to_le_bytes());
    uniforms.extend_from_slice(&high.to_le_bytes());
    uniforms.extend_from_slice(&dark.to_le_bytes());
    let children = [
        ChildPtr::Shader(image_shader(&fine)?),
        ChildPtr::Shader(image_shader(&coarse)?),
    ];
    let shader = effect(&LINE_EFFECT)?
        .make_shader(Data::new_copy(&uniforms), &children, None)
        .ok_or_else(|| filter_error("线稿着色器创建失败"))?;
    let ink = draw_shader(image.dimensions(), shader);

    if thickness > 0.05 {
        let filter = image_filters::dilate((thickness, thickness), None, None)
            .ok_or_else(|| filter_error("线条加粗失败"))?;
        Ok(gpu_backend::filtered(&ink, filter))
    } else {
        Ok(ink)
    }
}

fn create_diff_shader(
    image1: &Image,
    image2: &Image,
    light_cut: f32,
    dark_cut: f32,
) -> Result<Shader, Error> {
    let mut uniforms = Vec::with_capacity(8);
    uniforms.extend_from_slice(&light_cut.to_le_bytes());
    uniforms.extend_from_slice(&dark_cut.to_le_bytes());
    let children = [
        ChildPtr::Shader(image_shader(image1)?),
        ChildPtr::Shader(image_shader(image2)?),
    ];
    effect(&DIFF_EFFECT)?
        .make_shader(Data::new_copy(&uniforms), &children, None)
        .ok_or_else(|| filter_error("差分着色器创建失败"))
}

fn create_ink_shader(image: &Image, gain: f32) -> Result<Shader, Error> {
    let mut uniforms = Vec::with_capacity(4);
    uniforms.extend_from_slice(&gain.to_le_bytes());
    let children = [ChildPtr::Shader(image_shader(image)?)];
    effect(&INK_EFFECT)?
        .make_shader(Data::new_copy(&uniforms), &children, None)
        .ok_or_else(|| filter_error("墨色着色器创建失败"))
}

/// 粗细档位 → (线条尺度, 加粗半径)，两者都会再乘以图像尺寸归一化系数。
fn line_preset(convolute: &str) -> (f32, f32) {
    match convolute {
        "极细" => (0.50, 0.0),
        "精细" => (0.60, 0.0),
        "稍粗" => (0.95, 0.5),
        "超粗" => (1.25, 1.0),
        "极粗" => (1.65, 1.6),
        _ => (0.72, 0.0),
    }
}

fn contrast_filter(grayscale: bool, contrast: f32, denoise: bool) -> Result<ImageFilter, Error> {
    let color_filter = high_contrast_filter::new(&HighContrastConfig::new(
        grayscale,
        InvertStyle::NoInvert,
        contrast,
    ))
    .ok_or_else(|| filter_error("对比度滤镜创建失败"))?;
    let mut filter = image_filters::color_filter(color_filter, None, None)
        .ok_or_else(|| filter_error("对比度滤镜创建失败"))?;
    if denoise {
        let denoise_filter = separable_average_filter(3, 1.0, 0.0, None)
            .ok_or_else(|| filter_error("降噪滤镜创建失败"))?;
        filter = compose_filters(denoise_filter, filter)?;
    }
    Ok(filter)
}

fn separable_average_filter(
    width: usize,
    gain: scalar,
    bias: scalar,
    input: impl Into<Option<ImageFilter>>,
) -> Option<ImageFilter> {
    let n = width as i32;
    let kernel = vec![1.0 / width as f32; width];
    let offset = (n - 1) / 2;
    let horizontal = image_filters::matrix_convolution(
        (n, 1),
        &kernel,
        1.0,
        0.0,
        (offset, 0),
        TileMode::Clamp,
        false,
        input,
        None,
    )?;
    image_filters::matrix_convolution(
        (1, n),
        &kernel,
        gain,
        bias,
        (0, offset),
        TileMode::Clamp,
        false,
        Some(horizontal),
        None,
    )
}

fn dense_convolution_filter(
    kernel_size: impl Into<ISize>,
    kernel: &[scalar],
    kernel_offset: impl Into<IPoint>,
    gain: scalar,
    bias: scalar,
    input: impl Into<Option<ImageFilter>>,
) -> Option<ImageFilter> {
    image_filters::matrix_convolution(
        kernel_size,
        kernel,
        gain,
        bias,
        kernel_offset,
        TileMode::Clamp,
        false,
        input,
        None,
    )
}

fn compose_filters(outer: ImageFilter, inner: ImageFilter) -> Result<ImageFilter, Error> {
    image_filters::compose(outer, inner).ok_or_else(|| filter_error("滤镜组合失败"))
}

fn edge_filter(convolute: &str, gain: f32, bias: f32) -> Result<Option<ImageFilter>, Error> {
    let filter = match convolute {
        "线稿" => return Ok(None),
        "极细" => separable_average_filter(3, gain, bias, None),
        "精细" => separable_average_filter(5, gain, bias, None),
        "一般" => separable_average_filter(7, gain, bias, None),
        "稍粗" => separable_average_filter(9, gain, bias, None),
        "超粗" => separable_average_filter(11, gain, bias, None),
        "极粗" => separable_average_filter(13, gain, bias, None),
        "浮雕" => dense_convolution_filter(
            (3, 3),
            &[
                1.0, 1.0, 1.0, //
                1.0, 1.0, -1.0, //
                -1.0, -1.0, -1.0, //
            ],
            (1, 1),
            gain,
            bias,
            None,
        ),
        "超浮雕" => dense_convolution_filter(
            (5, 5),
            &[
                1.0, 1.0, 1.0, 1.0, 1.0, //
                1.0, 1.0, 1.0, 1.0, 1.0, //
                1.0, 1.0, 1.0, -1.0, -1.0, //
                -1.0, -1.0, -1.0, -1.0, -1.0, //
                -1.0, -1.0, -1.0, -1.0, -1.0, //
            ],
            (2, 2),
            gain,
            bias,
            None,
        ),
        other => return Err(filter_error(&format!("未知的卷积矩阵：{other}"))),
    };
    filter
        .map(Some)
        .ok_or_else(|| filter_error("卷积滤镜创建失败"))
}

fn create_kiss_gradient(isize: impl Into<ISize>) -> Option<Shader> {
    let colors = [
        Color::from_rgb(251, 186, 48), // #fbba30
        Color::from_rgb(252, 114, 53), // #fc7235
        Color::from_rgb(252, 53, 78),  // #fc354e
        Color::from_rgb(207, 54, 223), // #cf36df
        Color::from_rgb(55, 181, 217), // #37b5d9
        Color::from_rgb(62, 182, 218), // #3eb6da
    ];
    let isize = isize.into();
    gradient_shader::linear(
        (
            Point::new(0.0, 0.0),
            Point::new(isize.width as f32, isize.height as f32),
        ),
        GradientShaderColors::Colors(&colors),
        Some(&[0.0_f32, 0.4, 0.6, 0.7, 0.8, 1.0] as &[scalar]),
        TileMode::Clamp,
        None,
        None,
    )
}

fn limit_long_side(image: &Image, limit: i32) -> Image {
    let (width, height) = (image.width(), image.height());
    let long = width.max(height);
    if long <= limit {
        return image.clone();
    }
    let factor = limit as f32 / long as f32;
    image.resize_exact((
        ((width as f32 * factor).round() as i32).max(1),
        ((height as f32 * factor).round() as i32).max(1),
    ))
}

pub fn louvre(images: Vec<InputImage>, _: Vec<String>, options: Louvre) -> Result<Vec<u8>, Error> {
    let convolute = options.convolute.unwrap_or_else(|| "一般".to_string());
    let classic =
        options.classic.unwrap_or(false) || matches!(convolute.as_str(), "浮雕" | "超浮雕" | "线稿");
    let diff = convolute != "线稿" && !options.no_diff.unwrap_or(false);

    let (preset_scale, preset_thickness) = line_preset(&convolute);
    let line_scale = options.line_scale.unwrap_or(preset_scale).clamp(0.3, 4.0);
    let thickness = options.thickness.unwrap_or(preset_thickness).clamp(0.0, 8.0);
    let ink_low = options.ink_low.unwrap_or(0.0085).max(0.0005);
    let ink_high = options.ink_high.unwrap_or(0.028).max(ink_low + 0.0005);
    let dark_boost = options.dark_boost.unwrap_or(1.0).clamp(0.0, 3.0);

    let contrast = options.contrast.unwrap_or(0.05).clamp(0.0, 0.99);
    let gain = options.gain.unwrap_or(1.0);
    let bias = options.bias.unwrap_or(0.0);
    let light_cut = options.light_cut.unwrap_or(128.0);
    let dark_cut = options.dark_cut.unwrap_or(1.0);
    let denoise = options.denoise.unwrap_or(false);

    let resize = options.resize.unwrap_or(false);
    let kuma = !options.gray.unwrap_or(false);
    let full_bg = !options.no_bg.unwrap_or(false);

    let func = |images: Vec<Image>| -> Result<Image, Error> {
        let src = images.first().unwrap();
        // 带透明通道的图先铺白，否则透明区在预乘下等于纯黑，边缘会凭空多一圈。
        let flat = if src.is_opaque() {
            src.clone()
        } else {
            src.with_background(Color::WHITE)
        };
        let work = if resize {
            limit_long_side(&flat, FAST_LONG_SIDE)
        } else {
            flat
        };
        let img_size = work.dimensions();
        let scale =
            (img_size.width.min(img_size.height) as f32 / REFERENCE_SHORT_SIDE).clamp(0.45, 4.0);

        let mut final_img = gpu_backend::surface(img_size);
        {
            let canvas = final_img.canvas();
            if !classic {
                let ink = line_art_ink(
                    &work,
                    line_scale * scale,
                    ink_low,
                    ink_high,
                    dark_boost,
                    thickness * scale,
                )?;
                canvas.draw_image(&ink, (0, 0), None);
            } else if diff {
                let base_filter = contrast_filter(true, contrast, denoise)?;
                let edge = edge_filter(&convolute, gain, bias)?;
                let img2 = gpu_backend::filtered(&work, base_filter);
                let img1 = match edge {
                    Some(filter) => gpu_backend::filtered(&img2, filter),
                    None => img2.clone(),
                };
                let shader = create_diff_shader(&img1, &img2, light_cut, dark_cut)?;
                canvas.draw_rect(
                    Rect::from_isize(img_size),
                    Paint::default()
                        .set_shader(shader)
                        .set_blend_mode(BlendMode::Src),
                );
            } else {
                let base_filter = contrast_filter(true, contrast, denoise)?;
                let edge = edge_filter(&convolute, gain, bias)?;
                let filter = match edge {
                    Some(filter) => compose_filters(filter, base_filter)?,
                    None => base_filter,
                };
                // 这一支以前画的是不透明图，后面的 SrcIn 渐变会把整张图糊成色块，
                // 所以这里同样转成「墨色 alpha」。
                let filtered = gpu_backend::filtered(&work, filter);
                let shader = create_ink_shader(&filtered, 1.0)?;
                canvas.draw_rect(
                    Rect::from_isize(img_size),
                    Paint::default()
                        .set_shader(shader)
                        .set_blend_mode(BlendMode::Src),
                );
            }
        }

        let canvas = final_img.canvas();
        if kuma {
            let gradient =
                create_kiss_gradient(img_size).ok_or_else(|| filter_error("渐变创建失败"))?;
            canvas.draw_irect(
                IRect::from_size(img_size),
                Paint::default()
                    .set_shader(gradient)
                    .set_blend_mode(BlendMode::SrcIn),
            );
        }
        if full_bg {
            canvas.draw_irect(
                IRect::from_size(img_size),
                new_paint(Color::WHITE).set_blend_mode(BlendMode::DstOver),
            );
        }
        // GPU 后端下这一步把结果回读成光栅图，编码器才拿得到像素。
        Ok(gpu_backend::to_raster(&final_img.image_snapshot()))
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "louvre",
    louvre,
    min_images = 1,
    max_images = 1,
    keywords = &["卢浮宫"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2026, 8, 31),
);
