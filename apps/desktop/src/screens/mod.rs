//! One module per screen. Screens read app state and queue actions; they never own it.

pub(crate) mod avatars;
pub(crate) mod capture;
pub(crate) mod chat;
pub(crate) mod chat_panel;
pub(crate) mod coding_agents;
pub(crate) mod documents;
pub(crate) mod home;
pub(crate) mod onboarding;
pub(crate) mod profile;
pub(crate) mod projects;
pub(crate) mod settings;
pub(crate) mod subagents;

pub(crate) use avatars::{avatar_gallery, register_gallery};
pub(crate) use capture::capture;
pub(crate) use chat::chat;
pub(crate) use chat_panel::panel as chat_panel;
pub(crate) use documents::documents;
pub(crate) use home::home;
pub(crate) use profile::profile;
pub(crate) use projects::projects;
pub(crate) use subagents::subagents;
