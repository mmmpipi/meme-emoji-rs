use skia_safe::Image;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{FrameAlign, GifInfo, make_gif_or_combined_gif},
    image::{Fit, ImageExt},
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

fn behind_do(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let func = |i: usize, images: Vec<Image>| {
        let self_locs = [
            (57, -10),
            (57, -10),
            (56, -13),
            (55, -12),
            (51, -10),
            (51, -10),
            (51, -10),
            (46, -8),
            (46, -8),
            (54, -10),
        ];

        let user_locs = [
            (174, 91),
            (174, 91),
            (173, 86),
            (171, 87),
            (170, 85),
            (170, 85),
            (167, 82),
            (170, 85),
            (170, 85),
            (172, 88),
        ];
        let frame = load_image(format!("behind_do/{i}.png"))?;
        let mut surface = new_surface(frame.dimensions());
        let head = images
            .first()
            .unwrap()
            .resize_fit((110, 110), Fit::Cover)
            .circle()
            .rotate(-15.0);
        let user = images[1].resize_fit((116, 116), Fit::Cover).circle();
        let canves = surface.canvas();
        canves.draw_image(user, user_locs[i], None);
        canves.draw_image(frame, (0, 0), None);
        canves.draw_image(head, self_locs[i], None);
        Ok(surface.image_snapshot())
    };

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: 10,
            duration: 0.07,
        },
        FrameAlign::NoExtend,
    )
}

register_meme!(
    "behind_do",
    behind_do,
    min_images = 2,
    max_images = 2,
    keywords = &["后撅"],
    date_created = local_date(2025, 12, 6),
    date_modified = local_date(2025, 12, 6),
);
