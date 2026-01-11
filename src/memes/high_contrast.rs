use skia_safe::{
    HighContrastConfig, Image, high_contrast_config::InvertStyle, high_contrast_filter,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::local_date,
};

use crate::register_meme;

pub fn high_contrast(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: Options,
) -> Result<Vec<u8>, Error> {
    let grayscale = options.grayscale.unwrap_or(false);
    let invert_style = {
        let text = options.invert_style.unwrap_or("不反转".to_owned());
        match text.as_str() {
            "不反转" => InvertStyle::NoInvert,
            "反转亮度" => InvertStyle::InvertBrightness,
            "反转明度" => InvertStyle::InvertLightness,
            _ => unreachable!(),
        }
    };
    let contrast = options.contrast.unwrap_or(0.7);

    let single_func = |images: Vec<Image>| {
        let mut img = images.first().unwrap().to_owned();

        let filter =
            high_contrast_filter::new(&HighContrastConfig::new(grayscale, invert_style, contrast));

        if filter.is_none() {
            return Err(Error::MemeFeedback("".to_owned()));
        }

        img = img.color_filter(filter.unwrap());

        Ok(img)
    };

    make_png_or_gif(images, single_func)
}

#[derive(MemeOptions)]
pub(crate) struct Options {
    /// 黑白图片
    #[option(short,long,long_aliases = ["灰度"], default = false)]
    pub grayscale: Option<bool>,

    #[option(short, long,long_aliases = ["反转"], choices = ["不反转","反转亮度","反转明度"] , default = "不反转")]
    pub invert_style: Option<String>,

    #[option(short, long,long_aliases = ["对比度"], minimum = -1.0, maximum = 1.0, default = 0.7)]
    pub contrast: Option<f32>,
}

register_meme!(
    "high_contrast",
    high_contrast,
    min_images = 1,
    max_images = 1,
    keywords = &["高对比度"],
    date_created = local_date(2026, 1, 11),
    date_modified = local_date(2026, 1, 11),
);
