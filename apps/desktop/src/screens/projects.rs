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
    let mut list = col().w_full().gap(4.0);
    for project in projects {
        let active = selected == Some(project.id.as_str());
        let id = project.id.clone();
        list = list.child(
            col()
                .key(format!("project-{}", project.id))
                .hit()
                .focusable(true)
                .focus_ring(2.0, -2.0, theme::ink())
                .role("button")
                .label(&project.name)
                .w_full()
                .padding(12.0)
                .gap(4.0)
                .radius(6.0)
                .when(active, |this| this.fill(theme::accent_soft()))
                .when(!active, |this| this.hover(theme::pressed()))
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(
                            text(&project.name)
                                .font(LABEL)
                                .color(theme::ink())
                                .grow(1.0)
                                .min_w(0.0)
                                .truncate()
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
                        .truncate()
                        .a11y_hidden(),
                )
                .on_press(move |s: &mut CoderuneApp| s.load_project_form(&id)),
        );
    }
    if projects.is_empty() {
        list = list.child(ui::subtle("No projects yet."));
    }
    let editing = s.project_form.editing.is_some();
    let confirm_delete = s.confirm_delete_project.is_some();
    let identity = vec![
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
    let scope = vec![
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
            Some("Defaults are 1024 KiB per file and 20 MiB per capture."),
            row()
                .w_full()
                .gap(12.0)
                .flex_wrap()
                .child(col().grow(1.0).min_w(124.0).child(ui::field(
                    "Per file (KiB)",
                    None,
                    text_input(
                        &s.project_form.per_file_kib,
                        "1024",
                        "Per-file limit in KiB",
                        |s| &mut s.project_form.per_file_kib,
                        k,
                    ),
                )))
                .child(col().grow(1.0).min_w(124.0).child(ui::field(
                    "Per capture (MiB)",
                    None,
                    text_input(
                        &s.project_form.per_capture_mib,
                        "20",
                        "Per-capture limit in MiB",
                        |s| &mut s.project_form.per_capture_mib,
                        k,
                    ),
                ))),
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
    let mut editor = col()
        .key("project-editor")
        .grow(1.0)
        .min_w(336.0)
        .gap(16.0)
        .child(ui::card(
            Some(if editing {
                "Project details"
            } else {
                "New project details"
            }),
            None,
            identity,
        ))
        .child(ui::card(Some("Capture scope"), None, scope));
    if confirm_delete {
        editor = editor.child(ui::alert(None, "Deleting a project also deletes its captures and project-linked documents. Click Confirm delete to proceed."));
    }
    if let Some(error) = &s.error {
        editor = editor.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        editor = editor.child(ui::alert(None, notice));
    }
    let mut actions = row().flex_wrap().gap(8.0).child(primary(
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
                .flex_wrap()
                .gap(24.0)
                .items_start()
                .child(col().key("project-list").w(216.0).child(ui::card(
                    Some("Saved projects"),
                    None,
                    vec![list],
                )))
                .child(editor.child(actions)),
        )
}
