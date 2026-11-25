//! The home screen.

use crate::app::CoderuneApp;
use crate::widgets::{primary, primary_tx, secondary, secondary_tx, status_alerts};
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, HEADING_LG};
use rune::{row, text, El};
use rune_kit::Kit;

pub(crate) fn home(s: &CoderuneApp, k: &Kit) -> El {
    let project = s.workspace.as_ref().and_then(|workspace| {
        workspace
            .selected_project()
            .or_else(|| workspace.projects.first())
    });

    let mut header = row()
        .w_full()
        .items_center()
        .justify_between()
        .gap(12.0)
        .child(text("Home").font(HEADING_LG).color(theme::ink()));
    let start = if let Some(project) = project {
        header = header.child(
            secondary_tx("New project", |s, tx| s.choose_project_folder(tx), k)
                .key("home-new-project"),
        );
        let id = project.id.clone();
        let (done, total) = s.context_done(&id);
        ui::section(
            k,
            "home-project",
            "Continue with",
            [ui::setting_row(
                &project.name,
                Some(format!("{done} of {total} context items ready")),
                primary("New chat", move |s| s.new_chat(&id), k).key("home-new-chat"),
            )],
        )
    } else {
        ui::rows_card(
            k,
            "home-project",
            [ui::empty_state(
                Some(&icons::FOLDER),
                "No projects yet",
                "Add a project folder to start a chat and collect context.",
                Some(
                    primary_tx(
                        "Choose project folder",
                        |s, tx| s.choose_project_folder(tx),
                        k,
                    )
                    .key("home-new-project"),
                ),
            )],
        )
    };

    let profile_action = if s.profile_form.details_count() == 0 {
        "Complete profile"
    } else {
        "Edit profile"
    };
    let body = ui::page("home-page")
        .child(header)
        .child(start)
        .child(ui::section(
            k,
            "home-profile",
            "Profile",
            [ui::setting_row(
                "Developer profile",
                Some("Add details about how you work and what you want to learn.".to_string()),
                secondary(profile_action, CoderuneApp::open_profile_details, k)
                    .key("home-profile-action"),
            )],
        ));
    body.children(status_alerts(s))
}
