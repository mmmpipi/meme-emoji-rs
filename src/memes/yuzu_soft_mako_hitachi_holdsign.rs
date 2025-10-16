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

fn yuzu_soft_mako_hitachi_holdsign(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts.first().unwrap();
    let frame = load_image("yuzu_soft_mako_hitachi_holdsign/0.png")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    canvas.draw_text_area_auto_font_size(
        IRect::new(50, 309, 474, 598),
        text,
        25.0,
        120.0,
        text_params!(
            font_families = &["FZSJ-QINGCRJ"],
            paint = new_paint(Color::BLACK),
            text_align = TextAlign::Center,
        ),
    )?;
    encode_png(surface.image_snapshot())
}

register_meme!(
    "yuzu_soft_mako_hitachi_holdsign",
    yuzu_soft_mako_hitachi_holdsign,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["Ciallo～"],
    keywords = &["常陸茉子举牌","茉子举牌","常陆茉子举牌"],
    date_created = local_date(2025, 5, 17),
    date_modified = local_date(2025, 5, 17),
);
