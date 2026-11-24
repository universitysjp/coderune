//! The Settings dialog and its section navigation.

use crate::app::{CoderuneApp, SettingsSection};
use crate::updates::Status;
use crate::widgets::text_input;
use coderune_ui::{components as ui, components::PushButtonExt, icons, theme};
use rune::{col, el, row, text, El, ScrollSpec};
use rune_kit::{Button, ButtonVariant, Dialog, Kit, Switch};

pub(crate) fn dialog(s: &CoderuneApp, section: SettingsSection, k: &Kit, overlays: &mut Vec<El>) {
    let nav = col()
        .w(156.0)
        .h_full()
        .fill(theme::surface())
        .padding(12.0)
        .gap(4.0)
        .children(
            [
                (
                    SettingsSection::General,
                    "settings-general",
                    "General",
                    &icons::GEAR,
                ),
                (
                    SettingsSection::Storage,
                    "settings-storage",
                    "Storage",
                    &icons::FOLDER,
                ),
                (
                    SettingsSection::Agents,
                    "settings-agents",
                    "Agents",
                    &icons::ROBOT,
                ),
            ]
            .into_iter()
            .map(|(destination, key, label, icon)| {
                let selected = section == destination;
                ui::nav_item(key, label, icon, selected, move |s: &mut CoderuneApp| {
                    s.select_settings(destination);
                })
                .focus_selected(selected)
            }),
        );
    let content = match section {
        SettingsSection::General => general(s, k),
        SettingsSection::Storage => storage(s, k),
        SettingsSection::Agents => super::coding_agents::coding_agents(s, k),
    };
    let mut panel = col().w_full().padding(20.0).gap(16.0).child(content);
    if let Some(feedback) = &s.settings_feedback {
        panel = panel.child(match feedback {
            Ok(message) => ui::alert(None, message),
            Err(error) => ui::alert(Some(error.code.as_str()), &error.message),
        });
    }
    let mut dialog_kit = k.clone();
    dialog_kit.c.layer_01 = theme::surface();
    Dialog::new("settings", "Settings")
        .open(true)
        .width(760.0)
        .fit()
        .on_close(CoderuneApp::close_settings)
        .child(
            row()
                .w_full()
                .h(460.0)
                .clip_rounded(6.0)
                .child(nav)
                .child(el().w(1.0).h_full().fill(theme::row_line()))
                .child(
                    col()
                        .key(match section {
                            SettingsSection::General => "general-panel",
                            SettingsSection::Storage => "storage-panel",
                            SettingsSection::Agents => "agents-panel",
                        })
                        .grow(1.0)
                        .min_w(0.0)
                        .h_full()
                        .fill(theme::panel())
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(panel),
                ),
        )
        .build(&dialog_kit, overlays);
}

fn general(s: &CoderuneApp, k: &Kit) -> El {
    let busy = matches!(s.updates.status, Status::Checking | Status::Downloading);
    let check = Button::new("Check for updates")
        .disabled(busy)
        .on_press(|s: &mut CoderuneApp| s.updates.check())
        .build_push(k)
        .key("check-updates");
    let mut body = col().w_full().gap(12.0).child(setting_row(
        "Application updates",
        Some(format!("Installed version {}", crate::VERSION)),
        check,
    ));
    let status = match &s.updates.status {
        Status::Idle => None,
        Status::Checking => Some(ui::subtle("Checking for updates…")),
        Status::Current => Some(ui::subtle("Coderune is up to date.")),
        Status::NoRelease => Some(ui::subtle("No desktop release has been published yet.")),
        Status::Available(update) => Some(
            Button::new(format!("Download Coderune {}", update.version))
                .variant(ButtonVariant::Contrast)
                .on_press(|s: &mut CoderuneApp| s.updates.download())
                .build_push(k),
        ),
        Status::Downloading => Some(ui::subtle("Downloading and verifying the update…")),
        Status::Downloaded(folder) => {
            let folder = folder.clone();
            Some(setting_row(
                "Update downloaded",
                Some("Close Coderune before installing the update.".to_string()),
                Button::new("Open download folder")
                    .on_press(move |s: &mut CoderuneApp| s.open_settings_folder(&folder))
                    .build_push(k),
            ))
        }
        Status::Failed(message) => Some(ui::alert(None, message)),
    };
    if let Some(status) = status {
        body = body.child(col().w_full().items_start().child(status));
    }
    body.child(divider()).child(setting_row(
        "Dark mode",
        None,
        Switch::new(s.dark_mode())
            .label("Dark mode")
            .hide_label()
            .on_toggle(CoderuneApp::toggle_dark_mode)
            .build(k)
            .key("dark-mode"),
    ))
}

fn storage(s: &CoderuneApp, k: &Kit) -> El {
    let data_path = s.workspace.as_ref().map(|workspace| workspace.data_path());
    let folder = data_path
        .as_deref()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf);
    let open = Button::new("Open data folder")
        .disabled(folder.is_none())
        .on_press(move |s: &mut CoderuneApp| {
            if let Some(folder) = &folder {
                s.open_settings_folder(folder);
            }
        })
        .build_push(k)
        .key("open-data-folder");
    let export = Button::new("Export JSON")
        .variant(ButtonVariant::Contrast)
        .on_press(CoderuneApp::export_data)
        .build_push(k)
        .key("export-data");
    let delete = Button::new("Delete all data")
        .variant(ButtonVariant::Danger)
        .disabled(s.confirm_delete_all)
        .on_press(|s: &mut CoderuneApp| {
            s.confirm_delete_all = true;
            s.settings_feedback = None;
        })
        .build_push(k)
        .key("delete-data");

    let mut database = col()
        .w_full()
        .gap(4.0)
        .child(setting_row("Local database", None, open));
    if let Some(path) = data_path {
        database = database.child(
            text(path.display().to_string())
                .font(theme::CODE_SM)
                .color(theme::muted())
                .truncate(),
        );
    }
    let mut body = col()
        .w_full()
        .gap(16.0)
        .child(database)
        .child(divider())
        .child(
            col()
                .w_full()
                .gap(10.0)
                .child(setting_row(
                    "Export data",
                    Some(
                        "Save your profile, projects, documents, and captures as JSON.".to_string(),
                    ),
                    export,
                ))
                .child(ui::field(
                    "Export path",
                    None,
                    text_input(
                        &s.export_path,
                        "/path/to/export.json",
                        "Export path",
                        |s| &mut s.export_path,
                        k,
                    ),
                )),
        )
        .child(divider())
        .child(setting_row(
            "Delete all data",
            Some(
                "Permanently removes your profile, projects, documents, and captures.".to_string(),
            ),
            delete,
        ));
    if s.confirm_delete_all {
        body = body.child(setting_row(
            "This action cannot be undone.",
            None,
            row()
                .gap(8.0)
                .child(
                    Button::new("Cancel")
                        .on_press(|s: &mut CoderuneApp| s.confirm_delete_all = false)
                        .build_push(k)
                        .key("cancel-delete-data"),
                )
                .child(
                    Button::new("Confirm delete all")
                        .variant(ButtonVariant::Danger)
                        .on_press(CoderuneApp::delete_all_data)
                        .build_push(k)
                        .key("confirm-delete-data"),
                ),
        ));
    }
    body
}

fn setting_row(title: &str, detail: Option<String>, action: El) -> El {
    let mut title = col().grow(1.0).min_w(0.0).gap(4.0).child(
        text(title)
            .font(theme::HEADING_XS)
            .color(theme::ink())
            .wrap(),
    );
    if let Some(detail) = detail {
        title = title.child(ui::subtle(&detail));
    }
    row()
        .w_full()
        .items_center()
        .gap(16.0)
        .child(title)
        .child(action)
}

fn divider() -> El {
    el().w_full().h(1.0).fill(theme::row_line())
}
