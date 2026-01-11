use skia_safe::{FilterMode, Image, MipmapMode, Rect, SamplingOptions, image_filters};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::{InputImage, MemeOptions},
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::local_date,
};

use crate::register_meme;

pub fn fisheye(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: Options,
) -> Result<Vec<u8>, Error> {
    let zoom: f32 = options.zoom.unwrap_or(1.3);
    let inset: f32 = options.inset.unwrap_or(128.0);

    let single_func = |images: Vec<Image>| {
        let mut img = images.first().unwrap().to_owned();

        let rect = Rect::from_isize(img.dimensions());

        let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Nearest);

        let filter = image_filters::magnifier(rect, zoom, inset, sampling, None, None);

        if filter.is_none() {
            return Err(Error::MemeFeedback("error".to_owned()));
        }
        let filter = filter.unwrap();
        for _ in 0..options.time.unwrap_or(1) {
            img = img.image_filter(filter.clone())
        }
        Ok(img)
    };

    make_png_or_gif(images, single_func)
}

#[derive(MemeOptions)]
pub(crate) struct Options {
    #[option(short,long,long_aliases = ["缩放"], minimum = 0.0, maximum = 100.0, default = 1.3)]
    pub zoom: Option<f32>,

    #[option(short, long,long_aliases = ["边缘"], minimum = 0.0, maximum = 1000.0, default = 128.0)]
    pub inset: Option<f32>,

    #[option(short, long,long_aliases = ["次数"], minimum = 0, maximum = 100, default = 1)]
    pub time: Option<i32>,
}

register_meme!(
    "fisheye",
    fisheye,
    min_images = 1,
    max_images = 1,
    keywords = &["鱼眼"],
    date_created = local_date(2026, 1, 6),
    date_modified = local_date(2026, 1, 6),
);
