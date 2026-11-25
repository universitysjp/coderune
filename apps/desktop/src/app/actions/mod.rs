//! Application actions, grouped by the part of the workspace they change.
//!
//! Each module adds inherent methods to `CoderuneApp`; screens call those
//! methods and never touch fields directly.

mod capture;
mod chats;
mod documents;
mod profile;
mod projects;
mod settings;
