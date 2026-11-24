//! Local chats, the tabs that hold them, and the agent each chat points at.

use std::fs::{self, File};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use coderune_agents::CliAgentId;
use serde::{Deserialize, Serialize};

/// Who wrote a message. Coderune writes the notes the app itself adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Author {
    Developer,
    Coderune,
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
}

impl Chats {
    pub(crate) fn path(data_path: &Path) -> PathBuf {
        data_path.with_file_name("chats.json")
    }

    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let mut chats = match File::open(path) {
            Ok(file) => serde_json::from_reader(io::BufReader::new(file)).map_err(io::Error::other)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error),
        };
        chats.path = Some(path.to_path_buf());
        // Drop tabs whose chat vanished, then the active one if it closed elsewhere.
        chats.open.retain(|id| chats.chats.iter().any(|chat| &chat.id == id));
        if chats.active.as_ref().is_some_and(|id| !chats.open.contains(id)) {
            chats.active = chats.open.last().cloned();
        }
        Ok(chats)
    }

    pub(crate) fn save(&self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let folder = path.parent().filter(|parent| !parent.as_os_str().is_empty());
        fs::create_dir_all(folder.unwrap_or(Path::new(".")))?;
        let file = tempfile::NamedTempFile::new_in(folder.unwrap_or(Path::new(".")))?;
        serde_json::to_writer(file.as_file(), self).map_err(io::Error::other)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
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
        chat.retitle(text);
        chat.updated_at = coderune_domain::now_iso();
        chat.messages.push(Message {
            author,
            text: text.to_string(),
            at: chat.updated_at.clone(),
        });
        true
    }

    pub(crate) fn set_agent(&mut self, id: &str, agent: Option<CliAgentId>) -> bool {
        let Some(chat) = self.chat_mut(id) else {
            return false;
        };
        chat.agent = agent;
        chat.updated_at = coderune_domain::now_iso();
        true
    }
}
