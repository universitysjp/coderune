//! Coderune's Rune design system.
//!
//! Tokens and reusable primitives live here. Screens and application state stay in
//! `apps/desktop`; this crate never imports a Coderune service.

pub mod components;
pub mod icons;
pub mod theme;

pub use components::{
    alert, badge, card, code_well, code_well_tall, empty_state, field, mono_line, nav_item,
    page_header, section_title, subtle, tiny, WhenExt,
};
