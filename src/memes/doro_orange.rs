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

fn doro_orange(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("桃乐丝:和{name}一起品尝欧润吉真是一种享受\n \n{name}:欧润吉真好吃")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("doro_orange/0.png")?;
        let user_head = images[0].circle().resize_exact((270, 270));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head,  (588, 65), None);
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(1, 585, 949, 792),
            text,
            20.0,
            100.0,
            text_params!(
                font_families = &["FZXS14"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Left,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "doro_orange",
    doro_orange,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["欧润吉","润吉","润橘","橘子","橘","🍊"],
    date_created = local_date(2025, 9, 1),
    date_modified = local_date(2025, 9, 1),
);
