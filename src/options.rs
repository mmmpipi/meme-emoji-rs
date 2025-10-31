use meme_generator_utils::builder::MemeOptions;

#[derive(MemeOptions)]
pub(crate) struct NoOptions {}

macro_rules! number_option {
    ($name:ident, $min:tt, $max:tt) => {
        use meme_generator_utils::builder::MemeOptions;
        #[derive(MemeOptions)]
        struct $name {
            /// 图片编号
            #[option(short, long, minimum = $min, maximum = $max)]
            number: Option<i32>,
        }
    };
}

#[derive(MemeOptions)]
pub(crate) struct GrayStandard {
    /// 灰度图标准
    #[option(short, long,long_aliases = ["灰度图标准"], default = "rec601_gray", choices = ["rec601", "rec709","rec601_noprocess","rec601_gray"])]
    pub gray_standard: Option<String>,

    /// 缩放
    #[option(short, long,long_aliases = ["缩放"], default = false)]
    pub resize: Option<bool>,

    /// 不进行后处理
    #[option(short, long,long_aliases = ["取消后处理"], default = false)]
    pub no_postprocess: Option<bool>,

    
    #[option(short, long, minimum = 0, maximum = 255, default = 20)]
    pub limit: Option<i32>,

    #[option(short, long, minimum = 0.0, maximum = 30.0, default = 5.0)]
    pub gain: Option<f32>,
    
    #[option(short, long, minimum = -30.0, maximum = 30.0, default = 0.0)]
    pub bias: Option<f32>,
}

pub(crate) use number_option;
