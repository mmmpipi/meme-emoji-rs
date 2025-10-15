use skia_safe::{Color, IRect, Image, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage, canvas::CanvasExt, encoder::make_png_or_gif, image::ImageExt, text_params, tools::{load_image, local_date, new_paint, new_surface}
};

use crate::{options::NoOptions, register_meme};

fn mihoyo_sigewinne_fingered(
    images: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let name = &images[0].name;
    let text = if texts.is_empty() {
        &format!("希格雯:这{name}没救了\n希格雯:快拉去璃月往生堂\n希格雯:让胡堂主埋了吧")
    } else {
        texts.first().unwrap()
    };
    let func = |images: Vec<Image>| {
        let frame = load_image("mihoyo_sigewinne_fingered/0.png")?;
        let user_head = images[0].resize_exact((144, 144));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, (12, 47), None);
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_text_area_auto_font_size(
            IRect::new(1, 345, 351, 435),
            text,
            15.0,
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
    "mihoyo_sigewinne_fingered",
    mihoyo_sigewinne_fingered,
    min_images = 1,
    max_images = 1,
    min_texts=0,
    max_texts=1,
    keywords = &["没救了", "希格雯指"],
    date_created = local_date(2025, 9, 13),
    date_modified = local_date(2025, 9, 13),
);
