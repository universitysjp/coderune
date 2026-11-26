//! Project chat tabs, provider configuration, and streamed turns.

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use coderune_agents::chat::{
    ChatEvent, ChatOptions, ChatProvider, ChatRun, Model, ModelCatalogRun,
};
use coderune_agents::{CliAgentId, CliAgentStatus};
use coderune_domain::AppError;
use rune::{TextEdit, Tx};

use crate::agent_detection::Check;
use crate::app::{CoderuneApp, View};
use crate::chats::{Author, Chat};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatMenu {
    Agent,
    Effort,
}

pub(crate) struct ChatTurn {
    run: ChatRun,
    agent: CliAgentId,
    pub(crate) reply: String,
    pub(crate) approval: Option<String>,
    pub(crate) stopping: bool,
}

#[derive(Default)]
pub(crate) struct ChatCatalog {
    key: Option<CatalogKey>,
    pub(crate) status: CatalogStatus,
}

#[derive(Default)]
pub(crate) enum CatalogStatus {
    #[default]
    Unavailable,
    Loading(ModelCatalogRun),
    Ready(Vec<Model>),
    Failed(String),
}

#[derive(PartialEq, Eq)]
struct CatalogKey {
    provider: ChatProvider,
    project: PathBuf,
    executable: PathBuf,
    path: Option<OsString>,
    model: Option<String>,
}

impl CoderuneApp {
    pub(in crate::app) fn seed_chat_catalog(&mut self, models: Vec<Model>) {
        self.chat_catalog = ChatCatalog {
            key: self.active_chat().and_then(|chat| self.chat_target(chat)),
            status: CatalogStatus::Ready(models),
        };
    }

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
        if active {
            self.restore_chat_draft();
        }
        self.save_chats();
    }

    /// Puts a chat away, cancels its turn, and closes its tab.
    pub(crate) fn archive_chat(&mut self, id: &str) {
        let active = self.chats.active.as_deref() == Some(id);
        if active {
            self.stash_chat_draft();
        }
        self.hovered_chat = None;
        self.chat_turns.remove(id);
        self.chat_feedback.remove(id);
        self.chat_drafts.remove(id);
        if self.chats.archive_chat(id) {
            if active {
                self.restore_chat_draft();
            }
            self.save_chats();
            self.notice = Some("Chat archived.".to_string());
        }
    }

    pub(crate) fn unarchive_chat(&mut self, id: &str) {
        if self.chats.unarchive_chat(id) {
            self.save_chats();
            self.notice = Some("Chat restored.".to_string());
        }
    }

    pub(crate) fn delete_chat(&mut self, id: &str) {
        let active = self.chats.active.as_deref() == Some(id);
        if active {
            self.stash_chat_draft();
        }
        self.hovered_chat = None;
        self.chat_turns.remove(id);
        self.chat_feedback.remove(id);
        self.chat_drafts.remove(id);
        if self.chats.delete_chat(id) {
            if active {
                self.restore_chat_draft();
            }
            self.save_chats();
            self.notice = Some("Chat deleted.".to_string());
        }
    }

    pub(in crate::app) fn remove_project_chats(&mut self, project_id: &str) {
        let active = self
            .active_chat()
            .is_some_and(|chat| chat.project_id == project_id);
        for chat in self.chats.chats_for(project_id) {
            self.chat_turns.remove(&chat.id);
            self.chat_drafts.remove(&chat.id);
            self.chat_feedback.remove(&chat.id);
        }
        self.chats.delete_for_project(project_id);
        if active {
            self.close_chat_menu();
            self.restore_chat_draft();
        }
    }

    pub(in crate::app) fn cancel_agent_turns(&mut self, agent: CliAgentId) {
        for turn in self
            .chat_turns
            .values_mut()
            .filter(|turn| turn.agent == agent)
        {
            turn.run.cancel();
            turn.approval = None;
            turn.stopping = true;
        }
    }

    pub(in crate::app) fn open_terminal(&mut self) {
        let project = self
            .active_chat()
            .and_then(|chat| self.chat_project(chat))
            .or_else(|| {
                let workspace = self.workspace.as_ref()?;
                let selected = workspace.selected_project_id.as_deref()?;
                workspace
                    .projects
                    .iter()
                    .find(|project| project.id == selected)
            });
        let folder = project.map(|project| PathBuf::from(&project.root_path));
        let result = folder.or_else(std::env::home_dir).map_or_else(
            || {
                Err(std::io::Error::other(
                    "No project or home folder is available.",
                ))
            },
            |folder| crate::terminal::open(&folder),
        );
        if let Err(error) = result {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadablePath,
                format!("Could not open a terminal: {error}"),
            ));
        }
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
        self.close_chat_menu();
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

    pub(crate) fn chat_agents(&self) -> impl Iterator<Item = &CliAgentStatus> {
        self.agent_detection.results.iter().filter(|agent| {
            agent.executable.is_some()
                && self.agent_enabled(agent.id)
                && ChatProvider::for_agent(agent.id).is_some()
        })
    }

    pub(crate) fn chat_project(&self, chat: &Chat) -> Option<&coderune_domain::Project> {
        self.workspace
            .as_ref()?
            .projects
            .iter()
            .find(|project| project.id == chat.project_id)
    }

    fn chat_target(&self, chat: &Chat) -> Option<CatalogKey> {
        let agent = chat.agent?;
        let executable = self
            .chat_agents()
            .find(|status| status.id == agent)?
            .executable
            .clone()?;
        let provider = ChatProvider::for_agent(agent)?;
        Some(CatalogKey {
            provider,
            project: PathBuf::from(&self.chat_project(chat)?.root_path),
            executable,
            path: self.agent_detection.path.clone(),
            model: match provider {
                ChatProvider::OpenCode | ChatProvider::Antigravity => chat.options.model.clone(),
                _ => None,
            },
        })
    }

    pub(crate) fn chat_can_send(&self, chat: &Chat) -> bool {
        !self.chat_turns.contains_key(&chat.id)
            && chat
                .agent
                .is_some_and(|id| self.chat_agents().any(|agent| agent.id == id))
            && self.chat_project(chat).is_some()
    }

    pub(crate) fn send_chat_message(&mut self) {
        let Some(chat) = self.active_chat().filter(|chat| self.chat_can_send(chat)) else {
            return;
        };
        let text = self.chat_draft.text.trim();
        if text.is_empty() {
            return;
        }
        let Some((agent, target)) = chat.agent.zip(self.chat_target(chat)) else {
            return;
        };
        let id = chat.id.clone();
        if !target.project.is_dir() {
            self.chat_feedback
                .insert(id, "The project folder is unavailable.".into());
            return;
        }
        let session = chat.provider_session.clone();
        let mut options = chat.options.clone();
        if matches!(
            target.provider,
            ChatProvider::OpenCode | ChatProvider::Antigravity
        ) && options.model.is_none()
            && self.chat_catalog.key.as_ref() == Some(&target)
        {
            if let CatalogStatus::Ready(models) = &self.chat_catalog.status {
                options.model = models
                    .iter()
                    .find(|model| model.is_default)
                    .map(|model| model.id.clone());
            }
        }
        if let Err(error) = self.chats.record_prompt(&id, text) {
            self.chat_feedback
                .insert(id, format!("Could not save the prompt: {error}"));
            return;
        }
        self.chat_feedback.remove(&id);
        self.error = None;
        let run = ChatRun::start(
            target.provider,
            &target.executable,
            &target.project,
            target.path.as_deref(),
            session.as_deref(),
            &options,
            text,
        );
        match run {
            Ok(run) => {
                self.chat_turns.insert(
                    id,
                    ChatTurn {
                        run,
                        agent,
                        reply: String::new(),
                        approval: None,
                        stopping: false,
                    },
                );
                self.chat_draft = TextEdit::default();
            }
            Err(error) => {
                self.chat_feedback
                    .insert(id, format!("Could not start {}: {error}", agent.label()));
                self.chat_draft = TextEdit::default();
            }
        }
    }

    pub(crate) fn set_chat_agent(&mut self, agent: CliAgentId) {
        let Some(chat) = self.active_chat() else {
            return;
        };
        if self.chat_turns.contains_key(&chat.id)
            || !self.chat_agents().any(|status| status.id == agent)
        {
            return;
        }
        let id = chat.id.clone();
        self.close_chat_menu();
        self.chats.set_agent(&id, Some(agent));
        self.chat_feedback.remove(&id);
        self.error = None;
        self.save_chats();
        self.refresh_chat_catalog(false);
    }

    pub(crate) fn set_chat_model(&mut self, model: Option<String>) {
        self.change_chat_options(move |options| {
            if options.model != model {
                options.model = model;
                options.effort = None;
            }
        });
        self.refresh_chat_catalog(false);
    }

    pub(crate) fn set_chat_effort(&mut self, effort: Option<String>) {
        self.change_chat_options(move |options| options.effort = effort);
    }

    fn change_chat_options(&mut self, change: impl FnOnce(&mut ChatOptions)) {
        self.close_chat_menu();
        if let Some(chat) = self.active_chat().filter(|chat| self.chat_can_send(chat)) {
            let id = chat.id.clone();
            let mut options = chat.options.clone();
            change(&mut options);
            self.chats.set_options(&id, options);
            self.save_chats();
        }
    }

    pub(crate) fn toggle_chat_menu(&mut self, menu: ChatMenu) {
        self.chat_menu = (self.chat_menu != Some(menu)).then_some(menu);
    }

    pub(crate) fn close_chat_menu(&mut self) {
        self.chat_menu = None;
        self.chat_model_menu.close();
    }

    pub(crate) fn chat_model_menu_mut(&mut self) -> &mut rune_kit::ComboboxState {
        &mut self.chat_model_menu
    }

    pub(crate) fn refresh_chat_catalog(&mut self, force: bool) {
        let key = self.active_chat().and_then(|chat| self.chat_target(chat));
        if self.chat_catalog.key == key && !force {
            return;
        }
        let status = if let Some(key) = &key {
            match ModelCatalogRun::start(
                key.provider,
                &key.executable,
                &key.project,
                key.path.as_deref(),
                key.model.as_deref(),
            ) {
                Ok(run) => CatalogStatus::Loading(run),
                Err(error) => CatalogStatus::Failed(error.to_string()),
            }
        } else {
            CatalogStatus::Unavailable
        };
        self.chat_catalog = ChatCatalog { key, status };
    }

    pub(crate) fn stop_chat_agent(&mut self) {
        if let Some(turn) = self
            .chats
            .active
            .as_ref()
            .and_then(|id| self.chat_turns.get_mut(id))
        {
            turn.run.cancel();
            turn.approval = None;
            turn.stopping = true;
        }
    }

    pub(crate) fn answer_chat_approval(&mut self, approved: bool) {
        let Some((id, turn)) = self
            .chats
            .active
            .as_ref()
            .and_then(|id| self.chat_turns.get_mut(id).map(|turn| (id, turn)))
        else {
            return;
        };
        match turn.run.answer_approval(approved) {
            Ok(()) => turn.approval = None,
            Err(error) => {
                self.chat_feedback.insert(id.clone(), error.to_string());
            }
        }
    }

    pub(in crate::app) fn poll_chats(&mut self, tx: &mut Tx) -> bool {
        self.refresh_chat_catalog(false);
        let mut changed = false;
        if let CatalogStatus::Loading(receive) = &self.chat_catalog.status {
            match receive.try_recv() {
                Ok(result) => {
                    self.chat_catalog.status = match result {
                        Ok(models) => CatalogStatus::Ready(models),
                        Err(error) => CatalogStatus::Failed(error),
                    };
                    changed = true;
                }
                Err(TryRecvError::Disconnected) => {
                    self.chat_catalog.status =
                        CatalogStatus::Failed("Model lookup ended unexpectedly.".into());
                    changed = true;
                }
                Err(TryRecvError::Empty) => tx.wake_after(Duration::from_millis(80)),
            }
        }

        let turns_changed = self.poll_chat_turns();
        if !self.chat_turns.is_empty() {
            tx.wake_after(Duration::from_millis(32));
        }
        changed || turns_changed
    }

    fn poll_chat_turns(&mut self) -> bool {
        let mut changed = false;
        let mut save = false;
        self.chat_turns.retain(|id, turn| {
            for _ in 0..64 {
                let result = match turn.run.try_recv() {
                    Ok(ChatEvent::Session(session)) => {
                        self.chats.set_provider_session(id, session);
                        save = true;
                        continue;
                    }
                    Ok(ChatEvent::Delta(delta)) => {
                        turn.reply.push_str(&delta);
                        changed = true;
                        continue;
                    }
                    Ok(ChatEvent::Approval { summary }) => {
                        if !turn.stopping {
                            turn.approval = Some(summary);
                        }
                        changed = true;
                        continue;
                    }
                    Ok(ChatEvent::Finished(result)) => result,
                    Err(TryRecvError::Empty) => return true,
                    Err(TryRecvError::Disconnected) => {
                        Err("Agent connection closed unexpectedly.".into())
                    }
                };
                if !turn.reply.trim().is_empty() {
                    self.chats
                        .add_message(id, Author::Agent(turn.agent), &turn.reply);
                    save = true;
                }
                if turn.stopping {
                    self.chat_feedback
                        .insert(id.clone(), "Response stopped.".into());
                } else if let Err(error) = result {
                    self.chat_feedback.insert(id.clone(), error);
                }
                changed = true;
                return false;
            }
            true
        });
        if save {
            self.save_chats();
        }
        changed
    }

    pub(in crate::app) fn finish_chat_turns(&mut self) {
        for turn in self.chat_turns.values_mut() {
            turn.run.cancel();
            turn.stopping = true;
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !self.chat_turns.is_empty() && std::time::Instant::now() < deadline {
            self.poll_chat_turns();
            if !self.chat_turns.is_empty() {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
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
