use skia_safe::{textlayout::TextAlign, Color, IRect, Image};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::make_png_or_gif,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn yes(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("{name} YES!👍🏻👍🏻")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yes/0.png")?;
        let user_head = images[0].resize_exact((165, 165));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head,  (396, 452), None);
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(295, 817, 980, 936),
            text,
            15.0,
            120.0,
            text_params!(
                font_families = &["FZXS14"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Center,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yes",
    yes,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["yes","Yes","YES"],
    date_created = local_date(2025, 8, 19),
    date_modified = local_date(2025, 9, 8),
);
