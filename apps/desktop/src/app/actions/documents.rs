//! Document import, and the sharing preview prepared for an agent.

use std::path::PathBuf;

use coderune_domain::{AgentAvailability, AppError};

use crate::app::CoderuneApp;
use crate::forms::ExtractedDocument;

impl CoderuneApp {
    pub(crate) fn extract_document(&mut self) {
        let path = self.document_form.path.text.trim().to_string();
        if path.is_empty() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadableDocument,
                "Choose a document path first.",
            ));
            return;
        }
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.extract_document(PathBuf::from(&path), 20 * 1024 * 1024) {
            Ok((name, text, bytes)) => {
                self.document_form.extracted = Some(ExtractedDocument {
                    name,
                    text,
                    bytes,
                    source_path: Some(path),
                });
                self.error = None;
                self.notice = Some("Text extracted locally. Review it before saving.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_document_import(&mut self) {
        let Some(extracted) = self.document_form.extracted.take() else {
            return;
        };
        let result = self.workspace.as_mut().map(|workspace| {
            workspace.save_document(
                self.document_form.kind,
                self.document_form.project_id.clone(),
                &extracted.name,
                extracted.source_path.as_deref(),
                &extracted.text,
                true,
            )
        });
        match result {
            Some(Ok(document)) => {
                self.notice = Some(format!(
                    "Saved {} import {}.",
                    document.kind.label(),
                    document.name
                ));
                self.error = None;
                self.document_form.draft_preview = None;
                self.document_form.selected_document = Some(document.id);
            }
            Some(Err(error)) => {
                self.document_form.extracted = Some(extracted);
                self.error = Some(error);
            }
            None => self.document_form.extracted = Some(extracted),
        }
    }

    pub(crate) fn prepare_draft(&mut self) {
        let (Some(agent), Some(document_id)) = (
            self.document_form.agent,
            self.document_form.selected_document.clone(),
        ) else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Select an imported document and a coding agent first.",
            ));
            return;
        };
        if !self.agent_enabled(agent.into()) {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Turn on this coding agent in Settings before using it.",
            ));
            return;
        }
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        if !workspace
            .agents
            .iter()
            .any(|status| status.id == agent && status.availability == AgentAvailability::Ready)
        {
            self.document_form.draft_preview = None;
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "This coding agent is unavailable. Recheck installed agents in Settings.",
            ));
            return;
        }
        match workspace.prepare_draft(agent, &document_id) {
            Ok(preview) => {
                self.document_form.draft_preview = Some(preview);
                self.error = None;
                self.notice =
                    Some("Sharing preview prepared. No agent process has started.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }
}
