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

fn yuzu_soft_murasame_finger(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("原来{name}是柚子厨")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_murasame_finger/0.png")?;
        let user_head = images[0].circle().resize_exact((430,430));
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (980,420), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(805, 94, 1728, 274),
            text,
            20.0,
            100.0,
            text_params!(
                font_families = &["FZKaTong-M19S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::End,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_murasame_finger",
    yuzu_soft_murasame_finger,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["柚子厨","丛雨指"],
    date_created = local_date(2024, 7, 26),
    date_modified = local_date(2025, 5, 25),
);
