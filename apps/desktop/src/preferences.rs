//! Local desktop appearance and coding-agent preferences.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use coderune_agents::CliAgentId;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Preferences {
    pub(crate) dark_mode: bool,
    /// The family behind ordinary text, and the one behind code and paths.
    pub(crate) ui_font: Option<String>,
    pub(crate) mono_font: Option<String>,
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
