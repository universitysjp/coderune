//! Capture previews, history, and readiness for the selected project.

use crate::app::*;

impl CoderuneApp {
    pub(in crate::app) fn reset_capture_state(&mut self) {
        self.capture_session = None;
        self.capture_reviewed = false;
        self.capture_detail = None;
        self.confirm_delete_capture = None;
    }

    pub(in crate::app) fn refresh_captures(&mut self) {
        self.captures.clear();
        if let Some(workspace) = &self.workspace {
            if let Some(id) = &workspace.selected_project_id {
                match workspace.list_captures(id) {
                    Ok(captures) => {
                        if captures.is_empty() {
                            self.projects_with_captures.remove(id);
                        } else {
                            self.projects_with_captures.insert(id.clone());
                        }
                        self.captures = captures;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
        }
    }

    /// What a project still needs before a chat has full context.
    pub(crate) fn context_items(&self, project_id: &str) -> Vec<ContextItem> {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.projects.iter().find(|p| p.id == project_id))
        else {
            return Vec::new();
        };
        vec![
            ContextItem {
                label: "Project folder",
                done: std::path::Path::new(&project.root_path).is_dir(),
            },
            ContextItem {
                label: "Goals",
                done: !project.goals.is_empty(),
            },
            ContextItem {
                label: "Capture scope",
                done: !project.scope.include_paths.is_empty()
                    || !project.scope.exclude_paths.is_empty(),
            },
            ContextItem {
                label: "First capture",
                done: self.projects_with_captures.contains(project_id),
            },
        ]
    }

    pub(crate) fn context_done(&self, project_id: &str) -> (u8, u8) {
        let items = self.context_items(project_id);
        let done = items.iter().filter(|item| item.done).count() as u8;
        (done, items.len() as u8)
    }

    pub(crate) fn preview_capture(&mut self) {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project())
            .cloned()
        else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::NotFound,
                "Create or select a project before capturing context.",
            ));
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.preview_capture(&project.id, project.scope.clone(), project.limits.clone())
        {
            Ok(session) => {
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some(
                    "Preview collected. Review the exact file list before saving.".to_string(),
                );
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn refresh_capture(&mut self) {
        let Some(project_id) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project_id.clone())
        else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::NotFound,
                "Create or select a project before refreshing context.",
            ));
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.refresh_capture(&project_id) {
            Ok((latest, session)) => {
                let unchanged = latest.is_some();
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some(if unchanged {
                    "Repository unchanged. The latest capture already matches this state."
                        .to_string()
                } else {
                    "Changes found. A fresh preview is ready for review.".to_string()
                });
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_capture(&mut self) {
        let Some(session) = self.capture_session.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.save_capture(&session, self.capture_reviewed) {
            Ok(capture) => {
                self.notice = Some(format!("Context capture v{} saved.", capture.version));
                self.error = None;
                self.capture_reviewed = false;
                self.capture_session = None;
                self.refresh_captures();
            }
            Err(error) => {
                if error.code == coderune_domain::ErrorCode::CaptureStale {
                    self.capture_reviewed = false;
                }
                self.error = Some(error);
            }
        }
    }

    pub(crate) fn inspect_capture(&mut self, id: &str) {
        let Some(capture) = self
            .captures
            .iter()
            .find(|capture| capture.id == id)
            .cloned()
        else {
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.capture_contents(&capture.id) {
            Ok(contents) => {
                let selected_path = capture.files.first().map(|file| file.path.clone());
                self.capture_detail = Some(CaptureDetail {
                    capture,
                    contents,
                    selected_path,
                });
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn delete_capture(&mut self, id: &str) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_capture(id) {
            Ok(()) => {
                self.notice = Some("Context capture deleted.".to_string());
                self.error = None;
                if self
                    .capture_detail
                    .as_ref()
                    .is_some_and(|detail| detail.capture.id == id)
                {
                    self.capture_detail = None;
                }
                self.confirm_delete_capture = None;
                self.refresh_captures();
            }
            Err(error) => self.error = Some(error),
        }
    }
}
