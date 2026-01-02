use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn kurogames_phoebe_hug(
    images: Vec<InputImage>,
    _: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("kurogames_phoebe_hug/0.png")?;
        let user_head = images[0].resize_exact((96, 78));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (87, 218), None);
        canvas.draw_image(frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };
    make_png_or_gif(images, func)
}

register_meme!(
    "kurogames_phoebe_hug",
    kurogames_phoebe_hug,
    min_images = 1,
    max_images = 1,
    keywords = &["菲比抱"],
    date_created = local_date(2025, 12, 1),
    date_modified = local_date(2025, 12, 1),
);
