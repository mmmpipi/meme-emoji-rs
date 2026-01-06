use std::cell::LazyCell;

use skia_safe::{
    Data, IRect, Image, Paint, RuntimeEffect, SamplingOptions, runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_melt_effect(
    image: &Image,
    time: f32,
    edge_width: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    const SHADER_SOURCE: &str = r#"
        uniform shader image;
        uniform float threshold;
        uniform vec4 edgeColor;
        uniform float edgeWidth;
        
        vec4 main(float2 coord) {
            vec4 color = image.eval(coord);
            float brightness = (color.r + color.g + color.b) / 3.0;
            
            // 计算溶解 alpha
            float dissolve = 0.0;
            if (brightness >= threshold + edgeWidth) {
                dissolve = 1.0;
            } else if (brightness > threshold) {
                dissolve = (brightness - threshold) / edgeWidth;
            }
            
            // 边缘效果
            vec4 finalColor = color;
            if (brightness < threshold + edgeWidth && brightness > threshold) {
                float mixFactor = 1.0 - (brightness - threshold) / edgeWidth;
                finalColor = mix(color, edgeColor, mixFactor);
            }
            
            return vec4(finalColor.rgb, finalColor.a * dissolve);
        }
    "#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 准备uniform数据
    let mut uniforms = vec![];
    uniforms.extend_from_slice(&time.to_le_bytes());
    let edge_color: [f32; 4] = [0.9, 0.6, 0.9, 1.0]; // RGBA
    for component in edge_color {
        uniforms.extend_from_slice(&component.to_le_bytes());
    }
    uniforms.extend_from_slice(&edge_width.to_le_bytes());

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

pub fn disappear(images: Vec<InputImage>, _: Vec<String>, options: Options) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let edge_width: f32 = options.edge_width.unwrap_or(0.3);
    let frame = options.frame.unwrap_or(30) as u32;
    let func = |i: usize, images: Vec<Image>| {
        let time = i as f32 / frame as f32;
        let img = images.first().unwrap().to_owned();

        let mut result = new_surface(img.dimensions());
        let shader = create_melt_effect(&img, time, edge_width);
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
    "disappear",
    disappear,
    min_images = 1,
    max_images = 1,
    keywords = &["消失", "湮灭"],
    date_created = local_date(2026, 1, 5),
    date_modified = local_date(2026, 1, 5),
);
