//! Local data: export, delete-all, and opening folders or an editor.

use std::path::{Path, PathBuf};

use coderune_app::Workspace;
use coderune_domain::AppError;
use rune::{FileDialogOptions, TextEdit, Tx};

use crate::app::{CoderuneApp, View};
use crate::forms::{DocumentForm, ProjectForm};

impl CoderuneApp {
    /// Asks where to write the export, then writes it there. No path field: the
    /// system save dialog names the file.
    pub(crate) fn export_data(&mut self, tx: &mut Tx) {
        let directory = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.data_path())
            .and_then(|path| path.parent().map(Path::to_path_buf));
        tx.save_file(
            FileDialogOptions {
                title: "Export local context".to_string(),
                directory,
                file_name: Some("coderune-export.json".to_string()),
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, _| match result {
                Ok(Some(path)) => s.export_data_to(&path),
                Ok(None) => {}
                Err(error) => {
                    s.settings_feedback = Some(Err(AppError::new(
                        coderune_domain::ErrorCode::UnreadablePath,
                        format!("Could not open the save dialog: {error}"),
                    )));
                }
            },
        );
    }

    fn export_data_to(&mut self, path: &Path) {
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        self.settings_feedback = Some(workspace.export_json().and_then(|json| {
            std::fs::write(path, json)
                .map(|()| "Local context exported.".to_string())
                .map_err(|error| {
                    AppError::new(
                        coderune_domain::ErrorCode::Validation,
                        format!("Could not write {}: {error}", path.display()),
                    )
                })
        }));
    }

    pub(crate) fn delete_all_data(&mut self) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_all_data() {
            Ok(()) => {
                self.chat_turns.clear();
                self.chat_feedback.clear();
                self.chat_catalog = Default::default();
                self.close_chat_menu();
                self.chat_drafts.clear();
                self.chat_draft = TextEdit::default();
                let clear_chats = self.chats.clear();
                self.projects_with_captures.clear();
                self.notice = clear_chats.is_ok().then(|| {
                    "All workspace data deleted. Onboarding is required again.".to_string()
                });
                self.error = None;
                self.close_settings();
                self.view = View::Home;
                self.onboarding_visible = true;
                self.reset_capture_state();
                self.confirm_delete_project = None;
                self.confirm_delete_capture = None;
                self.project_form = ProjectForm::empty(None);
                self.project_dialog = false;
                self.document_form = DocumentForm::empty();
                self.refresh_profile_form();
                self.refresh_captures();
                if let Err(error) = clear_chats {
                    self.error = Some(AppError::new(
                        coderune_domain::ErrorCode::UnreadablePath,
                        format!("Could not remove saved chats: {error}"),
                    ));
                }
            }
            Err(error) => self.settings_feedback = Some(Err(error)),
        }
    }

    pub(crate) fn open_folder(&mut self, path: PathBuf) {
        if let Err(error) = Workspace::open_directory(&path) {
            self.error = Some(error);
        }
    }

    pub(crate) fn open_settings_folder(&mut self, path: &std::path::Path) {
        self.settings_feedback = Workspace::open_directory(path).err().map(Err);
    }

    pub(crate) fn open_editor(&mut self, editor: &str, path: PathBuf) {
        if let Err(error) = Workspace::launch_editor(editor, &path) {
            self.error = Some(error);
        }
    }
}
