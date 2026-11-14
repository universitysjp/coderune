mod app;
mod screens;
mod widgets;

use rune::AppSpec;

fn main() {
    rune::run(AppSpec {
        min_size: Some((900.0, 600.0)),
        fonts: &[
            include_bytes!("../../../packages/tokens/fonts/Geist-Variable.ttf"),
            include_bytes!("../../../packages/tokens/fonts/PaperMono-Variable.ttf"),
        ],
        families: ("Geist", "Paper Mono"),
        on_key: Some(app::on_key),
        scenes: &[
            ("home", app::scene_home),
            ("profile", app::scene_profile),
            ("projects", app::scene_projects),
            ("documents", app::scene_documents),
            ("agents", app::scene_agents),
            ("capture", app::scene_capture),
            ("capture-preview", app::scene_capture_preview),
            ("data", app::scene_data),
        ],
        ..AppSpec::new(
            "Coderune",
            (1280.0, 860.0),
            app::CoderuneApp::init,
            app::view,
        )
    });
}
