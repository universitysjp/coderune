//! Home: the project list on the left, the open project on the right.
//!
//! Home is the app's two-pane landing page. The rail lists projects (the same
//! destinations the sidebar shows) and links to Settings; the page shows what
//! the selected project needs next, its context checklist, and its chats.

use crate::app::{CoderuneApp, SettingsSection, View};
use crate::widgets::{primary, primary_tx, secondary, status_alerts};
use coderune_domain::Project;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, LABEL_SM, SETTINGS_RAIL};
use rune::{col, icon, row, text, El};
use rune_kit::{ControlSize, IconButton, IconButtonVariant, Kit};

pub(crate) fn home(s: &CoderuneApp, k: &Kit) -> El {
    let projects: Vec<Project> = s
        .workspace
        .iter()
        .flat_map(|workspace| workspace.projects.iter().cloned())
        .collect();
    let selected = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.selected_project())
        .cloned();
    ui::two_pane(
        "home-page",
        home_rail(s, k, &projects, selected.as_ref()),
        home_page(s, k, selected.as_ref()),
    )
}

fn home_rail(s: &CoderuneApp, k: &Kit, projects: &[Project], selected: Option<&Project>) -> El {
    let mut list = ui::pane_nav("home-project-list");
    for project in projects {
        let id = project.id.clone();
        let selected = selected.is_some_and(|open| open.id == project.id);
        list = list.child(ui::pane_nav_item(
            &format!("home-project-{}", project.id),
            &project.name,
            &icons::FOLDER,
            selected,
            move |s: &mut CoderuneApp| s.select_home_project(&id),
        ));
    }
    let mut rail = ui::pane_rail("home-rail").child(
        row()
            .w_full()
            .h(32.0)
            .px(6.0)
            .items_center()
            .child(
                text("Projects")
                    .font(LABEL_SM)
                    .color(theme::faint())
                    .grow(1.0),
            )
            .child(
                IconButton::new(&icons::PLUS)
                    .variant(IconButtonVariant::Ghost)
                    .control_size(ControlSize::Small)
                    .label("Add a project folder")
                    .on_press_tx(|s: &mut CoderuneApp, tx: &mut rune::Tx| {
                        s.choose_project_folder(tx)
                    })
                    .build(k)
                    .key("home-new-project"),
            ),
    );
    if projects.is_empty() {
        rail = rail.child(
            row().w_full().px(6.0).py(12.0).child(
                text("No projects yet")
                    .font(SETTINGS_RAIL)
                    .color(theme::muted())
                    .wrap(),
            ),
        );
    }
    rail = rail.child(list);
    rail.child(rune::rule(0.5, theme::line()).w_full().my(4.0))
        .child(ui::pane_nav_item(
            "home-settings",
            "Settings",
            &icons::GEAR,
            s.settings.is_some(),
            |s: &mut CoderuneApp| s.select_settings(SettingsSection::Preferences),
        ))
}

fn home_page(s: &CoderuneApp, k: &Kit, project: Option<&Project>) -> El {
    let Some(project) = project else {
        return ui::pane_page("home-content")
            .child(ui::pane_heading(
                "home-header",
                "Home",
                "Pick a project on the left, or add one to start collecting context.",
            ))
            .child(
                ui::pane_body("home-body", "Home").child(ui::settings_card(
                    k,
                    "home-empty",
                    [ui::empty_state(
                        Some(&icons::FOLDER),
                        "No project open",
                        "Add a project folder to start a chat and collect shared context.",
                        Some(
                            primary_tx(
                                "Choose project folder",
                                |s, tx| s.choose_project_folder(tx),
                                k,
                            )
                            .key("home-choose-folder"),
                        ),
                    )],
                )),
            );
    };

    let id = project.id.clone();
    let (done, total) = s.context_done(&id);
    let actions = vec![
        primary(
            "New chat",
            {
                let id = id.clone();
                move |s: &mut CoderuneApp| s.new_chat(&id)
            },
            k,
        )
        .key("home-new-chat"),
        secondary(
            "Project settings",
            {
                let id = id.clone();
                move |s: &mut CoderuneApp| {
                    s.load_project_form(&id);
                    s.navigate(View::Projects);
                }
            },
            k,
        )
        .key("home-project-settings"),
    ];

    let mut body = ui::pane_body("home-body", "Home").children(status_alerts(s));

    body = body.child(ui::settings_section(
        k,
        "home-overview",
        Some("Overview"),
        ui::settings_card(
            k,
            "home-overview-card",
            overview_rows(project, done, total, k),
        ),
    ));
    body = body.child(ui::settings_section(
        k,
        "home-context",
        Some("Context"),
        ui::settings_card(k, "home-context-card", context_rows(s, &id, done, total)),
    ));
    body = body.child(ui::settings_section(
        k,
        "home-chats",
        Some("Chats"),
        ui::settings_card(k, "home-chats-card", chat_rows(s, &id, k)),
    ));

    let profile_action = if s.profile_form.details_count() == 0 {
        "Complete profile"
    } else {
        "Edit profile"
    };
    body = body.child(ui::settings_section(
        k,
        "home-profile",
        Some("Profile"),
        ui::settings_card(
            k,
            "home-profile-card",
            [ui::settings_row(
                "Developer profile",
                Some("Add details about how you work and what you want to learn.".to_string()),
                secondary(profile_action, CoderuneApp::open_profile_details, k)
                    .key("home-profile-action"),
            )],
        ),
    ));

    ui::pane_page("home-content")
        .child(ui::pane_heading_row(
            "home-header",
            &project.name,
            &format!("{} · v{}", project.root_path, project.version),
            actions,
        ))
        .child(body)
}

fn overview_rows(project: &Project, done: u8, total: u8, k: &Kit) -> Vec<El> {
    let goals = if project.goals.is_empty() {
        "No goals yet".to_string()
    } else {
        project
            .goals
            .iter()
            .map(|goal| goal.text.as_str())
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let captured = done >= total && total > 0;
    let folder = std::path::PathBuf::from(&project.root_path);
    vec![
        ui::settings_row(
            "Folder",
            Some(project.root_path.clone()),
            secondary(
                "Open",
                move |s: &mut CoderuneApp| s.open_folder(folder.clone()),
                k,
            )
            .key("home-open-folder"),
        )
        .key("home-overview-folder"),
        ui::settings_row("Goals", Some(goals), col().shrink()).key("home-overview-goals"),
        ui::settings_row(
            "Context",
            Some(format!("{done} of {total} context items ready")),
            col().shrink(),
        )
        .key("home-overview-context"),
        ui::settings_row(
            if captured {
                "Context captured"
            } else {
                "Context not captured"
            },
            Some(if captured {
                "The project's files are saved for chats and validation.".to_string()
            } else {
                "Run a capture to save the files the agent may need.".to_string()
            }),
            secondary(
                if captured {
                    "View captures"
                } else {
                    "Capture now"
                },
                |s: &mut CoderuneApp| s.navigate(View::Capture),
                theme::kit(),
            )
            .key("home-capture"),
        )
        .key("home-overview-capture"),
    ]
}

fn context_rows(s: &CoderuneApp, project_id: &str, done: u8, total: u8) -> Vec<El> {
    let mut rows: Vec<El> = s
        .context_items(project_id)
        .into_iter()
        .map(|item| {
            ui::settings_row(
                item.label,
                None,
                icon(
                    if item.done {
                        &icons::CHECK_CIRCLE
                    } else {
                        &icons::CARET_RIGHT
                    },
                    16.0,
                )
                .color(if item.done {
                    theme::muted()
                } else {
                    theme::faint()
                })
                .a11y_hidden(),
            )
            .key(format!("home-context-{}", item.label))
        })
        .collect();
    rows.push(
        ui::settings_row(
            "Readiness",
            Some(format!("{done} of {total} context items ready")),
            primary_tx(
                "Capture context",
                |s, tx| {
                    let _ = tx;
                    s.navigate(View::Capture)
                },
                theme::kit(),
            )
            .key("home-context-capture"),
        )
        .key("home-context-readiness"),
    );
    rows
}

fn chat_rows(s: &CoderuneApp, project_id: &str, k: &Kit) -> Vec<El> {
    let chats: Vec<_> = s.chats.chats_for(project_id).take(6).collect();
    if chats.is_empty() {
        return vec![ui::empty_state(
            Some(&icons::MESSAGE),
            "No chats yet",
            "Start a chat to ask the agent about this project.",
            Some(
                primary(
                    "New chat",
                    {
                        let id = project_id.to_string();
                        move |s: &mut CoderuneApp| s.new_chat(&id)
                    },
                    k,
                )
                .key("home-chats-new"),
            ),
        )];
    }
    chats
        .into_iter()
        .map(|chat| {
            let id = chat.id.clone();
            let agent = chat.agent.map_or("No agent", |agent| agent.label());
            ui::settings_row(
                &chat.title,
                Some(format!(
                    "{} · {}",
                    agent,
                    crate::chats::when(&chat.updated_at)
                )),
                secondary("Open", move |s: &mut CoderuneApp| s.open_chat(&id), k)
                    .key(format!("home-open-chat-{}", chat.id)),
            )
            .key(format!("home-chat-{}", chat.id))
        })
        .collect()
}
