use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use memelith_core::{APPLICATION_NAME, Memelith};

struct MemelithView {
    core: Memelith,
}

impl Render for MemelithView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .justify_center()
            .items_center()
            .gap_3()
            .bg(rgb(0x151821))
            .text_color(rgb(0xf5f0ff))
            .child(
                div()
                    .text_3xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(APPLICATION_NAME),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .rounded_full()
                    .bg(rgb(0x2a2038))
                    .text_color(rgb(0xd8b4fe))
                    .child(self.core.status_summary()),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(720.0), px(480.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| MemelithView {
                    core: Memelith::new(),
                })
            },
        )
        .expect("failed to open the Memelith window");

        cx.activate(true);
    });
}
