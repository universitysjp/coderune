//! The home screen.

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, HEADING_XS, LABEL};
use rune::{col, row, text, El};
use rune_kit::Kit;
use std::path::PathBuf;

pub(crate) fn home(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let profile = workspace.and_then(|workspace| workspace.profile());
    let projects = workspace
        .map(|workspace| workspace.projects.len())
        .unwrap_or(0);
    let captures = s.captures.len();

    let baseline = col()
        .items_end()
        .gap(2.0)
        .child(
            text(
                profile
                    .map(|p| p.display_name.as_str())
                    .unwrap_or("Not set"),
            )
            .font(LABEL)
            .color(theme::ink()),
        )
        .child(ui::tiny(
            &profile
                .map(|p| format!("v{} · {}", p.version, p.experience_stage.label()))
                .unwrap_or_else(|| "Complete onboarding first".to_string()),
        ));
    let mut body = ui::page("home-page")
        .child(ui::page_header(
            "Workspace",
            "Local context about you and the projects you work on. Nothing leaves this machine unless you start an explicit agent draft.",
            vec![secondary("Open projects", |s| s.navigate(View::Projects), k)],
        ))
        .child(ui::section(
            k,
            "home-summary",
            "Workspace",
            [
                ui::setting_row(
                    "Developer baseline",
                    Some("Starting description for later progress.".to_string()),
                    baseline,
                ),
                ui::setting_row(
                    "Projects",
                    Some(format!(
                        "{} configured",
                        if projects == 1 { "project" } else { "projects" }
                    )),
                    text(projects.to_string())
                        .font(HEADING_XS)
                        .color(theme::ink()),
                ),
                ui::setting_row(
                    "Context captures",
                    Some("Reviewed snapshots for the selected project.".to_string()),
                    text(captures.to_string())
                        .font(HEADING_XS)
                        .color(theme::ink()),
                ),
            ],
        ));

    if let Some(project) = workspace.and_then(|workspace| workspace.selected_project()) {
        let root = PathBuf::from(&project.root_path);
        let root2 = root.clone();
        body = body.child(ui::section(
            k,
            "home-project",
            "Project",
            [col()
                .w_full()
                .gap(10.0)
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(text(&project.name).font(HEADING_XS).color(theme::ink()))
                        .child(ui::badge(&format!("v{}", project.version), k)),
                )
                .child(ui::mono_line(&project.root_path, theme::muted()))
                .child(ui::subtle(&format!(
                    "{} goal(s) · active project context",
                    project.goals.len()
                )))
                .child(
                    row()
                        .gap(8.0)
                        .child(secondary(
                            "Open folder",
                            move |s| s.open_folder(root.clone()),
                            k,
                        ))
                        .child(ghost(
                            "Open in VS Code",
                            move |s| s.open_editor("code", root2.clone()),
                            k,
                        )),
                )],
        ));
    } else {
        body = body.child(ui::section(
            k,
            "home-project",
            "Project",
            [col()
                .w_full()
                .gap(10.0)
                .child(text("No project yet").font(LABEL).color(theme::ink()))
                .child(ui::subtle(
                    "Choose a folder, name the project, and add at least one goal.",
                ))
                .child(primary("Create project", |s| s.navigate(View::Projects), k))],
        ));
    }

    let details = s.profile_form.details_count();
    body = body.child(ui::section(
        k,
        "home-profile",
        "Profile",
        [col()
            .w_full()
            .gap(10.0)
            .child(
                text(if details == 0 {
                    "Finish your profile when you are ready"
                } else {
                    "Your profile has more to say"
                })
                .font(LABEL)
                .color(theme::ink()),
            )
            .child(ui::subtle(if details == 0 {
                "Onboarding kept it short. Add detail later; nothing is required to start a project."
            } else {
                "You have added optional profile detail. You can keep it current from here."
            }))
            .child(row().child(ghost(
                "Complete profile",
                |s| s.open_profile_details(),
                k,
            )))],
    ));

    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    body
}
