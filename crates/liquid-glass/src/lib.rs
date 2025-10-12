//! Liquid Glass —— gpui 的 Apple 设计语言组件库。
//!
//! 材质模型：真实模糊由窗口背景（[`surface::window_options`] 开启
//! `WindowBackgroundAppearance::Blurred`）提供；组件只负责白色浮层、
//! 受光边缘与投影。亮度分层避免半透明白互叠发灰：
//! 侧边栏（低不透明度玻璃）< 内容面板（近白）< 卡片（几乎不透明）< 选中态（纯白）。
//!
//! # 使用
//!
//! ```ignore
//! Application::new().run(|cx| {
//!     liquid_glass::init(cx); // 安装主题 + 输入按键
//!     cx.open_window(liquid_glass::window_options("App"), |_, cx| { ... });
//! });
//! ```
//!
//! 控件均为构造函数风格：返回 `Div`/`Stateful<Div>`，
//! 调用方继续链式挂 `.on_click(...)`、`.child(...)`。

mod button;
mod feedback;
mod input;
mod navigation;
mod overlay;
mod selection;
mod surface;
mod theme;

pub use button::{accent_fill, danger_button, glass_button, icon_button, primary_button};
pub use feedback::{BannerKind, banner, empty_state, hud, progress_bar, skeleton, spinner};
pub use input::TextInput;
pub use navigation::{page_header, search_field, sidebar, sidebar_item, sidebar_title};
pub use overlay::{
    dialog, dialog_footer, menu, menu_item, menu_separator, popover, tooltip_bubble,
};
pub use selection::{checkbox, segmented_control, switch, tag_chip};
pub use surface::{
    card_shadow, control_shadow, glass_card, glass_group, group_row, hairline, panel_shadow,
    section_header, window_options,
};
pub use theme::{Theme, init, init_with_theme, theme, to_hsla};
