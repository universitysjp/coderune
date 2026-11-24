//! The subagents screen: the reusable specialists a developer defines.

use crate::app::CoderuneApp;
use coderune_ui::components as ui;
use coderune_ui::theme;
use rune::{col, text, El};

pub(crate) fn subagents(_s: &CoderuneApp) -> El {
    let k = theme::kit();
    ui::page("subagents-page")
        .child(ui::page_header(
            "Subagents",
            "Reusable Coderune specialists, each with its own purpose and default model assignment.",
            vec![],
        ))
        .child(ui::section(
            &k,
            "subagents-list",
            "Defined subagents",
            [col()
                .w_full()
                .gap(4.0)
                .child(
                    text("No subagents yet")
                        .font(theme::LABEL)
                        .color(theme::ink()),
                )
                .child(ui::subtle(
                    "A defined subagent pairs a bot avatar with a purpose and a default model assignment. Define one to reuse it across projects.",
                ))],
        ))
}
