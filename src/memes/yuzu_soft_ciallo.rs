use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage, encoder::make_png_or_gif, image::ImageExt, shortcut, tools::{load_image, local_date, new_surface}
};

use crate::{options::NoOptions, register_meme};

fn yuzu_soft_ciallo(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_ciallo/0.png")?;
        let user_head = images[0].resize_exact((355, 295));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (364, 363), None);
        canvas.draw_image(&frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_ciallo",
    yuzu_soft_ciallo,
    min_images = 1,
    max_images = 1,
    keywords = &["ciallo~"],
    shortcuts = &[
        shortcut!(
            "[cC][iI][aA][lL]{2}[oO]",
            humanized = "Ciallo"
        ),
        shortcut!(
            "[cC][iI][aA][lL]{2}[oO]～\\(∠・ω< \\)⌒[★☆]",
            humanized = "Ciallo～(∠・ω< )⌒★"
        ),
    ],
    date_created = local_date(2025, 9, 5),
    date_modified = local_date(2025, 9, 25),
);
