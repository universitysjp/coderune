//! The home screen.

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, HEADING, HEADING_XS};
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

    let mut body = col()
        .w_full()
        .gap(24.0)
        .child(ui::page_header(
            "Workspace",
            "Local context about you and the projects you work on. Nothing leaves this machine unless you start an explicit agent draft.",
            vec![secondary("Open projects", |s| s.navigate(View::Projects), k)],
        ))
        .child(
            row()
                .w_full()
                .gap(16.0)
                .items_start()
                .child(ui::card(
                    Some("Developer baseline"),
                    Some("Starting description for later progress."),
                    vec![
                        text(profile.map(|p| p.display_name.as_str()).unwrap_or("Not set")).font(HEADING_XS).color(theme::ink()),
                        ui::subtle(&profile.map(|p| format!("v{} · {}", p.version, p.experience_stage.label())).unwrap_or_else(|| "Complete onboarding first".to_string())),
                    ],
                ).grow(1.0))
                .child(ui::card(
                    Some("Projects"),
                    Some("Project context and repository scope."),
                    vec![
                        text(projects.to_string()).font(HEADING).color(theme::ink()),
                        ui::subtle(if projects == 1 { "project configured" } else { "projects configured" }),
                    ],
                ).grow(1.0))
                .child(ui::card(
                    Some("Context captures"),
                    Some("Reviewed snapshots for the selected project."),
                    vec![
                        text(captures.to_string()).font(HEADING).color(theme::ink()),
                        ui::subtle("saved captures"),
                    ],
                ).grow(1.0)),
        );

    if let Some(project) = workspace.and_then(|workspace| workspace.selected_project()) {
        let root = PathBuf::from(&project.root_path);
        let root2 = root.clone();
        body = body.child(ui::card(
            Some(&project.name),
            Some("Active project context."),
            vec![
                ui::mono_line(&project.root_path, theme::muted()),
                ui::subtle(&format!(
                    "{} goal(s) · version {}",
                    project.goals.len(),
                    project.version
                )),
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
            ],
        ));
    } else {
        body = body.child(ui::card(
            Some("No project yet"),
            Some("Create a project to capture repository context."),
            vec![
                ui::subtle("Choose a folder, name the project, and add at least one goal."),
                primary("Create project", |s| s.navigate(View::Projects), k),
            ],
        ));
    }

    let details = s.profile_form.details_count();
    body = body.child(ui::card(
        Some(if details == 0 {
            "Finish your profile when you are ready"
        } else {
            "Your profile has more to say"
        }),
        Some("Education, employment, project experience, skills, certifications, links, and learning preferences are optional."),
        vec![
            ui::subtle(if details == 0 {
                "Onboarding kept it short. Add detail later; nothing is required to start a project."
            } else {
                "You have added optional profile detail. You can keep it current from here."
            }),
            ghost("Complete profile", |s| s.open_profile_details(), k),
        ],
    ));

    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    body
}
