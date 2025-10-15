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

fn doro_surrounding_photos(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("桃乐丝:我这边里有Doro动漫周边公仔\n桃乐丝:还有{name}的写真~\n桃乐丝:你要吗？")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("doro_surrounding_photos/0.png")?;
        let user_head = &images[0];
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(user_head.resize_exact((123, 109)), (860, 1095), None);
        canvas.draw_image(user_head.resize_exact((68, 68)), (864, 1227), None);
        canvas.draw_image(
            user_head.resize_exact((100, 100)).rotate(15.0),
            (815, 1296),
            None,
        );
        canvas.draw_image(frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(1, 1, 1080, 250),
            text,
            30.0,
            60.0,
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
    "doro_surrounding_photos",
    doro_surrounding_photos,
    min_images = 1,
    max_images = 1,
    min_texts = 0,
    max_texts = 1,
    keywords = &["周边写真"],
    date_created = local_date(2025, 9, 13),
    date_modified = local_date(2025, 9, 13),
);
