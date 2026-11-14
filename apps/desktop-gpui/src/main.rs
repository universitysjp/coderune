mod app;
mod assets;

use std::borrow::Cow;

use assets::Assets;
use coderune_ui::{Backspace, Copy, Cut, Delete, Left, Paste, Right, SelectAll};
use gpui::{
    px, size, App, AppContext, Application, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions,
};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("coderune=info")),
        )
        .init();

    Application::new().with_assets(Assets).run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(coderune_ui::theme::GEIST_TTF),
                Cow::Borrowed(coderune_ui::theme::PAPER_MONO_TTF),
            ])
            .expect("Coderune fonts must register before the first layout");

        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, Some("TextInput")),
            KeyBinding::new("delete", Delete, Some("TextInput")),
            KeyBinding::new("left", Left, Some("TextInput")),
            KeyBinding::new("right", Right, Some("TextInput")),
            KeyBinding::new("cmd-a", SelectAll, Some("TextInput")),
            KeyBinding::new("ctrl-a", SelectAll, Some("TextInput")),
            KeyBinding::new("cmd-v", Paste, Some("TextInput")),
            KeyBinding::new("ctrl-v", Paste, Some("TextInput")),
            KeyBinding::new("cmd-c", Copy, Some("TextInput")),
            KeyBinding::new("ctrl-c", Copy, Some("TextInput")),
            KeyBinding::new("cmd-x", Cut, Some("TextInput")),
            KeyBinding::new("ctrl-x", Cut, Some("TextInput")),
        ]);

        let launch_directory = std::env::args().nth(1).map(std::path::PathBuf::from);
        let bounds = Bounds::centered(None, size(px(1280.0), px(860.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.0), px(600.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Coderune".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| app::CoderuneApp::new(launch_directory, cx)),
        )
        .expect("Coderune window should open");
        cx.activate(true);
    });
}
