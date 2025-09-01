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

fn doro_contact(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("桃乐丝:{name}同学,这是你的头像照片吗？\n桃乐丝:你长得好帅呀\n桃乐丝:{name},你愿意和我交往在一起吗？\n{name}:我愿意,我愿意\n{name}:桃乐丝,我{name}愿意和你在一起一生一世")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("doro_contact/0.png")?;
        let user_head = images[0].resize_exact((108, 108));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (145, 278), None);
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(1, 1, 677, 207),
            text,
            15.0,
            60.0,
            text_params!(
                font_families = &["FZShaoEr-M11S"],
                paint = new_paint(Color::BLACK),
                text_align = TextAlign::Center,
            ),
        )?;
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "doro_contact",
    doro_contact,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["交往","doro交往","Doro交往","DORO交往","桃乐丝交往"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
