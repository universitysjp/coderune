//! Small layout helpers shared by every screen.
//!
//! Components that need editing, focus, accessibility, or animation come from
//! `rune-kit`. These helpers add Coderune's frame, type, and spacing roles on
//! top, split by responsibility: `layout`, `text`, and `controls`.

mod controls;
mod layout;
mod text;

pub use controls::*;
pub use layout::*;
pub use text::*;

/// Rune has no conditional builder; this keeps the call sites declarative.
pub use controls::WhenExt;
