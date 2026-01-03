use std::cell::LazyCell;

use skia_safe::{
    Color, ColorMatrix, Data, IRect, Image, Paint, Point, Rect, RuntimeEffect, SamplingOptions,
    Size, runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::make_png_or_gif,
    image::ImageExt,
    shortcut,
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_swirl_effect(
    image: &Image,
    center: impl Into<Point>,
    dot_size: f32,
    dot_spacing: f32,
    angle: f32,
    gray_threshold: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    let center = center.into();
    const SHADER_SOURCE: &str = r#"
uniform shader image;

uniform float2 center;        // 中心点
uniform float dotSize;        // 网点大小
uniform float dotSpacing;     // 网点间距
uniform float angle;          // 旋转角度（弧度）
uniform float grayThreshold;  // 灰度阈值

// 将颜色转换为灰度
float rgb2gray(vec3 color) {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

// 旋转坐标
vec2 rotate(vec2 uv, float angle) {
    float s = sin(angle);
    float c = cos(angle);
    mat2 rot = mat2(c, -s, s, c);
    return rot * (uv - center) + center;
}

// 创建半调网点
float halftoneDot(vec2 pos, float gray) {
    // 将坐标映射到网点网格
    vec2 gridPos = fract(pos / dotSpacing);
    
    // 计算到网格中心的距离
    vec2 gridCenter = vec2(0.5, 0.5);
    float dist = distance(gridPos, gridCenter);
    
    // 根据灰度值计算网点半径
    float radius = (1.0 - gray) * (dotSize / 2.0);
    
    // 生成网点（使用平滑过渡）
    return 1.0 - smoothstep(radius - 0.1, radius + 0.1, dist * dotSpacing);
}

vec4 main(vec2 coord) {
    // 获取原始颜色
    vec4 color = image.eval(coord);
    
    // 转换到灰度
    float gray = rgb2gray(color.rgb);
    
    // 旋转坐标系统
    vec2 rotatedCoord = rotate(coord, angle);
    
    // 生成半调图案
    float pattern = halftoneDot(rotatedCoord, gray);
    
    // 应用阈值进行二值化
    float thresholded = step(grayThreshold, gray);
    
    // 混合模式：使用网点图案与阈值结果混合
    float finalPattern = pattern * thresholded;
    
    // 输出结果（黑白半调）
    return vec4(finalPattern, finalPattern, finalPattern, color.a);
}
"#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 准备uniform数据
    let mut uniforms = vec![];
    // 添加漩涡中心 (vec2)
    uniforms.extend_from_slice(&center.x.to_le_bytes());
    uniforms.extend_from_slice(&center.y.to_le_bytes());

    uniforms.extend_from_slice(&dot_size.to_le_bytes());
    uniforms.extend_from_slice(&dot_spacing.to_le_bytes());
    uniforms.extend_from_slice(&angle.to_le_bytes());
    uniforms.extend_from_slice(&gray_threshold.to_le_bytes());

    //uniforms.extend_from_slice(&(pattern_type as i32).to_le_bytes());

    // 添加图像尺寸 (vec2)
    // uniforms.extend_from_slice(&(image_size.width as f32).to_le_bytes());
    // uniforms.extend_from_slice(&(image_size.height as f32).to_le_bytes());

    let image_shader = image
        .to_shader(None, SamplingOptions::default(), None)
        .expect("图像转shader错误");
    let data = Data::new_copy(&uniforms);
    // 构建漩涡着色器
    let children = vec![ChildPtr::Shader(image_shader)];
    let swirl_shader = effect
        .make_shader(data, &children, None)
        .ok_or("无法创建halftone着色器")?;

    Ok(swirl_shader)
}

pub fn halftone(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: ColorfulHalftoneEffect,
) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let colorful = options.colorful.unwrap_or(false);
    let colorful = if options.colorful_inner.unwrap_or(0) > 0 {
        true
    } else {
        colorful
    };
    let dot_size: f32 = options.dot_size.unwrap_or(1.0);
    let angle: f32 = 0.0;
    let dot_spacing: f32 = if colorful {
        options.dot_spacing.unwrap_or(0.6)
    } else {
        options.dot_spacing.unwrap_or(0.8)
    };
    let inverse_color: bool = options.inverse_color.unwrap_or(false);
    let gray_threshold: f32 = options.gray_threshold.unwrap_or(0.0);
    let min_size: i32 = 600;
    let inverse_color = if colorful {
        !inverse_color
    } else {
        inverse_color
    };

    let single_func = |images: Vec<Image>| {
        let inverse_color_matrix = ColorMatrix::new(
            -1.0, 0.0, 0.0, 0.0, 1.0, //
            0.0, -1.0, 0.0, 0.0, 1.0, //
            0.0, 0.0, -1.0, 0.0, 1.0, //
            0.0, 0.0, 0.0, 1.0, 0.0, //
        );
        let last_color_matrix = ColorMatrix::new(
            1.0, 0.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, 0.0, //
            1.0, 0.0, 0.0, 0.0, 0.0, //
        );

        let mut img = images.first().unwrap().to_owned();
        if img.dimensions().height < min_size {
            let value = min_size as f32 / img.dimensions().height as f32;
            img = img.resize_exact(
                Size::new(
                    img.dimensions().width as f32 * value,
                    img.dimensions().height as f32 * value,
                )
                .to_round(),
            )
        }
        if img.dimensions().width < min_size {
            let value = min_size as f32 / img.dimensions().width as f32;
            img = img.resize_exact(
                Size::new(
                    img.dimensions().width as f32 * value,
                    img.dimensions().height as f32 * value,
                )
                .to_round(),
            )
        }
        let mut result = new_surface(img.dimensions());
        let shader = create_swirl_effect(
            &img,
            (0.5, 0.5),
            dot_size,
            dot_spacing,
            angle,
            gray_threshold,
        );
        if let Err(e) = shader {
            return Err(Error::MemeFeedback(format!("着色器错误：{:?}", e)));
        }
        let shader = shader.unwrap();
        result.canvas().draw_irect(
            IRect::from_size(img.dimensions()),
            Paint::default().set_shader(shader),
        );

        if colorful {
            result = result
                .image_snapshot()
                .color_matrix(last_color_matrix)
                .to_surface();
            result.canvas().draw_image(
                &img,
                (0, 0),
                Some(Paint::default().set_blend_mode(skia_safe::BlendMode::SrcIn)),
            );
            result.canvas().draw_rect(
                Rect::from_isize(img.dimensions()),
                Paint::default()
                    .set_color(Color::WHITE)
                    .set_blend_mode(skia_safe::BlendMode::DstOver),
            );
        }

        if !inverse_color {
            result = result
                .image_snapshot()
                .color_matrix(inverse_color_matrix)
                .to_surface();
        }

        Ok(result.image_snapshot())
    };

    make_png_or_gif(images, single_func)
}

#[derive(MemeOptions)]
pub(crate) struct ColorfulHalftoneEffect {
    /// 点大小
    #[option(long,long_aliases = ["ds","点大小"], minimum = 0.1, maximum = 10.0, default = 1.0)]
    pub dot_size: Option<f32>,

    /// 点间距
    #[option(short, long,long_aliases = ["点间距"], minimum = 0.0, maximum = 10.0, default = 0.8)]
    pub dot_spacing: Option<f32>,

    /// 将会在图像上减去此灰度
    #[option(short, long,long_aliases = ["灰度大小"], minimum = -1.0, maximum = 1.0, default = 0.0)]
    pub gray_threshold: Option<f32>,

    /// 反色
    #[option(short, long,long_aliases = ["反色"],default = false)]
    pub inverse_color: Option<bool>,

    /// 彩色
    #[option(short, long,long_aliases = ["彩色"],default = false)]
    pub colorful: Option<bool>,

    /// 调试选项
    #[option(long, default = 0)]
    pub colorful_inner: Option<i32>,
}

register_meme!(
    "halftone",
    halftone,
    min_images = 1,
    max_images = 1,
    keywords = &["半色调", "打印机"],
    shortcuts = &[shortcut!("彩色打印机", options = &[("colorful_inner", 1)])],
    date_created = local_date(2026, 1, 4),
    date_modified = local_date(2026, 1, 4),
);
