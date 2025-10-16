use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage, encoder::encode_png, image::{Fit, ImageExt}, tools::{load_image, local_date, new_surface}
};

use crate::{options::NoOptions, register_meme};

fn xueli_think(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let frame = load_image("xueli_think/0.png")?;
    let input_img = &images.first().unwrap().image;

    let final_img = input_img.resize_fit((824,824), Fit::Cover);

    let mut surface = new_surface(frame.dimensions());
    let canvas = surface.canvas();
    canvas.draw_image(final_img, (0, 0), None);
    canvas.draw_image(frame, (0, 0), None);
    encode_png(surface.image_snapshot())
}

register_meme!(
    "xueli_think",
    xueli_think,
    min_images = 1,
    max_images = 1,
    keywords = &["雪莉想", "雪梨想", "橘雪莉想"],
    date_created = local_date(2025, 10, 5),
    date_modified = local_date(2025, 10, 5),
);
