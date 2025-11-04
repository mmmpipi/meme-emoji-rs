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
    /// 缩放
    #[option(short, long,long_aliases = ["缩放"], default = false)]
    pub resize: Option<bool>,

    /// 不进行后处理
    #[option(short, long,long_aliases = ["取消绘制背景"], default = false)]
    pub no_bg: Option<bool>,

    /// 黑白
    #[option(long,long_aliases = ["黑白"], default = false)]
    pub gray: Option<bool>,

    /// 取消差异化
    #[option(short, long,long_aliases = ["取消差异化"], default = false)]
    pub no_diff: Option<bool>,

    /// 卷积矩阵
    #[option(short, long,long_aliases = ["卷积矩阵"], default = "一般",choices = ["精细","一般","稍粗","超粗","极粗","浮雕","线稿","极细","超浮雕"])]
    pub convolute: Option<String>,

    #[option(short, long, minimum = 0.0, maximum = 30.0, default = 1.0)]
    pub gain: Option<f32>,

    #[option(short, long, minimum = -30.0, maximum = 30.0, default = 0.0)]
    pub bias: Option<f32>,

    /// 线迹轻重
    #[option(long,long_aliases = ["线迹轻重"], minimum = 80.0, maximum = 126.0, default = 118.0)]
    pub dark_cut: Option<f32>,

    /// 降噪
    #[option(short, long,long_aliases = ["降噪"], default = false)]
    pub denoise: Option<bool>,
}

pub(crate) use number_option;
