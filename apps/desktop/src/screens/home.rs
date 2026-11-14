//! The home screen.

#![allow(unused_imports)]

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, BODY_SM, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, Cx, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit, TextInput};
use std::path::PathBuf;

pub(crate) fn home(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let profile = workspace.and_then(|workspace| workspace.profile());
    let projects = workspace
        .map(|workspace| workspace.projects.len())
        .unwrap_or(0);
    let captures = workspace
        .and_then(|workspace| workspace.selected_project())
        .and_then(|project| {
            workspace.and_then(|workspace| workspace.list_captures(&project.id).ok())
        })
        .map(|captures| captures.len())
        .unwrap_or(0);

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
                    Some("Starting description used for later progress."),
                    vec![
                        text(profile.map(|p| p.display_name.as_str()).unwrap_or("Not set")).font(HEADING_XS).color(theme::ink()),
                        ui::subtle(&profile.map(|p| format!("v{} · {}", p.version, p.experience_stage.label())).unwrap_or_else(|| "Complete onboarding first".to_string())),
                    ],
                ))
                .child(ui::card(
                    Some("Projects"),
                    Some("Project context and repository scope."),
                    vec![
                        text(projects.to_string()).font(HEADING).color(theme::ink()),
                        ui::subtle(if projects == 1 { "project configured" } else { "projects configured" }),
                    ],
                ))
                .child(ui::card(
                    Some("Context captures"),
                    Some("Reviewed repository snapshots for the selected project."),
                    vec![
                        text(captures.to_string()).font(HEADING).color(theme::ink()),
                        ui::subtle("saved captures"),
                    ],
                )),
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
    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    body
}
