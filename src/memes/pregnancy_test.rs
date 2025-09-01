use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn pregnancy_test(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("pregnancy_test/0.png")?;
        let user_head = images[0].resize_exact((640, 640));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (1, 1), None);
        canvas.draw_image(&frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "pregnancy_test",
    pregnancy_test,
    min_images = 1,
    max_images = 1,
    keywords = &["验孕棒"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
