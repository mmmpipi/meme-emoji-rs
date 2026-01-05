use std::cell::LazyCell;

use skia_safe::{
    Data, IRect, Image, Paint, RuntimeEffect,
    SamplingOptions, Size, runtime_effect::ChildPtr,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    tools::{local_date, new_surface},
};

use crate::register_meme;

fn create_error_effect(
    image: &Image,
    size: impl Into<Size>,
    time: f32,
    intensity: f32,
) -> Result<skia_safe::Shader, Box<dyn std::error::Error>> {
    let size = size.into();
    const SHADER_SOURCE: &str = r#"
    uniform shader image;
    uniform float2 iResolution;
    uniform float iTime;
    uniform float intensity;
    
    float random(vec2 st) {
        return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
    }
    
    vec4 main(vec2 fragCoord) {
        vec2 uv = fragCoord / iResolution;
        float t = iTime * 5.0 * 100.0;
        
        // 随时间变化的强度波动
        float pulse = 0.5 + 0.5 * sin(t * 0.5);
        float dynamicIntensity = intensity * (0.7 + 0.3 * pulse);
        
        vec4 color = image.eval(fragCoord);

        // 周期性大故障
        float bigGlitch = step(0.9, sin(t * 0.3)) * 0.8;
        float totalIntensity = dynamicIntensity + bigGlitch;
        
        
        // 动态通道偏移
        float channelOffset = totalIntensity * 0.015;
        float offsetX = sin(uv.y * 25.0 + t * 3.0) * channelOffset;
        offsetX += cos(uv.y * 18.0 + t * 2.0) * channelOffset * 0.6;
        
        vec2 offsetUV = uv + vec2(offsetX, 0.0);
        offsetUV.x = fract(offsetUV.x);
        
        vec4 offsetColor = image.eval(offsetUV * iResolution);
        
        // 通道分离（随时间变化强度）
        float sepAmount = totalIntensity * 0.01;
        vec4 redSep = image.eval((offsetUV + vec2(sepAmount, 0.0)) * iResolution);
        vec4 blueSep = image.eval((offsetUV - vec2(sepAmount * 0.8, 0.0)) * iResolution);
        
        color.r = mix(color.r, redSep.r, totalIntensity * 0.8);
        color.b = mix(color.b, blueSep.b, totalIntensity * 0.8);
        color.g = offsetColor.g;
        
        // 动态扫描线
        float scanFreq = 500.0 + sin(t * 0.7) * 200.0;
        float scanline = sin(uv.y * scanFreq + t * 8.0) * 0.1 * totalIntensity;
        color.rgb += vec3(scanline);
        
        // 闪烁效果
        float flicker = random(vec2(t * 0.2, 0.0));
        if (flicker < 0.1 * totalIntensity) {
            color.rgb *= (1.0 + flicker * 5.0);
        }
        
        return color;
    }
    "#;

    let effect = LazyCell::new(|| {
        RuntimeEffect::make_for_shader(SHADER_SOURCE, None).expect("着色器编译失败")
    });

    // 准备uniform数据
    let mut uniforms = vec![];
    uniforms.extend_from_slice(&size.width.to_le_bytes());
    uniforms.extend_from_slice(&size.height.to_le_bytes());
    uniforms.extend_from_slice(&time.to_le_bytes());
    uniforms.extend_from_slice(&intensity.to_le_bytes());

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

pub fn error_meme(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: ErrorMemeEffect,
) -> Result<Vec<u8>, Error> {
    //let base_color: Color = Color::from_argb(155, 0, 0, 0);
    let gray_threshold: f32 = options.intensity.unwrap_or(1.0);
    let frame = options.frame.unwrap_or(30) as u32;
    let size_set_1 = options.size.unwrap_or(false);
    let func = |i: usize, images: Vec<Image>| {
        let time = i as f32 / frame as f32;
        let img = images.first().unwrap().to_owned();

        let mut result = new_surface(img.dimensions());
        let size = if size_set_1{
            Size::new(1.0,1.0)
        }else{
            img.dimensions().into()
        };
        let shader = create_error_effect(&img, size, time, gray_threshold);
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
pub(crate) struct ErrorMemeEffect {
    /// 帧数
    #[option(short, long,long_aliases = ["帧数"], minimum = 0, maximum = 100, default = 30)]
    pub frame: Option<i32>,

    /// 强度
    #[option(short, long,long_aliases = ["强度"], minimum = -100.0, maximum = 100.0, default = 1.0)]
    pub intensity: Option<f32>,

    /// 强制将size设置为1,故障+故障
    #[option(short, long,long_aliases = ["大小设1"], default = false)]
    pub size: Option<bool>,
}

register_meme!(
    "error_meme",
    error_meme,
    min_images = 1,
    max_images = 1,
    keywords = &["故障"],
    date_created = local_date(2026, 1, 5),
    date_modified = local_date(2026, 1, 5),
);
