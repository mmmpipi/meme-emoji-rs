use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::encode_png,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint},
};
use rand::Rng;
use skia_safe::{Color, IRect, textlayout::TextAlign};

use crate::{options::number_option, register_meme};

number_option!(Number, 1, 4);

fn anan_hs(
    _: Vec<InputImage>,
    texts: Vec<String>,
    options: Number,
) -> Result<Vec<u8>, Error> {
    let params = [(150, 470, 400, 600), (310, 740, 790, 990),(147, 810, 736, 1105),(179, 344, 464, 413)];
    let num = options.number.unwrap_or({
        let mut rng = rand::rng();
        rng.random_range(1..=4)
    })-1;

    let text = texts.first().unwrap();
    let frame = load_image(format!("anan_hs/{num}.jpg"))?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    let (l, t, r, b) = params[num as usize];
    canvas.draw_text_area_auto_font_size(
        IRect::new(l, t, r, b),
        text,
        5.0,
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
    "anan_hs",
    anan_hs,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["宝宝求你去看看医生吧\n吾辈没法同时做你的\n心理医生、妈妈\n最好的朋友、性玩具\n最坏的敌人和人生导师"],
    keywords = &["安安举牌", "夏目安安举牌"],
    date_created = local_date(2025, 10, 5),
    date_modified = local_date(2025, 10, 28),
);
