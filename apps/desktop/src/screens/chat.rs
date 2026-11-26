//! A project chat's conversation and composer.

use coderune_domain::{AppError, ErrorCode};
use coderune_ui::components as ui;
use coderune_ui::theme::{self, BODY, BODY_SM, LABEL, LABEL_SM};
use rune::{col, row, text, El, ScrollSpec, Tx};
use rune_kit::{
    Combobox, ComboboxOption, Kit, Markdown, Placement, Progress, Select, Spinner, TextArea,
};

use crate::agent_detection::Check;
use crate::app::actions::chats::{CatalogStatus, ChatMenu, ChatTurn};
use crate::app::{CoderuneApp, SettingsSection};
use crate::chats::{Author, Chat};
use crate::widgets::{error_alert, ghost, primary_enabled};

use super::coding_agents::compact_logo;

const BUBBLE_MAX: f32 = 720.0;
const BUBBLE_PAD: f32 = 12.0;

pub(crate) fn chat(s: &CoderuneApp, k: &Kit, overlays: &mut Vec<El>) -> El {
    let Some(chat) = s.active_chat() else {
        return ui::page("chat-missing").child(ui::subtle("This chat is no longer available."));
    };
    let project = s.chat_project(chat);
    let turn = s.chat_turns.get(&chat.id);
    let header = row()
        .w_full()
        .h(theme::TOPBAR)
        .px(16.0)
        .gap(12.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
        .child(
            text(&chat.title)
                .font(LABEL)
                .color(theme::ink())
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
        )
        .children(project.map(|project| {
            text(&project.name)
                .font(LABEL_SM)
                .color(theme::muted())
                .truncate()
        }))
        .child(
            ghost("Project settings", CoderuneApp::show_chat_project, k)
                .key("chat-project-settings"),
        );

    col()
        .key("chat")
        .w_full()
        .h_full()
        .child(header)
        .children(error_alert(s).map(|alert| col().w_full().px(16.0).py(8.0).child(alert)))
        .child(transcript(s, k, chat, turn))
        .child(composer(s, k, chat, turn, overlays))
}

fn transcript(s: &CoderuneApp, k: &Kit, chat: &Chat, turn: Option<&ChatTurn>) -> El {
    let mut messages = col().w_full().gap(20.0);
    if chat.messages.is_empty() {
        messages = messages.child(empty_chat(s, k, chat));
    }
    for (index, message) in chat.messages.iter().enumerate() {
        messages = messages.child(message_row(
            message.author,
            &message.text,
            k,
            &format!("chat-message-{index}"),
            false,
        ));
    }
    if let Some(turn) = turn {
        if !turn.reply.is_empty() {
            if let Some(agent) = chat.agent {
                messages = messages.child(message_row(
                    Author::Agent(agent),
                    &turn.reply,
                    k,
                    "chat-message-live",
                    true,
                ));
            }
        }
        messages = messages.child(working_line(turn, k));
    }
    if let Some(feedback) = s.chat_feedback.get(&chat.id) {
        messages = messages.child(ui::alert(None, feedback));
    }
    col()
        .key("chat-transcript")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec { stick_bottom: true })
        .scroll_edge_fade(12.0)
        .px(20.0)
        .py(20.0)
        .items_center()
        .child(col().w_full().max_w(theme::CONTAINER).child(messages))
}

fn message_row(author: Author, value: &str, k: &Kit, key: &str, streaming: bool) -> El {
    let (name, agent, mine) = match author {
        Author::Developer => ("You", None, true),
        Author::Coderune => ("Coderune", None, false),
        Author::Agent(agent) => (agent.label(), Some(agent), false),
    };
    let body = if value.trim().is_empty() {
        text("…").font(BODY).color(theme::muted())
    } else {
        Markdown::new(value)
            .selectable(true)
            .streaming(streaming)
            .scroll_wide(true)
            .on_link_tx(|s: &mut CoderuneApp, url, tx| {
                if let Err(error) = tx.open_url(url) {
                    s.error = Some(AppError::new(
                        ErrorCode::Validation,
                        format!("Could not open the link: {error}"),
                    ));
                }
            })
            .build(k)
            .key(format!("{key}-markdown"))
            .max_w(BUBBLE_MAX - BUBBLE_PAD * 2.0)
    };
    let bubble = col()
        .key(key)
        .max_w(BUBBLE_MAX)
        .min_w(0.0)
        .padding(BUBBLE_PAD)
        .radius(k.radius.lg)
        .fill(if mine {
            theme::accent_soft()
        } else {
            theme::panel()
        })
        .border(
            1.0,
            if mine {
                theme::line()
            } else {
                theme::row_line()
            },
        )
        .child(body);
    let meta = row()
        .items_center()
        .gap(7.0)
        .children(agent.map(|agent| compact_logo(agent, k.dark)))
        .child(text(name).font(LABEL_SM).color(theme::muted()));
    let mut message = col().w_full().gap(6.0);
    message = if mine {
        message.items_end()
    } else {
        message.items_start()
    };
    message.child(meta).child(bubble)
}

fn working_line(turn: &ChatTurn, k: &Kit) -> El {
    row()
        .key("chat-working")
        .items_center()
        .gap(7.0)
        .h(24.0)
        .child(Spinner::new().size(13.0).color(theme::muted()).build(k))
        .child(
            text(if turn.stopping {
                "Stopping…"
            } else {
                "Working…"
            })
            .font(LABEL_SM)
            .color(theme::muted()),
        )
        .child(text(turn.elapsed()).font(LABEL_SM).color(theme::faint()))
}

fn empty_chat(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let (done, total) = s.context_done(&chat.project_id);
    let mut body = col().w_full().py(20.0).gap(10.0).items_start().child(
        text("Start a conversation")
            .font(theme::HEADING_XS)
            .color(theme::ink()),
    );
    if total > 0 {
        body = body.child(
            row()
                .items_center()
                .gap(10.0)
                .child(
                    Progress::new(Some(f32::from(done) / f32::from(total)))
                        .build(k)
                        .w(96.0),
                )
                .child(ui::tiny(&format!("{done} of {total} context items ready"))),
        );
    }
    body
}

fn composer(
    s: &CoderuneApp,
    k: &Kit,
    chat: &Chat,
    turn: Option<&ChatTurn>,
    overlays: &mut Vec<El>,
) -> El {
    let available = s.chat_can_send(chat);
    let prompt = if let Some(agent) = chat.agent {
        format!("Message {}…", agent.label())
    } else {
        "Choose a coding agent to send a message".to_string()
    };
    let input = TextArea::new(&s.chat_draft)
        .placeholder(prompt)
        .label("Message")
        .width(None)
        .auto_grow(2, 6)
        .wrap_anywhere()
        .on_edit(|s: &mut CoderuneApp| &mut s.chat_draft)
        .on_submit(|s: &mut CoderuneApp, _: &mut Tx| s.send_chat_message())
        .build(k)
        .key("chat-message");
    let action = if let Some(turn) = turn {
        primary_enabled(
            if turn.stopping { "Stopping…" } else { "Stop" },
            !turn.stopping,
            CoderuneApp::stop_chat_agent,
            k,
        )
        .key("stop-chat-agent")
    } else {
        primary_enabled(
            "Send",
            available && !s.chat_draft.text.trim().is_empty(),
            CoderuneApp::send_chat_message,
            k,
        )
        .key("send-message")
    };
    let agents: Vec<_> = s.chat_agents().map(|status| status.id).collect();
    let agent = row()
        .gap(2.0)
        .items_center()
        .min_w(0.0)
        .children(chat.agent.map(|agent| compact_logo(agent, k.dark)))
        .child(
            select(
                s,
                ChatMenu::Agent,
                &agents.iter().map(|agent| agent.label()).collect::<Vec<_>>(),
                chat.agent.map(|agent| agent.label()),
            )
            .disabled(turn.is_some() || agents.is_empty())
            .on_choose(move |s: &mut CoderuneApp, name, _| {
                if let Some(&agent) = agents.iter().find(|agent| agent.label() == name) {
                    s.set_chat_agent(agent);
                }
            })
            .build(k, overlays),
        );
    let models = model_controls(s, k, chat, available, overlays);
    let mut controls = row()
        .w_full()
        .min_w(0.0)
        .items_center()
        .gap(6.0)
        .child(agent)
        .children(models)
        .child(row().grow(1.0))
        .child(action);
    if !available && turn.is_none() && !matches!(s.agent_detection.check, Check::Checking(_)) {
        let missing = s.chat_agents().next().is_none();
        if missing {
            controls = controls.child(ghost(
                "Agents settings",
                |s| s.select_settings(SettingsSection::Agents),
                k,
            ));
        }
    }
    let mut body = col().w_full().max_w(theme::CONTAINER).gap(6.0);
    if let Some(summary) = turn.and_then(|turn| turn.approval.as_deref()) {
        body = body.child(
            col()
                .w_full()
                .gap(8.0)
                .padding(10.0)
                .radius(k.radius.md)
                .fill(theme::panel())
                .border(1.0, theme::line())
                .child(text("Agent approval").font(LABEL).color(theme::ink()))
                .child(
                    col()
                        .w_full()
                        .max_h(144.0)
                        .min_h(0.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(
                            text(summary)
                                .font(BODY_SM)
                                .color(theme::muted())
                                .wrap_anywhere(),
                        ),
                )
                .child(
                    row()
                        .items_center()
                        .gap(6.0)
                        .child(ghost("Decline", |s| s.answer_chat_approval(false), k))
                        .child(primary_enabled(
                            "Approve once",
                            true,
                            |s| s.answer_chat_approval(true),
                            k,
                        )),
                ),
        );
    }
    col()
        .w_full()
        .px(16.0)
        .py(12.0)
        .items_center()
        .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
        .child(body.child(input).child(controls))
}

fn model_controls(
    s: &CoderuneApp,
    k: &Kit,
    chat: &Chat,
    available: bool,
    overlays: &mut Vec<El>,
) -> Vec<El> {
    const DEFAULT_MODEL: &str = "Default model";
    const DEFAULT_EFFORT: &str = "Default effort";
    match &s.chat_catalog.status {
        CatalogStatus::Unavailable => Vec::new(),
        CatalogStatus::Loading(_) => vec![ui::tiny("Loading models…")],
        CatalogStatus::Failed(error) => {
            vec![ghost("Retry models", |s| s.refresh_chat_catalog(true), k)
                .description(error.clone())]
        }
        CatalogStatus::Ready(models) => {
            let selected = chat.options.model.as_deref();
            let active = models
                .iter()
                .find(|model| selected.map_or(model.is_default, |id| model.id == id));
            let choices: Vec<_> = std::iter::once(ComboboxOption::new("", DEFAULT_MODEL))
                .chain(
                    models
                        .iter()
                        .filter(|model| model.id != "default")
                        .map(|model| {
                            let mut option =
                                ComboboxOption::new(format!("model:{}", model.id), &model.label);
                            if option.label != model.id {
                                option.detail = model.id.clone();
                            }
                            option
                        }),
                )
                .collect();
            let label = selected.map(|id| active.map_or(id, |model| model.label.as_str()));
            let current = selected.map(|id| format!("model:{id}")).unwrap_or_default();
            let disabled = !available || (choices.len() == 1 && selected.is_none());
            let trigger = Select::new::<&str>("chat-model", &[])
                .label("Model")
                .current(label.or(Some(DEFAULT_MODEL)))
                .open(s.chat_model_menu.is_open())
                .disabled(disabled)
                .trigger(k)
                .max_w(200.0)
                .min_w(0.0);
            let model = Combobox::new(
                "chat-model-picker",
                &s.chat_model_menu,
                &choices,
                CoderuneApp::chat_model_menu_mut,
            )
            .label("Model")
            .placement(Placement::TopStart)
            .search_placeholder("Search models")
            .current(Some(&current))
            .disabled(disabled)
            .trigger(trigger)
            .on_choose(|s, id, _| s.set_chat_model(id.strip_prefix("model:").map(str::to_owned)))
            .build(k, overlays)
            .max_w(200.0)
            .min_w(0.0);
            let mut controls = vec![model];
            if let Some(active) = active.filter(|model| !model.efforts.is_empty()) {
                let efforts = active.efforts.clone();
                let labels: Vec<_> = std::iter::once(DEFAULT_EFFORT)
                    .chain(efforts.iter().map(|effort| effort.label.as_str()))
                    .collect();
                let current = chat.options.effort.as_deref().and_then(|id| {
                    efforts
                        .iter()
                        .find(|effort| effort.id == id)
                        .map(|effort| effort.label.as_str())
                });
                controls.push(
                    select(
                        s,
                        ChatMenu::Effort,
                        &labels,
                        Some(current.unwrap_or(DEFAULT_EFFORT)),
                    )
                    .disabled(!available)
                    .on_choose(move |s: &mut CoderuneApp, label, _| {
                        s.set_chat_effort(
                            efforts
                                .iter()
                                .find(|effort| effort.label == label)
                                .map(|effort| effort.id.clone()),
                        );
                    })
                    .build(k, overlays)
                    .max_w(155.0)
                    .min_w(0.0),
                );
            }
            controls
        }
    }
}

fn select(s: &CoderuneApp, menu: ChatMenu, options: &[&str], current: Option<&str>) -> Select {
    let (key, label, placeholder) = match menu {
        ChatMenu::Agent => ("chat-agent", "Coding agent", "Choose agent"),
        ChatMenu::Effort => ("chat-effort", "Reasoning effort", "Effort"),
    };
    Select::new(key, options)
        .current(current)
        .label(label)
        .placeholder(placeholder)
        .placement(Placement::TopStart)
        .open(s.chat_menu == Some(menu))
        .on_toggle(move |s: &mut CoderuneApp| s.toggle_chat_menu(menu))
        .on_close(CoderuneApp::close_chat_menu)
}
