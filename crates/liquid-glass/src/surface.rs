//! 玻璃表面与布局原语：卡片、分组、分隔线、阴影、窗口配置。

use gpui::{
    App, BoxShadow, Div, SharedString, Styled, TitlebarOptions, WindowBackgroundAppearance,
    WindowOptions, div, hsla, point, prelude::*, px,
};

use crate::theme::theme;

/// 面板级投影（侧边栏、弹出层等大表面）。
pub fn panel_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0., 0., 0., 0.10),
            offset: point(px(0.), px(10.)),
            blur_radius: px(30.),
            spread_radius: px(-10.),
        },
        BoxShadow {
            color: hsla(0., 0., 0., 0.04),
            offset: point(px(0.), px(1.)),
            blur_radius: px(3.),
            spread_radius: px(0.),
        },
    ]
}

/// 卡片贴地投影。
pub fn card_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0., 0., 0., 0.06),
            offset: point(px(0.), px(2.)),
            blur_radius: px(8.),
            spread_radius: px(-1.),
        },
        BoxShadow {
            color: hsla(0., 0., 0., 0.04),
            offset: point(px(0.), px(0.5)),
            blur_radius: px(1.),
            spread_radius: px(0.),
        },
    ]
}

/// 控件级细投影（按钮、选中胶囊）。
pub fn control_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0., 0., 0., 0.10),
        offset: point(px(0.), px(1.)),
        blur_radius: px(2.5),
        spread_radius: px(0.),
    }]
}

/// 玻璃卡片：亮白浮层 + 定义边 + 贴地投影。
pub fn glass_card(cx: &App) -> Div {
    let t = theme(cx);
    div()
        .rounded(px(14.))
        .bg(t.card)
        .border_1()
        .border_color(t.edge)
        .shadow(card_shadow())
}

/// 分组列表容器（macOS 设置的 inset group）。配合 [`group_row`] 与 [`hairline`]。
pub fn glass_group(cx: &App) -> Div {
    glass_card(cx).rounded(px(12.)).overflow_hidden()
}

/// 分组内的一行。
pub fn group_row() -> Div {
    div()
        .min_h(px(44.))
        .px_4()
        .py_3()
        .flex()
        .items_center()
        .gap_4()
}

/// 分组内发丝分隔线（左侧留出行内边距）。
pub fn hairline(cx: &App) -> Div {
    div().h(px(1.)).ml_4().bg(theme(cx).separator)
}

/// 分组上方的节标题。
pub fn section_header(cx: &App, label: impl Into<SharedString>) -> Div {
    div()
        .px_2()
        .pb_2()
        .text_size(px(13.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme(cx).label_2)
        .child(label.into())
}

/// Liquid Glass 窗口配置：整窗模糊背景 + 透明标题栏。
/// 调用方在此基础上补 `window_bounds`、`app_id` 等字段。
pub fn window_options(title: impl Into<SharedString>) -> WindowOptions {
    WindowOptions {
        window_background: WindowBackgroundAppearance::Blurred,
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(24.), px(24.))),
        }),
        ..Default::default()
    }
}
