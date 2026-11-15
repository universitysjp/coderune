//! The projects screen.

use crate::app::CoderuneApp;
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, CODE_SM, LABEL};
use coderune_ui::WhenExt;
use rune::{col, row, text, El};
use rune_kit::{Badge, Kit};

pub(crate) fn projects(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let projects = workspace
        .map(|workspace| workspace.projects.as_slice())
        .unwrap_or(&[]);
    let selected = s.project_form.editing.as_deref();
    let mut rows = Vec::new();
    for project in projects {
        let active = selected == Some(project.id.as_str());
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
        ui::field(
            "Include paths",
            Some("Empty means the whole repository. Comma separated, relative to the root."),
            text_input(
                &s.project_form.include_paths,
                "src, docs",
                "Include paths",
                |s| &mut s.project_form.include_paths,
                k,
            ),
        ),
        ui::field(
            "Exclude paths",
            Some("Comma separated paths, directory names, or glob patterns."),
            text_input(
                &s.project_form.exclude_paths,
                "generated, dist",
                "Exclude paths",
                |s| &mut s.project_form.exclude_paths,
                k,
            ),
        ),
        ui::field(
            "Capture limits",
            Some("KiB per file and MiB per capture. Defaults are 1024 KiB and 20 MiB."),
            row()
                .gap(8.0)
                .items_center()
                .child(
                    text_input(
                        &s.project_form.per_file_kib,
                        "1024",
                        "Per-file limit in KiB",
                        |s| &mut s.project_form.per_file_kib,
                        k,
                    )
                    .w(160.0),
                )
                .child(
                    text_input(
                        &s.project_form.per_capture_mib,
                        "20",
                        "Per-capture limit in MiB",
                        |s| &mut s.project_form.per_capture_mib,
                        k,
                    )
                    .w(160.0),
                ),
        ),
        ui::field(
            "Gitignore",
            Some("Only the root .gitignore is read."),
            row().gap(8.0).child(option_chip(
                "project-gitignore".to_string(),
                "Respect .gitignore",
                s.project_form.respect_gitignore,
                |s: &mut CoderuneApp| {
                    s.project_form.respect_gitignore = !s.project_form.respect_gitignore
                },
            )),
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
