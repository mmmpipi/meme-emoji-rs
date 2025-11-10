use std::{cell::LazyCell, f32::consts::PI};

use skia_safe::{
    Data, IRect, Image, Paint, RuntimeEffect, SamplingOptions, runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif, make_png_or_gif},
    tools::{local_date, new_surface},
};

use crate::{options::WaveEffect, register_meme};

fn create_wave_effect(
    image: &Image,
    amplitude: f32, // 振幅 - 控制波纹的强度
    frequency: f32, // 频率 - 控制波纹的密度
    phase: f32,     // 相位 - 用于动画效果
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    // 定义 SkSL 着色器源码
    const SHADER_SOURCE: &str = r#"
        uniform shader image;
        uniform float amplitude;
        uniform float frequency; 
        uniform float phase;
        uniform vec2 image_size;

        vec4 main(vec2 coord) {
            // 归一化坐标 [0,1]
            vec2 uv = coord / image_size;
            
            // 计算波纹偏移 - 使用正弦函数在X和Y方向都产生扭曲
            float wave_x = sin(uv.y * frequency + phase) * amplitude;
            float wave_y = sin(uv.x * frequency * 0.7 + phase * 1.3) * amplitude * 0.8;
            
            // 应用偏移到纹理坐标
            vec2 distorted_uv = uv + vec2(wave_x, wave_y);
            
            // 将归一化坐标转换回像素坐标
            vec2 sample_coord = distorted_uv * image_size;
            
            // 使用扭曲后的坐标采样图像
            return image.eval(sample_coord);
        }
    "#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("wave着色器编译失败")
    });

    let image_size = image.dimensions();

    // uniform数据
    let mut uniforms = vec![];

    // 添加振幅
    uniforms.extend_from_slice(&amplitude.to_le_bytes());
    // 添加频率
    uniforms.extend_from_slice(&frequency.to_le_bytes());
    // 添加相位
    uniforms.extend_from_slice(&phase.to_le_bytes());

    // 添加图像尺寸 (vec2)
    uniforms.extend_from_slice(&(image_size.width as f32).to_le_bytes());
    uniforms.extend_from_slice(&(image_size.height as f32).to_le_bytes());

    let data = Data::new_copy(&uniforms);

    // 创建输入图像的着色器
    let image_shader = image
        .to_shader(None, SamplingOptions::default(), None)
        .unwrap();

    // 构建波纹扭曲着色器
    let children = [ChildPtr::Shader(image_shader)];
    let wave_shader = effect
        .make_shader(data, &children, None)
        .ok_or("无法创建波纹着色器")?;
    Ok(wave_shader)
}

fn wave_distortion(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: WaveEffect,
) -> Result<Vec<u8>, Error> {
    let amplitude = options.amplitude.unwrap_or(0.02);
    let frequency = options.frequency.unwrap_or(12.0);
    let phase = options.phase.unwrap_or(0.0);
    let frame = options.frame.unwrap_or(10) as u32;

    let single_func = |images: Vec<Image>| {
        let img = images.first().unwrap();
        let mut result = new_surface(img.dimensions());
        let wave_shader = create_wave_effect(img, amplitude, frequency, phase);
        if let Err(e) = wave_shader {
            return Err(Error::MemeFeedback(format!("着色器错误：{:?}", e)));
        }
        let wave_shader = wave_shader.unwrap();
        result.canvas().draw_irect(
            IRect::from_size(img.dimensions()),
            Paint::default().set_shader(wave_shader),
        );
        Ok(result.image_snapshot())
    };

    let muti_func = |i: usize, images: Vec<Image>| {
        let img = images.first().unwrap();
        let mut result = new_surface(img.dimensions());
        let phase = phase + i as f32 * PI * 2.0 / frame as f32;
        let wave_shader = create_wave_effect(img, amplitude, frequency, phase);
        if let Err(e) = wave_shader {
            return Err(Error::MemeFeedback(format!("着色器错误：{:?}", e)));
        }
        let wave_shader = wave_shader.unwrap();
        result.canvas().draw_irect(
            IRect::from_size(img.dimensions()),
            Paint::default().set_shader(wave_shader),
        );
        Ok(result.image_snapshot())
    };

    if options.single.unwrap_or(false) {
        make_png_or_gif(images, single_func)
    } else {
        make_gif_or_combined_gif(
            images,
            muti_func,
            GifInfo {
                frame_num: frame - 1,
                duration: 0.02,
            },
            FrameAlign::NoExtend,
        )
    }
}

register_meme!(
    "wave_distortion",
    wave_distortion,
    min_images = 1,
    max_images = 1,
    keywords = &["波纹扭曲"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
