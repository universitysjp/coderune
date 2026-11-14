//! Native GPUI design system for Coderune.
//!
//! The visual language comes from the existing Coderune UI tokens and native
//! adapters. Layout and composition are native and intentionally new.

pub mod components;
pub mod theme;

pub use components::*;
pub use theme::*;

use gpui::{div, Div, ParentElement, Styled};

/// A plain text block with a role and tone.
pub fn text(content: impl Into<String>, role: TextRole, tone: Tone) -> Div {
    div()
        .type_role(role)
        .tone(tone)
        .child(content.into())
}

/// A vertical flex container with no visual styling.
pub fn stack() -> Div {
    div().flex().flex_col().min_w(gpui::px(0.0))
}

/// A horizontal flex container with no visual styling.
pub fn row() -> Div {
    div().flex().flex_row().items_center().min_w(gpui::px(0.0))
}

/// A flexible spacer.
pub fn spacer() -> Div {
    div().flex_1()
}
