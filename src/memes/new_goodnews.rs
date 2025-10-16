use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::encode_png,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint, new_surface},
};
use skia_safe::{Color, IRect, textlayout::TextAlign};

use crate::{
    options::NoOptions,
    register_meme,
};

fn new_goodnews(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = if let Some(text) = texts.first() {
        text
    } else {
        &images.first().unwrap().name
    };
    let frame = load_image("new_goodnews/0.png")?;
    let user_head = images[0].image.circle().resize_exact((300, 300));
    let mut surface = new_surface(frame.dimensions());
    let canvas = surface.canvas();
    canvas.draw_image(user_head, (210, 270), None);
    canvas.draw_image(frame, (0,0), None);
    canvas.draw_text_area_auto_font_size(
        IRect::new(235, 615, 478, 659),
        text,
        10.0,
        100.0,
        text_params!(
            font_families = &["FZXS14"],
            paint = new_paint(Color::from_rgb(255, 215, 0)),
            text_align = TextAlign::Center,
        ),
    )?;
    if let Some(text) = texts.get(1){
        canvas.draw_text_area_auto_font_size(
        IRect::new(199, 708, 512, 923),
        text,
        10.0,
        60.0,
        text_params!(
            font_families = &["FZXS14"],
            paint = new_paint(Color::from_rgb(255, 215, 0)),
            text_align = TextAlign::Center,
        ),
    )?;
    }
    encode_png(surface.image_snapshot())
}

register_meme!(
    "new_goodnews",
    new_goodnews,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 2,
    default_texts = &["天命之人", "喜报传佳讯\n福星高照\n满门庭"],
    keywords = &["新喜报"],
    date_created = local_date(2025, 7, 26),
    date_modified = local_date(2025, 10, 3),
);
