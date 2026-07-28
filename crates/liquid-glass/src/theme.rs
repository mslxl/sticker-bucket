//! Liquid Glass 主题：Apple 浅色系统色板与材质常量。
//!
//! 亮度层级规则：根层透出窗口模糊底，侧边栏是低不透明度活性玻璃，
//! 内容面板近白，卡片几乎不透明——避免半透明白互叠发灰。

use gpui::{App, Global, Hsla, Rgba, rgb, rgba};

/// 全局主题。通过 [`init`] 安装，之后用 [`theme`] 读取。
#[derive(Clone, Debug)]
pub struct Theme {
    /// 强调色（systemBlue）
    pub accent: Rgba,
    /// 强调色 hover 加深
    pub accent_hover: Rgba,
    /// 强调色按压加深
    pub accent_press: Rgba,
    /// 成功（systemGreen）
    pub success: Rgba,
    /// 警告（systemOrange）
    pub warning: Rgba,
    /// 危险（systemRed）
    pub danger: Rgba,

    /// 主文字（label）
    pub ink: Rgba,
    /// 次级文字（secondaryLabel）
    pub label_2: Rgba,
    /// 三级文字（tertiaryLabel）
    pub label_3: Rgba,
    /// 反色文字（选中/强调填充上的白字）
    pub ink_on_accent: Rgba,

    /// 侧边栏活性玻璃（低不透明度，透出窗口模糊）
    pub sidebar_glass: Rgba,
    /// 内容面板底（近白）
    pub content_bg: Rgba,
    /// 卡片 / 分组表面
    pub card: Rgba,
    /// 选中 / hover 提亮表面
    pub card_strong: Rgba,
    /// 控件白底（输入框、次级按钮）
    pub control_bg: Rgba,
    /// 玻璃受光亮边
    pub glass_edge: Rgba,
    /// 深色定义边（卡片、控件描边）
    pub edge: Rgba,
    /// 分组内发丝分隔线
    pub separator: Rgba,
    /// hover 浮现的中性底色
    pub wash_hover: Rgba,
    /// 按压加深的中性底色
    pub wash_press: Rgba,
    /// 模态遮罩
    pub scrim: Rgba,
}

impl Theme {
    /// Apple 浅色外观默认主题。
    pub fn light() -> Self {
        Self {
            accent: rgb(0x007aff),
            accent_hover: rgb(0x0070e8),
            accent_press: rgb(0x0063cc),
            success: rgb(0x34c759),
            warning: rgb(0xff9500),
            danger: rgb(0xff3b30),

            ink: rgb(0x1d1d1f),
            label_2: rgba(0x3c3c4399),
            label_3: rgba(0x3c3c4366),
            ink_on_accent: rgb(0xffffff),

            sidebar_glass: rgba(0xffffff8c),
            content_bg: rgba(0xfbfafde8),
            card: rgba(0xfffffff2),
            card_strong: rgba(0xffffffff),
            control_bg: rgb(0xffffff),
            glass_edge: rgba(0xffffff73),
            edge: rgba(0x00000010),
            separator: rgba(0x3c3c431f),
            wash_hover: rgba(0x3c3c4314),
            wash_press: rgba(0x3c3c4326),
            scrim: rgba(0x00000033),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

struct GlobalTheme(Theme);

impl Global for GlobalTheme {}

/// 安装主题（含文本输入按键绑定）。在 `Application::run` 回调里调用一次。
pub fn init(cx: &mut App) {
    init_with_theme(cx, Theme::default());
}

/// 以自定义主题安装。
pub fn init_with_theme(cx: &mut App, theme: Theme) {
    cx.set_global(GlobalTheme(theme));
    crate::input::init(cx);
}

/// 读取当前主题。必须先 [`init`]。
pub fn theme(cx: &App) -> &Theme {
    &cx.global::<GlobalTheme>().0
}

/// Hsla 快捷转换（gpui 的部分 API 只收 Hsla）。
pub fn to_hsla(color: Rgba) -> Hsla {
    color.into()
}
