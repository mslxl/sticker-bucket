//! 导航结构：玻璃侧边栏、导航项、搜索框、大标题页头。

use gpui::{
    App, Div, ElementId, Entity, SharedString, Stateful, Styled, WindowControlArea, div,
    prelude::*, px,
};

use crate::{input::TextInput, surface::control_shadow, theme::theme};

/// 侧边栏容器：低不透明度活性玻璃 + 右侧定义边，
/// 顶部预留 macOS 红绿灯区域（同时是窗口拖拽区）。
pub fn sidebar(cx: &App, width: f32) -> Div {
    let t = theme(cx);
    div()
        .w(px(width))
        .h_full()
        .flex_none()
        .bg(t.sidebar_glass)
        .border_r_1()
        .border_color(gpui::rgba(0x0000000d))
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(56.))
                .flex_none()
                .window_control_area(WindowControlArea::Drag),
        )
}

/// 侧边栏应用名（拖拽区的一部分）。
pub fn sidebar_title(label: impl Into<SharedString>) -> Div {
    div()
        .px_5()
        .pb_3()
        .text_size(px(15.))
        .font_weight(gpui::FontWeight::BOLD)
        .window_control_area(WindowControlArea::Drag)
        .child(label.into())
}

/// 侧边栏导航项：选中为亮白胶囊 + 贴地阴影，图标着强调色。
pub fn sidebar_item(
    cx: &App,
    id: impl Into<ElementId>,
    icon: impl Into<SharedString>,
    label: impl Into<SharedString>,
    selected: bool,
) -> Stateful<Div> {
    let t = theme(cx);
    let strong = t.card_strong;
    div()
        .id(id)
        .h(px(32.))
        .px_3()
        .rounded(px(9.))
        .flex()
        .items_center()
        .gap_3()
        .cursor_pointer()
        .text_size(px(13.))
        .text_color(t.ink)
        .when(selected, |style| {
            style
                .bg(strong)
                .shadow(control_shadow())
                .font_weight(gpui::FontWeight::SEMIBOLD)
        })
        .when(!selected, |style| {
            style.hover(|style| style.bg(gpui::rgba(0xffffff73)))
        })
        .active(move |style| style.bg(strong))
        .child(
            div()
                .w(px(20.))
                .text_center()
                .text_color(t.accent)
                .child(icon.into()),
        )
        .child(label.into())
}

/// 内容区大标题页头（可拖拽窗口）。
pub fn page_header(
    cx: &App,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
) -> Div {
    let t = theme(cx);
    div()
        .flex_none()
        .px_8()
        .pt_6()
        .pb_4()
        .flex()
        .flex_col()
        .gap_1()
        .window_control_area(WindowControlArea::Drag)
        .child(
            div()
                .text_size(px(26.))
                .font_weight(gpui::FontWeight::BOLD)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(13.))
                .text_color(t.label_2)
                .child(subtitle.into()),
        )
}

/// 搜索框：圆角灰底 + 放大镜前缀，包装一个 [`TextInput`]。
pub fn search_field(cx: &App, input: Entity<TextInput>) -> Div {
    let t = theme(cx);
    div()
        .h(px(30.))
        .px(px(9.))
        .rounded(px(9.))
        .bg(t.wash_hover)
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex_none()
                .text_size(px(13.))
                .text_color(t.label_3)
                .child("⌕"),
        )
        .child(div().flex_1().min_w_0().child(input))
}
