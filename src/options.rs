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
pub(crate) struct Louvre {
    /// 缩放
    #[option(short, long,long_aliases = ["缩放"], default = false)]
    pub resize: Option<bool>,

    /// 不渲染背景
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

    /// 取消降噪
    #[option(short, long,long_aliases = ["取消降噪"], default = false)]
    pub no_denoise: Option<bool>,

    /// 降噪次数
    #[option(long,long_aliases = ["降噪次数"], default = 1)]
    pub denoise_times: Option<i32>,
}

#[derive(MemeOptions)]
pub(crate) struct WaveEffect {
    /// 控制波纹的强度
    #[option(short, long,long_aliases = ["振幅"], minimum = -1.0, maximum = 1.0, default = 0.01)]
    pub amplitude: Option<f32>,

    /// 控制波纹的密度
    #[option(short, long,long_aliases = ["频率"], minimum = -80.0, maximum = 80.0, default = 40.0)]
    pub frequency: Option<f32>,

    /// 偏移
    #[option(short, long,long_aliases = ["相位"], minimum = -80.0, maximum = 80.0, default = 0.0)]
    pub phase: Option<f32>,
    /// 帧数量
    #[option(long,long_aliases = ["帧数量"], minimum = 5, maximum = 100, default = 10)]
    pub frame: Option<i32>,
    /// 单图
    #[option(short, long,long_aliases = ["单图"], default = false)]
    pub single: Option<bool>,
}

#[derive(MemeOptions)]
pub(crate) struct SwirlEffect {
    /// 强度
    #[option(short, long,long_aliases = ["强度"], minimum = -30.0, maximum = 30.0, default = 0.5)]
    pub strength: Option<f32>,

    /// 半径
    #[option(short, long,long_aliases = ["半径"], minimum = 0.0, maximum = 4.0, default = 0.5)]
    pub radius: Option<f32>,

    /// 偏移
    #[option(short, long,long_aliases = ["相位"], minimum = -80.0, maximum = 80.0, default = 0.0)]
    pub phase: Option<f32>,
    /// 动画时间
    #[option(short, long,long_aliases = ["时间"], minimum = 0.01, maximum = 30.0, default = 2.0)]
    pub time: Option<f32>,
    /// 动画前空转时间
    #[option(short, long,long_aliases = ["时间"], minimum = 0.01, maximum = 30.0, default = 0.2)]
    pub empty_time: Option<f32>,
    /// 单图
    #[option(long,long_aliases = ["单图"], default = false)]
    pub single: Option<bool>,
    /// 循环
    #[option(long,long_aliases = ["循环"], default = false)]
    pub loopback: Option<bool>,
}
