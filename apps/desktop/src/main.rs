mod app;
mod screens;
mod widgets;

use rune::AppSpec;

fn main() {
    // A `cargo run` binary has no bundle, so give the window and Dock the
    // composed mark explicitly. A packaged build can also use `icon.icns`.
    let _ = rune::set_app_icon(include_bytes!("../assets/icon-1024.png").to_vec());
    rune::run(AppSpec {
        min_size: Some((960.0, 640.0)),
        fonts: &[
            include_bytes!("../../../packages/tokens/fonts/Geist-Variable.ttf"),
            include_bytes!("../../../packages/tokens/fonts/PaperMono-Variable.ttf"),
        ],
        families: ("Geist", "Paper Mono"),
        on_key: Some(app::on_key),
        scenes: &[
            ("home", app::scene_home),
            ("profile", app::scene_profile),
            ("profile-details", app::scene_profile_details),
            ("projects", app::scene_projects),
            ("documents", app::scene_documents),
            ("agents", app::scene_agents),
            ("capture", app::scene_capture),
            ("capture-preview", app::scene_capture_preview),
            ("data", app::scene_data),
            ("onboarding-start", app::scene_onboarding_start),
            ("onboarding-review", app::scene_onboarding_review),
        ],
        ..AppSpec::new(
            "Coderune",
            (1280.0, 860.0),
            app::CoderuneApp::init,
            app::view,
        )
    });
}
