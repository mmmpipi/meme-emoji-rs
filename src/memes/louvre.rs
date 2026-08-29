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
    tools::{local_date, new_paint, new_surface},
};

use crate::{options::Louvre, register_meme};

const DIFF_SHADER_SOURCE: &str = r#"
    uniform shader u_texture1;
    uniform shader u_texture2;
    uniform float u_light_cut;
    uniform float u_dark_cut;

    vec4 main(vec2 uv) {
        vec4 color1 = u_texture1.eval(uv);
        vec4 color2 = u_texture2.eval(uv);
        float r = 0.5 + color2.r - color1.r; // 128/255 ≈ 0.5
        float scale = 255.0 / (255.0 - u_light_cut - u_dark_cut);
        r = clamp((r * 255.0 - u_dark_cut) * scale, 0.0, 255.0) / 255.0;
        return vec4(r, r, r, 1.0 - r);
    }
"#;

const BILATERAL_SHADER_SOURCE: &str = r#"
    uniform shader u_image;
    uniform float u_dx;
    uniform float u_dy;
    uniform float u_sigma_s;
    uniform float u_sigma_r;

    vec4 main(vec2 uv) {
        vec4 center = u_image.eval(uv);
        float center_luma = dot(center.rgb, vec3(0.299, 0.587, 0.114));
        float two_s = 2.0 * u_sigma_s * u_sigma_s;
        float two_r = 2.0 * u_sigma_r * u_sigma_r;
        vec4 sum = vec4(0.0);
        float wsum = 0.0;
        for (int i = -6; i <= 6; ++i) {
            float fi = float(i);
            vec4 c = u_image.eval(uv + vec2(u_dx, u_dy) * fi);
            float luma = dot(c.rgb, vec3(0.299, 0.587, 0.114));
            float d = luma - center_luma;
            float w = exp(-(fi * fi) / two_s - (d * d) / two_r);
            sum += w * c;
            wsum += w;
        }
        return sum / max(wsum, 0.000001);
    }
"#;

thread_local! {
    static DIFF_EFFECT: RuntimeEffect = RuntimeEffect::make_for_shader(DIFF_SHADER_SOURCE, None)
        .expect("着色器编译失败");
    static BILATERAL_EFFECT: RuntimeEffect = RuntimeEffect::make_for_shader(BILATERAL_SHADER_SOURCE, None)
        .expect("着色器编译失败");
}

fn filter_error(msg: &str) -> Error {
    Error::MemeFeedback(msg.to_string())
}

fn contrast_filter(grayscale: bool, contrast: f32, denoise: bool) -> Result<ImageFilter, Error> {
    let color_filter = high_contrast_filter::new(&HighContrastConfig::new(
        grayscale,
        InvertStyle::NoInvert,
        contrast,
    ))
    .ok_or_else(|| filter_error("luma filter create Failed!"))?;
    let mut filter = image_filters::color_filter(color_filter, None, None)
        .ok_or_else(|| filter_error("luma filter create Failed!"))?;
    if denoise {
        let denoise_filter = separable_average_filter(3, 1.0, 0.0, None)
            .ok_or_else(|| filter_error("denoise filter create Failed!"))?;
        filter = compose_filters(denoise_filter, filter)?;
    }
    Ok(filter)
}

fn create_diff_effect(
    image1: &Image,
    image2: &Image,
    light_cut: f32,
    dark_cut: f32,
) -> Result<Shader, Box<dyn std::error::Error>> {
    let mut uniforms = Vec::with_capacity(8);
    uniforms.extend_from_slice(&light_cut.to_le_bytes());
    uniforms.extend_from_slice(&dark_cut.to_le_bytes());

    let image1_shader = image1
        .to_shader(None, SamplingOptions::default(), None)
        .ok_or("图像转shader错误")?;
    let image2_shader = image2
        .to_shader(None, SamplingOptions::default(), None)
        .ok_or("图像转shader错误")?;
    let children = [
        ChildPtr::Shader(image1_shader),
        ChildPtr::Shader(image2_shader),
    ];
    DIFF_EFFECT.with(|effect| {
        effect
            .make_shader(Data::new_copy(&uniforms), &children, None)
            .ok_or_else(|| "无法创建着色器".into())
    })
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
    image_filters::compose(outer, inner).ok_or_else(|| filter_error("filter compose Failed!"))
}

fn bilateral_pass(
    image: &Image,
    dx: f32,
    dy: f32,
    sigma_s: f32,
    sigma_r: f32,
) -> Result<Image, Error> {
    let image_shader = image
        .to_shader(
            Some((TileMode::Clamp, TileMode::Clamp)),
            SamplingOptions::default(),
            None,
        )
        .ok_or_else(|| filter_error("图像转shader错误"))?;
    let mut uniforms = Vec::with_capacity(16);
    uniforms.extend_from_slice(&dx.to_le_bytes());
    uniforms.extend_from_slice(&dy.to_le_bytes());
    uniforms.extend_from_slice(&sigma_s.to_le_bytes());
    uniforms.extend_from_slice(&sigma_r.to_le_bytes());
    let shader = BILATERAL_EFFECT
        .with(|effect| {
            effect.make_shader(
                Data::new_copy(&uniforms),
                &[ChildPtr::Shader(image_shader)],
                None,
            )
        })
        .ok_or_else(|| filter_error("无法创建着色器"))?;
    let mut surface = new_surface(image.dimensions());
    let canvas = surface.canvas();
    canvas.draw_rect(
        Rect::from_isize(image.dimensions()),
        Paint::default().set_shader(shader),
    );
    Ok(surface.image_snapshot())
}

fn bilateral_smooth(image: &Image, sigma_s: f32, sigma_r: f32) -> Result<Image, Error> {
    let horizontal = bilateral_pass(image, 1.0, 0.0, sigma_s, sigma_r)?;
    bilateral_pass(&horizontal, 0.0, 1.0, sigma_s, sigma_r)
}

fn toon_preprocess(image: &Image, scale: f32, sigma_s: f32, sigma_r: f32) -> Result<Image, Error> {
    let width = image.width();
    let height = image.height();
    let min_side = width.min(height).max(1) as f32;
    let scale = scale.max(1.0);
    let small_min = (min_side / scale).max(480.0).min(min_side);
    let (small_w, small_h, scaled) = if small_min + 0.5 < min_side {
        let factor = min_side / small_min;
        (
            ((width as f32 / factor).round() as i32).max(1),
            ((height as f32 / factor).round() as i32).max(1),
            true,
        )
    } else {
        (width, height, false)
    };
    let low = if scaled {
        image.resize_exact((small_w, small_h))
    } else {
        image.clone()
    };
    let smoothed = bilateral_smooth(&low, sigma_s.max(0.5), sigma_r.max(0.01))?;
    if scaled {
        Ok(smoothed.resize_exact((width, height)))
    } else {
        Ok(smoothed)
    }
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
        _ => unreachable!(""),
    };
    filter
        .map(Some)
        .ok_or_else(|| filter_error("edge filter create Failed!"))
}

pub fn louvre(images: Vec<InputImage>, _: Vec<String>, options: Louvre) -> Result<Vec<u8>, Error> {
    let convolute = options.convolute.unwrap_or_else(|| "一般".to_string());
    let contrast = options.contrast.unwrap_or(0.05);
    let gain = options.gain.unwrap_or(2.0);
    let bias = options.bias.unwrap_or(0.0);
    let light_cut = options.light_cut.unwrap_or(128.0);
    let dark_cut = options.dark_cut.unwrap_or(118.0);
    let denoise = options.denoise.unwrap_or(false);
    let toon = !options.no_toon.unwrap_or(false);
    let toon_scale = options.toon_scale.unwrap_or(1.6);
    let sigma_s = options.sigma_s.unwrap_or(3.0);
    let sigma_r = options.sigma_r.unwrap_or(0.08);
    let diff = convolute != "线稿" && !options.no_diff.unwrap_or(false);
    let kuma = !options.gray.unwrap_or(false);
    let full_bg = !options.no_bg.unwrap_or(false);

    let func = |images: Vec<Image>| {
        let src = images.first().unwrap();
        let work = if toon {
            toon_preprocess(src, toon_scale, sigma_s, sigma_r)?
        } else {
            src.clone()
        };
        let img_size = work.dimensions();
        let mut final_img = new_surface(img_size);
        {
            let canvas = final_img.canvas();
            if diff {
                let base_filter = contrast_filter(true, contrast, denoise)?;
                let edge = edge_filter(&convolute, gain, bias)?;
                let img2 = work.image_filter(base_filter);
                let img1 = match edge {
                    Some(filter) => img2.image_filter(filter),
                    None => img2.clone(),
                };
                let shader = create_diff_effect(&img1, &img2, light_cut, dark_cut)
                    .map_err(|e| Error::MemeFeedback(format!("着色器错误：{e:?}")))?;
                canvas.draw_rect(
                    Rect::from_isize(img_size),
                    Paint::default().set_shader(shader),
                );
            } else {
                let base_filter = contrast_filter(true, contrast, denoise)?;
                let edge = edge_filter(&convolute, gain, bias)?;
                let filter = match edge {
                    Some(filter) => compose_filters(filter, base_filter)?,
                    None => base_filter,
                };
                let mut paint = Paint::default();
                paint.set_image_filter(filter);
                canvas.draw_image(&work, (0, 0), Some(&paint));
            }
        }
        let canvas = final_img.canvas();
        if kuma {
            canvas.draw_irect(
                IRect::from_size(img_size),
                Paint::default()
                    .set_shader(create_kiss_gradient(img_size).unwrap())
                    .set_blend_mode(BlendMode::SrcIn),
            );
        }
        if full_bg {
            canvas.draw_irect(
                IRect::from_size(img_size),
                new_paint(Color::WHITE).set_blend_mode(BlendMode::DstOver),
            );
        }
        Ok(final_img.image_snapshot())
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
    date_modified = local_date(2026, 8, 29),
);
