//! Chat tabs, their drafts, and the coding agent running in them.

use crate::app::*;

impl CoderuneApp {
    pub(crate) fn active_chat(&self) -> Option<&Chat> {
        self.chats.active_chat()
    }

    pub(crate) fn show_chat_project(&mut self) {
        if let Some(project_id) = self.active_chat().map(|chat| chat.project_id.clone()) {
            self.load_project_form(&project_id);
            self.navigate(View::Projects);
        }
    }

    /// Creates a chat in the selected project and opens its tab.
    pub(crate) fn new_chat(&mut self, project_id: &str) {
        self.error = None;
        self.notice = None;
        if !self.has_project(project_id) {
            return;
        }
        self.stash_chat_draft();
        self.chats.new_chat(project_id);
        self.update_preferences(|preferences| preferences.set_project_expanded(project_id, true));
        self.save_chats();
        self.refresh_chat_agents();
    }

    pub(crate) fn open_chat(&mut self, id: &str) {
        let project_id = self.chats.chat(id).map(|chat| chat.project_id.clone());
        if self.chats.active.as_deref() == Some(id) {
            return;
        }
        let Some(project_id) = project_id else { return };
        self.error = None;
        self.notice = None;
        if !self.has_project(&project_id) {
            return;
        }
        self.stash_chat_draft();
        self.chats.open_chat(id);
        self.restore_chat_draft();
        self.save_chats();
        self.refresh_chat_agents();
    }

    pub(crate) fn show_workspace(&mut self) {
        if self.chats.active.is_some() {
            self.stash_chat_draft();
            self.chats.active = None;
            self.save_chats();
        }
    }

    pub(crate) fn close_chat(&mut self, id: &str) {
        let active = self.chats.active.as_deref() == Some(id);
        if active {
            self.stash_chat_draft();
        }
        self.chats.close_chat(id);
        self.chat_terminals.remove(id);
        if active {
            self.restore_chat_draft();
        }
        self.save_chats();
    }

    fn has_project(&self, project_id: &str) -> bool {
        self.workspace.as_ref().is_some_and(|workspace| {
            workspace
                .projects
                .iter()
                .any(|project| project.id == project_id)
        })
    }

    pub(in crate::app) fn refresh_chat_agents(&mut self) {
        if matches!(self.agent_detection.check, Check::NotChecked) {
            self.agent_detection.refresh();
        }
    }

    fn stash_chat_draft(&mut self) {
        if let Some(id) = &self.chats.active {
            self.chat_drafts
                .insert(id.clone(), std::mem::take(&mut self.chat_draft));
        }
    }

    fn restore_chat_draft(&mut self) {
        self.chat_draft = self
            .chats
            .active
            .as_ref()
            .and_then(|id| self.chat_drafts.remove(id))
            .unwrap_or_default();
    }

    /// Sends the message to the running agent in the active tab.
    pub(crate) fn send_chat_message(&mut self) {
        let text = self.chat_draft.text.trim();
        let Some(id) = self.chats.active.clone() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let Some(terminal) = self
            .chat_terminals
            .get_mut(&id)
            .filter(|term| term.is_running())
        else {
            return;
        };
        if let Err(error) = terminal.send_line(text) {
            self.chat_agent_error("Could not send message", error);
            return;
        }
        self.error = None;
        self.chats.add_message(&id, Author::Developer, text);
        self.chat_draft = TextEdit::default();
        self.save_chats();
    }

    pub(crate) fn set_chat_agent(&mut self, agent: Option<coderune_agents::CliAgentId>) {
        let Some(id) = self.chats.active.clone() else {
            return;
        };
        if self.active_chat().is_some_and(|chat| chat.agent == agent) {
            return;
        }
        self.chat_terminals.remove(&id);
        self.chats.set_agent(&id, agent);
        self.error = None;
        self.save_chats();
    }

    pub(crate) fn start_chat_agent(&mut self) {
        let Some(chat) = self.active_chat() else {
            return;
        };
        let Some(agent) = chat.agent.filter(|agent| self.agent_enabled(*agent)) else {
            return;
        };
        let Some(executable) = self
            .agent_detection
            .results
            .iter()
            .find(|status| status.id == agent)
            .and_then(|status| status.executable.as_deref())
        else {
            return;
        };
        let Some(project) = self.workspace.as_ref().and_then(|workspace| {
            workspace
                .projects
                .iter()
                .find(|project| project.id == chat.project_id)
        }) else {
            return;
        };
        if self
            .chat_terminals
            .get(&chat.id)
            .is_some_and(Terminal::is_running)
        {
            return;
        }
        match Terminal::launch(
            executable,
            std::path::Path::new(&project.root_path),
            self.agent_detection.path.as_deref(),
        ) {
            Ok(terminal) => {
                self.chat_terminals.insert(chat.id.clone(), terminal);
                self.error = None;
            }
            Err(error) => self.chat_agent_error("Could not start agent", error),
        }
    }

    pub(crate) fn active_terminal_mut(&mut self) -> Option<&mut Terminal> {
        self.chat_terminals.get_mut(self.chats.active.as_deref()?)
    }

    pub(crate) fn stop_chat_agent(&mut self) {
        if let Some(Err(error)) = self.active_terminal_mut().map(Terminal::stop) {
            self.chat_agent_error("Could not stop agent", error);
        }
    }

    pub(crate) fn clear_chat_terminal(&mut self) {
        if let Some(id) = &self.chats.active {
            self.chat_terminals.remove(id);
        }
    }

    pub(in crate::app) fn chat_agent_error(&mut self, message: &str, error: std::io::Error) {
        self.error = Some(AppError::new(
            coderune_domain::ErrorCode::AgentFailed,
            format!("{message}: {error}"),
        ));
    }

    pub(crate) fn agent_dock(&self) -> AgentDock {
        self.preferences.agent_dock
    }

    pub(crate) fn set_agent_dock(&mut self, dock: AgentDock) {
        self.update_preferences(|preferences| preferences.agent_dock = dock);
    }

    pub(in crate::app) fn save_chats(&mut self) {
        if let Err(error) = self.chats.save() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadablePath,
                format!("Could not save chats: {error}"),
            ));
        }
    }
}
