use skia_safe::{Color, IRect, Image, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::make_png_or_gif,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint},
};

use crate::{options::NoOptions, register_meme};

fn yuzu_soft_shocked(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if let Some(text) = texts.first() {
        text
    } else {
        &format!("{name},你是柚...柚子厨?!")
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_shocked/0.png")?;
        let user_head = images[0].circle().resize_exact((33, 33));
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (0, 100), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(34, 100, 257, 133),
            text,
            20.0,
            70.0,
            text_params!(
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::End,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_shocked",
    yuzu_soft_shocked,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["震惊柚子厨"],
    date_created = local_date(2024, 7, 26),
    date_modified = local_date(2025, 5, 25),
);
