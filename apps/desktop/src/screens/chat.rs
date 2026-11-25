//! A chat tab: the conversation, its project context, and the agent panel.

use crate::agent_detection::Check;
use crate::app::CoderuneApp;
use crate::chat_terminal::Terminal;
use crate::chats::{Author, Chat};
use crate::preferences::AgentDock;
use crate::widgets::*;
use coderune_agents::CliAgentId;
use coderune_ui::components as ui;
use coderune_ui::components::PushButtonExt;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, El, ScrollSpec};
use rune_kit::{Button, ButtonVariant, ControlSize, IconButton, IconButtonVariant, Kit, Progress};

const PANEL_WIDTH: f32 = 320.0;
const FLOAT_WIDTH: f32 = 316.0;
const FLOAT_INSET: f32 = 12.0;

pub(crate) fn chat(s: &CoderuneApp, k: &Kit) -> El {
    let Some(chat) = s.active_chat() else {
        return ui::page("chat-missing").child(ui::subtle("This chat is no longer available."));
    };
    let project = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.projects.iter().find(|p| p.id == chat.project_id));
    let project_name = project.map_or("No project", |project| project.name.as_str());
    let dock = s.agent_dock();
    let header = row()
        .w_full()
        .h(theme::TOPBAR)
        .px(16.0)
        .items_center()
        .gap(10.0)
        .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
        .child(
            text(&chat.title)
                .font(LABEL)
                .color(theme::ink())
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
        )
        .child(
            text(project_name)
                .font(LABEL_SM)
                .color(theme::muted())
                .truncate(),
        )
        .child(
            Button::new("Project settings")
                .variant(ButtonVariant::Ghost)
                .on_press(CoderuneApp::show_chat_project)
                .build_push(k)
                .key("chat-project-settings"),
        )
        .when(dock == AgentDock::Hidden, |header| {
            header.child(
                IconButton::new(&icons::PANEL_RIGHT)
                    .variant(IconButtonVariant::Ghost)
                    .label("Show agent panel")
                    .on_press(|s: &mut CoderuneApp| s.set_agent_dock(AgentDock::Right))
                    .build(k)
                    .key("show-agent-panel"),
            )
        });
    let transcript = transcript(s, k, chat, dock == AgentDock::Floating);
    let body = match dock {
        AgentDock::Hidden => transcript,
        AgentDock::Left => row()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .child(agent_panel(s, k, chat, dock))
            .child(transcript),
        AgentDock::Right => row()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .child(transcript)
            .child(agent_panel(s, k, chat, dock)),
        AgentDock::Floating => transcript,
    };
    col()
        .key("chat")
        .w_full()
        .h_full()
        .child(header)
        .children(error_alert(s).map(|alert| col().w_full().px(16.0).py(8.0).child(alert)))
        .child(body)
}

fn transcript(s: &CoderuneApp, k: &Kit, chat: &Chat, floating: bool) -> El {
    let mut messages = col().w_full().gap(14.0).pb(8.0);
    if chat.messages.is_empty() {
        messages = messages.child(empty_chat(s, k, chat));
    }
    let running = s
        .chat_terminals
        .get(&chat.id)
        .is_some_and(Terminal::is_running);
    for message in &chat.messages {
        let mine = message.author == Author::Developer;
        messages = messages.child(
            col()
                .w_full()
                .gap(3.0)
                .items_start()
                .child(
                    text(match message.author {
                        Author::Developer => "You",
                        Author::Coderune => "Coderune",
                    })
                    .font(LABEL_SM)
                    .color(if mine { theme::ink() } else { theme::faint() }),
                )
                .child(
                    text(&message.text)
                        .font(BODY)
                        .color(if mine { theme::ink() } else { theme::muted() })
                        .wrap(),
                ),
        );
    }
    let composer = row()
        .w_full()
        .padding(12.0)
        .gap(8.0)
        .items_center()
        .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
        .child(
            col().grow(1.0).min_w(0.0).child(
                text_input(
                    &s.chat_draft,
                    if running {
                        "Write a message…"
                    } else {
                        "Start an agent to send a message"
                    },
                    "Message",
                    |s| &mut s.chat_draft,
                    k,
                )
                .on_submit(|s: &mut CoderuneApp, _: &mut rune::Tx| s.send_chat_message()),
            ),
        )
        .child(
            Button::new("Send")
                .variant(ButtonVariant::Contrast)
                .disabled(!running || s.chat_draft.text.trim().is_empty())
                .on_press(|s: &mut CoderuneApp| s.send_chat_message())
                .build_push(k)
                .key("send-message"),
        );
    col()
        .key("chat-transcript")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(
            col()
                .key("chat-message-area")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .child(
                    col()
                        .w_full()
                        .grow(1.0)
                        .min_h(0.0)
                        .scroll(ScrollSpec { stick_bottom: true })
                        .pl(16.0)
                        .pr(if floating {
                            FLOAT_WIDTH + 2.0 * FLOAT_INSET
                        } else {
                            16.0
                        })
                        .py(14.0)
                        .items_center()
                        .child(col().w_full().max_w(720.0).child(messages)),
                )
                .when(floating, |area| area.child(floating_panel(s, k, chat))),
        )
        .child(composer)
}

fn empty_chat(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let done = s.context_done(&chat.project_id);
    col()
        .w_full()
        .py(24.0)
        .gap(10.0)
        .items_start()
        .child(
            text("This chat has no messages")
                .font(theme::HEADING_XS)
                .color(theme::ink()),
        )
        .child(ui::subtle(
            "Start an agent in the panel to write your first message.",
        ))
        .child(
            row()
                .items_center()
                .gap(10.0)
                .child(
                    Progress::new((done.1 > 0).then(|| f32::from(done.0) / f32::from(done.1)))
                        .build(k)
                        .w(140.0),
                )
                .child(ui::tiny(&format!("{} of {} context items", done.0, done.1))),
        )
}

fn agent_panel(s: &CoderuneApp, k: &Kit, chat: &Chat, dock: AgentDock) -> El {
    let terminal = s.chat_terminals.get(&chat.id);
    let running = terminal.is_some_and(Terminal::is_running);
    let available = chat.agent.is_some_and(|id| {
        s.agent_enabled(id)
            && s.agent_detection
                .results
                .iter()
                .any(|status| status.id == id && status.executable.is_some())
    });
    let mut panel = col()
        .key("agent-panel")
        .w(PANEL_WIDTH)
        .min_h(0.0)
        .fill(theme::surface())
        .when(dock == AgentDock::Right, |panel| {
            panel
                .h_full()
                .border_sides([0.0, 0.0, 0.0, 1.0], theme::row_line())
        })
        .when(dock == AgentDock::Left, |panel| {
            panel
                .h_full()
                .border_sides([0.0, 1.0, 0.0, 0.0], theme::row_line())
        })
        .child(panel_header(s, k, chat));
    if let Some(terminal) = terminal {
        panel = panel.child(terminal.view::<CoderuneApp>(CoderuneApp::active_terminal_mut));
    } else {
        panel = panel.child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .padding(12.0)
                .gap(14.0)
                .child(agent_list(s, k, chat))
                .child(context_section(s, k, chat)),
        );
    }
    panel.child(
        row()
            .w_full()
            .padding(8.0)
            .gap(6.0)
            .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
            .child(if running {
                Button::new("Stop agent")
                    .variant(ButtonVariant::Ghost)
                    .on_press(CoderuneApp::stop_chat_agent)
                    .build_push(k)
                    .key("stop-chat-agent")
            } else {
                Button::new("Start agent")
                    .variant(ButtonVariant::Contrast)
                    .disabled(!available)
                    .on_press(CoderuneApp::start_chat_agent)
                    .build_push(k)
                    .key("start-chat-agent")
            })
            .when(terminal.is_some() && !running, |footer| {
                footer.child(
                    Button::new("Choose agent")
                        .variant(ButtonVariant::Ghost)
                        .on_press(CoderuneApp::clear_chat_terminal)
                        .build_push(k),
                )
            }),
    )
}

fn floating_panel(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let mut panel = agent_panel(s, k, chat, AgentDock::Floating)
        .pin(
            None,
            Some(FLOAT_INSET),
            Some(FLOAT_INSET),
            Some(FLOAT_INSET),
        )
        .w(FLOAT_WIDTH)
        .border(1.0, theme::row_line())
        .radius(k.radius.xl)
        .continuous_corners()
        .clip_rounded(k.radius.xl);
    for shadow in &k.elevation.overlay {
        panel = panel.box_shadow(*shadow);
    }
    panel
}

fn panel_header(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let dock = s.agent_dock();
    let dock_button =
        |key: &'static str, icon: &'static rune::Icon, target: AgentDock, hint: &str| {
            IconButton::new(icon)
                .variant(if dock == target {
                    IconButtonVariant::Neutral
                } else {
                    IconButtonVariant::Ghost
                })
                .control_size(ControlSize::Small)
                .label(hint)
                .on_press(move |s: &mut CoderuneApp| s.set_agent_dock(target))
                .build(k)
                .key(key)
        };
    row()
        .w_full()
        .h(theme::TOPBAR)
        .px(12.0)
        .gap(4.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
        .child(
            text(
                s.agent_detection
                    .results
                    .iter()
                    .find(|agent| Some(agent.id) == chat.agent)
                    .map_or("Agents", |agent| agent.name),
            )
            .font(LABEL)
            .color(theme::ink())
            .grow(1.0)
            .min_w(0.0)
            .truncate(),
        )
        .child(dock_button(
            "dock-left",
            &icons::PANEL_LEFT,
            AgentDock::Left,
            "Dock agent panel left",
        ))
        .child(dock_button(
            "dock-right",
            &icons::PANEL_RIGHT,
            AgentDock::Right,
            "Dock agent panel right",
        ))
        .child(dock_button(
            "dock-float",
            &icons::PANEL_FLOAT,
            AgentDock::Floating,
            "Float agent panel",
        ))
        .child(
            IconButton::new(&icons::PANEL_HIDDEN)
                .variant(IconButtonVariant::Ghost)
                .control_size(ControlSize::Small)
                .label("Hide agent panel")
                .on_press(|s: &mut CoderuneApp| s.set_agent_dock(AgentDock::Hidden))
                .build(k)
                .key("dock-hidden"),
        )
}

fn agent_list(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let mut ready: Vec<(CliAgentId, &'static str)> = Vec::new();
    for status in &s.agent_detection.results {
        if status.executable.is_some() && s.agent_enabled(status.id) {
            ready.push((status.id, status.name));
        }
    }
    let mut rows = col().w_full().gap(2.0);
    if ready.is_empty() {
        rows = rows
            .child(ui::subtle(
                if matches!(s.agent_detection.check, Check::Checking(_)) {
                    "Checking for installed agents…"
                } else {
                    "No coding agent is ready."
                },
            ))
            .child(row().child(ghost(
                "Open Agents settings",
                |s| s.select_settings(crate::app::SettingsSection::Agents),
                k,
            )));
    }
    for (id, name) in ready {
        let selected = chat.agent == Some(id);
        rows = rows.child(
            row()
                .key(format!("chat-agent-{id:?}"))
                .hit()
                .focusable(true)
                .role("button")
                .label(name)
                .w_full()
                .h(32.0)
                .px(8.0)
                .gap(8.0)
                .items_center()
                .radius(6.0)
                .when(selected, |this| this.fill(theme::accent_soft()))
                .when(!selected, |this| this.hover(theme::hover()))
                .child(super::coding_agents::logo(id, k.dark))
                .child(text(name).font(LABEL).color(theme::ink()).grow(1.0))
                .child(ui::tiny(if selected { "Selected" } else { "Ready" }))
                .on_press(move |s: &mut CoderuneApp| s.set_chat_agent(Some(id))),
        );
    }
    col()
        .key("chat-agent-list")
        .w_full()
        .gap(6.0)
        .child(text("Coding agent").font(LABEL_SM).color(theme::faint()))
        .child(rows)
}

fn context_section(s: &CoderuneApp, k: &Kit, chat: &Chat) -> El {
    let items = s.context_items(&chat.project_id);
    let done = items.iter().filter(|item| item.done).count();
    let total = items.len();
    let mut rows = col().w_full().gap(6.0).child(
        row()
            .items_center()
            .gap(8.0)
            .child(
                Progress::new((total > 0).then(|| done as f32 / total as f32))
                    .build(k)
                    .w(120.0),
            )
            .child(ui::tiny(&format!("{done} of {total}"))),
    );
    for item in items {
        rows = rows.child(
            row()
                .w_full()
                .gap(8.0)
                .items_center()
                .child(
                    row()
                        .size(6.0, 6.0)
                        .radius(3.0)
                        .fill(if item.done {
                            theme::muted()
                        } else {
                            theme::line()
                        })
                        .a11y_hidden(),
                )
                .child(
                    text(item.label)
                        .font(theme::BODY_SM)
                        .color(if item.done {
                            theme::muted()
                        } else {
                            theme::ink()
                        })
                        .grow(1.0),
                ),
        );
    }
    rows.child(row().child(ghost(
        "Review in project settings",
        CoderuneApp::show_chat_project,
        k,
    )))
}
