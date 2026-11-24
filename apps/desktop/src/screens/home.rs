//! The home screen.

use crate::app::{CoderuneApp, View};
use crate::widgets::{primary, secondary};
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, HEADING_LG, HEADING_SM};
use rune::{col, icon, row, text, El};
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
        header = header.child(secondary("New project", new_project, k).key("home-new-project"));
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
            [col()
                .w_full()
                .items_center()
                .py(32.0)
                .gap(12.0)
                .child(
                    row()
                        .size(36.0, 36.0)
                        .center()
                        .radius(k.radius.md)
                        .fill(theme::code())
                        .child(icon(&icons::FOLDER, theme::ICON).color(theme::muted()))
                        .a11y_hidden(),
                )
                .child(text("No projects yet").font(HEADING_SM).color(theme::ink()))
                .child(ui::subtle(
                    "Add a project folder to start a chat and collect context.",
                ))
                .child(primary("Create project", new_project, k).key("home-new-project"))],
        )
    };

    let profile_action = if s.profile_form.details_count() == 0 {
        "Complete profile"
    } else {
        "Edit profile"
    };
    let mut body = ui::page("home-page")
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
    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    body
}

fn new_project(s: &mut CoderuneApp) {
    s.new_project_form();
    s.navigate(View::Projects);
}
