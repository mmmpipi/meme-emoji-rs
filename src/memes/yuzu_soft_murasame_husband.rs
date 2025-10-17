use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{load_image, local_date},
};

use crate::{options::NoOptions, register_meme};

fn yuzu_soft_murasame_husband(
    images: Vec<InputImage>,
    _: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_murasame_husband/0.png")?;
        let user_head = images[0].circle().resize_exact((200, 200));
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (160, 40), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_murasame_husband",
    yuzu_soft_murasame_husband,
    min_images = 1,
    max_images = 1,
    keywords = &["柚子厨", "丛雨指"],
    date_created = local_date(2024, 7, 26),
    date_modified = local_date(2025, 5, 25),
);
