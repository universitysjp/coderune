//! Local chats, the tabs that hold them, and the agent each chat points at.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use coderune_agents::{chat::ChatOptions, CliAgentId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Who wrote a message. Coderune writes the notes the app itself adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Author {
    Developer,
    Coderune,
    Agent(CliAgentId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Message {
    pub author: Author,
    pub text: String,
    pub at: String,
}

/// One conversation inside a project. A chat keeps the agent chosen for it,
/// so returning to a tab returns to the same context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Chat {
    pub id: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub agent: Option<CliAgentId>,
    #[serde(default)]
    pub options: ChatOptions,
    #[serde(default)]
    pub provider_session: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub messages: Vec<Message>,
}

impl Chat {
    pub(crate) fn new(project_id: &str) -> Self {
        let at = coderune_domain::now_iso();
        Self {
            id: coderune_domain::new_id(),
            project_id: project_id.to_string(),
            title: "New chat".to_string(),
            agent: None,
            options: ChatOptions::default(),
            provider_session: None,
            created_at: at.clone(),
            updated_at: at,
            messages: Vec::new(),
        }
    }

    /// The first words of a message make a better tab label than "New chat".
    fn retitle(&mut self, text: &str) {
        if self.title != "New chat" || self.messages.iter().any(|m| m.author == Author::Developer) {
            return;
        }
        let mut title: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if title.is_empty() {
            return;
        }
        if title.chars().count() > 42 {
            title = title.chars().take(42).collect::<String>();
            title.push('…');
        }
        self.title = title;
    }
}

/// The chats on disk and the tabs open in this window.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Chats {
    pub chats: Vec<Chat>,
    /// Tab order. A chat can exist without being open.
    pub open: Vec<String>,
    pub active: Option<String>,
    #[serde(skip)]
    path: Option<PathBuf>,
    #[serde(skip)]
    saved_digest: Option<[u8; 32]>,
}

impl Chats {
    pub(crate) fn path(data_path: &Path) -> PathBuf {
        data_path.with_file_name("chats.json")
    }

    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let mut chats = match read_saved_chats(path)? {
            Some(contents) => {
                let mut chats: Self =
                    serde_json::from_slice(&contents).map_err(io::Error::other)?;
                chats.saved_digest = Some(Sha256::digest(&contents).into());
                chats
            }
            None => Self::default(),
        };
        chats.path = Some(path.to_path_buf());
        // Drop tabs whose chat vanished, then the active one if it closed elsewhere.
        chats
            .open
            .retain(|id| chats.chats.iter().any(|chat| &chat.id == id));
        if chats
            .active
            .as_ref()
            .is_some_and(|id| !chats.open.contains(id))
        {
            chats.active = chats.open.last().cloned();
        }
        Ok(chats)
    }

    pub(crate) fn unreadable(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            ..Self::default()
        }
    }

    pub(crate) fn save(&mut self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let folder = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(folder)?;
        let _lock = lock_chats(path)?;
        let current_digest: Option<[u8; 32]> =
            read_saved_chats(path)?.map(|bytes| Sha256::digest(bytes).into());
        if current_digest != self.saved_digest {
            return Err(io::Error::other(
                "Saved chats changed. Reopen Coderune before editing them.",
            ));
        }
        let contents = serde_json::to_vec(self).map_err(io::Error::other)?;
        let mut file = tempfile::NamedTempFile::new_in(folder)?;
        file.write_all(&contents)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        self.saved_digest = Some(Sha256::digest(contents).into());
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> io::Result<()> {
        self.chats.clear();
        self.open.clear();
        self.active = None;
        if let Some(path) = &self.path {
            let _lock = lock_chats(path)?;
            match fs::remove_file(path) {
                Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
                _ => {}
            }
        }
        self.saved_digest = None;
        Ok(())
    }

    pub(crate) fn active_chat(&self) -> Option<&Chat> {
        let id = self.active.as_deref()?;
        self.chat(id)
    }

    pub(crate) fn chat(&self, id: &str) -> Option<&Chat> {
        self.chats.iter().find(|chat| chat.id == id)
    }

    fn chat_mut(&mut self, id: &str) -> Option<&mut Chat> {
        self.chats.iter_mut().find(|chat| chat.id == id)
    }

    pub(crate) fn chats_for<'a>(&'a self, project_id: &'a str) -> impl Iterator<Item = &'a Chat> {
        self.chats
            .iter()
            .filter(move |chat| chat.project_id == project_id)
    }

    /// Creates a chat in `project_id`, opens its tab, and makes it the active one.
    pub(crate) fn new_chat(&mut self, project_id: &str) -> String {
        let chat = Chat::new(project_id);
        let id = chat.id.clone();
        self.chats.push(chat);
        self.open.push(id.clone());
        self.active = Some(id.clone());
        id
    }

    pub(crate) fn open_chat(&mut self, id: &str) {
        if self.chat(id).is_none() {
            return;
        }
        if !self.open.iter().any(|open| open == id) {
            self.open.push(id.to_string());
        }
        self.active = Some(id.to_string());
    }

    /// Closes a tab and activates its neighbour. The chat itself is kept.
    pub(crate) fn close_chat(&mut self, id: &str) {
        let Some(index) = self.open.iter().position(|open| open == id) else {
            return;
        };
        self.open.remove(index);
        if self.active.as_deref() == Some(id) {
            self.active = self
                .open
                .get(index.saturating_sub(1))
                .or_else(|| self.open.first())
                .cloned();
        }
    }

    pub(crate) fn delete_for_project(&mut self, project_id: &str) {
        let closed: Vec<String> = self
            .chats
            .iter()
            .filter(|chat| chat.project_id == project_id)
            .map(|chat| chat.id.clone())
            .collect();
        self.chats.retain(|chat| chat.project_id != project_id);
        self.open.retain(|id| !closed.contains(id));
        if self.active.as_ref().is_some_and(|id| closed.contains(id)) {
            self.active = self.open.first().cloned();
        }
    }

    pub(crate) fn add_message(&mut self, id: &str, author: Author, text: &str) -> bool {
        let Some(chat) = self.chat_mut(id) else {
            return false;
        };
        if author == Author::Developer {
            chat.retitle(text);
        }
        chat.updated_at = coderune_domain::now_iso();
        chat.messages.push(Message {
            author,
            text: text.to_string(),
            at: chat.updated_at.clone(),
        });
        true
    }

    /// Save the prompt before the provider sees it. Leave the draft retryable
    /// if another window changed local storage or the disk write failed.
    pub(crate) fn record_prompt(&mut self, id: &str, text: &str) -> io::Result<()> {
        let Some(chat) = self.chat(id) else {
            return Err(io::Error::new(ErrorKind::NotFound, "Chat was removed"));
        };
        let title = chat.title.clone();
        let updated_at = chat.updated_at.clone();
        self.add_message(id, Author::Developer, text);
        if let Err(error) = self.save() {
            if let Some(chat) = self.chat_mut(id) {
                chat.messages.pop();
                chat.title = title;
                chat.updated_at = updated_at;
            }
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn set_agent(&mut self, id: &str, agent: Option<CliAgentId>) -> bool {
        let Some(chat) = self.chat_mut(id) else {
            return false;
        };
        if chat.agent != agent {
            chat.agent = agent;
            chat.options = ChatOptions::default();
            chat.provider_session = None;
        }
        chat.updated_at = coderune_domain::now_iso();
        true
    }

    pub(crate) fn set_options(&mut self, id: &str, options: ChatOptions) {
        if let Some(chat) = self.chat_mut(id) {
            chat.options = options;
        }
    }

    pub(crate) fn set_provider_session(&mut self, id: &str, session: String) {
        if let Some(chat) = self.chat_mut(id) {
            chat.provider_session = Some(session);
        }
    }
}

fn read_saved_chats(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn lock_chats(path: &Path) -> io::Result<File> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))?;
    lock.lock()?;
    Ok(lock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_window_cannot_resurrect_deleted_chats() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("chats.json");
        let mut first = Chats::load(&path).unwrap();
        let chat = first.new_chat("project-id");
        first.save().unwrap();
        let mut other = Chats::load(&path).unwrap();
        first.clear().unwrap();
        other.add_message(&chat, Author::Developer, "A late message");
        assert!(other.save().is_err());
        assert!(!path.exists());
    }

    #[test]
    fn damaged_chat_file_can_be_deleted_without_being_overwritten() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("chats.json");
        fs::write(&path, "incomplete data").unwrap();
        assert!(Chats::load(&path).is_err());
        let mut chats = Chats::unreadable(path.clone());
        assert!(chats.save().is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "incomplete data");
        chats.clear().unwrap();
        assert!(!path.exists());
    }
}
