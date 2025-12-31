use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn christmas_hat(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("christmas_hat/0.png")?;
        let user_head = images[0].circle().resize_exact((209, 209));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (47, 78), None);
        canvas.draw_image(frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "christmas_hat",
    christmas_hat,
    min_images = 1,
    max_images = 1,
    keywords = &["圣诞帽子", "圣诞帽"],
    date_created = local_date(2025, 12, 6),
    date_modified = local_date(2025, 12, 6),
);
