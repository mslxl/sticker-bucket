//! 选择类控件：Switch、Checkbox、SegmentedControl。
//!
//! Switch/Checkbox 是受控组件：传入当前值，调用方挂 `.on_click` 翻转状态。

use std::rc::Rc;

use gpui::{App, Div, ElementId, SharedString, Stateful, Styled, Window, div, prelude::*, px};

use crate::{
    surface::control_shadow,
    theme::{theme, to_hsla},
};

/// macOS 风格开关（38×22）。`checked` 由调用方持有，`.on_click` 翻转。
pub fn switch(cx: &App, id: impl Into<ElementId>, checked: bool) -> Stateful<Div> {
    let t = theme(cx);
    let track = if checked {
        t.accent
    } else {
        // 未选中：中性灰轨道
        gpui::rgba(0x78788033)
    };
    div()
        .id(id)
        .w(px(38.))
        .h(px(22.))
        .flex_none()
        .rounded_full()
        .bg(track)
        .p(px(2.))
        .flex()
        .items_center()
        .when(checked, |style| style.justify_end())
        .cursor_pointer()
        .child(
            div()
                .size(px(18.))
                .rounded_full()
                .bg(t.control_bg)
                .shadow(control_shadow()),
        )
}

/// 复选框（16×16）。`checked` 受控。
pub fn checkbox(cx: &App, id: impl Into<ElementId>, checked: bool) -> Stateful<Div> {
    let t = theme(cx);
    div()
        .id(id)
        .size(px(16.))
        .flex_none()
        .rounded(px(4.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .map(|style| {
            if checked {
                style
                    .bg(t.accent)
                    .text_color(t.ink_on_accent)
                    .text_size(px(11.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("✓")
            } else {
                style.bg(t.control_bg).border_1().border_color(t.edge)
            }
        })
}

/// 分段控件：灰轨道内的白色选中段（macOS segmented control）。
///
/// `on_select` 在点击未选中的段时收到该段下标。
pub fn segmented_control(
    cx: &App,
    id: impl Into<ElementId>,
    labels: &[SharedString],
    selected: usize,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let t = theme(cx);
    let on_select = Rc::new(on_select);
    div()
        .id(id)
        .h(px(28.))
        .p(px(2.))
        .rounded(px(9.))
        .bg(t.wash_hover)
        .flex()
        .items_center()
        .children(labels.iter().enumerate().map(|(index, label)| {
            let on_select = Rc::clone(&on_select);
            let is_selected = index == selected;
            div()
                .id(("segment", index))
                .h_full()
                .px_3()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(7.))
                .text_size(px(13.))
                .text_color(t.ink)
                .cursor_pointer()
                .when(is_selected, |style| {
                    style
                        .bg(t.control_bg)
                        .shadow(control_shadow())
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                })
                .on_click(move |_, window, cx| {
                    if !is_selected {
                        on_select(index, window, cx);
                    }
                })
                .child(label.clone())
        }))
}

/// 小型标签胶囊（Tag/Chip）。中性底；`accent` 变体给选中态。
pub fn tag_chip(cx: &App, id: impl Into<ElementId>, accent: bool) -> Stateful<Div> {
    let t = theme(cx);
    let mut accent_bg = to_hsla(t.accent);
    accent_bg.a = 0.14;
    div()
        .id(id)
        .h(px(22.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap_1()
        .rounded_full()
        .text_size(px(12.))
        .map(|style| {
            if accent {
                style.bg(accent_bg).text_color(t.accent)
            } else {
                style.bg(t.wash_hover).text_color(t.ink)
            }
        })
}
