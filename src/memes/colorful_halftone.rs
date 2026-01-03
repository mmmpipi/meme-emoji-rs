use skia_safe::{Data, Image};

use meme_generator_core::{
    error::Error,
    meme::{self},
};
use meme_generator_utils::{
    builder::InputImage, encoder::{encode_png, make_png_or_gif}, image::ImageExt, tools::local_date
};

use crate::{
    memes::{
        halftone::{self, ColorfulHalftoneEffect},
        louvre::louvre,
    },
    options::{self},
    register_meme,
};

// 在渲染

fn colorful_halftone(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: ColorfulHalftoneEffect,
) -> Result<Vec<u8>, Error> {
    let name = &images.first().unwrap().name.clone();
    let func = |images: Vec<Image>| {
        let name = name.clone();
        let options = ColorfulHalftoneEffect { ..options };
        let image = images.first().unwrap();
        let data = encode_png(image.clone())?;
        let data: meme::Image = meme::Image {
            data,
            name: name.clone(),
        };
        let halftone = halftone::halftone(vec![InputImage::from(&data)?], vec![], options)?;
        let data: meme::Image = meme::Image {
            data: halftone,
            name,
        };
        let louvre = louvre(
            vec![InputImage::from(&data)?],
            vec![],
            options::Louvre::default(),
        )?;
        let data = Data::new_copy(&louvre);
        let image = Image::from_encoded(data);
        if image.is_none() {
            return Err(Error::MemeFeedback("encode png".to_owned()));
        }
        let image = image.unwrap();
        Ok(image)
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "colorful_halftone",
    colorful_halftone,
    min_images = 1,
    max_images = 1,
    keywords = &["炫彩打印机"],
    date_created = local_date(2026, 1, 4),
    date_modified = local_date(2026, 1, 4),
);
