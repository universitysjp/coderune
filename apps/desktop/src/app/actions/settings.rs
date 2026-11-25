//! Desktop preferences, fonts, agents, and local data actions.

use std::path::{Path, PathBuf};

use coderune_agents::CliAgentId;
use coderune_app::Workspace;
use coderune_domain::AppError;
use rune::{FileDialogOptions, TextEdit, Tx};

use crate::agent_detection::Check;
use crate::app::{CoderuneApp, SettingsSection, View};
use crate::forms::{DocumentForm, ProjectForm};
use crate::preferences::Preferences;

impl CoderuneApp {
    pub(crate) fn select_settings(&mut self, section: SettingsSection) {
        self.settings = Some(section);
        self.settings_feedback = None;
        self.font_menu = None;
        self.confirm_delete_all = false;
        if section == SettingsSection::Agents
            && matches!(self.agent_detection.check, Check::NotChecked)
        {
            self.agent_detection.refresh();
        }
    }

    pub(crate) fn close_settings(&mut self) {
        self.settings = None;
        self.settings_feedback = None;
        self.font_menu = None;
        self.confirm_delete_all = false;
    }

    pub(crate) fn dark_mode(&self) -> bool {
        self.preferences.dark_mode
    }

    pub(crate) fn toggle_dark_mode(&mut self) {
        let dark_mode = !self.dark_mode();
        self.update_preferences(|preferences| preferences.dark_mode = dark_mode);
    }

    pub(crate) fn ui_font(&self) -> &str {
        self.preferences
            .ui_font
            .as_deref()
            .unwrap_or(crate::DEFAULT_UI_FONT)
    }

    pub(crate) fn mono_font(&self) -> &str {
        self.preferences
            .mono_font
            .as_deref()
            .unwrap_or(crate::DEFAULT_MONO_FONT)
    }

    pub(crate) fn set_ui_font(&mut self, family: String) {
        self.update_preferences(|preferences| preferences.ui_font = Some(family));
    }

    pub(crate) fn set_mono_font(&mut self, family: String) {
        self.update_preferences(|preferences| preferences.mono_font = Some(family));
    }

    /// The CSS family lists the text system should use: the chosen family
    /// first, then the shipped one so missing glyphs still resolve.
    pub(in crate::app) fn font_families(&self) -> (String, String) {
        (
            format!("{}, {}", self.ui_font(), crate::DEFAULT_UI_FONT),
            format!("{}, {}", self.mono_font(), crate::DEFAULT_MONO_FONT),
        )
    }

    pub(crate) fn agent_enabled(&self, agent: CliAgentId) -> bool {
        self.preferences.agent_enabled(agent)
    }

    pub(crate) fn set_agent_enabled(&mut self, agent: CliAgentId, enabled: bool) {
        self.update_preferences(|preferences| preferences.set_agent_enabled(agent, enabled));
        if !enabled {
            self.chat_terminals.retain(|id, _| {
                self.chats
                    .chat(id)
                    .is_some_and(|chat| chat.agent != Some(agent))
            });
        }
        if self
            .document_form
            .agent
            .is_some_and(|selected| !self.agent_enabled(selected.into()))
        {
            self.document_form.agent = None;
            self.document_form.draft_preview = None;
        }
    }

    pub(in crate::app) fn update_preferences(&mut self, change: impl FnOnce(&mut Preferences)) {
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match Preferences::update(&Preferences::path(&workspace.data_path()), change) {
            Ok(next) => {
                self.preferences = next;
                self.settings_feedback = None;
            }
            Err(error) => {
                self.settings_feedback = Some(Err(AppError::new(
                    coderune_domain::ErrorCode::UnreadablePath,
                    format!("Could not save desktop settings: {error}"),
                )));
            }
        }
    }

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
                self.chat_terminals.clear();
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
                    self.chat_agent_error("Could not remove saved chats", error);
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
