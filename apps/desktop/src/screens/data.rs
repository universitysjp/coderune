//! The data screen.

use crate::app::CoderuneApp;
use crate::widgets::*;
use coderune_ui::components as ui;
use rune::{col, El};
use rune_kit::Kit;
use std::path::PathBuf;

pub(crate) fn data(s: &CoderuneApp, k: &Kit) -> El {
    let data_path = s
        .workspace
        .as_ref()
        .map(|workspace| workspace.data_path().display().to_string())
        .unwrap_or_else(|| "Unavailable".to_string());
    let data_path_for_open = PathBuf::from(&data_path);
    let mut body = col()
        .w_full()
        .gap(24.0)
        .child(ui::page_header("Data", "Inspect the local database, export a JSON bundle, or remove all application data.", vec![]))
        .child(ui::card(
            Some("Local database"),
            Some("One per-user SQLite database, outside project repositories."),
            vec![
                ui::code_well(&data_path),
                secondary("Open data folder", move |s| {
                    if let Some(parent) = data_path_for_open.parent() {
                        s.open_folder(parent.to_path_buf());
                    }
                }, k),
            ],
        ))
        .child(ui::card(
            Some("Export"),
            Some("Exports the profile, every profile and project version, documents, and all captures with content."),
            vec![
                ui::field("Export path", Some("A JSON file. No data is uploaded."), text_input(&s.export_path, "/path/to/export.json", "Export path", |s| &mut s.export_path, k)),
                primary("Export JSON", |s| s.export_data(), k),
            ],
        ))
        .child(ui::card(
            Some("Delete all data"),
            Some("Removes the profile, onboarding draft, projects, documents, captures, and stored content. Schema migrations remain."),
            vec![
                ui::subtle("This cannot be undone. Export first if you need a copy."),
                destructive(if s.confirm_delete_all { "Confirm delete all" } else { "Delete all data" }, |s| {
                    if s.confirm_delete_all {
                        s.delete_all_data();
                    } else {
                        s.confirm_delete_all = true;
                    }
                }, k),
            ],
        ));
    if s.confirm_delete_all {
        body = body.child(ui::alert(Some("confirmation-required"), "Click Confirm delete all to remove every stored profile, project, document, and capture."));
    }
    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }
    body
}
