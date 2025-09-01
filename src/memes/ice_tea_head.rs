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

fn ice_tea_head(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = if texts.is_empty() {
        ""
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("ice_tea_head/0.png")?;
        let user_head = images[0].circle().resize_exact((500, 500));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(235, 615, 478, 659),
            text,
            20.0,
            100.0,
            text_params!(
                font_families = &["FZShaoEr-M11S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Start,
            ),
        )?;
        canvas.draw_image(&user_head, (320, 35), None);
        Ok(surface.image_snapshot())
    };
    make_png_or_gif(images, func)
}

register_meme!(
    "ice_tea_head",
    ice_tea_head,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["冰红茶", "牢","man!","man！"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
