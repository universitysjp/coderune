//! The capture screen.

use crate::app::CoderuneApp;
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, BODY_SM, CODE_SM, LABEL};
use coderune_ui::WhenExt;
use rune::{col, row, text, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit};

pub(crate) fn capture(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let project = workspace
        .and_then(|workspace| workspace.selected_project())
        .cloned();
    let captures = project
        .as_ref()
        .and_then(|project| {
            workspace.and_then(|workspace| workspace.list_captures(&project.id).ok())
        })
        .unwrap_or_default();
    let mut body = col()
        .w_full()
        .gap(24.0)
        .child(ui::page_header(
            "Capture",
            "Collect repository context only on explicit request. Nothing is saved until you review and confirm the exact file list.",
            vec![
                primary("Preview capture", |s| s.preview_capture(), k),
                secondary("Refresh", |s| s.refresh_capture(), k),
            ],
        ));
    if let Some(project) = &project {
        body = body.child(ui::card(
            Some(&project.name),
            Some("Effective scope and limits for the next preview."),
            vec![
                ui::mono_line(&project.root_path, theme::muted()),
                ui::subtle(&format!(
                    "Includes: {} · Excludes: {} · .gitignore: {} · per file: {} KiB · per capture: {} MiB",
                    if project.scope.include_paths.is_empty() { "whole repository".to_string() } else { project.scope.include_paths.join(", ") },
                    if project.scope.exclude_paths.is_empty() { "none".to_string() } else { project.scope.exclude_paths.join(", ") },
                    if project.scope.respect_gitignore { "yes" } else { "no" },
                    project.limits.per_file_bytes / 1024,
                    project.limits.per_capture_bytes / (1024 * 1024),
                )),
            ],
        ));
    }
    if let Some(session) = &s.capture_session {
        let preview = &session.collected.preview;
        let mut file_rows = Vec::new();
        for file in &preview.files {
            file_rows.push(
                row()
                    .w_full()
                    .gap(8.0)
                    .items_center()
                    .py(4.0)
                    .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
                    .child(text(&file.path).font(CODE_SM).color(theme::ink()))
                    .child(Badge::new(&file.category).build(k))
                    .child(
                        text(format!("{} B", file.bytes))
                            .font(BODY_SM)
                            .color(theme::faint()),
                    ),
            );
        }
        let mut review_children = vec![
            row()
                .gap(8.0)
                .items_center()
                .child(ui::badge(&format!("{} files", preview.files.len()), k))
                .child(ui::badge(&format!("{} bytes", preview.total_bytes), k))
                .child(ui::badge(&format!("{} skipped", preview.skipped.len()), k)),
            text(&preview.fingerprint)
                .font(CODE_SM)
                .color(theme::faint())
                .wrap(),
            col()
                .w_full()
                .h(280.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .children(file_rows),
        ];
        if let Some(git) = &preview.git {
            review_children.insert(
                1,
                ui::subtle(&format!(
                    "{} · {}",
                    git.branch.clone().unwrap_or_else(|| "detached".to_string()),
                    git.summary
                )),
            );
        }
        if !preview.skipped.is_empty() {
            review_children.push(ui::section_title(
                "Skipped",
                "Every skipped path is shown with its reason.",
            ));
            review_children.push(
                col()
                    .w_full()
                    .h(160.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .children(preview.skipped.iter().map(|file| {
                        row()
                            .gap(8.0)
                            .items_center()
                            .py(4.0)
                            .child(text(&file.path).font(CODE_SM).color(theme::muted()))
                            .child(Badge::new(&file.reason).build(k))
                    })),
            );
        }
        review_children.push(
            row()
                .gap(8.0)
                .items_center()
                .child(option_chip(
                    "capture-reviewed".to_string(),
                    "I reviewed this exact file list",
                    s.capture_reviewed,
                    |s: &mut CoderuneApp| s.capture_reviewed = !s.capture_reviewed,
                ))
                .child(
                    Button::new("Save capture")
                        .variant(ButtonVariant::Contrast)
                        .control_size(ControlSize::ExtraLarge)
                        .disabled(!s.capture_reviewed)
                        .on_press(|s: &mut CoderuneApp| s.save_capture())
                        .build(k),
                ),
        );
        if let Some(error) = &s.error {
            review_children.push(ui::alert(Some(error.code.as_str()), &error.message));
        }
        if let Some(notice) = &s.notice {
            review_children.push(ui::alert(None, notice));
        }
        body = body.child(ui::card(
            Some("Review preview"),
            Some("Saving re-collects the repository. If anything changed, you must preview again."),
            review_children,
        ));
    } else {
        if let Some(error) = &s.error {
            body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
        }
        if let Some(notice) = &s.notice {
            body = body.child(ui::alert(None, notice));
        }
    }

    let mut capture_rows = Vec::new();
    for capture in &captures {
        let id = capture.id.clone();
        let inspect_id = id.clone();
        let delete_id = id.clone();
        let confirming = s.confirm_delete_capture.as_deref() == Some(id.as_str());
        capture_rows.push(
            row()
                .w_full()
                .flex_wrap()
                .gap(8.0)
                .items_center()
                .padding(12.0)
                .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
                .child(
                    text(format!("v{}", capture.version))
                        .font(LABEL)
                        .color(theme::ink()),
                )
                .child(Badge::new(format!("{} files", capture.files.len())).build(k))
                .child(Badge::new(format!("{} bytes", capture.total_bytes)).build(k))
                .child(
                    text(&capture.created_at)
                        .font(BODY_SM)
                        .color(theme::faint()),
                )
                .child(rune::el().grow(1.0))
                .child(
                    Button::new("Inspect")
                        .variant(ButtonVariant::Ghost)
                        .on_press(move |s: &mut CoderuneApp| s.inspect_capture(&inspect_id))
                        .build(k),
                )
                .child(
                    Button::new(if confirming { "Confirm" } else { "Delete" })
                        .variant(ButtonVariant::Danger)
                        .on_press(move |s: &mut CoderuneApp| {
                            if s.confirm_delete_capture.as_deref() == Some(delete_id.as_str()) {
                                s.delete_capture(&delete_id);
                            } else {
                                s.confirm_delete_capture = Some(delete_id.clone());
                            }
                        })
                        .build(k),
                ),
        );
    }
    if capture_rows.is_empty() {
        capture_rows.push(ui::subtle("No captures for this project yet."));
    }
    body = body.child(ui::card(
        Some("Capture history"),
        Some("Older captures remain immutable when the profile or project changes."),
        capture_rows,
    ));

    if let Some(detail) = &s.capture_detail {
        let selected_content = detail
            .selected_path
            .as_ref()
            .and_then(|path| detail.contents.get(path).cloned())
            .unwrap_or_default();
        let mut file_buttons = Vec::new();
        for file in &detail.capture.files {
            let path = file.path.clone();
            let selected = detail.selected_path.as_deref() == Some(path.as_str());
            file_buttons.push(
                row()
                    .key(format!("detail-file-{}", file.path))
                    .hit()
                    .focusable(true)
                    .role("button")
                    .label(&file.path)
                    .px(8.0)
                    .py(4.0)
                    .radius(6.0)
                    .when(selected, |this| this.fill(theme::accent_soft()))
                    .when(!selected, |this| this.hover(theme::hover()))
                    .child(
                        text(&file.path)
                            .font(CODE_SM)
                            .color(theme::ink())
                            .a11y_hidden(),
                    )
                    .on_press(move |s: &mut CoderuneApp| {
                        if let Some(detail) = s.capture_detail.as_mut() {
                            detail.selected_path = Some(path.clone());
                        }
                    }),
            );
        }
        body = body.child(ui::card(
            Some(&format!("Capture v{}", detail.capture.version)),
            Some("Saved content is read back from the local content store by hash."),
            vec![
                col()
                    .w_full()
                    .h(200.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .children(file_buttons),
                ui::code_well_tall(if selected_content.is_empty() {
                    "No stored content for this path.".to_string()
                } else {
                    selected_content
                }),
            ],
        ));
    }
    body
}
