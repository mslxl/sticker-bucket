//! 按钮族：主操作胶囊、玻璃胶囊、无边框图标钮。
//!
//! 全部返回 `Stateful<Div>`，调用方继续链式挂 `.on_click(...)`、`.child(...)`。

use gpui::{
    App, Background, Div, ElementId, Stateful, Styled, div, linear_color_stop, linear_gradient,
    prelude::*, px,
};

use crate::{
    surface::control_shadow,
    theme::{theme, to_hsla},
};

/// 强调色填充：顶部微亮的纵向渐变，模拟受光。
pub fn accent_fill(cx: &App) -> Background {
    let t = theme(cx);
    let mut top = to_hsla(t.accent);
    top.l = (top.l + 0.08).min(1.);
    linear_gradient(
        180.,
        linear_color_stop(top, 0.),
        linear_color_stop(t.accent, 1.),
    )
}

/// 主操作按钮：macOS push button 规格的强调色胶囊（28px 高、13px 字号）。
pub fn primary_button(cx: &App, id: impl Into<ElementId>) -> Stateful<Div> {
    let t = theme(cx);
    let hover = t.accent_hover;
    let press = t.accent_press;
    div()
        .id(id)
        .h(px(28.))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(accent_fill(cx))
        .text_color(t.ink_on_accent)
        .text_size(px(13.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .cursor_pointer()
        .shadow(control_shadow())
        .hover(move |style| style.bg(hover))
        .active(move |style| style.bg(press))
}

/// 危险操作按钮：systemRed 填充，规格同主按钮。
pub fn danger_button(cx: &App, id: impl Into<ElementId>) -> Stateful<Div> {
    let t = theme(cx);
    let base = t.danger;
    let mut press = to_hsla(base);
    press.l = (press.l - 0.08).max(0.);
    div()
        .id(id)
        .h(px(28.))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(base)
        .text_color(t.ink_on_accent)
        .text_size(px(13.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .cursor_pointer()
        .shadow(control_shadow())
        .hover(move |style| style.opacity(0.92))
        .active(move |style| style.bg(press))
}

/// 次级按钮：macOS bordered 白胶囊。
pub fn glass_button(cx: &App, id: impl Into<ElementId>) -> Stateful<Div> {
    let t = theme(cx);
    let strong = t.card_strong;
    let press = t.wash_press;
    div()
        .id(id)
        .h(px(28.))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(t.control_bg)
        .border_1()
        .border_color(t.edge)
        .text_color(t.ink)
        .text_size(px(13.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .cursor_pointer()
        .shadow(control_shadow())
        .hover(move |style| style.bg(strong))
        .active(move |style| style.bg(press))
}

/// 无边框圆形图标钮，hover 才浮现底色（如 ✕ 移除）。
pub fn icon_button(cx: &App, id: impl Into<ElementId>) -> Stateful<Div> {
    let t = theme(cx);
    let ink = t.ink;
    let hover = t.wash_hover;
    let press = t.wash_press;
    div()
        .id(id)
        .size(px(22.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_color(t.label_2)
        .text_xs()
        .cursor_pointer()
        .hover(move |style| style.bg(hover).text_color(ink))
        .active(move |style| style.bg(press))
}
