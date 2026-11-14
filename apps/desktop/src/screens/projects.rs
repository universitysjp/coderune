//! The projects screen.

#![allow(unused_imports)]

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, BODY_SM, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, Cx, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit, TextInput};

pub(crate) fn projects(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let projects = workspace
        .map(|workspace| workspace.projects.clone())
        .unwrap_or_default();
    let selected = workspace.and_then(|workspace| workspace.selected_project_id.clone());
    let mut rows = Vec::new();
    for project in &projects {
        let active = selected.as_deref() == Some(project.id.as_str());
        let id = project.id.clone();
        rows.push(
            col()
                .key(format!("project-{}", project.id))
                .hit()
                .focusable(true)
                .role("button")
                .label(&project.name)
                .w_full()
                .padding(12.0)
                .gap(4.0)
                .radius(6.0)
                .when(active, |this| this.fill(theme::accent_soft()))
                .when(!active, |this| this.hover(theme::hover()))
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(
                            text(&project.name)
                                .font(LABEL)
                                .color(theme::ink())
                                .a11y_hidden(),
                        )
                        .child(
                            Badge::new(format!("v{}", project.version))
                                .build(k)
                                .a11y_hidden(),
                        ),
                )
                .child(
                    text(&project.root_path)
                        .font(CODE_SM)
                        .color(theme::faint())
                        .a11y_hidden(),
                )
                .on_press(move |s: &mut CoderuneApp| s.load_project_form(&id)),
        );
    }
    if rows.is_empty() {
        rows.push(ui::card(
            None,
            None,
            vec![ui::subtle("No projects yet. Create one on the right.")],
        ));
    }
    let editing = s.project_form.editing.is_some();
    let confirm_delete = s.confirm_delete_project.is_some();
    let mut form_children = vec![
        ui::field(
            "Project name",
            None,
            text_input(
                &s.project_form.name,
                "Project name",
                "Project name",
                |s| &mut s.project_form.name,
                k,
            ),
        ),
        ui::field(
            "Root folder",
            Some("Absolute path. The folder must exist and be readable."),
            text_input(
                &s.project_form.root,
                "/path/to/project",
                "Root folder",
                |s| &mut s.project_form.root,
                k,
            ),
        ),
        ui::field(
            "Goals",
            Some("At least one. Separate several goals with commas."),
            text_input(
                &s.project_form.goals,
                "At least one goal",
                "Goals",
                |s| &mut s.project_form.goals,
                k,
            ),
        ),
    ];
    if confirm_delete {
        form_children.push(ui::alert(None, "Deleting a project also deletes its captures and project-linked documents. Click Confirm delete to proceed."));
    }
    if let Some(error) = &s.error {
        form_children.push(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        form_children.push(ui::alert(None, notice));
    }
    let mut actions = row().gap(8.0).child(primary(
        if editing {
            "Save new version"
        } else {
            "Create project"
        },
        |s| s.save_project(),
        k,
    ));
    if editing {
        actions = actions.child(destructive(
            if confirm_delete {
                "Confirm delete"
            } else {
                "Delete project"
            },
            |s| {
                if s.confirm_delete_project.is_some() {
                    s.delete_project();
                } else {
                    s.confirm_delete_project = s.project_form.editing.clone();
                }
            },
            k,
        ));
    } else {
        actions = actions.child(ghost("Clear", |s| s.new_project_form(), k));
    }
    form_children.push(actions);

    col()
        .w_full()
        .gap(24.0)
        .child(ui::page_header(
            "Projects",
            "Project context is stored locally and reused on later captures.",
            vec![secondary("New project", |s| s.new_project_form(), k)],
        ))
        .child(
            row()
                .w_full()
                .gap(16.0)
                .items_start()
                .child(
                    col()
                        .w(300.0)
                        .shrink()
                        .gap(4.0)
                        .child(ui::section_title("Configured", "Select a project to edit."))
                        .children(rows),
                )
                .child(col().grow(1.0).min_w(360.0).child(ui::card(
                    Some(if editing {
                        "Edit project"
                    } else {
                        "New project"
                    }),
                    Some("A project needs a name, a readable root folder, and at least one goal."),
                    form_children,
                ))),
        )
}
