use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn doro_knight(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let params = [
        (126, 203),
        (125, 197),
        (125, 204),
        (125, 209),
        (128, 197),
        (125, 200),
        (124, 207),
        (125, 201),
        (125, 201),
        (125, 207),
        (125, 209),
        (126, 201),
        (125, 205),
        (125, 198),
        (127, 197),
        (127, 206),
    ];

    let func = |i: usize, images: Vec<Image>| {
        let frame = load_image(format!("doro_knight/{i}.png"))?;
        let user_head = images[0].circle().resize_exact((133, 133));
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.draw_image(&user_head, params[i], None);
        canvas.draw_image(&frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: 16,
            duration: 0.1,
        },
        FrameAlign::ExtendLoop,
    )
}

register_meme!(
    "doro_knight",
    doro_knight,
    min_images = 1,
    max_images = 1,
    keywords = &["骑士", "doro骑士", "Doro骑士", "DORO骑士"],
    date_created = local_date(2025, 9, 13),
    date_modified = local_date(2025, 9, 13),
);
