use skia_safe::{
    BlendMode, Color, ColorMatrix, IPoint, IRect, ISize, Image, ImageFilter, Paint, Point, Shader, TileMode, gradient_shader::{self, GradientShaderColors}, image_filters::{self, CropRect}, scalar
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::make_png_or_gif,
    image::ImageExt,
    tools::{local_date, new_paint, new_surface},
};

use crate::{options::GrayStandard, register_meme};

fn create_convolute_average(width: usize) -> Vec<f32> {
    let value = 1.0 / (width * width) as f32;
    vec![value; width * width]
}

fn simple_convolute_average_builder(width: usize) -> MatrixConvolutionBuilder {
    let width_int = width as i32;
    let mut builder = MatrixConvolutionBuilder::new(
        (width_int, width_int),
        create_convolute_average(width),
        ((width_int - 1) / 2, (width_int - 1) / 2),
    );
    builder.convolve_alpha = false;
    builder
}

#[derive(Clone)]
struct MatrixConvolutionBuilder {
    pub kernel_size: ISize,
    pub kernel: Vec<scalar>,
    pub gain: scalar,
    pub bias: scalar,
    pub kernel_offset: IPoint,
    pub tile_mode: TileMode,
    pub convolve_alpha: bool,
    pub input: Option<ImageFilter>,
    pub crop_rect: CropRect,
}

impl MatrixConvolutionBuilder {
    fn new(
        kernel_size: impl Into<ISize>,
        kernel: impl Into<Vec<scalar>>,
        kernel_offset: impl Into<IPoint>,
    ) -> Self {
        Self {
            kernel_size: kernel_size.into(),
            kernel: kernel.into(),
            gain: 1.0,
            bias: 0.0,
            kernel_offset: kernel_offset.into(),
            tile_mode: TileMode::Clamp,
            convolve_alpha: true,
            input: None,
            crop_rect: None.into(),
        }
    }
    fn build(self) -> Option<ImageFilter> {
        image_filters::matrix_convolution(
            self.kernel_size,
            &self.kernel.to_vec(),
            self.gain,
            self.bias,
            self.kernel_offset,
            self.tile_mode,
            self.convolve_alpha, // convolve_alpha，是否对alpha通道进行卷积
            self.input,
            self.crop_rect,
        )
    }
}

fn create_kiss_gradient(isize: impl Into<ISize>) -> Option<Shader> {
    let colors = [
        Color::from_rgb(251, 186, 48), // #fbba30
        Color::from_rgb(252, 114, 53), // #fc7235
        Color::from_rgb(252, 53, 78),  // #fc354e
        Color::from_rgb(207, 54, 223), // #cf36df
        Color::from_rgb(55, 181, 217), // #37b5d9
        Color::from_rgb(62, 182, 218), // #3eb6da
    ];
    let isize = isize.into();
    gradient_shader::linear(
        (
            Point::new(0.0, 0.0),
            Point::new(isize.width as f32, isize.height as f32),
        ),
        GradientShaderColors::Colors(&colors),
        Some(&[0.0_f32, 0.4, 0.6, 0.7, 0.8, 1.0] as &[scalar]),
        TileMode::Clamp,
        None,
        None,
    )
}

// 在渲染

fn louvre(
    images: Vec<InputImage>,
    _: Vec<String>,
    options: GrayStandard,
) -> Result<Vec<u8>, Error> {
    let binding = options.convolute.unwrap_or("一般".to_string());
    let convolute = binding.as_str();
    let gain = options.gain.unwrap_or(2.0);
    let bias = options.bias.unwrap_or(0.0);
    let light_cut = 128.0;
    let dark_cut = options.dark_cut.unwrap_or(118.0);
    let mut diff = !options.no_diff.unwrap_or(false);
    let kuma = !options.gray.unwrap_or(false);
    let full_bg = !options.no_bg.unwrap_or(false);
    let func = |images: Vec<Image>| {
        let luma_matrix = ColorMatrix::new(
            0.299, 0.587, 0.114, 0.0, 0.0, //
            0.299, 0.587, 0.114, 0.0, 0.0, //
            0.299, 0.587, 0.114, 0.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, 0.0, //
        );
        let mut img = images.first().unwrap().to_owned();
        img = img.color_matrix(luma_matrix);
        if options.denoise.unwrap_or(false) {
            const NINE: f32 = 1.0 / 9.0;
            img = img.image_filter(
                image_filters::matrix_convolution(
                    (3, 3),
                    &[NINE; 9],
                    1.0,
                    0.0,
                    (1, 1),
                    TileMode::Clamp,
                    false,
                    None,
                    None,
                )
                .unwrap(),
            )
        }
        let img_size = img.dimensions();
        let filter = if convolute == "线稿" {
            diff = false;
            None
        } else {
            let mut builder = match convolute {
                "极细" => simple_convolute_average_builder(3),
                "精细" => simple_convolute_average_builder(5),
                "一般" => simple_convolute_average_builder(7),
                "稍粗" => simple_convolute_average_builder(9),
                "超粗" => simple_convolute_average_builder(11),
                "极粗" => simple_convolute_average_builder(13),
                "浮雕" => MatrixConvolutionBuilder::new(
                    (3, 3),
                    [
                        1.0, 1.0, 1.0, //
                        1.0, 1.0, -1.0, //
                        -1.0, -1.0, -1.0, //
                    ],
                    (1, 1),
                ),
                "超浮雕" => MatrixConvolutionBuilder::new(
                    (5, 5),
                    [
                        1.0, 1.0, 1.0, 1.0, 1.0, //
                        1.0, 1.0, 1.0, 1.0, 1.0, //
                        1.0, 1.0, 1.0, -1.0, -1.0, //
                        -1.0, -1.0, -1.0, -1.0, -1.0, //
                        -1.0, -1.0, -1.0, -1.0, -1.0, //
                    ],
                    (2, 2),
                ),
                _ => unreachable!(""),
            };
            builder.bias = bias;
            builder.gain = gain;
            let filter = builder.build();
            if filter.is_none() {
                return Err(Error::MemeFeedback(
                    "edge filter create Failed!".to_string(),
                ));
            };
            filter
        };
        let mut img1 = img.clone();
        if let Some(filter) = filter {
            img1 = img.image_filter(filter)
        };
        let img2 = img.to_owned();
        let img1_data = img1.peek_pixels().unwrap();
        let img2_data = img2.peek_pixels().unwrap();
        let mut final_img = new_surface(img_size);
        let canvas = final_img.canvas();
        for x in 0..img.width() {
            for y in 0..img.height() {
                let rgb = img1_data.get_color((x, y));
                let mut r = rgb.r();
                let mut g = rgb.g();
                let mut b = rgb.b();
                let mut a = rgb.a();
                if diff {
                    r = 128 + img2_data.get_color((x, y)).r() - r;
                    let scale: f32 = 255.0 / (255.0 - light_cut - dark_cut);
                    r = ((r as f32 - dark_cut) * scale).clamp(0.0, 255.0) as u8;
                    (g, b) = (r, r);
                    a = 255 - r;
                }
                canvas.draw_point((x, y), &new_paint(Color::from_argb(a, r, g, b)));
            }
        }
        if kuma {
            canvas.draw_irect(
                IRect::from_size(img_size),
                Paint::default()
                    .set_shader(create_kiss_gradient(img_size).unwrap())
                    .set_blend_mode(BlendMode::SrcIn),
            );
        }
        if full_bg {
            canvas.draw_irect(
                IRect::from_size(img_size),
                new_paint(Color::WHITE).set_blend_mode(BlendMode::DstOver),
            );
        }
        Ok(final_img.image_snapshot())
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
