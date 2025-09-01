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

fn doro_dear(images: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("桃乐丝:{name},你永远是桃乐丝的最爱之人❤️")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("doro_dear/0.png")?;
        let user_head = images[0].circle().resize_exact((100, 100));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head, (576, 529), None);
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(1, 675, 1200, 812),
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
    "doro_dear",
    doro_dear,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["最爱","doro最爱","Doro最爱","DORO最爱","桃乐丝最爱"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
