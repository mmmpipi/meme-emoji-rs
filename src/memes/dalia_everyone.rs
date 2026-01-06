//来自meme_emoji
use skia_safe::{textlayout::TextAlign, Color, IRect, Image};

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

fn dalia_everyone(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("所有人，给我{name}生成黄图")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("dalia_everyone/0.png")?;
        let user_head = images[0].resize_exact((95, 95)).rotate(5.0);
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head,  (611, 387), None);
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(211, 0, 1060, 132),
            text,
            15.0,
            60.0,
            text_params!(
                font_families = &["FZXS14"],
                paint = new_paint(Color::WHITE),
                text_align = TextAlign::Left,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "dalia_everyone",
    dalia_everyone,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["所有人"],
    date_created = local_date(2025, 12, 1),
    date_modified = local_date(2025, 12, 2),
);
