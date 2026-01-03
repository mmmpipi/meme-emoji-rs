use std::cell::LazyCell;

use skia_safe::{
    ColorMatrix, Data, IRect, Image, Paint, Point, RuntimeEffect, SamplingOptions, Size,
    runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_swirl_effect(
    image: &Image,
    center: impl Into<Point>,
    dot_size: f32,
    dot_spacing: f32,
    angle: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    let center = center.into();
    const SHADER_SOURCE: &str = r#"
uniform shader image;

uniform float2 center;
uniform float dotSize;
uniform float dotSpacing;
uniform float angle;

// 分离RGB通道并分别处理
vec3 separateChannels(vec3 color) {
    return vec3(
        color.r > 0.5 ? 1.0 : 0.0,
        color.g > 0.5 ? 1.0 : 0.0,
        color.b > 0.5 ? 1.0 : 0.0
    );
}

// 旋转坐标
vec2 rotate(vec2 uv, float angle, vec2 center) {
    float s = sin(angle);
    float c = cos(angle);
    mat2 rot = mat2(c, -s, s, c);
    return rot * (uv - center) + center;
}

// 创建彩色半调网点
vec3 colorHalftone(vec2 pos, vec3 color, float angleOffset) {
    vec3 result = vec3(0.0);
    
    // 为每个通道创建不同的角度偏移
    float angles[3];
    angles[0] = angle;           // 红色通道
    angles[1] = angle + 0.785;   // 绿色通道 + 45度
    angles[2] = angle + 1.57;    // 蓝色通道 + 90度
    
    for (int i = 0; i < 3; i++) {
        vec2 rotatedPos = rotate(pos, angles[i], center);
        vec2 gridPos = fract(rotatedPos / dotSpacing);
        vec2 gridCenter = vec2(0.5, 0.5);
        float dist = distance(gridPos, gridCenter);
        
        float radius = (1.0 - color[i]) * (dotSize / 2.0);
        float pattern = 1.0 - smoothstep(radius - 0.1, radius + 0.1, dist * dotSpacing);
        
        result[i] = pattern * step(0.5, color[i]);
    }
    
    return result;
}

vec4 main(vec2 coord) {
    vec4 original = image.eval(coord);
    vec3 pattern = colorHalftone(coord, original.rgb, 0.0);
    
    // 使用原图的透明度
    return vec4(pattern, original.a);
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

fn colorful_halftone(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: ColorfulHalftoneEffect,
) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let dot_size: f32 = options.dot_size.unwrap_or(1.0);
    let angle: f32 = 0.0;
    let dot_spacing: f32 = options.dot_spacing.unwrap_or(0.8);
    let inverse_color: bool = options.inverse_color.unwrap_or(false);
    let min_size: i32 = 600;

    let single_func = |images: Vec<Image>| {
        let inverse_color_matrix = ColorMatrix::new(
            -1.0, 0.0, 0.0, 0.0, 1.0, //
            0.0, -1.0, 0.0, 0.0, 1.0, //
            0.0, 0.0, -1.0, 0.0, 1.0, //
            0.0, 0.0, 0.0, 1.0, 0.0, //
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
        );
        if let Err(e) = shader {
            return Err(Error::MemeFeedback(format!("着色器错误：{:?}", e)));
        }
        let shader = shader.unwrap();
        result.canvas().draw_irect(
            IRect::from_size(img.dimensions()),
            Paint::default().set_shader(shader),
        );
        if inverse_color {
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

    /// 反色
    #[option(short, long,long_aliases = ["反色"],default = false)]
    pub inverse_color: Option<bool>,
}

register_meme!(
    "colorful_halftone",
    colorful_halftone,
    min_images = 1,
    max_images = 1,
    keywords = &["彩色半色调","彩色打印机"],
    date_created = local_date(2025, 11, 11),
    date_modified = local_date(2025, 11, 11),
);
