//! The subagents screen: the reusable specialists a developer defines.

use crate::app::CoderuneApp;
use coderune_ui::components as ui;
use rune::{col, El};
use rune_kit::Kit;

pub(crate) fn subagents(_s: &CoderuneApp, k: &Kit) -> El {
    ui::page("subagents-page")
        .child(ui::page_header(
            "Subagents",
            "Reusable Coderune specialists, each with its own purpose and default model assignment.",
            vec![],
        ))
        .child(ui::section(
            k,
            "subagents-list",
            "Defined subagents",
            [col().w_full().py(ui::ROW_PAD).child(ui::empty_state(
                None,
                "No subagents yet",
                "A defined subagent pairs a bot avatar with a purpose and a default model assignment. Define one to reuse it across projects.",
                None,
            ))],
        ))
}
