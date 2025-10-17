use skia_safe::{Color, IRect, Image, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::make_png_or_gif,
    image::{Fit, ImageExt},
    text_params,
    tools::{load_image, local_date, new_paint, new_stroke_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

const DEFAULT_TEXT: &str = "丛雨讨厌这个";

fn yuzu_soft_murasame_dislike(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = if let Some(text) = texts.first() {
        text
    } else {
        DEFAULT_TEXT
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_murasame_dislike/0.png")?;
        let user_head = images[0].resize_fit((305, 235), Fit::Cover);
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (106, 72), None);
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(
                5,
                frame.height() - 60,
                frame.width() - 5,
                frame.height() - 10,
            ),
            text,
            20.0,
            40.0,
            text_params!(
                paint = new_paint(Color::WHITE),
                stroke_paint = new_stroke_paint(Color::BLACK, 4.0),
                text_align = TextAlign::Center,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_murasame_dislike",
    yuzu_soft_murasame_dislike,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    default_texts = &[DEFAULT_TEXT],
    keywords = &["丛雨讨厌"],
    date_created = local_date(2025, 5, 25),
    date_modified = local_date(2025, 5, 25),
);
