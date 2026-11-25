mod agent_detection;
mod app;
mod chat_terminal;
mod chats;
mod fonts;
mod forms;
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
            ("home", app::scenes::scene_home),
            ("profile", app::scenes::scene_profile),
            ("profile-details", app::scenes::scene_profile_details),
            ("projects", app::scenes::scene_projects),
            ("new-project", app::scenes::scene_new_project),
            ("documents", app::scenes::scene_documents),
            ("agents", app::scenes::scene_subagents),
            ("subagents", app::scenes::scene_subagents),
            ("avatars", app::scenes::scene_avatars),
            ("capture", app::scenes::scene_capture),
            ("capture-preview", app::scenes::scene_capture_preview),
            ("data", app::scenes::scene_settings),
            ("settings", app::scenes::scene_settings),
            ("storage", app::scenes::scene_storage),
            ("settings-shortcuts", app::scenes::scene_shortcuts),
            ("settings-agents", app::scenes::scene_settings_agents),
            ("home-dark", app::scenes::scene_home_dark),
            ("chat", app::scenes::scene_chat),
            ("chat-floating", app::scenes::scene_chat_floating),
            ("settings-fonts", app::scenes::scene_settings_fonts),
            ("settings-dark", app::scenes::scene_settings_dark),
            (
                "settings-agents-dark",
                app::scenes::scene_settings_agents_dark,
            ),
            ("onboarding-start", app::scenes::scene_onboarding_start),
            ("onboarding-review", app::scenes::scene_onboarding_review),
        ],
        ..AppSpec::new(
            "Coderune",
            (1280.0, 860.0),
            app::CoderuneApp::init,
            app::view,
        )
    });
}
