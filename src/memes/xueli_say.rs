use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::encode_png,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint},
};
use skia_safe::{Color, IRect, textlayout::TextAlign};

use crate::{options::NoOptions, register_meme};

fn xueli_say(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts.first().unwrap();
    let frame = load_image("xueli_say/0.jpg")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    canvas.draw_text_area_auto_font_size(
        IRect::new(240, 30, 300, 240),
        text,
        10.0,
        120.0,
        text_params!(
            font_families = &["FZShaoEr-M11S"],
            paint = new_paint(Color::BLACK),
            text_align = TextAlign::Left,
        ),
    )?;
    encode_png(surface.image_snapshot())
}

register_meme!(
    "xueli_say",
    xueli_say,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["你是高手？"],
    keywords = &["雪莉说", "雪梨说", "橘雪莉说"],
    date_created = local_date(2025, 10, 5),
    date_modified = local_date(2025, 10, 5),
);
