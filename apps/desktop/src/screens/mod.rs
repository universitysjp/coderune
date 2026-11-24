//! One module per screen. Screens read app state and queue actions; they never own it.

pub(crate) mod avatars;
pub(crate) mod capture;
mod coding_agents;
pub(crate) mod documents;
pub(crate) mod home;
pub(crate) mod onboarding;
pub(crate) mod profile;
pub(crate) mod projects;
pub(crate) mod settings;
pub(crate) mod subagents;

pub(crate) use avatars::{avatar_gallery, register_gallery};
pub(crate) use capture::capture;
pub(crate) use documents::documents;
pub(crate) use home::home;
pub(crate) use onboarding::onboarding;
pub(crate) use profile::profile;
pub(crate) use projects::projects;
pub(crate) use subagents::subagents;
