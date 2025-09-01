use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn torture_yourself(
    images: Vec<InputImage>,
    _: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let func = |images: Vec<Image>| {
        let frame = load_image("torture_yourself/0.png")?;
        let main_user_head = images[0].resize_exact((400, 400));
        let user_head = images[1].circle().resize_exact((200, 200));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&main_user_head, (70, 283), None);
        canvas.draw_image(&frame, (0, 0), None);
        canvas.draw_image(&user_head, (725, 780), None);
        Ok(surface.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "torture_yourself",
    torture_yourself,
    min_images = 2,
    max_images = 2,
    keywords = &["折磨自己"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
