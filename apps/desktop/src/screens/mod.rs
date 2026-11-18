//! One module per screen. Screens read app state and queue actions; they never own it.

pub(crate) mod agents;
pub(crate) mod avatars;
pub(crate) mod capture;
pub(crate) mod data;
pub(crate) mod documents;
pub(crate) mod home;
pub(crate) mod onboarding;
pub(crate) mod profile;
pub(crate) mod projects;

pub(crate) use agents::agents;
pub(crate) use avatars::{avatar_gallery, gallery_id};
pub(crate) use capture::capture;
pub(crate) use data::data;
pub(crate) use documents::documents;
pub(crate) use home::home;
pub(crate) use onboarding::onboarding;
pub(crate) use profile::profile;
pub(crate) use projects::projects;
