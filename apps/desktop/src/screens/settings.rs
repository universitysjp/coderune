//! Settings: the rail of pages beside the open page, laid out like the reference
//! desktop's Settings screen — a sticky rail, a 15/640 page title, and raised cards
//! whose rows are split by hairlines.

use crate::app::{CoderuneApp, SettingsSection, VIEW_SHORTCUTS};
use crate::preferences::{NavigationMode, Notify};
use crate::updates::Status;
use crate::widgets::{destructive, primary, primary_tx, secondary};
use coderune_ui::theme::{self, SETTINGS_MODE_NAME, SETTINGS_RAIL};
use coderune_ui::{components as ui, components::PushButtonExt, icons};
use rune::{col, el, icon, row, text, El};
use rune_kit::{Button, ButtonVariant, Kit, Placement, Select, SelectAppearance, Switch};

pub(crate) fn page(
    s: &CoderuneApp,
    section: SettingsSection,
    k: &Kit,
    overlays: &mut Vec<El>,
) -> El {
    let mut menus = Vec::new();
    let content = match section {
        SettingsSection::Preferences => preferences(s, k),
        SettingsSection::Appearance => appearance(s, k, &mut menus),
        SettingsSection::Notifications => notifications(s, k),
        SettingsSection::Providers => super::coding_agents::coding_agents(s, k),
        SettingsSection::Projects => projects(s, k),
        SettingsSection::Shortcuts => shortcuts(k),
    };
    let mut body = match &s.settings_feedback {
        Some(Ok(message)) => col().w_full().child(ui::alert(None, message)),
        Some(Err(error)) => col()
            .w_full()
            .child(ui::alert(Some(error.code.as_str()), &error.message)),
        None => col().w_full(),
    };
    body = body.child(content);

    let column = ui::pane_page("settings-content")
        .child(ui::pane_heading(
            "settings-tab-header-band",
            section.label(),
            section.description(),
        ))
        .child(
            ui::pane_body(&format!("settings-{}-body", section.id()), section.label()).child(body),
        );
    overlays.extend(menus);
    ui::two_pane("settings-page", rail(section), column)
}

fn rail(current: SettingsSection) -> El {
    let mut items = ui::pane_nav("settings-nav-group");
    for section in SettingsSection::ALL {
        items = items.child(nav_item(section, current));
    }
    ui::pane_rail("settings-sidebar")
        .child(back_button())
        .child(items)
}

fn back_button() -> El {
    row()
        .key("settings-back")
        .role("button")
        .label("Back to app")
        .h(28.0)
        .px(6.0)
        .gap(8.0)
        .center_y()
        .radius(6.0)
        .self_start()
        .focusable(true)
        .focus_ring(2.0, -2.0, theme::ink())
        .hover(theme::rail_hover())
        .hover_motion(theme::HOVER_FADE, theme::HOVER_FADE)
        .on_press(CoderuneApp::close_settings)
        .child(
            icon(&icons::ARROW_LEFT, 16.0)
                .color(theme::muted())
                .a11y_hidden(),
        )
        .child(
            text("Back to app")
                .font(SETTINGS_RAIL)
                .color(theme::muted())
                .a11y_hidden(),
        )
}

/// The rail row's ink brightens under the pointer, as the reference rail does.
fn nav_item(section: SettingsSection, current: SettingsSection) -> El {
    ui::pane_nav_item(
        &format!("settings-nav-{}", section.id()),
        section.label(),
        section.icon(),
        section == current,
        move |s: &mut CoderuneApp| s.select_settings(section),
    )
}

fn preferences(s: &CoderuneApp, k: &Kit) -> El {
    col()
        .w_full()
        .gap(32.0)
        .child(ui::settings_section(
            k,
            "preferences-navigation",
            Some("Navigation"),
            ui::settings_card(k, "preferences-navigation-card", [layout_control(s)]),
        ))
        .child(ui::settings_section(
            k,
            "preferences-application",
            Some("Application"),
            ui::settings_card(k, "preferences-application-card", [updates_row(s, k)]),
        ))
        .child(ui::settings_section(
            k,
            "preferences-local-data",
            Some("Local data"),
            ui::settings_card(k, "preferences-local-data-card", local_data_rows(s, k)),
        ))
}

/// The Window layout row: its words over the two cards, so the cards keep their
/// 120 pt width instead of squeezing the description.
fn layout_control(s: &CoderuneApp) -> El {
    col()
        .w_full()
        .key("setting-window-layout")
        .gap(16.0)
        .py(20.0)
        .child(
            col()
                .w_full()
                .gap(8.0)
                .child(
                    text("Window layout")
                        .key("title")
                        .font(theme::SETTINGS_ROW_TITLE)
                        .color(theme::ink()),
                )
                .child(
                    text("Where destinations and chats live. Picking one switches the shell around this page.")
                        .key("about")
                        .font(theme::SETTINGS_ROW_DESCRIPTION)
                        .color(theme::muted())
                        .wrap(),
                ),
        )
        .child(layout_cards(s))
}

/// The two 120 pt layout cards, each a window showing where its navigation sits.
fn layout_cards(s: &CoderuneApp) -> El {
    row()
        .key("navigation-mode")
        .role("radiogroup")
        .label("Window layout")
        .focus_group()
        .gap(10.0)
        .shrink()
        .children(
            NavigationMode::ALL
                .into_iter()
                .map(|mode| layout_card(mode, s.navigation_mode() == mode)),
        )
}

fn layout_card(mode: NavigationMode, chosen: bool) -> El {
    let ring = if chosen {
        theme::control_hover()
    } else {
        theme::line()
    };
    let body = col()
        .w_full()
        .h_full()
        .px(10.0)
        .pt(10.0)
        .pb(8.0)
        .gap(6.0)
        .child(layout_sketch(mode, chosen).a11y_hidden())
        .child(
            text(mode.label())
                .font(SETTINGS_MODE_NAME)
                .color(if chosen { theme::ink() } else { theme::muted() })
                .truncate(),
        );
    let mut card = el()
        .key(format!("navigation-mode-{}", mode.id()))
        .role("radio")
        .label(mode.label())
        .description(mode.detail())
        .focusable(true)
        .focus_ring(2.0, 2.0, theme::ink())
        .w(120.0)
        .h(84.0)
        .shrink()
        .radius(8.0)
        .fill(theme::raised())
        .border(if chosen { 2.0 } else { 1.0 }, ring)
        .hover(theme::rail_hover())
        .on_press(move |s: &mut CoderuneApp| s.set_navigation_mode(mode))
        .child(body);
    if chosen {
        card = card.child(
            el().key("check")
                .pin(None, Some(-6.0), Some(-6.0), None)
                .size(16.0, 16.0)
                .radius(8.0)
                .center()
                .fill(theme::accent())
                .box_shadow(rune::Shadow {
                    spread: 2.0,
                    color: theme::panel(),
                    ..rune::Shadow::default()
                })
                .a11y_hidden()
                .child(icon(&icons::CHECK_SMALL, 16.0).color(theme::inverse())),
        );
    }
    card
}

/// A 44 pt window: its navigation down one side, or across the top, and the room
/// left for the page. The chosen card's navigation takes the accent.
fn layout_sketch(mode: NavigationMode, chosen: bool) -> El {
    let ground = theme::raised();
    let chrome = theme::code();
    let ink = if chosen {
        theme::accent().alpha(0.55)
    } else {
        theme::muted().alpha(0.4)
    };
    let faint = theme::muted().alpha(0.2);
    let frame = |inner: El| {
        el().w_full()
            .h(44.0)
            .radius(5.0)
            .clip_rounded(5.0)
            .fill(ground)
            .border(1.0, theme::line())
            .child(inner)
    };
    let line = |width: f32, fill: rune::Color| el().h(4.0).w(width).radius(2.0).fill(fill);
    match mode {
        NavigationMode::Sidebar => frame(
            row()
                .w_full()
                .h_full()
                .child(
                    col()
                        .w(30.0)
                        .h_full()
                        .gap(5.0)
                        .padding(6.0)
                        .fill(chrome)
                        .child(line(18.0, ink))
                        .child(line(18.0, faint))
                        .child(line(18.0, faint)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(5.0)
                        .padding(7.0)
                        .child(line(46.0, faint))
                        .child(line(32.0, faint)),
                ),
        ),
        NavigationMode::TopBar => frame(
            col()
                .w_full()
                .h_full()
                .child(
                    row()
                        .w_full()
                        .h(15.0)
                        .gap(4.0)
                        .px(6.0)
                        .center_y()
                        .fill(chrome)
                        .child(line(14.0, ink))
                        .child(line(14.0, faint))
                        .child(line(14.0, faint))
                        .child(line(14.0, faint)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(5.0)
                        .px(7.0)
                        .py(6.0)
                        .child(line(46.0, faint))
                        .child(line(32.0, faint)),
                ),
        ),
    }
}

fn updates_row(s: &CoderuneApp, k: &Kit) -> El {
    let busy = matches!(
        s.updates.status,
        Status::Checking | Status::Downloading | Status::Ready(_)
    );
    let check = Button::new("Check for updates")
        .disabled(busy)
        .on_press(|s: &mut CoderuneApp| s.updates.check())
        .build_push(k)
        .key("check-updates");
    let mut row = ui::settings_row(
        "Application updates",
        Some(format!("Installed version {}", crate::VERSION)),
        check,
    );
    let status = match &s.updates.status {
        Status::Idle => None,
        Status::Checking => Some(ui::subtle("Checking for updates…")),
        Status::Current => Some(ui::subtle("Coderune is up to date.")),
        Status::NoRelease => Some(ui::subtle("No desktop release has been published yet.")),
        Status::Available(update) => Some(primary(
            &format!("Download Coderune {}", update.version),
            |s| s.updates.download(),
            k,
        )),
        Status::Downloading => Some(ui::subtle("Downloading and verifying the update…")),
        Status::Ready(install) => {
            let version = crate::update_install::version(install).to_string();
            Some(
                ui::settings_row(
                    "Update ready",
                    Some("Restarting installs the update and reopens Coderune.".to_string()),
                    primary(
                        &format!("Restart to update to {version}"),
                        |s| s.updates.restart(),
                        k,
                    )
                    .key("restart-update"),
                )
                .key("update-ready"),
            )
        }
        Status::Manual(path) => {
            let folder = path
                .parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| path.clone());
            Some(ui::settings_row(
                "Update downloaded",
                Some(
                    "Open the downloaded package to install it, then reopen Coderune.".to_string(),
                ),
                secondary(
                    "Open download folder",
                    move |s| s.open_settings_folder(&folder),
                    k,
                ),
            ))
        }
        Status::Failed(message) => Some(ui::alert(None, message)),
    };
    if let Some(status) = status {
        row = row.child(col().w_full().pt(8.0).items_start().child(status));
    }
    row
}

fn local_data_rows(s: &CoderuneApp, k: &Kit) -> Vec<El> {
    let folder = s
        .workspace
        .as_ref()
        .map(|workspace| workspace.data_path())
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf));
    let open = Button::new("Open data folder")
        .disabled(folder.is_none())
        .on_press(move |s: &mut CoderuneApp| {
            if let Some(folder) = &folder {
                s.open_settings_folder(folder);
            }
        })
        .build_push(k)
        .key("open-data-folder");
    let export = primary_tx("Export JSON", |s, tx| s.export_data(tx), k).key("export-data");
    let delete = Button::new("Delete all data")
        .variant(ButtonVariant::Danger)
        .disabled(s.confirm_delete_all)
        .on_press(|s: &mut CoderuneApp| {
            s.confirm_delete_all = true;
            s.settings_feedback = None;
        })
        .build_push(k)
        .key("delete-data");
    let mut rows = vec![
        ui::settings_row(
            "Local database",
            Some(
                "Your profile, projects, chats, documents, and captures stay on this computer."
                    .to_string(),
            ),
            open,
        )
        .key("data-folder"),
        ui::settings_row(
            "Export data",
            Some("Save everything as one JSON file to move or archive it.".to_string()),
            export,
        )
        .key("data-export"),
        ui::settings_row(
            "Delete all data",
            Some(
                "Permanently removes your profile, projects, chats, documents, and captures."
                    .to_string(),
            ),
            delete,
        )
        .key("data-delete"),
    ];
    if s.confirm_delete_all {
        rows.push(
            ui::settings_row(
                "This action cannot be undone.",
                None,
                row()
                    .gap(8.0)
                    .child(
                        secondary("Cancel", |s| s.confirm_delete_all = false, k)
                            .key("cancel-delete-data"),
                    )
                    .child(
                        destructive("Confirm delete all", CoderuneApp::delete_all_data, k)
                            .key("confirm-delete-data"),
                    ),
            )
            .key("data-confirm"),
        );
    }
    rows
}

fn appearance(s: &CoderuneApp, k: &Kit, menus: &mut Vec<El>) -> El {
    let dark = ui::settings_row(
        "Dark mode",
        Some("Use the dark colour scheme everywhere in the window.".to_string()),
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
        FontRow {
            key: "ui-font",
            title: "Interface font",
            detail: "Text, labels, and headings.",
            current: s.ui_font(),
            choose: CoderuneApp::set_ui_font,
        },
        menus,
    );
    let mono_font = font_row(
        s,
        k,
        FontRow {
            key: "mono-font",
            title: "Code font",
            detail: "Code, paths, IDs, and hashes.",
            current: s.mono_font(),
            choose: CoderuneApp::set_mono_font,
        },
        menus,
    );
    col()
        .w_full()
        .gap(32.0)
        .child(ui::settings_section(
            k,
            "appearance-scheme",
            Some("Colour scheme"),
            ui::settings_card(k, "appearance-scheme-card", [dark]),
        ))
        .child(ui::settings_section(
            k,
            "appearance-fonts",
            Some("Fonts"),
            ui::settings_card(k, "appearance-fonts-card", [ui_font, mono_font]),
        ))
}

/// One font row's copy and the setter it saves through.
struct FontRow<'a> {
    key: &'static str,
    title: &'a str,
    detail: &'a str,
    current: &'a str,
    choose: fn(&mut CoderuneApp, String),
}

fn font_row(s: &CoderuneApp, k: &Kit, row: FontRow<'_>, menus: &mut Vec<El>) -> El {
    let FontRow {
        key,
        title,
        detail,
        current,
        choose,
    } = row;
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
    ui::settings_row(title, Some(detail.to_string()), select).key(format!("font-{key}"))
}

fn notifications(s: &CoderuneApp, k: &Kit) -> El {
    col().w_full().gap(32.0).child(ui::settings_section(
        k,
        "notifications-desktop",
        Some("Desktop notifications"),
        ui::settings_card(
            k,
            "notifications-desktop-card",
            [
                notification_row(
                    s,
                    "notifications-approval",
                    Notify::Approvals,
                    "Permission requests",
                    "An agent asks before it runs a command or edits a file.",
                ),
                notification_row(
                    s,
                    "notifications-finished",
                    Notify::Finished,
                    "Finished turns",
                    "A reply arrives while you are in another app.",
                ),
                notification_row(
                    s,
                    "notifications-errors",
                    Notify::Errors,
                    "Errors",
                    "A turn fails, or an agent stops early.",
                ),
            ],
        ),
    ))
}

/// A notification switch, stored in the desktop preferences.
fn notification_row(s: &CoderuneApp, key: &str, kind: Notify, title: &str, detail: &str) -> El {
    let kit = theme::kit();
    ui::settings_row(
        title,
        Some(detail.to_string()),
        Switch::new(s.notification_enabled(kind))
            .label(title)
            .hide_label()
            .on_toggle(move |s: &mut CoderuneApp| {
                let on = !s.notification_enabled(kind);
                s.set_notification(kind, on);
            })
            .build(theme::switch_kit(kit.dark))
            .key(key),
    )
    .key(key)
}

fn projects(s: &CoderuneApp, k: &Kit) -> El {
    let add = Button::new("Add project")
        .on_press_tx(|s: &mut CoderuneApp, tx: &mut rune::Tx| s.choose_project_folder(tx))
        .build_push(k)
        .key("add-project");
    let mut project_rows: Vec<El> = s
        .workspace
        .iter()
        .flat_map(|workspace| &workspace.projects)
        .map(|project| {
            let id = project.id.clone();
            let select = id.clone();
            let (done, total) = s.context_done(&id);
            ui::settings_row(
                &project.name,
                Some(format!(
                    "{} · {done} of {total} context items",
                    project.root_path
                )),
                secondary(
                    "Open",
                    move |s: &mut CoderuneApp| {
                        s.load_project_form(&select);
                        s.navigate(crate::app::View::Projects);
                    },
                    k,
                )
                .key(format!("open-project-{}", project.id)),
            )
            .key(format!("settings-project-{id}"))
        })
        .collect();
    if project_rows.is_empty() {
        project_rows.push(
            ui::settings_row(
                "No projects yet",
                Some("Add a folder to start collecting shared context for an agent.".to_string()),
                col(),
            )
            .key("settings-projects-empty"),
        );
    }
    col()
        .w_full()
        .gap(32.0)
        .child(ui::settings_section(
            k,
            "settings-projects",
            Some("Projects"),
            ui::settings_card(
                k,
                "settings-projects-card",
                project_rows.into_iter().chain(std::iter::once(
                    ui::settings_row(
                        "Add a project",
                        Some("Pick a folder to keep context and chats for.".to_string()),
                        add,
                    )
                    .key("settings-add-project"),
                )),
            ),
        ))
        .child(archived(s, k))
}

fn archived(s: &CoderuneApp, k: &Kit) -> El {
    let rows: Vec<El> = s
        .chats
        .archived()
        .map(|chat| {
            let id = chat.id.clone();
            let restore = id.clone();
            let delete = id.clone();
            let cancel = id.clone();
            let confirm = id.clone();
            let project = s
                .chat_project(chat)
                .map_or("No project", |project| project.name.as_str());
            let agent = chat.agent.map_or("No agent", |agent| agent.label());
            let detail = format!(
                "{} · {} · {}",
                agent,
                project,
                crate::chats::when(&chat.updated_at)
            );
            let mut actions = row().gap(8.0).items_center();
            if s.confirm_delete_chat.as_deref() == Some(id.as_str()) {
                actions = actions
                    .child(
                        secondary(
                            "Cancel",
                            move |s| {
                                if s.confirm_delete_chat.as_deref() == Some(cancel.as_str()) {
                                    s.confirm_delete_chat = None;
                                }
                            },
                            k,
                        )
                        .key(format!("cancel-delete-chat-{id}")),
                    )
                    .child(
                        destructive(
                            "Delete permanently",
                            move |s| {
                                s.confirm_delete_chat = None;
                                s.delete_chat(&delete);
                            },
                            k,
                        )
                        .key(format!("confirm-delete-chat-{id}")),
                    );
            } else {
                actions = actions
                    .child(
                        secondary("Restore", move |s| s.unarchive_chat(&restore), k)
                            .key(format!("restore-chat-{id}")),
                    )
                    .child(
                        Button::new("Delete")
                            .variant(ButtonVariant::Danger)
                            .on_press(move |s: &mut CoderuneApp| {
                                s.confirm_delete_chat = Some(confirm.clone());
                            })
                            .build_push(k)
                            .key(format!("delete-chat-{id}")),
                    );
            }
            ui::settings_row(&chat.title, Some(detail), actions).key(format!("archived-chat-{id}"))
        })
        .collect();
    if rows.is_empty() {
        return ui::settings_section(
            k,
            "archived-chats",
            Some("Archived chats"),
            ui::settings_card(
                k,
                "archived-chats-card",
                [ui::empty_state(
                    Some(&icons::ARCHIVE),
                    "No archived chats",
                    "Archive a chat from the sidebar to put it away without deleting its transcript.",
                    None,
                )],
            ),
        );
    }
    ui::settings_section(
        k,
        "archived-chats",
        Some("Archived chats"),
        ui::settings_card(k, "archived-chats-card", rows),
    )
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
        .gap(32.0)
        .child(ui::settings_section(
            k,
            "navigation-shortcuts",
            Some("Navigation"),
            ui::settings_card(k, "navigation-shortcuts-card", view_rows),
        ))
        .child(ui::settings_section(
            k,
            "window-shortcuts",
            Some("Window"),
            ui::settings_card(
                k,
                "window-shortcuts-card",
                [
                    shortcut_row(k, "Open Settings", &[&[modifier, ","], &[modifier, "7"]]),
                    shortcut_row(k, "Toggle sidebar", &sidebar_keys),
                    shortcut_row(k, "Open terminal", &[&[modifier, "J"]]),
                    shortcut_row(k, "Close Settings", &[&["Esc"]]),
                    shortcut_row(k, "Quit Coderune", &[&[modifier, "Q"]]),
                ],
            ),
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
    ui::settings_row(title, None, keys)
        .key(title)
        .role("group")
        .description(description)
}
