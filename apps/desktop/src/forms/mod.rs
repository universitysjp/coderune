//! Editable forms and the text codecs that convert them to domain drafts.
//!
//! Ops run on drafts: forms never write storage directly. Keeping the
//! conversion here keeps `app` focused on state transitions and screens on
//! rendering.

mod document;
mod profile;
mod project;
mod text;

pub(crate) use document::{DocumentForm, ExtractedDocument};
pub(crate) use profile::ProfileForm;
pub(crate) use project::{project_name, ProjectForm};
pub(crate) use text::{edit, split_list};
