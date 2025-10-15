use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::encode_png,
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn xueli_think(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    const SCREEN_WIDTH: i32 = 824;
    const SCREEN_HEIGHT: i32 = 824;
    const SCREEN_ASPECT_RATIO: f32 = (SCREEN_WIDTH as f32) / (SCREEN_HEIGHT as f32);
    let frame = load_image("xueli_think/0.png")?;
    let input_img = &images.first().unwrap().image;
    let img_aspect_ratio = input_img.width() as f32 / input_img.height() as f32;

    let final_img = if img_aspect_ratio > SCREEN_ASPECT_RATIO {
        let new_height = SCREEN_HEIGHT as f32;
        let new_width = new_height * img_aspect_ratio;

        let resized_img = input_img.resize_exact((new_width as i32, new_height as i32));

        let left_crop = (new_width - (SCREEN_WIDTH as f32)) / 2.0;

        let mut surface = new_surface((new_width as i32, new_height as i32));

        let canvas = surface.canvas();

        canvas.draw_image(resized_img, (-left_crop as i32, 0), None);

        surface.image_snapshot()
    } else {
        let new_width = SCREEN_WIDTH as f32;
        let new_height = new_width / img_aspect_ratio;
        let resized_img = input_img.resize_exact((new_width as i32, new_height as i32));

        let top_crop = (new_height - (SCREEN_HEIGHT as f32)) / 2.0;

        let mut surface = new_surface((new_width as i32, new_height as i32));

        let canvas = surface.canvas();

        canvas.draw_image(resized_img, (0, -top_crop as i32), None);

        surface.image_snapshot()
    };

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
