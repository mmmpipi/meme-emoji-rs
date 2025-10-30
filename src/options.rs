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
}

pub(crate) use number_option;
