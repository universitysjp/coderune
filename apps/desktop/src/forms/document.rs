//! The CV and SRS import form.

use coderune_domain::{AgentId, DocumentKind};
use rune::TextEdit;

use super::edit;

#[derive(Clone)]
pub(crate) struct ExtractedDocument {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) bytes: usize,
    pub(crate) source_path: Option<String>,
}

pub(crate) struct DocumentForm {
    pub(crate) path: TextEdit,
    pub(crate) kind: DocumentKind,
    pub(crate) project_id: Option<String>,
    pub(crate) extracted: Option<ExtractedDocument>,
    pub(crate) selected_document: Option<String>,
    pub(crate) agent: Option<AgentId>,
    pub(crate) draft_preview: Option<coderune_agents::DraftSharingPreview>,
}

impl DocumentForm {
    pub(crate) fn empty() -> Self {
        Self {
            path: edit(""),
            kind: DocumentKind::Cv,
            project_id: None,
            extracted: None,
            selected_document: None,
            agent: None,
            draft_preview: None,
        }
    }
}
