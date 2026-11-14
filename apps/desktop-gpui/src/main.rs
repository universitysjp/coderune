use gpui::{
    div, prelude::*, px, rgb, App, Application, Bounds, Context, Window, WindowBounds,
    WindowOptions,
};

struct Root;

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xfcfcfc))
            .text_color(rgb(0x161616))
            .p(px(24.0))
            .child("Coderune native")
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, gpui::size(px(1280.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Root),
        )
        .unwrap();
        cx.activate(true);
    });
}
