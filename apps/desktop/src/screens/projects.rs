//! Project creation and settings. Project navigation lives in the sidebar.

use std::path::PathBuf;

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_domain::Project;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, CODE_SM, LABEL};
use rune::{col, icon, row, text, El};
use rune_kit::{Accordion, AccordionItem, Dialog, Kit, Progress};

pub(crate) fn projects(s: &CoderuneApp, k: &Kit) -> El {
    let project = s.project_form.editing.as_deref().and_then(|id| {
        s.workspace
            .as_ref()?
            .projects
            .iter()
            .find(|project| project.id == id)
    });
    let (title, description, save_label) = match project {
        Some(project) => (
            "Project settings",
            format!("{} · v{}", project.name, project.version),
            "Save new version",
        ),
        None => (
            "New project",
            if s.project_form.folder().is_some() {
                "Add a goal to finish, or leave the rest for later.".to_string()
            } else {
                "Pick a folder with the add button in the sidebar.".to_string()
            },
            "Create project",
        ),
    };
    let mut actions = vec![primary(save_label, |s| s.save_project(), k).key("save-project")];
    if project.is_none() {
        actions.insert(0, ghost("Clear", |s| s.new_project_form(), k));
    }
    let mut page = ui::page("projects-page")
        .max_w(640.0)
        .child(ui::page_header(title, &description, actions));
    if let Some(error) = &s.error {
        page = page.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        page = page.child(ui::alert(None, notice));
    }
    if let Some(project) = project {
        page = page.child(context_progress(s, project, k));
    }
    page = page
        .child(project_details(s, project, k))
        .child(project_goals(s, k));
    if project.is_some() {
        page = page.child(capture_scope(s, k)).child(delete_project(s, k));
    }
    page
}

/// The quick questions asked right after a folder is picked. Creating the
/// project needs a name and one goal; everything else can wait for the panel.
pub(crate) fn dialog(s: &CoderuneApp, k: &Kit, overlays: &mut Vec<El>) {
    if s.project_form.editing.is_some() {
        return;
    }
    let folder = s.project_form.folder();
    let folder_name = folder
        .as_deref()
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("No folder chosen");
    let folder_detail = folder
        .as_deref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let mut copy = col().grow(1.0).min_w(0.0).gap(2.0).child(
        text(folder_name)
            .font(LABEL)
            .color(theme::ink())
            .truncate(),
    );
    if !folder_detail.is_empty() {
        copy = copy.child(
            text(folder_detail)
                .font(CODE_SM)
                .color(theme::faint())
                .truncate(),
        );
    }
    let mut body = col()
        .w_full()
        .padding(16.0)
        .gap(16.0)
        .child(
            row()
                .w_full()
                .gap(10.0)
                .items_center()
                .child(icon(&icons::FOLDER, theme::ICON).color(theme::muted()).a11y_hidden())
                .child(copy),
        )
        .child(ui::field(
            "Project name",
            None,
            text_input(
                &s.project_form.name,
                "Project name",
                "Project name",
                |s| &mut s.project_form.name,
                k,
            )
            .key("project-dialog-name"),
        ))
        .child(ui::field(
            "What are you building?",
            Some("One goal is enough. Separate more with commas."),
            text_input(
                &s.project_form.goals,
                "Ship the native desktop app",
                "Goals",
                |s| &mut s.project_form.goals,
                k,
            )
            .key("project-dialog-goals"),
        ))
        .child(
            Accordion::new()
                .item(
                    AccordionItem::new(if s.project_scope_expanded {
                        "Hide capture options"
                    } else {
                        "Capture options"
                    })
                    .expanded(s.project_scope_expanded)
                    .on_toggle(|s: &mut CoderuneApp| {
                        s.project_scope_expanded = !s.project_scope_expanded;
                    })
                    .child(capture_fields(s, k)),
                )
                .build(k)
                .key("project-dialog-scope"),
        );
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    body = body.child(
        row()
            .w_full()
            .gap(8.0)
            .items_center()
            .child(rune::el().grow(1.0))
            .child(ghost("Not now", |s| s.close_project_dialog(), k).key("project-dialog-later"))
            .child(primary(
                "Create project",
                |s| s.create_project_from_dialog(),
                k,
            )
            .key("project-dialog-create")),
    );
    Dialog::new("new-project", "New project")
        .open(true)
        .description("Coderune reads this folder only when you ask it to capture context. The rest can wait.")
        .width(520.0)
        .fit()
        .on_close(CoderuneApp::close_project_dialog)
        .child(body)
        .build(k, overlays);
}

fn context_progress(s: &CoderuneApp, project: &Project, k: &Kit) -> El {
    let items = s.context_items(&project.id);
    let total = items.len();
    let missing: Vec<_> = items
        .iter()
        .filter(|item| !item.done)
        .map(|item| item.label)
        .collect();
    let done = total - missing.len();
    let summary = if missing.is_empty() {
        "All context items are ready.".to_string()
    } else {
        format!("Still to add · {}", missing.join(", "))
    };
    let context = col()
        .w_full()
        .py(ui::ROW_PAD)
        .gap(8.0)
        .child(
            Progress::new(Some(done as f32 / total.max(1) as f32))
                .label(format!("{done} of {total} context items ready"))
                .build(k),
        )
        .child(ui::subtle(&summary));
    let captured = items
        .iter()
        .find(|item| item.label == "First capture")
        .is_some_and(|item| item.done);
    let (detail, action) = if captured {
        ("Context is saved for this project.", "View captures")
    } else {
        ("Review and save the repository context.", "Start a capture")
    };
    ui::section(
        k,
        "project-context",
        "Context readiness",
        [
            context,
            ui::setting_row(
                "First capture",
                Some(detail.to_string()),
                secondary(action, |s| s.navigate(View::Capture), k).key("project-capture"),
            )
            .py(6.0),
        ],
    )
}

fn project_details(s: &CoderuneApp, project: Option<&Project>, k: &Kit) -> El {
    let folder = s.project_form.folder();
    let folder_name = folder
        .as_deref()
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("No folder chosen");
    let folder_detail = folder
        .as_deref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Pick the folder Coderune should read context from.".to_string());
    let mut folder_actions = row()
        .flex_wrap()
        .gap(8.0)
        .child(
            secondary_tx("Choose folder", |s, tx| s.change_project_folder(tx), k)
                .key("project-choose-folder"),
        );
    if let Some(project) = project {
        let root = PathBuf::from(&project.root_path);
        let editor_folder = root.clone();
        folder_actions = folder_actions
            .child(
                secondary("Open folder", move |s| s.open_folder(root.clone()), k)
                    .key("open-project-folder"),
            )
            .child(
                secondary(
                    "Open in VS Code",
                    move |s| s.open_editor("code", editor_folder.clone()),
                    k,
                )
                .key("open-project-editor"),
            );
    }
    let folder_row = col()
        .w_full()
        .gap(8.0)
        .child(
            row()
                .key("project-folder")
                .w_full()
                .h(32.0)
                .px(10.0)
                .gap(8.0)
                .items_center()
                .radius(6.0)
                .border(1.0, theme::control_line())
                .fill(theme::panel())
                .child(
                    icon(&icons::FOLDER, theme::ICON)
                        .color(theme::muted())
                        .a11y_hidden(),
                )
                .child(text(folder_name).font(LABEL).color(theme::ink()).truncate())
                .child(
                    text(&folder_detail)
                        .font(CODE_SM)
                        .color(theme::faint())
                        .truncate()
                        .grow(1.0),
                ),
        )
        .child(folder_actions);
    let fields = col()
        .w_full()
        .py(ui::ROW_PAD)
        .gap(12.0)
        .child(ui::field(
            "Project name",
            None,
            text_input(
                &s.project_form.name,
                "Project name",
                "Project name",
                |s| &mut s.project_form.name,
                k,
            )
            .key("project-name"),
        ))
        .child(ui::field(
            "Project folder",
            Some("Coderune reads this folder only when you ask it to capture context."),
            folder_row,
        ));
    ui::section(k, "project-identity", "Project details", [fields])
}

fn project_goals(s: &CoderuneApp, k: &Kit) -> El {
    ui::section(
        k,
        "project-goals",
        "Goals",
        [ui::field(
            "What are you building?",
            Some("Add at least one goal. Separate multiple goals with commas."),
            text_input(
                &s.project_form.goals,
                "Ship the native desktop app",
                "Goals",
                |s| &mut s.project_form.goals,
                k,
            )
            .key("project-goals-input"),
        )
        .py(ui::ROW_PAD)],
    )
}

/// The scope and limit fields, shared by the panel's disclosure and the new
/// project dialog.
fn capture_fields(s: &CoderuneApp, k: &Kit) -> El {
    col()
        .w_full()
        .padding(ui::ROW_PAD)
        .gap(12.0)
        .child(ui::field(
            "Include paths",
            Some("Use comma-separated paths relative to the root."),
            text_input(
                &s.project_form.include_paths,
                "src, docs",
                "Include paths",
                |s| &mut s.project_form.include_paths,
                k,
            )
            .key("project-include-paths"),
        ))
        .child(ui::field(
            "Exclude paths",
            Some("Use comma-separated paths, directory names, or glob patterns."),
            text_input(
                &s.project_form.exclude_paths,
                "generated, dist",
                "Exclude paths",
                |s| &mut s.project_form.exclude_paths,
                k,
            ),
        ))
        .child(ui::field(
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
        ))
        .child(ui::field(
            "Gitignore",
            Some("Only the root .gitignore is read."),
            option_chip(
                "project-gitignore".to_string(),
                "Respect .gitignore",
                s.project_form.respect_gitignore,
                |s: &mut CoderuneApp| {
                    s.project_form.respect_gitignore = !s.project_form.respect_gitignore
                },
            ),
        ))
}

fn capture_scope(s: &CoderuneApp, k: &Kit) -> El {
    ui::section(
        k,
        "project-scope",
        "Capture scope",
        [col()
            .w_full()
            .py(ui::ROW_PAD)
            .gap(8.0)
            .child(ui::subtle(
                "Captures default to allowed files across the repository. Set include or exclude paths to narrow the scope.",
            ))
            .child(
                Accordion::new()
                    .item(
                        AccordionItem::new(if s.project_scope_expanded {
                            "Hide paths and limits"
                        } else {
                            "Show paths and limits"
                        })
                        .expanded(s.project_scope_expanded)
                        .on_toggle(|s: &mut CoderuneApp| {
                            s.project_scope_expanded = !s.project_scope_expanded;
                        })
                        .child(capture_fields(s, k)),
                    )
                    .build(k)
                    .key("project-scope-disclosure"),
            )],
    )
}

fn delete_project(s: &CoderuneApp, k: &Kit) -> El {
    let action = if s.confirm_delete_project.is_some() {
        row()
            .key("confirm-project-delete")
            .flex_wrap()
            .gap(8.0)
            .child(ghost("Cancel", |s| s.confirm_delete_project = None, k))
            .child(destructive("Confirm delete", |s| s.delete_project(), k))
    } else {
        secondary(
            "Delete project",
            |s| s.confirm_delete_project = s.project_form.editing.clone(),
            k,
        )
        .key("request-project-delete")
    };
    ui::section(
        k,
        "project-delete",
        "Manage project",
        [ui::setting_row(
            if s.confirm_delete_project.is_some() {
                "Permanently delete this project?"
            } else {
                "Remove this project from Coderune"
            },
            Some(
                "This also deletes its chats, captures, and project-linked documents.".to_string(),
            ),
            action,
        )
        .py(6.0)],
    )
}
