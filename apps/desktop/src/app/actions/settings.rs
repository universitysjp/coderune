//! Desktop preferences, fonts, agents, and local data actions.

use coderune_agents::CliAgentId;
use coderune_domain::AppError;

use crate::agent_detection::Check;
use crate::app::{CoderuneApp, SettingsSection};
use crate::preferences::Preferences;

impl CoderuneApp {
    pub(crate) fn select_settings(&mut self, section: SettingsSection) {
        self.settings = Some(section);
        self.close_chat_menu();
        self.settings_feedback = None;
        self.font_menu = None;
        self.confirm_delete_all = false;
        self.confirm_delete_chat = None;
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
        self.confirm_delete_chat = None;
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
        if !self.agent_enabled(agent) {
            self.cancel_agent_turns(agent);
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
}
