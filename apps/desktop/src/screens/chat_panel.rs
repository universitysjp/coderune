//! A chat's side panel: the project context and documents the conversation reads.

use coderune_domain::Project;
use coderune_ui::theme::{self, BODY_SM, LABEL, LABEL_SM};
use coderune_ui::{components as ui, icons};
use rune::{col, el, icon, row, text, El, ScrollSpec};
use rune_kit::{ControlSize, IconButton, IconButtonVariant, Kit};

use crate::app::{ChatPanel, CoderuneApp};
use crate::chats::Chat;

/// The panel beside a chat: its tabs, then the open tab's content.
pub(crate) fn panel(s: &CoderuneApp, chat: &Chat, k: &Kit, tab: ChatPanel) -> El {
    let close = IconButton::new(k.glyph.close)
        .variant(IconButtonVariant::Ghost)
        .control_size(ControlSize::Mini)
        .label("Close panel")
        .on_press(CoderuneApp::close_chat_panel)
        .build(k)
        .key("close-chat-panel");
    let mut tabs = row()
        .key("chat-panel-tabs")
        .role("tablist")
        .gap(2.0)
        .grow(1.0)
        .min_w(0.0)
        .items_center();
    for candidate in ChatPanel::ALL {
        let selected = candidate == tab;
        tabs = tabs.child(
            row()
                .key(format!("chat-panel-{}", candidate.id()))
                .role("tab")
                .label(candidate.label())
                .focusable(true)
                .focus_selected(selected)
                .h(26.0)
                .px(8.0)
                .center_y()
                .radius(5.0)
                .when_selected(selected)
                .on_press(move |s: &mut CoderuneApp| s.select_chat_panel(candidate))
                .child(text(candidate.label()).font(LABEL).color(if selected {
                    theme::ink()
                } else {
                    theme::muted()
                })),
        );
    }
    let body = match tab {
        ChatPanel::Context => context(s, chat),
        ChatPanel::Documents => documents(s, chat),
    };
    col()
        .key("chat-panel")
        .role("complementary")
        .label("Chat panel")
        .w_full()
        .h_full()
        .fill(theme::panel())
        .border_sides([0.0, 1.0, 0.0, 0.0], theme::row_line())
        .child(
            row()
                .w_full()
                .h(40.0)
                .px(8.0)
                .gap(6.0)
                .items_center()
                .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
                .child(tabs)
                .child(close),
        )
        .child(
            col()
                .key("chat-panel-body")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .scroll_edge_fade(12.0)
                .px(14.0)
                .py(14.0)
                .gap(20.0)
                .child(body),
        )
}

trait SelectedFill {
    fn when_selected(self, selected: bool) -> Self;
}

impl SelectedFill for El {
    fn when_selected(self, selected: bool) -> Self {
        if selected {
            self.fill(theme::rail_selected())
        } else {
            self
        }
    }
}

fn context(s: &CoderuneApp, chat: &Chat) -> El {
    let Some(project) = s.chat_project(chat) else {
        return ui::empty_state(
            Some(&icons::FOLDER),
            "No project",
            "This chat is not attached to a project, so it has no shared context.",
            None,
        );
    };
    let items = s.context_items(&project.id);
    let (done, total) = s.context_done(&project.id);
    let mut list = col().key("chat-panel-context-items").w_full().gap(10.0);
    for item in items {
        list = list.child(
            row()
                .w_full()
                .gap(8.0)
                .items_center()
                .child(
                    el().size(14.0, 14.0)
                        .radius(7.0)
                        .center()
                        .fill(if item.done {
                            theme::accent_soft()
                        } else {
                            theme::code()
                        })
                        .when_checked(item.done)
                        .a11y_hidden(),
                )
                .child(
                    text(item.label)
                        .font(BODY_SM)
                        .color(if item.done {
                            theme::ink()
                        } else {
                            theme::muted()
                        })
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                ),
        );
    }
    col()
        .w_full()
        .gap(16.0)
        .child(ui::section_title(
            &project.name,
            &format!("{done} of {total} context items ready"),
        ))
        .child(project_facts(project))
        .child(list)
}

trait CheckMark {
    fn when_checked(self, done: bool) -> Self;
}

impl CheckMark for El {
    fn when_checked(self, done: bool) -> Self {
        if done {
            self.child(icon(&icons::CHECK_SMALL, 10.0).color(theme::muted()))
        } else {
            self
        }
    }
}

fn project_facts(project: &Project) -> El {
    let goals: Vec<&str> = project
        .goals
        .iter()
        .map(|statement| statement.text.as_str())
        .collect();
    let mut facts = col()
        .w_full()
        .gap(10.0)
        .child(fact("Folder", &project.root_path));
    if !goals.is_empty() {
        facts = facts.child(
            col()
                .w_full()
                .gap(2.0)
                .child(text("Goals").font(LABEL_SM).color(theme::faint()))
                .children(goals.into_iter().map(|goal| {
                    text(format!("· {goal}"))
                        .font(BODY_SM)
                        .color(theme::muted())
                        .wrap()
                })),
        );
    }
    facts
}

fn fact(label: &str, value: &str) -> El {
    col()
        .w_full()
        .gap(2.0)
        .child(text(label).font(LABEL_SM).color(theme::faint()))
        .child(
            text(value)
                .font(BODY_SM)
                .color(theme::muted())
                .wrap_anywhere(),
        )
}

fn documents(s: &CoderuneApp, chat: &Chat) -> El {
    let project = s.chat_project(chat).map(|project| project.id.as_str());
    let docs: Vec<_> = s
        .workspace
        .iter()
        .flat_map(|workspace| &workspace.documents)
        .filter(|document| document.project_id.as_deref() == project)
        .collect();
    if docs.is_empty() {
        return ui::empty_state(
            Some(&icons::FILES),
            "No documents",
            "Import a CV or SRS for this project from Documents to ground the chat.",
            None,
        );
    }
    let mut list = col().key("chat-panel-documents").w_full().gap(4.0);
    for document in docs {
        list = list.child(
            col()
                .key(format!("chat-panel-document-{}", document.id))
                .w_full()
                .gap(2.0)
                .px(8.0)
                .py(6.0)
                .radius(5.0)
                .hover(theme::hover())
                .child(
                    text(&document.name)
                        .font(BODY_SM)
                        .color(theme::ink())
                        .truncate(),
                )
                .child(
                    text(format!(
                        "{} · {} kB",
                        document.kind.label(),
                        document.text_bytes.div_ceil(1000).max(1)
                    ))
                    .font(LABEL_SM)
                    .color(theme::faint()),
                ),
        );
    }
    list
}
