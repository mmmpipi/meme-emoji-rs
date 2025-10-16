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

fn yuzu_soft_ayachi_nene(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if let Some(text) = texts.first() {
        text
    } else {
        &format!("{name},这是你的照片吗？")
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_ayachi_nene/0.png")?;
        let user_head = images[0].resize_exact((165, 165)).rotate(45.0);
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.clear(Color::WHITE);
        canvas.draw_image(&user_head, (500, 410), None);
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(0, 0, 716, 128),
            text,
            20.0,
            150.0,
            text_params!(
                font_families = &["FZShaoEr-M11S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Left,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_ayachi_nene",
    yuzu_soft_ayachi_nene,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["宁宁困惑","绫地宁宁困惑"],
    date_created = local_date(2025, 3, 24),
    date_modified = local_date(2025, 3, 24),
);
