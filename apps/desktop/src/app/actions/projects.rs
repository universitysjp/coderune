//! Project selection, editing, folder picking, and deletion.

use std::path::Path;

use coderune_domain::AppError;
use rune::{FileDialogOptions, Tx};

use crate::app::{CoderuneApp, View};
use crate::forms::{edit, project_name, split_list, ProjectForm};

impl CoderuneApp {
    pub(crate) fn project_expanded(&self, id: &str) -> bool {
        self.preferences.project_expanded(id)
    }

    pub(crate) fn toggle_project_chats(&mut self, id: &str) {
        let expanded = !self.project_expanded(id);
        let id = id.to_string();
        self.update_preferences(move |preferences| preferences.set_project_expanded(&id, expanded));
    }

    /// Opens a project on Home without leaving the page.
    pub(crate) fn select_home_project(&mut self, id: &str) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        if workspace.projects.iter().any(|project| project.id == id) {
            workspace.select_project(Some(id.to_string()));
            self.refresh_captures();
            self.error = None;
        }
    }

    pub(crate) fn load_project_form(&mut self, id: &str) {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.projects.iter().find(|project| project.id == id))
            .cloned()
        else {
            return;
        };
        self.project_form = ProjectForm::from_project(&project);
        self.project_scope_expanded = false;
        self.confirm_delete_project = None;
        self.error = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(Some(project.id));
        }
        self.refresh_captures();
    }

    pub(crate) fn new_project_form(&mut self) {
        let initial_root = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        self.project_form = ProjectForm::empty(initial_root);
        self.project_dialog = false;
        self.project_scope_expanded = false;
        self.confirm_delete_project = None;
        self.error = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(None);
        }
        self.refresh_captures();
    }

    /// Opens the system folder picker. A picked folder fills a new project form
    /// and opens its quick questions; cancelling changes nothing.
    pub(crate) fn choose_project_folder(&mut self, tx: &mut Tx) {
        let directory = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        tx.open_folder(
            FileDialogOptions {
                title: "Choose a project folder".to_string(),
                directory,
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, tx| match result {
                Ok(Some(path)) => {
                    s.start_new_project(&path);
                    tx.instant();
                }
                Ok(None) => {}
                Err(error) => s.error = Some(folder_picker_error(error)),
            },
        );
    }

    /// The panel's folder row. It replaces the form's root, keeping the name the
    /// developer typed unless it is still untouched.
    pub(crate) fn change_project_folder(&mut self, tx: &mut Tx) {
        let directory = self.project_form.folder();
        tx.open_folder(
            FileDialogOptions {
                title: "Choose a project folder".to_string(),
                directory,
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, tx| match result {
                Ok(Some(path)) => {
                    if s.project_form.name.text.trim().is_empty() {
                        s.project_form.name = edit(project_name(&path));
                    }
                    s.project_form.root = edit(path.display().to_string());
                    s.error = None;
                    tx.instant();
                }
                Ok(None) => {}
                Err(error) => s.error = Some(folder_picker_error(error)),
            },
        );
    }

    pub(in crate::app) fn start_new_project(&mut self, root: &Path) {
        self.project_form = ProjectForm::from_folder(root);
        self.project_dialog = true;
        self.project_scope_expanded = false;
        self.confirm_delete_project = None;
        self.error = None;
        self.notice = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(None);
        }
        self.view = View::Projects;
        self.show_workspace();
        self.refresh_captures();
    }

    /// Closes the quick questions without creating anything: the picked folder
    /// stays in the panel, where the rest can be completed.
    pub(crate) fn close_project_dialog(&mut self) {
        self.project_dialog = false;
        self.error = None;
    }

    /// Creates the project from the dialog. Only the name and one goal are
    /// required; scope and limits take their defaults.
    pub(crate) fn create_project_from_dialog(&mut self) {
        if self.project_form.name.text.trim().is_empty() {
            self.error = Some(AppError::validation(vec!["name".to_string()]));
            return;
        }
        if split_list(&self.project_form.goals.text).is_empty() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Add at least one goal, or finish the project in the panel.",
            ));
            return;
        }
        self.save_project();
        if self.project_form.editing.is_some() {
            self.project_dialog = false;
        }
    }

    pub(crate) fn save_project(&mut self) {
        let previous = self.project_form.editing.as_deref().and_then(|id| {
            self.workspace
                .as_ref()
                .and_then(|workspace| workspace.projects.iter().find(|project| project.id == id))
                .cloned()
        });
        let draft = match self.project_form.to_draft(previous.as_ref()) {
            Ok(draft) => draft,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        let result = match self.project_form.editing.as_deref() {
            Some(id) => workspace.update_project(id, draft),
            None => workspace.create_project(draft),
        };
        match result {
            Ok(project) => {
                self.notice = Some(format!(
                    "Project {} saved at version {}.",
                    project.name, project.version
                ));
                self.error = None;
                self.project_form = ProjectForm::from_project(&project);
                self.confirm_delete_project = None;
                self.reset_capture_state();
                self.refresh_captures();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn delete_project(&mut self) {
        let Some(id) = self.confirm_delete_project.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_project(&id) {
            Ok(()) => {
                self.remove_project_chats(&id);
                self.projects_with_captures.remove(&id);
                self.new_project_form();
                self.save_chats();
                self.notice = Some("Project, captures, and chats deleted.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }
}
fn folder_picker_error(error: rune::FileDialogError) -> AppError {
    AppError::new(
        coderune_domain::ErrorCode::UnreadablePath,
        format!("Could not open the folder picker: {error}"),
    )
}
