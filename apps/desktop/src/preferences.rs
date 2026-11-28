//! Local desktop appearance and coding-agent preferences.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use coderune_agents::CliAgentId;
use serde::{Deserialize, Serialize};

/// Where the workspace's top-level navigation lives. Settings › Preferences draws
/// one card for each, and the shell switches around the page while you pick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NavigationMode {
    #[default]
    Sidebar,
    TopBar,
}

impl NavigationMode {
    pub(crate) const ALL: [Self; 2] = [Self::Sidebar, Self::TopBar];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sidebar => "Sidebar",
            Self::TopBar => "Top bar",
        }
    }

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::TopBar => "top-bar",
        }
    }

    pub(crate) fn detail(self) -> &'static str {
        match self {
            Self::Sidebar => "Destinations and chats down the left side.",
            Self::TopBar => "Destinations across the window's title bar.",
        }
    }
}

/// The desktop notifications an agent turn may raise. The switches persist now;
/// delivery is wired where each event is handled.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Notifications {
    pub(crate) approvals: bool,
    pub(crate) finished: bool,
    pub(crate) errors: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            approvals: true,
            finished: true,
            errors: true,
        }
    }
}

/// One of the notification switch rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Notify {
    Approvals,
    Finished,
    Errors,
}

impl Notifications {
    pub(crate) fn get(self, kind: Notify) -> bool {
        match kind {
            Notify::Approvals => self.approvals,
            Notify::Finished => self.finished,
            Notify::Errors => self.errors,
        }
    }

    pub(crate) fn set(&mut self, kind: Notify, on: bool) {
        match kind {
            Notify::Approvals => self.approvals = on,
            Notify::Finished => self.finished = on,
            Notify::Errors => self.errors = on,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Preferences {
    pub(crate) dark_mode: bool,
    pub(crate) notifications: Notifications,
    /// Where top-level navigation lives: the sidebar or the title bar.
    pub(crate) navigation: NavigationMode,
    /// The family behind ordinary text, and the one behind code and paths.
    pub(crate) ui_font: Option<String>,
    pub(crate) mono_font: Option<String>,
    /// Projects whose chats are listed under them.
    pub(crate) expanded_projects: Vec<String>,
    disabled_agents: Vec<CliAgentId>,
}

impl Preferences {
    pub(crate) fn path(data_path: &Path) -> PathBuf {
        data_path.with_file_name("desktop-settings.json")
    }

    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        match File::open(path) {
            Ok(file) => serde_json::from_reader(io::BufReader::new(file)).map_err(io::Error::other),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn update(path: &Path, change: impl FnOnce(&mut Self)) -> io::Result<Self> {
        let folder = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
        let folder = folder.unwrap_or(Path::new("."));
        fs::create_dir_all(folder)?;
        // Lock a separate file so replacing the JSON leaves the lock in place.
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("lock"))?;
        lock.lock()?;
        let mut preferences = Self::load(path)?;
        change(&mut preferences);
        let file = tempfile::NamedTempFile::new_in(folder)?;
        serde_json::to_writer(file.as_file(), &preferences).map_err(io::Error::other)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(preferences)
    }

    pub(crate) fn project_expanded(&self, id: &str) -> bool {
        self.expanded_projects.iter().any(|project| project == id)
    }

    pub(crate) fn set_project_expanded(&mut self, id: &str, expanded: bool) {
        if expanded {
            if !self.project_expanded(id) {
                self.expanded_projects.push(id.to_string());
            }
        } else {
            self.expanded_projects.retain(|project| project != id);
        }
    }

    pub(crate) fn agent_enabled(&self, agent: CliAgentId) -> bool {
        !self.disabled_agents.contains(&agent)
    }

    pub(crate) fn set_agent_enabled(&mut self, agent: CliAgentId, enabled: bool) {
        if enabled {
            self.disabled_agents.retain(|&id| id != agent);
        } else if self.agent_enabled(agent) {
            self.disabled_agents.push(agent);
        }
    }
}
