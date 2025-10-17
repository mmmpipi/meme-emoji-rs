use skia_safe::{Color, Image};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::{Fit, ImageExt},
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn yuzu_soft_murasame_ipad(
    images: Vec<InputImage>,
    _: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_murasame_ipad/0.png")?;
        let user_head = images[0]
            .resize_fit((520, 350), Fit::Cover)
            .rotate(3.0);
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.clear(Color::BLACK);
        canvas.draw_image(user_head, (280, 475), None);
        canvas.draw_image(frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_murasame_ipad",
    yuzu_soft_murasame_ipad,
    min_images = 1,
    max_images = 1,
    keywords = &["丛雨平板"],
    date_created = local_date(2025, 6, 20),
    date_modified = local_date(2025, 6, 20),
);
