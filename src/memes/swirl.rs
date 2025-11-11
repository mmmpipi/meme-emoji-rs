use std::cell::LazyCell;

use skia_safe::{
    Data, IRect, Image, Paint, Point, RuntimeEffect, SamplingOptions, runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage, encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif, make_png_or_gif}, image::ImageExt, shortcut, tools::{local_date, new_surface}
};
use tween::Tweener;

use crate::{options::SwirlEffect, register_meme};

fn create_swirl_effect(
    image: &Image,
    center: impl Into<Point>, // 漩涡中心 (归一化坐标，范围 0-1)
    radius: f32,              // 漩涡半径 (归一化值)
    strength: f32,            // 漩涡强度
    phase: f32,               // 相位 (用于动画)
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    let center = center.into();
    const SHADER_SOURCE: &str = r#"
        uniform shader image;
        uniform vec2 center;
        uniform float radius;
        uniform float strength;
        uniform float phase;
        uniform vec2 image_size;

        vec4 main(vec2 coord) {
            // 归一化坐标
            vec2 uv = coord / image_size;
            
            // 计算到中心的向量和距离
            vec2 to_center = center - uv;
            float dist = length(to_center);
            
            // 计算扭曲效果
            float percent = (radius - dist) / radius;
            if (percent < 0.0) percent = 0.0;
            
            // 计算扭曲角度
            float angle = percent * percent * strength * 8.0 + phase;
            float cos_angle = cos(angle);
            float sin_angle = sin(angle);
            
            // 应用旋转
            vec2 rotated_coord;
            rotated_coord.x = cos_angle * to_center.x - sin_angle * to_center.y;
            rotated_coord.y = sin_angle * to_center.x + cos_angle * to_center.y;
            
            vec2 new_uv = center + rotated_coord;
            vec2 sample_coord = new_uv * image_size;
            
            return image.eval(sample_coord);
        }
    "#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 获取图像尺寸
    let image_size = image.dimensions();

    // 准备uniform数据
    let mut uniforms = vec![];

    // 添加漩涡中心 (vec2)
    uniforms.extend_from_slice(&center.x.to_le_bytes());
    uniforms.extend_from_slice(&center.y.to_le_bytes());

    // 添加漩涡半径
    uniforms.extend_from_slice(&radius.to_le_bytes());

    // 添加漩涡强度
    uniforms.extend_from_slice(&strength.to_le_bytes());

    // 添加相位
    uniforms.extend_from_slice(&phase.to_le_bytes());

    // 添加图像尺寸 (vec2)
    uniforms.extend_from_slice(&(image_size.width as f32).to_le_bytes());
    uniforms.extend_from_slice(&(image_size.height as f32).to_le_bytes());

    let image_shader = image
        .to_shader(None, SamplingOptions::default(), None)
        .expect("图像转shader错误");
    let data = Data::new_copy(&uniforms);
    // 构建漩涡着色器
    let children = vec![ChildPtr::Shader(image_shader)];
    let swirl_shader = effect
        .make_shader(data, &children, None)
        .ok_or("无法创建漩涡着色器")?;

    Ok(swirl_shader)
}

fn swirl(images: Vec<InputImage>, _: Vec<String>, options: SwirlEffect) -> Result<Vec<u8>, Error> {
    let strength = options.strength.unwrap_or(0.5);
    let radius = options.radius.unwrap_or(0.5);
    let phase = options.phase.unwrap_or(0.0);
    let time = options.time.unwrap_or(2.0);
    let empty_time = options.empty_time.unwrap_or(0.2);
    let duration = 0.02;
    let frame = (time / duration) as u32;
    let empty_frame = (empty_time / duration) as u32;
    let loopback = options.loopback.unwrap_or(false);

    let single_func = |images: Vec<Image>| {
        let img = images.first().unwrap().rotate(180.0);
        let mut result = new_surface(img.dimensions());
        let shader = create_swirl_effect(&img, (0.5, 0.5), radius, strength, phase);
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

    let mut strength_list = vec![];
    let mut tween = Tweener::expo_out(0.0, strength, frame - empty_frame);
    for i in 1..=frame - empty_frame {
        strength_list.push(tween.move_to(i));
    }

    let muti_func = |i: usize, images: Vec<Image>| {
        let frame = frame as usize;
        let i = if i < frame { i } else { frame * 2 - i - 1 };
        if i < empty_frame as usize {
            return Ok(images.first().unwrap().to_owned());
        }
        let img = images.first().unwrap().rotate(180.0);
        let mut result = new_surface(img.dimensions());
        let shader = create_swirl_effect(
            &img,
            (0.5, 0.5),
            radius,
            strength_list[i - empty_frame as usize],
            phase,
        );
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

    if options.single.unwrap_or(false) {
        make_png_or_gif(images, single_func)
    } else {
        make_gif_or_combined_gif(
            images,
            muti_func,
            GifInfo {
                frame_num: if loopback { frame * 2 } else { frame },
                duration,
            },
            FrameAlign::NoExtend,
        )
    }
}

register_meme!(
    "swirl",
    swirl,
    min_images = 1,
    max_images = 1,
    keywords = &["漩涡扭曲"],
    shortcuts = &[
        shortcut!(
            "柠檬酸",
            humanized = "柠檬酸",
            options = &[("time",1.0),("strength",0.8)]
        ),
    ],
    date_created = local_date(2025, 11, 11),
    date_modified = local_date(2025, 11, 11),
);
