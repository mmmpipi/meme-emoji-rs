use std::cell::LazyCell;

use skia_safe::{
    Data, IRect, Image, Paint, Point, RuntimeEffect, SamplingOptions,
    runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::make_png_or_gif,
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_effect(
    image: &Image,
    center: impl Into<Point>,
    dot_size: f32,
    dot_spacing: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    let center = center.into();
    const SHADER_SOURCE: &str = r#"
        uniform shader source;
        uniform vec2 center;
        uniform float aberration_amount;
        uniform float aberration_falloff;
        
        // 计算色差偏移
        vec4 chromatic_aberration(vec2 uv) {
            // 计算到中心的向量
            vec2 dir = uv - center;
            float dist = length(dir);
            vec2 dir_normalized = normalize(dir);
            
            // 基于距离的偏移量（非线性）
            float offset = aberration_amount * pow(dist / aberration_falloff, 1.5);
            
            // 每个通道不同的偏移
            float r = source.eval(uv + dir_normalized * offset * 1.0).r;
            float g = source.eval(uv + dir_normalized * offset * 0.5).g;
            float b = source.eval(uv).b;  // 蓝色通道不偏移
            
            return vec4(r, g, b, 1.0);
        }
        
        vec4 main(vec2 coord) {
            return chromatic_aberration(coord);
        }
    "#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 准备uniform数据
    let mut uniforms = vec![];

    uniforms.extend_from_slice(&center.x.to_le_bytes());
    uniforms.extend_from_slice(&center.y.to_le_bytes());

    uniforms.extend_from_slice(&dot_size.to_le_bytes());
    uniforms.extend_from_slice(&dot_spacing.to_le_bytes());

    let image_shader = image
        .to_shader(None, SamplingOptions::default(), None)
        .expect("图像转shader错误");
    let data = Data::new_copy(&uniforms);
    // 构建漩涡着色器
    let children = vec![ChildPtr::Shader(image_shader)];
    let swirl_shader = effect
        .make_shader(data, &children, None)
        .ok_or("无法创建着色器")?;

    Ok(swirl_shader)
}

pub fn aberration(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: Options,
) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let dot_size: f32 = options.amount.unwrap_or(0.1);
    let dot_spacing: f32 = options.falloff.unwrap_or(10.0);

    let single_func = |images: Vec<Image>| {
        let img = images.first().unwrap().to_owned();

        let mut result = new_surface(img.dimensions());
        let center = (img.dimensions().width/2,img.dimensions().height/2);
        let shader = create_effect(&img, center, dot_size, dot_spacing);
        if let Err(e) = shader {
            return Err(Error::MemeFeedback(format!("着色器错误：{:?}", e)));
        }
        let shader = shader.unwrap();
        result.canvas().draw_irect(
            IRect::from_size(img.dimensions()),
            Paint::default().set_shader(shader),
        );

        Ok(result.image_snapshot())
    };

    make_png_or_gif(images, single_func)
}

#[derive(MemeOptions)]
pub(crate) struct Options {
    #[option(short,long,long_aliases = ["色差强度"], minimum = 0.0, maximum = 100.0, default = 0.1)]
    pub amount: Option<f32>,

    #[option(short, long,long_aliases = ["色差衰减"], minimum = -100.0, maximum = 100.0, default = 10.0)]
    pub falloff: Option<f32>,
}

register_meme!(
    "aberration",
    aberration,
    min_images = 1,
    max_images = 1,
    keywords = &["色差"],
    date_created = local_date(2026, 1, 6),
    date_modified = local_date(2026, 1, 6),
);
