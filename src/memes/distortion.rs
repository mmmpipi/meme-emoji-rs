use std::cell::LazyCell;

use skia_safe::{
    Data, IRect, Image, Paint, RuntimeEffect, SamplingOptions,
    runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_effect(
    image: &Image,
    time: f32,
    _edge_width: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    const SHADER_SOURCE: &str = r#"
uniform shader image;
uniform float time;
uniform float frequency;
uniform float imageWidth;
uniform float contrast;


float get_shade(float4 pixel){
    return (pixel.r+pixel.g+pixel.b)/3.0;
}

float4 main(float2 coord) {
    float result = 0.0;
    for (float x = 0.0;x<=10;x++){
    float4 pixel = image.eval(float2(x*imageWidth/10.0,coord.y));
        result += get_shade(pixel);
    }
    result = sin(result*coord.x+time);
    result *= (1.0-contrast) + contrast * get_shade(image.eval(coord));

    return float4(result,result,result,1.0);

}"#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 准备uniform数据
    let mut uniforms = vec![];
    // let center = Point::from((image.dimensions().width / 2, image.dimensions().height / 2));
    uniforms.extend_from_slice(&time.to_le_bytes());
    uniforms.extend_from_slice(&10.0_f32.to_le_bytes());
    uniforms.extend_from_slice(&(image.dimensions().width as f32).to_le_bytes());
    uniforms.extend_from_slice(&0.8_f32.to_le_bytes());

    let image_shader = image
        .to_shader(None, SamplingOptions::default(), None)
        .expect("图像转shader错误");
    let data = Data::new_copy(&uniforms);

    let children = vec![ChildPtr::Shader(image_shader)];
    let swirl_shader = effect
        .make_shader(data, &children, None)
        .ok_or("无法创建着色器")?;

    Ok(swirl_shader)
}

pub fn distortion(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: Options,
) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let edge_width: f32 = options.edge_width.unwrap_or(0.3);
    let frame = options.frame.unwrap_or(30) as u32;
    let func = |i: usize, images: Vec<Image>| {
        let time = i as f32 * 2.0 / frame as f32;
        let img = images.first().unwrap().to_owned();

        let mut result = new_surface(img.dimensions());
        let shader = create_effect(&img, time, edge_width);
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

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: frame,
            duration: 0.1,
        },
        FrameAlign::NoExtend,
    )
}

#[derive(MemeOptions)]
pub(crate) struct Options {
    /// 帧数
    #[option(short, long,long_aliases = ["帧数"], minimum = 0, maximum = 100, default = 30)]
    pub frame: Option<i32>,

    /// 消失的边缘宽度
    #[option(short, long,long_aliases = ["边缘宽度"], minimum = 0.0, maximum = 100.0, default = 0.3)]
    pub edge_width: Option<f32>,
}

register_meme!(
    "distortion",
    distortion,
    min_images = 1,
    max_images = 1,
    keywords = &["失真"],
    date_created = local_date(2026, 1, 14),
    date_modified = local_date(2026, 1, 14),
);
