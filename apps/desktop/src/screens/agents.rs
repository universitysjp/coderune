//! The agents screen.

#![allow(unused_imports)]

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_domain::AgentAvailability;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, BODY_SM, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, Cx, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit, TextInput};

pub(crate) fn agents(s: &CoderuneApp, k: &Kit) -> El {
    let agents = s
        .workspace
        .as_ref()
        .map(|workspace| workspace.agents.clone())
        .unwrap_or_default();
    let mut cards = Vec::new();
    for status in &agents {
        let ready = status.availability == AgentAvailability::Ready;
        cards.push(ui::card(
            Some(status.id.label()),
            Some("Optional. Coderune never owns this tool's sign-in, provider, or model choice."),
            vec![
                row()
                    .gap(8.0)
                    .items_center()
                    .child(ui::badge(if ready { "Ready" } else { "Not installed" }, k))
                    .child(ui::badge(
                        if status.verified {
                            "Verified"
                        } else {
                            "Unverified"
                        },
                        k,
                    ))
                    .children(
                        status
                            .capabilities
                            .iter()
                            .map(|capability| Badge::new(capability).build(k)),
                    ),
                ui::subtle(&status.detail),
                ui::tiny(&format!("Restrictions: {}", status.restrictions.join("; "))),
                text(&status.setup_url)
                    .font(CODE_SM)
                    .color(theme::muted())
                    .wrap(),
            ],
        ));
    }
    if let Some(error) = &s.error {
        cards.push(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        cards.push(ui::alert(None, notice));
    }
    col()
        .w_full()
        .gap(16.0)
        .child(ui::page_header(
            "Agents",
            "Coding agents detected on this machine. Compatibility is reported honestly and is not claimed until demonstrated.",
            vec![secondary("Refresh detection", |s| s.refresh_agents(), k)],
        ))
        .child(ui::alert(None, "Each agent keeps its own sign-in, provider, and model configuration. Coderune does not read or copy credential files. Manual entry always remains available."))
        .children(cards)
}
