use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    image::ImageExt,
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn laydown_do(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |i: usize, images: Vec<Image>| {
        let positions = [
            (135, 18),
            (135, 18),
            (136, 33),
            (136, 33),
            (136, 33), // 1-5
            (133, 61),
            (133, 61),
            (133, 61),
            (138, 26),
            (138, 26), // 5-10
            (138, 26), // 11
        ];
        let frame = load_image(format!("laydown_do/{i}.png"))?;
        let mut surface = new_surface(frame.dimensions());
        let head = images.first().unwrap().resize_exact((110, 110));
        let canves = surface.canvas();
        canves.draw_image(head, positions[i], None);
        canves.draw_image(frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: 11,
            duration: 0.03,
        },
        FrameAlign::NoExtend,
    )
}

register_meme!(
    "laydown_do",
    laydown_do,
    min_images = 1,
    max_images = 1,
    keywords = &["躺撅"],
    date_created = local_date(2025, 8, 21),
    date_modified = local_date(2025, 8, 21),
);
