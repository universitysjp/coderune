//! The Settings dialog and its section navigation.

use crate::app::{CoderuneApp, SettingsSection, VIEW_SHORTCUTS};
use crate::updates::Status;
use crate::widgets::text_input;
use coderune_ui::{components as ui, components::PushButtonExt, icons, theme};
use rune::{col, el, row, text, El, ScrollSpec};
use rune_kit::{
    Button, ButtonVariant, ControlSize, IconButton, IconButtonVariant, Kit, Placement, Select,
    SelectAppearance, Switch,
};

pub(crate) fn dialog(s: &CoderuneApp, section: SettingsSection, k: &Kit, overlays: &mut Vec<El>) {
    let nav = col().w(166.0).h_full().px(10.0).py(8.0).gap(4.0).children(
        [
            (SettingsSection::General, "settings-general", &icons::GEAR),
            (SettingsSection::Storage, "settings-storage", &icons::FOLDER),
            (SettingsSection::Agents, "settings-agents", &icons::ROBOT),
            (
                SettingsSection::Shortcuts,
                "settings-shortcuts",
                &icons::KEYBOARD,
            ),
        ]
        .into_iter()
        .map(|(destination, key, icon)| {
            let selected = section == destination;
            ui::nav_item(
                key,
                destination.label(),
                icon,
                selected,
                move |s: &mut CoderuneApp| s.select_settings(destination),
            )
            .focus_selected(selected)
        }),
    );
    let mut menus = Vec::new();
    let content = match section {
        SettingsSection::General => general(s, k, &mut menus),
        SettingsSection::Storage => storage(s, k),
        SettingsSection::Agents => super::coding_agents::coding_agents(s, k),
        SettingsSection::Shortcuts => shortcuts(k),
    };
    let mut panel = col().w_full().padding(20.0).gap(16.0).child(content);
    if let Some(feedback) = &s.settings_feedback {
        panel = panel.child(match feedback {
            Ok(message) => ui::alert(None, message),
            Err(error) => ui::alert(Some(error.code.as_str()), &error.message),
        });
    }
    let close = IconButton::new(k.glyph.close)
        .variant(IconButtonVariant::Ghost)
        .control_size(ControlSize::Regular)
        .label("Close Settings")
        .on_press(CoderuneApp::close_settings)
        .build(k)
        .key("close");
    let panel_key = match section {
        SettingsSection::General => "general-panel",
        SettingsSection::Storage => "storage-panel",
        SettingsSection::Agents => "agents-panel",
        SettingsSection::Shortcuts => "shortcuts-panel",
    };
    let content = ui::surface_frame(k, theme::canvas(), k.radius.lg)
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(
            col()
                .key(panel_key)
                .role("group")
                .label(section.label())
                .w_full()
                .h_full()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .scroll_edge_fade(16.0)
                .child(panel),
        );
    let mut dialog = ui::surface_frame(k, theme::surface(), k.radius.xl)
        .key("settings-dialog")
        .role("dialog")
        .label("Settings")
        .w_full()
        .max_w(760.0)
        .h(520.0)
        .child(
            row()
                .w_full()
                .h(46.0)
                .px(14.0)
                .items_center()
                .justify_between()
                .child(text("Settings").font(theme::HEADING_XS).color(theme::ink()))
                .child(close),
        )
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .pr(8.0)
                .pb(8.0)
                .child(nav)
                .child(content),
        )
        .modal(true, CoderuneApp::close_settings)
        .token_motion(k.motion.overlay);
    for shadow in &k.elevation.overlay {
        dialog = dialog.box_shadow(*shadow);
    }
    let layer = |key| el().key(key).absolute(0.0, 0.0).w_full().h_full();
    overlays.extend([
        layer("settings-scrim")
            .fill(k.c.scrim)
            .a11y_hidden()
            .token_motion(k.motion.overlay),
        layer("settings-layer").padding(16.0).center().child(dialog),
    ]);
    // A font list opens above the dialog that owns it.
    overlays.extend(menus);
}

fn general(s: &CoderuneApp, k: &Kit, menus: &mut Vec<El>) -> El {
    let busy = matches!(
        s.updates.status,
        Status::Checking | Status::Downloading | Status::Ready(_)
    );
    let check = Button::new("Check for updates")
        .disabled(busy)
        .on_press(|s: &mut CoderuneApp| s.updates.check())
        .build_push(k)
        .key("check-updates");
    let mut updates = col().w_full().child(ui::setting_row(
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
        Status::Ready(install) => {
            let version = crate::update_install::version(install).to_string();
            Some(ui::setting_row(
                "Update ready",
                Some("Restarting installs the update and reopens Coderune.".to_string()),
                Button::new(format!("Restart to update to {version}"))
                    .variant(ButtonVariant::Contrast)
                    .on_press(|s: &mut CoderuneApp| s.updates.restart())
                    .build_push(k)
                    .key("restart-update"),
            ))
        }
        Status::Manual(path) => {
            let folder = path
                .parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| path.clone());
            Some(ui::setting_row(
                "Update downloaded",
                Some(
                    "Open the downloaded package to install it, then reopen Coderune.".to_string(),
                ),
                Button::new("Open download folder")
                    .on_press(move |s: &mut CoderuneApp| s.open_settings_folder(&folder))
                    .build_push(k),
            ))
        }
        Status::Failed(message) => Some(ui::alert(None, message)),
    };
    if let Some(status) = status {
        updates = updates.child(col().w_full().pb(12.0).items_start().child(status));
    }
    let dark = ui::setting_row(
        "Dark mode",
        None,
        Switch::new(s.dark_mode())
            .label("Dark mode")
            .hide_label()
            .on_toggle(CoderuneApp::toggle_dark_mode)
            .build(theme::switch_kit(k.dark))
            .key("dark-mode"),
    );
    let ui_font = font_row(
        s,
        k,
        "ui-font",
        "Interface font",
        "Text, labels, and headings.",
        s.ui_font(),
        CoderuneApp::set_ui_font,
        menus,
    );
    let mono_font = font_row(
        s,
        k,
        "mono-font",
        "Code font",
        "Code, paths, IDs, and hashes.",
        s.mono_font(),
        CoderuneApp::set_mono_font,
        menus,
    );
    col()
        .w_full()
        .gap(ui::SECTION_GAP)
        .child(ui::section(
            k,
            "general-application",
            "Application",
            [updates],
        ))
        .child(ui::section(
            k,
            "general-appearance",
            "Appearance",
            [dark, ui_font, mono_font],
        ))
}

fn font_row(
    s: &CoderuneApp,
    k: &Kit,
    key: &'static str,
    title: &str,
    detail: &str,
    current: &str,
    choose: fn(&mut CoderuneApp, String),
    menus: &mut Vec<El>,
) -> El {
    let choices = crate::fonts::choices(Some(current));
    let open = s.font_menu == Some(key);
    let select = Select::new(key, &choices)
        .appearance(SelectAppearance::Inline)
        .current(Some(current))
        .width(220.0)
        .label(title)
        .open(open)
        .placement(Placement::BottomEnd)
        .on_toggle(move |s: &mut CoderuneApp| {
            s.font_menu = if s.font_menu == Some(key) {
                None
            } else {
                Some(key)
            };
        })
        .on_close(|s: &mut CoderuneApp| s.font_menu = None)
        .on_choose(move |s: &mut CoderuneApp, value: &str, _: &mut rune::Tx| {
            s.font_menu = None;
            choose(s, value.to_string());
        })
        .build(k, menus);
    ui::setting_row(title, Some(detail.to_string()), select)
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
        .child(ui::setting_row("Local database", None, open));
    if let Some(path) = data_path {
        database = database.child(
            text(path.display().to_string())
                .pb(12.0)
                .font(theme::CODE_SM)
                .color(theme::muted())
                .truncate(),
        );
    }
    let mut rows = vec![
        database,
        col()
            .w_full()
            .pb(12.0)
            .child(ui::setting_row(
                "Export data",
                Some("Save your profile, projects, documents, and captures as JSON.".to_string()),
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
        ui::setting_row(
            "Delete all data",
            Some(
                "Permanently removes your profile, projects, chats, documents, and captures."
                    .to_string(),
            ),
            delete,
        ),
    ];
    if s.confirm_delete_all {
        rows.push(ui::setting_row(
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
    ui::settings_card(k, "storage-options", rows)
}

fn shortcuts(k: &Kit) -> El {
    let modifier = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    };
    let view_rows = VIEW_SHORTCUTS.into_iter().map(|(key, view)| {
        let key = key.to_string();
        shortcut_row(k, view.label(), &[&[modifier, &key]])
    });
    let primary_sidebar = &["Shift", modifier, "B"][..];
    let alternate_sidebar = &["Ctrl", modifier, "S"][..];
    let sidebar_keys = if cfg!(target_os = "macos") {
        vec![primary_sidebar, alternate_sidebar]
    } else {
        vec![primary_sidebar]
    };
    col()
        .w_full()
        .gap(12.0)
        .child(ui::section(
            k,
            "navigation-shortcuts",
            "Navigation",
            view_rows,
        ))
        .child(ui::section(
            k,
            "window-shortcuts",
            "Window",
            [
                shortcut_row(k, "Open Settings", &[&[modifier, ","], &[modifier, "7"]]),
                shortcut_row(k, "Toggle sidebar", &sidebar_keys),
                shortcut_row(k, "Close Settings", &[&["Esc"]]),
                shortcut_row(k, "Quit Coderune", &[&[modifier, "Q"]]),
            ],
        ))
}

fn shortcut_row(k: &Kit, title: &str, alternatives: &[&[&str]]) -> El {
    let mut keys = row().items_center().gap(6.0);
    for (index, shortcut) in alternatives.iter().enumerate() {
        if index > 0 {
            keys = keys.child(ui::tiny("or"));
        }
        keys = keys.child(ui::keycaps(&format!("keys-{index}"), shortcut, k));
    }
    let description = alternatives
        .iter()
        .map(|shortcut| shortcut.join(" + "))
        .collect::<Vec<_>>()
        .join(" or ");
    ui::setting_row(title, None, keys)
        .key(title)
        .role("group")
        .description(description)
}
