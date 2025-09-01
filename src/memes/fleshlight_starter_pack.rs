use skia_safe::{Color, IRect, Image, textlayout::TextAlign};

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

fn fleshlight_starter_pack(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if let Some(text) = texts.first() {
        text
    } else {
        &format!("{name}の❤️新手礼包")
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("fleshlight_starter_pack/0.png")?;
        let user_head = images[0].circle().resize_exact((580, 580));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (15, 185), None);
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(93, 688, 493, 769),
            text,
            20.0,
            100.0,
            text_params!(
                font_families = &["FZShaoEr-M11S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Center,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "fleshlight_starter_pack",
    fleshlight_starter_pack,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["新手礼包"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
