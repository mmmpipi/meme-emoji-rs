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

fn yuzu_soft_murasame_clothes(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("{name},你好变态,穿我的衣物")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("yuzu_soft_murasame_clothes/0.jpg")?;
        let user_head = images[0].circle().resize_exact((140, 140));
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (700, 80), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(0, 0, 130, 339),
            text,
            25.0,
            100.0,
            text_params!(
                font_families = &["FZKaTong-M19S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Center,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "yuzu_soft_murasame_clothes",
    yuzu_soft_murasame_clothes,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["丛雨衣服", "丛雨衣物"],
    date_created = local_date(2025, 3, 24),
    date_modified = local_date(2025, 3, 24),
);
