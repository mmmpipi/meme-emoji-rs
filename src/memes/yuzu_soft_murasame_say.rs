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

fn yuzu_soft_murasame_say(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts.first().unwrap();
    let frame = load_image("yuzu_soft_murasame_say/0.jpg")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    canvas.draw_text_area_auto_font_size(
        IRect::new(922, 0, 1275, 659),
        text,
        20.0,
        150.0,
        text_params!(
            font_families = &["FZKaTong-M19S"],
            paint = new_paint(Color::BLACK),
            text_align = TextAlign::Center,
        ),
    )?;
    encode_png(surface.image_snapshot())
}

register_meme!(
    "yuzu_soft_murasame_say",
    yuzu_soft_murasame_say,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["非酋，不要再涩涩了"],
    keywords = &["丛雨黑板"],
    date_created = local_date(2024, 12, 21),
    date_modified = local_date(2024, 12, 21),
);
