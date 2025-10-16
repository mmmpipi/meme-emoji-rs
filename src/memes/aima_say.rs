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

number_option!(Number, 1, 2);

fn aima_say(
    _: Vec<InputImage>,
    texts: Vec<String>,
    options: Number,
) -> Result<Vec<u8>, Error> {
    let params = [(40, 35, 140, 210), (360, 10, 490, 200)];
    let num = options.number.unwrap_or({
        let mut rng = rand::rng();
        rng.random_range(1..=2)
    })-1;

    let text = texts.first().unwrap();
    let frame = load_image(format!("aima_say/{num}.jpg"))?;
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
    "aima_say",
    aima_say,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["何意味"],
    keywords = &["艾玛说", "樱羽艾玛说"],
    date_created = local_date(2025, 10, 5),
    date_modified = local_date(2025, 10, 5),
);
