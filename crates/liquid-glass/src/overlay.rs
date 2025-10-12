//! 浮层类控件：菜单、模态对话框、工具提示。
//!
//! 浮层都遵循同一材质：近白玻璃 + 定义边 + 面板投影，
//! 模态额外配 scrim 遮罩压暗背景（Apple 的 dim-to-focus）。

use std::rc::Rc;

use gpui::{
    AnyElement, App, Corner, Div, ElementId, SharedString, Stateful, Styled, Window, anchored,
    deferred, div, prelude::*, px,
};

use crate::{surface::panel_shadow, theme::theme};

/// 菜单面板容器：放入 [`menu_item`] / [`menu_separator`]。
/// 需要浮层定位时套 `deferred(anchored().child(...))`，见 [`popover`]。
pub fn menu(cx: &App) -> Div {
    let t = theme(cx);
    div()
        .min_w(px(180.))
        .p(px(5.))
        .rounded(px(12.))
        .bg(t.card)
        .border_1()
        .border_color(t.edge)
        .shadow(panel_shadow())
        .flex()
        .flex_col()
}

/// 菜单项：hover 时整行变强调色白字（macOS 菜单交互）。
pub fn menu_item(
    cx: &App,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    destructive: bool,
) -> Stateful<Div> {
    let t = theme(cx);
    let accent = t.accent;
    let on_accent = t.ink_on_accent;
    div()
        .id(id)
        .h(px(28.))
        .px_3()
        .rounded(px(8.))
        .flex()
        .items_center()
        .gap_2()
        .text_size(px(13.))
        .text_color(if destructive { t.danger } else { t.ink })
        .cursor_pointer()
        .hover(move |style| style.bg(accent).text_color(on_accent))
        .child(label.into())
}

/// 菜单分隔线。
pub fn menu_separator(cx: &App) -> Div {
    div().h(px(1.)).mx_2().my_1().bg(theme(cx).separator)
}

/// 把内容包装成锚定在触发元素处的浮层（渲染在普通内容之上）。
///
/// 在触发元素的 render 里条件展开：
/// `.when(open, |d| d.child(popover(corner, menu(cx).child(...))))`
pub fn popover(anchor: Corner, content: impl IntoElement) -> AnyElement {
    deferred(
        anchored()
            .anchor(anchor)
            .snap_to_window_with_margin(px(8.))
            .child(div().mt_1().child(content)),
    )
    .with_priority(1)
    .into_any_element()
}

/// 模态对话框：scrim 遮罩 + 居中玻璃面板。
///
/// 作为窗口根元素的最后一个 child 条件渲染。`on_dismiss` 在点击遮罩时触发。
pub fn dialog(
    cx: &App,
    id: impl Into<ElementId>,
    content: impl IntoElement,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let t = theme(cx);
    let on_dismiss = Rc::new(on_dismiss);
    deferred(
        div()
            .id(id)
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(t.scrim)
            .on_click({
                let on_dismiss = Rc::clone(&on_dismiss);
                move |_, window, cx| on_dismiss(window, cx)
            })
            .child(
                div()
                    .id("liquid-glass-dialog-panel")
                    .w(px(420.))
                    .p_6()
                    .rounded(px(16.))
                    .bg(t.card_strong)
                    .border_1()
                    .border_color(t.edge)
                    .shadow(panel_shadow())
                    // 拦截面板内点击，避免冒泡到遮罩关闭
                    .on_click(|_, _, _| {})
                    .child(content),
            ),
    )
    .with_priority(2)
    .into_any_element()
}

/// 对话框标准页脚：右对齐按钮排。
pub fn dialog_footer() -> Div {
    div().pt_5().flex().justify_end().gap_2()
}

/// 深色小工具提示（配 gpui 的 `.tooltip(...)` 或自行锚定）。
pub fn tooltip_bubble(cx: &App, label: impl Into<SharedString>) -> Div {
    let t = theme(cx);
    div()
        .px(px(10.))
        .py(px(5.))
        .rounded(px(7.))
        .bg(gpui::rgba(0x28282ce6))
        .shadow(panel_shadow())
        .text_size(px(12.))
        .text_color(t.ink_on_accent)
        .child(label.into())
}
