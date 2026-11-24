mod agent_detection;
mod app;
mod chats;
mod fonts;
mod preferences;
mod screens;
mod shell;
mod update_install;
mod updates;
mod widgets;

use rune::AppSpec;

pub(crate) const VERSION: &str = match option_env!("CODERUNE_BUILD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// The families the app ships with, used until another is chosen.
pub(crate) const DEFAULT_UI_FONT: &str = "Geist";
pub(crate) const DEFAULT_MONO_FONT: &str = "Paper Mono";

fn main() {
    update_install::finish_on_launch();
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--version")
    {
        println!("Coderune {VERSION}");
        return;
    }

    #[cfg(target_os = "macos")]
    {
        // AppKit applies the native mask to the bundled icon. Standalone
        // binaries need an icon with the mask already applied.
        let has_bundle_icon = std::env::current_exe().is_ok_and(|executable| {
            executable
                .with_file_name("../Resources/AppIcon.icns")
                .is_file()
        });
        if !has_bundle_icon {
            let _ = rune::set_app_icon(include_bytes!("../assets/icon-dock.png").as_slice());
        }
    }
    #[cfg(target_os = "windows")]
    let _ = rune::set_app_icon(include_bytes!("../assets/icon-1024.png").as_slice());
    rune::run(AppSpec {
        min_size: Some((960.0, 640.0)),
        fonts: &[
            include_bytes!("../../../packages/tokens/fonts/Geist-Variable.ttf"),
            include_bytes!("../../../packages/tokens/fonts/PaperMono-Variable.ttf"),
        ],
        families: (DEFAULT_UI_FONT, DEFAULT_MONO_FONT),
        on_key: Some(app::on_key),
        poll: Some(app::poll),
        scenes: &[
            ("home", app::scene_home),
            ("profile", app::scene_profile),
            ("profile-details", app::scene_profile_details),
            ("projects", app::scene_projects),
            ("documents", app::scene_documents),
            ("agents", app::scene_subagents),
            ("subagents", app::scene_subagents),
            ("avatars", app::scene_avatars),
            ("capture", app::scene_capture),
            ("capture-preview", app::scene_capture_preview),
            ("data", app::scene_settings),
            ("settings", app::scene_settings),
            ("storage", app::scene_storage),
            ("settings-shortcuts", app::scene_shortcuts),
            ("settings-agents", app::scene_settings_agents),
            ("home-dark", app::scene_home_dark),
            ("settings-fonts", app::scene_settings_fonts),
            ("settings-dark", app::scene_settings_dark),
            ("settings-agents-dark", app::scene_settings_agents_dark),
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
