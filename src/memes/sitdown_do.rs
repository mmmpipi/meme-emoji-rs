use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn sitdown_do(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |i: usize, images: Vec<Image>| {
        let positions = [(180, 55), (180, 68), (181, 111)];
        let frame = load_image(format!("sitdown_do/{i}.png"))?;
        let mut surface = new_surface(frame.dimensions());
        let head = images.first().unwrap().resize_exact((215, 215));
        let canves = surface.canvas();
        canves.draw_image(head, positions[i], None);
        canves.draw_image(frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: 3,
            duration: 0.15,
        },
        FrameAlign::NoExtend,
    )
}

register_meme!(
    "sitdown_do",
    sitdown_do,
    min_images = 1,
    max_images = 1,
    keywords = &["坐撅"],
    date_created = local_date(2025, 8, 21),
    date_modified = local_date(2025, 9, 4),
);
