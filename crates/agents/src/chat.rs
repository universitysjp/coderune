//! Structured conversations with installed coding agents.

mod acp;
mod claude;
mod codex;
mod process;

use std::cell::Cell;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use self::process::SessionProcess;
use crate::CliAgentId;

/// Installed coding agents with a native project chat adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatProvider {
    Codex,
    ClaudeCode,
    OpenCode,
    Antigravity,
}

impl ChatProvider {
    /// Returns the adapter for an agent that supports project chats.
    pub fn for_agent(agent: CliAgentId) -> Option<Self> {
        match agent {
            CliAgentId::Codex => Some(Self::Codex),
            CliAgentId::ClaudeCode => Some(Self::ClaudeCode),
            CliAgentId::OpenCode => Some(Self::OpenCode),
            CliAgentId::Antigravity => Some(Self::Antigravity),
            _ => None,
        }
    }
}

/// Choices passed to the coding agent for a single turn. `None` leaves the
/// corresponding option to the agent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatOptions {
    pub model: Option<String>,
    pub effort: Option<String>,
}

/// A selectable model in the agent's catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub label: String,
    pub efforts: Vec<Effort>,
    pub is_default: bool,
}

/// A reasoning effort supported by a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effort {
    pub id: String,
    pub label: String,
}

/// Changes produced by a turn. A run emits exactly one final result while its
/// receiver remains connected.
#[derive(Debug)]
pub enum ChatEvent {
    Session(String),
    Delta(String),
    Approval { summary: String },
    Finished(Result<(), String>),
}

struct TurnRequest {
    session: Option<String>,
    options: ChatOptions,
    prompt: String,
}

/// A background turn bound to one project chat. Dropping the handle cancels it
/// and leaves process cleanup to its worker.
pub struct ChatRun {
    events: Receiver<ChatEvent>,
    approvals: SyncSender<bool>,
    approval_pending: Cell<bool>,
    stopped: Arc<AtomicBool>,
}

impl ChatRun {
    /// Starts one turn, optionally resuming a provider session. Startup and
    /// protocol failures are reported through [`ChatEvent::Finished`].
    pub fn start(
        provider: ChatProvider,
        executable: &Path,
        cwd: &Path,
        path: Option<&OsStr>,
        session: Option<&str>,
        options: &ChatOptions,
        prompt: &str,
    ) -> io::Result<Self> {
        let command = provider_command(executable, cwd, path)?;
        let request = TurnRequest {
            session: session.map(str::to_owned),
            options: options.clone(),
            prompt: prompt.to_owned(),
        };
        let (sender, events) = mpsc::sync_channel(64);
        let (approvals, answers) = mpsc::sync_channel(1);
        let stopped = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::clone(&stopped);
        thread::Builder::new()
            .name("coderune-chat-turn".into())
            .spawn(move || {
                let adapter = match provider {
                    ChatProvider::Codex => codex::run,
                    ChatProvider::ClaudeCode => claude::run,
                    ChatProvider::OpenCode | ChatProvider::Antigravity => acp::run,
                };
                let result = adapter(command, &request, &sender, &cancelled, &answers);
                let _ = sender.send(ChatEvent::Finished(result));
            })?;
        Ok(Self {
            events,
            approvals,
            approval_pending: Cell::new(false),
            stopped,
        })
    }

    /// Reads the next queued event without blocking the caller.
    pub fn try_recv(&self) -> Result<ChatEvent, TryRecvError> {
        let event = self.events.try_recv()?;
        match &event {
            ChatEvent::Approval { .. } if !self.stopped.load(Ordering::Relaxed) => {
                self.approval_pending.set(true);
            }
            ChatEvent::Finished(_) => self.approval_pending.set(false),
            _ => {}
        }
        Ok(event)
    }

    /// Answers the approval currently displayed for this run.
    pub fn answer_approval(&self, approved: bool) -> io::Result<()> {
        if !self.approval_pending.get() {
            return Err(io::Error::other("No agent approval is pending."));
        }
        self.approvals
            .try_send(approved)
            .map_err(io::Error::other)?;
        self.approval_pending.set(false);
        Ok(())
    }

    /// Requests cancellation while keeping the event stream available for the
    /// worker's final status.
    pub fn cancel(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.approval_pending.set(false);
    }
}

impl Drop for ChatRun {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// A background model lookup. Dropping the handle cancels the provider probe.
pub struct ModelCatalogRun {
    result: Receiver<Result<Vec<Model>, String>>,
    stopped: Arc<AtomicBool>,
}

impl ModelCatalogRun {
    /// Starts reading the available models and reasoning efforts for a project.
    pub fn start(
        provider: ChatProvider,
        executable: &Path,
        cwd: &Path,
        path: Option<&OsStr>,
        selected_model: Option<&str>,
    ) -> io::Result<Self> {
        let command = provider_command(executable, cwd, path)?;
        let selected_model = selected_model.map(str::to_owned);
        let (sender, result) = mpsc::channel();
        let stopped = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::clone(&stopped);
        thread::Builder::new()
            .name("coderune-chat-models".into())
            .spawn(move || {
                let models = read_models(provider, command, selected_model.as_deref(), &cancelled);
                let _ = sender.send(models);
            })?;
        Ok(Self { result, stopped })
    }

    /// Reads the lookup result if the provider has replied.
    pub fn try_recv(&self) -> Result<Result<Vec<Model>, String>, TryRecvError> {
        self.result.try_recv()
    }
}

impl Drop for ModelCatalogRun {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
}

/// Loads available models and reasoning efforts in the project directory.
/// Use [`ModelCatalogRun`] for lookups from the desktop event loop.
pub fn load_models(
    provider: ChatProvider,
    executable: &Path,
    cwd: &Path,
    path: Option<&OsStr>,
    selected_model: Option<&str>,
) -> Result<Vec<Model>, String> {
    let command = provider_command(executable, cwd, path).map_err(|error| error.to_string())?;
    read_models(provider, command, selected_model, &AtomicBool::new(false))
}

fn read_models(
    provider: ChatProvider,
    command: Command,
    selected_model: Option<&str>,
    stopped: &AtomicBool,
) -> Result<Vec<Model>, String> {
    if stopped.load(Ordering::Relaxed) {
        return Err("Model lookup cancelled.".into());
    }
    match provider {
        ChatProvider::Codex => codex::models(command, stopped),
        ChatProvider::ClaudeCode => claude::models(command, stopped),
        ChatProvider::OpenCode | ChatProvider::Antigravity => {
            acp::models(command, selected_model, stopped)
        }
    }
}

fn provider_command(executable: &Path, cwd: &Path, path: Option<&OsStr>) -> io::Result<Command> {
    if !executable.is_absolute() || !cwd.is_absolute() || !cwd.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Choose an installed agent and an available project folder.",
        ));
    }
    let mut command = Command::new(executable);
    command.current_dir(cwd);
    if let Some(path) = path {
        command.env("PATH", path);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW keeps the CLI's console hidden in the desktop app.
        command.creation_flags(0x08000000);
    }
    Ok(command)
}

fn request_approval(
    events: &SyncSender<ChatEvent>,
    approvals: &Receiver<bool>,
    stopped: &AtomicBool,
    summary: String,
) -> Result<bool, String> {
    emit(events, ChatEvent::Approval { summary })?;
    while !stopped.load(Ordering::Relaxed) {
        match approvals.recv_timeout(Duration::from_millis(50)) {
            Ok(approved) if !stopped.load(Ordering::Relaxed) => return Ok(approved),
            Err(RecvTimeoutError::Timeout) => continue,
            _ => break,
        }
    }
    Err("Turn cancelled.".into())
}

fn emit(events: &SyncSender<ChatEvent>, event: ChatEvent) -> Result<(), String> {
    events.send(event).map_err(|_| "Chat closed.".into())
}
