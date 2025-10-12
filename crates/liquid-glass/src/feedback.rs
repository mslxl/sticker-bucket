//! 反馈类控件：进度条、加载指示、横幅通知、空状态。

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, App, Div, SharedString, Styled, div, prelude::*, px, relative,
};

use crate::{
    surface::{card_shadow, glass_card},
    theme::theme,
};

/// 横幅语义。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BannerKind {
    /// 成功（绿点）
    Success,
    /// 提示（蓝点）
    Info,
    /// 警告（橙点）
    Warning,
    /// 错误（红点）
    Error,
}

/// 玻璃横幅：状态圆点 + 文字。用于操作结果通知。
pub fn banner(cx: &App, kind: BannerKind, message: impl Into<SharedString>) -> Div {
    let t = theme(cx);
    let dot = match kind {
        BannerKind::Success => t.success,
        BannerKind::Info => t.accent,
        BannerKind::Warning => t.warning,
        BannerKind::Error => t.danger,
    };
    div()
        .w_full()
        .px_4()
        .py_3()
        .rounded(px(12.))
        .bg(t.card)
        .border_1()
        .border_color(t.edge)
        .shadow(card_shadow())
        .flex()
        .items_center()
        .gap_3()
        .child(div().size(px(8.)).flex_none().rounded_full().bg(dot))
        .child(
            div()
                .text_size(px(13.))
                .text_color(t.ink)
                .child(message.into()),
        )
}

/// 定值进度条。`fraction` ∈ [0, 1]。
pub fn progress_bar(cx: &App, fraction: f32) -> Div {
    let t = theme(cx);
    let fraction = fraction.clamp(0., 1.);
    div()
        .w_full()
        .h(px(4.))
        .rounded_full()
        .bg(t.wash_hover)
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(fraction))
                .rounded_full()
                .bg(t.accent),
        )
}

/// 不定时长的加载指示：三个相位错开呼吸的圆点（gpui 无逐边描边/旋转，
/// 点阵脉冲是等价的 macOS 式安静反馈）。
pub fn spinner(cx: &App, id: impl Into<gpui::ElementId>, diameter: f32) -> impl IntoElement {
    let t = theme(cx);
    let dot = px((diameter / 4.).max(3.));
    let accent = t.accent;
    div()
        .id(id.into())
        .flex_none()
        .flex()
        .items_center()
        .gap(px((diameter / 6.).max(2.)))
        .children((0_usize..3).map(move |index| {
            div().size(dot).rounded_full().bg(accent).with_animation(
                ("liquid-glass-spinner-dot", index),
                Animation::new(Duration::from_millis(900)).repeat(),
                move |style, delta| {
                    let phase = (delta + index as f32 / 3.) % 1.;
                    let wave = (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
                    style.opacity(0.25 + 0.75 * wave)
                },
            )
        }))
}

/// 居中空状态：标题 + 说明 + 可选操作。macOS 原生风格（无背景卡片）。
pub fn empty_state(
    cx: &App,
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    action: Option<gpui::AnyElement>,
) -> Div {
    let t = theme(cx);
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(
            div()
                .text_size(px(17.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(t.label_2)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(13.))
                .text_color(t.label_3)
                .child(detail.into()),
        )
        .when_some(action, |style, action| {
            style.child(div().pt_2().child(action))
        })
}

/// 浮层 HUD（如“已复制”）：居中的深色玻璃块。配合 `anchored`/绝对定位使用。
pub fn hud(cx: &App, message: impl Into<SharedString>) -> Div {
    let t = theme(cx);
    div()
        .px_5()
        .py_4()
        .rounded(px(14.))
        .bg(gpui::rgba(0x28282cd9))
        .shadow(card_shadow())
        .text_size(px(13.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(t.ink_on_accent)
        .child(message.into())
}

/// 骨架占位块：低对比灰块，用于图片加载中。
pub fn skeleton(cx: &App) -> Div {
    glass_card(cx)
        .bg(theme(cx).wash_hover)
        .border_0()
        .shadow(vec![])
}
