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
    #[option(long,long_aliases = ["线迹轻重"], minimum = 0.0, maximum = 126.0, default = 1.0)]
    pub dark_cut: Option<f32>,

    /// 降噪
    #[option(short, long,long_aliases = ["降噪"], default = false)]
    pub denoise: Option<bool>,

    /// 关闭赛璐璐预处理
    #[option(long, long_aliases = ["关闭赛璐璐"], default = false)]
    pub no_toon: Option<bool>,

    /// 关闭 Canny，回退卷积差分
    #[option(long, long_aliases = ["关闭Canny"], default = false)]
    pub no_canny: Option<bool>,

    /// Canny 高斯，不填则跟卷积矩阵
    #[option(long, long_aliases = ["高斯"], minimum = 0.3, maximum = 4.0, default = 0.5)]
    pub canny_sigma: Option<f32>,

    /// Canny 高阈值
    #[option(long, long_aliases = ["高阈值"], minimum = 0.001, maximum = 0.8, default = 0.02)]
    pub canny_high: Option<f32>,

    /// Canny 低阈值
    #[option(long, long_aliases = ["低阈值"], minimum = 0.0, maximum = 0.8, default = 0.008)]
    pub canny_low: Option<f32>,

    /// NMS 软度，0 硬抑制，越大越软
    #[option(long, long_aliases = ["NMS"], minimum = 0.0, maximum = 1.0, default = 0.3)]
    pub nms: Option<f32>,

    /// 滞后连接次数
    #[option(long, long_aliases = ["滞后"], minimum = 0, maximum = 16, default = 16)]
    pub hyst: Option<i32>,

    /// Canny 后膨胀，不填则跟卷积矩阵
    #[option(long, long_aliases = ["膨胀"], minimum = 0.0, maximum = 8.0)]
    pub canny_dilate: Option<f32>,

    /// 预处理缩小倍数
    #[option(long, long_aliases = ["缩小倍数"], minimum = 1.0, maximum = 4.0, default = 1.0)]
    pub toon_scale: Option<f32>,

    /// 保边平滑空间半径
    #[option(long, long_aliases = ["平滑半径"], minimum = 0.5, maximum = 16.0, default = 1.0)]
    pub sigma_s: Option<f32>,

    /// 保边平滑色差
    #[option(long, long_aliases = ["平滑色差"], minimum = 0.01, maximum = 0.5, default = 0.08)]
    pub sigma_r: Option<f32>,

    /// 高光裁切
    #[option(long, long_aliases = ["高光裁切"], minimum = 0.0, maximum = 200.0, default = 128.0)]
    pub light_cut: Option<f32>,

    /// 处理前进行高对比度对于某些图片很合适
    #[option(long,long_aliases = ["对比度","co"],minimum = 0.0, maximum = 1.0, default = 0.05)]
    pub contrast: Option<f32>,
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
