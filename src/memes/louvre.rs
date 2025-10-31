use skia_safe::{Color, ColorFilter, ColorMatrix, HSV, IPoint, Image, TileMode, image_filters};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{local_date, new_paint, new_surface},
};

use crate::{options::GrayStandard, register_meme};

fn louvre(images: Vec<InputImage>, _: Vec<String>, options: GrayStandard) -> Result<Vec<u8>, Error> {
    let gray_standard = options.gray_standard.unwrap_or_else(|| {"rec601_gray".to_string()});
    let gain = options.gain.unwrap_or(2.0);
    let bias = options.bias.unwrap_or(0.0);
    let limit = options.limit.unwrap_or(20) as u8;
    let func = |images: Vec<Image>| {
        
        #[rustfmt::skip]
        let luma_matrix = ColorMatrix::new(
            0.299, 0.587, 0.114, 0.0, 0.0,
            0.299, 0.587, 0.114, 0.0, 0.0,
            0.299, 0.587, 0.114, 0.0, 0.0,
            0.0,   0.0,   0.0,   1.0, 0.0
        );

        let luma_filter = ColorFilter::luma();
        let Some(blur_filter) = image_filters::blur((0.5, 0.5), TileMode::Clamp, None, None) else {
            return Err(Error::MemeFeedback(
                "blur filter create Failed!".to_string(),
            ));
        };
        #[rustfmt::skip]
        let out = -1.0;
        let sobel_x: [f32; 9] = 
        [
            out, out, out, 
            out, 8.0, out,
            out, out, out
        ];
        let kernel_offset = IPoint::new(1, 1); 
        // 通常卷积核偏移为其尺寸的一半减一

        let Some(edge_filter) = image_filters::matrix_convolution(
            (3,3),
            &sobel_x,
            gain,
            bias,
            kernel_offset,
            TileMode::Clamp,
            true, // convolve_alpha，是否对alpha通道进行卷积
            None,
            None
        ) else {
            return Err(Error::MemeFeedback(
                "edge filter create Failed!".to_string(),
            ));
        };
        #[rustfmt::skip]
        let matrix = ColorMatrix::new(
            2.0, 0.0, 0.0, 0.0, 0.0, // R' = 2*R + 0*G + 0*B + 0*A + 0
            0.0, 2.0, 0.0, 0.0, 0.0, // G' = 0*R + 2*G + 0*B + 0*A + 0
            0.0, 0.0, 2.0, 0.0, 0.0, // B' = 0*R + 0*G + 2*B + 0*A + 0
            0.0, 0.0, 0.0, 1.0, 0.0  // A' = 0*R + 0*G + 0*B + 1*A + 0
        );

        let img = images
            .first()
            .unwrap();
        
        let img = if options.resize.unwrap_or(false){
            &img.resize_exact((500,500))
        }else{img};

        let img = img.image_filter(blur_filter);

        let img = 
        match gray_standard.as_str(){
            "rec709"=>img.color_filter(luma_filter),
            "rec601"=>img.color_matrix(luma_matrix),
            "rec601_gray"=>img.color_matrix(luma_matrix).color_filter(luma_filter),
            "rec601_noprocess"=>{return Ok(img.color_matrix(luma_matrix));},
            _ => return Err(Error::MemeFeedback("not support gray".to_string()))
        }
        .image_filter(edge_filter);
        if options.no_postprocess.unwrap_or(false){
            return Ok(img);
        }
        let img = img.color_matrix(matrix);
        let mut bg = new_surface(img.dimensions());
        bg.canvas().clear(Color::WHITE);

        let Some(data) = img.peek_pixels() else {
            return Err(Error::MemeFeedback(
                "cannot decode image!".to_string(),
            ));
        };

        let width = img.width() as f32;
        let height = img.height() as f32;
        let canvas = bg.canvas();

        for y in 0..img.height() {
            for x in 0..img.width() {
                let alpha = data.get_alpha_f((x,y));
                let color = data.get_color((x,y));
                if color.a()<limit{
                    continue;
                }
                let gradient_factor = (x + y) as f32 / (width + height);

                let hue = 0.05 + 0.65 * gradient_factor;
                let hue = hue.clamp(0.0, 1.0);
                let saturation: f32 = 0.9;
                let brightness = 0.8 + 0.1 * (1.0 - gradient_factor);
                let brightness = brightness.clamp(0.0, 1.0);
                let alpha = (alpha*2.0).min(1.0);
                let color = HSV::from((hue*360.0,saturation,brightness)).to_color((alpha*255.0) as u8);
                canvas.draw_point((x,y), &new_paint(color));
            }
        };
        Ok(bg.image_snapshot())
    };

    make_png_or_gif(images, func)
}

register_meme!(
    "louvre",
    louvre,
    min_images = 1,
    max_images = 1,
    keywords = &["卢浮宫"],
    date_created = local_date(2025, 8, 10),
    date_modified = local_date(2025, 8, 10),
);
