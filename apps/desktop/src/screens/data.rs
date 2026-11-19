//! The data screen.

use crate::app::CoderuneApp;
use crate::updates::Status;
use crate::widgets::*;
use coderune_ui::components as ui;
use rune::{col, El};
use rune_kit::{Kit, TextInput};
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
        .child(ui::page_header("Settings", "Check for updates and manage local data.", vec![]))
        .child(updates_card(s, k))
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

fn updates_card(s: &CoderuneApp, k: &Kit) -> El {
    let token = TextInput::new(&s.updates.token)
        .secure()
        .placeholder("GitHub token")
        .label("GitHub token")
        .width(None)
        .height(32.0)
        .on_edit(|s: &mut CoderuneApp| &mut s.updates.token)
        .build(k);
    let mut controls = vec![ui::field(
        "GitHub token",
        Some("For private releases, use a token with read access to this repository's contents. It stays in this session."),
        token,
    )];
    match &s.updates.status {
        Status::Idle => {}
        Status::Checking => controls.push(ui::subtle("Checking for updates…")),
        Status::Current => controls.push(ui::subtle("Coderune is up to date.")),
        Status::Available(update) => controls.push(primary(
            &format!("Download Coderune {}", update.version),
            |s| s.updates.download(),
            k,
        )),
        Status::Downloading => controls.push(ui::subtle("Downloading and verifying the update…")),
        Status::Downloaded(folder) => {
            let folder = folder.clone();
            controls.push(ui::subtle("The package is ready to install. Close Coderune before replacing the current version."));
            controls.push(primary(
                "Open download folder",
                move |s| s.open_folder(folder.clone()),
                k,
            ));
        }
        Status::Failed(message) => controls.push(ui::alert(None, message)),
    }
    if !matches!(s.updates.status, Status::Checking | Status::Downloading) {
        controls.push(secondary("Check for updates", |s| s.updates.check(), k));
    }
    ui::card(
        Some("Application updates"),
        Some(&format!("Installed version {}", crate::VERSION)),
        controls,
    )
}
