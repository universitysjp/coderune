//! Read versions from local CLIs and their publishers off the UI thread.

use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use coderune_agents::{CliAgentId, CliAgentStatus};
use rune::Tx;
use semver::Version;
use serde::Deserialize;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const RELEASE_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_POLL: Duration = Duration::from_millis(40);
const MAX_VERSION_BYTES: u64 = 4096;
const MAX_RELEASE_BYTES: u64 = 256 * 1024;

#[derive(Default)]
pub(super) struct VersionChecks {
    found: Vec<(CliAgentId, VersionInfo)>,
    job: Option<VersionJob>,
}

pub(crate) struct VersionInfo {
    pub(crate) installed: Version,
    pub(crate) update: Option<AvailableUpdate>,
}

pub(crate) struct AvailableUpdate {
    pub(crate) version: Version,
    pub(crate) instructions: &'static str,
}

struct VersionJob {
    receiver: Receiver<(CliAgentId, VersionInfo)>,
    cancelled: Arc<AtomicBool>,
}

impl Drop for VersionJob {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl VersionChecks {
    pub(super) fn start(agents: &[CliAgentStatus], path: Option<&OsStr>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        for agent in agents {
            let Some(executable) = agent.executable.clone() else {
                continue;
            };
            let id = agent.id;
            let path = path.map(OsStr::to_os_string);
            let sender = sender.clone();
            let cancelled = Arc::clone(&cancelled);
            let _ = thread::Builder::new()
                .name("coderune-agent-version".into())
                .spawn(move || {
                    check_agent(id, &executable, path.as_deref(), &sender, &cancelled);
                });
        }
        Self {
            found: Vec::new(),
            job: Some(VersionJob {
                receiver,
                cancelled,
            }),
        }
    }

    pub(super) fn pending(&self) -> bool {
        self.job.is_some()
    }

    pub(super) fn get(&self, agent: CliAgentId) -> Option<&VersionInfo> {
        self.found
            .iter()
            .find_map(|(id, info)| (*id == agent).then_some(info))
    }

    pub(super) fn poll(&mut self, tx: &mut Tx) -> bool {
        let Some(job) = &self.job else {
            return false;
        };
        let mut changed = false;
        loop {
            match job.receiver.try_recv() {
                Ok((id, info)) => {
                    if let Some((_, previous)) =
                        self.found.iter_mut().find(|(agent, _)| *agent == id)
                    {
                        *previous = info;
                    } else {
                        self.found.push((id, info));
                    }
                    changed = true;
                }
                Err(TryRecvError::Empty) => {
                    tx.wake_after(Duration::from_millis(100));
                    return changed;
                }
                Err(TryRecvError::Disconnected) => {
                    self.job = None;
                    return true;
                }
            }
        }
    }
}

fn check_agent(
    id: CliAgentId,
    executable: &Path,
    path: Option<&OsStr>,
    sender: &Sender<(CliAgentId, VersionInfo)>,
    cancelled: &AtomicBool,
) {
    let Some(installed) = installed_version(id, executable, path, cancelled) else {
        return;
    };
    if sender
        .send((
            id,
            VersionInfo {
                installed: installed.clone(),
                update: None,
            },
        ))
        .is_err()
        || cancelled.load(Ordering::Relaxed)
    {
        return;
    }
    if let Some(update) =
        Publisher::for_agent(id, &installed).and_then(|publisher| publisher.check(&installed))
    {
        let _ = sender.send((
            id,
            VersionInfo {
                installed,
                update: Some(update),
            },
        ));
    }
}

fn installed_version(
    id: CliAgentId,
    executable: &Path,
    path: Option<&OsStr>,
    cancelled: &AtomicBool,
) -> Option<Version> {
    if cancelled.load(Ordering::Relaxed) {
        return None;
    }
    let mut output = tempfile::tempfile().ok()?;
    let mut command = Command::new(executable);
    command
        .arg(if id == CliAgentId::Grok {
            "version"
        } else {
            "--version"
        })
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::null())
        .stdout(output.try_clone().ok()?)
        .stderr(Stdio::null());
    if let Some(path) = path {
        command.env("PATH", path);
    }
    let mut child = command.spawn().ok()?;
    let deadline = Instant::now() + PROBE_TIMEOUT;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if Instant::now() < deadline && !cancelled.load(Ordering::Relaxed) => {
                thread::sleep(PROBE_POLL);
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
        }
    };
    if !success {
        return None;
    }
    output.rewind().ok()?;
    let mut version = String::new();
    output
        .take(MAX_VERSION_BYTES)
        .read_to_string(&mut version)
        .ok()?;
    version
        .split_whitespace()
        .find_map(|word| Version::parse(word.trim_start_matches('v')).ok())
}

struct Publisher {
    release_url: &'static str,
    instructions: &'static str,
    format: ReleaseFormat,
}

enum ReleaseFormat {
    GitHub,
    Package,
    Plain,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

#[derive(Deserialize)]
struct PackageRelease {
    version: String,
}

impl Publisher {
    fn for_agent(id: CliAgentId, installed: &Version) -> Option<Self> {
        Some(match id {
            CliAgentId::Codex => Self {
                release_url: "https://api.github.com/repos/openai/codex/releases/latest",
                instructions: "https://learn.chatgpt.com/docs/codex/cli#install-codex",
                format: ReleaseFormat::GitHub,
            },
            CliAgentId::ClaudeCode => Self {
                release_url: "https://downloads.claude.ai/claude-code-releases/stable",
                instructions: "https://code.claude.com/docs/en/setup#update-manually",
                format: ReleaseFormat::Plain,
            },
            CliAgentId::OpenCode => Self {
                release_url: if installed.major >= 2 {
                    "https://registry.npmjs.org/@opencode%2Fcli/latest"
                } else {
                    "https://registry.npmjs.org/opencode-ai/latest"
                },
                instructions: "https://opencode.ai/docs/cli/commands/#upgrade",
                format: ReleaseFormat::Package,
            },
            _ => return None,
        })
    }

    fn check(&self, installed: &Version) -> Option<AvailableUpdate> {
        let mut response = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(RELEASE_TIMEOUT))
            .build()
            .new_agent()
            .get(self.release_url)
            .header(
                "User-Agent",
                concat!("Coderune/", env!("CARGO_PKG_VERSION")),
            )
            .call()
            .ok()?;
        let mut payload = String::new();
        response
            .body_mut()
            .with_config()
            .limit(MAX_RELEASE_BYTES)
            .reader()
            .read_to_string(&mut payload)
            .ok()?;
        self.available_from(installed, &payload)
    }

    fn available_from(&self, installed: &Version, payload: &str) -> Option<AvailableUpdate> {
        let raw = match self.format {
            ReleaseFormat::GitHub => {
                let release: GitHubRelease = serde_json::from_str(payload).ok()?;
                if release.draft || release.prerelease {
                    return None;
                }
                release.tag_name.strip_prefix("rust-v")?.to_owned()
            }
            ReleaseFormat::Package => {
                serde_json::from_str::<PackageRelease>(payload)
                    .ok()?
                    .version
            }
            ReleaseFormat::Plain => payload.trim().to_owned(),
        };
        let version = Version::parse(&raw).ok()?;
        (version.pre.is_empty() && version.cmp_precedence(installed).is_gt()).then_some(
            AvailableUpdate {
                version,
                instructions: self.instructions,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn installed_cli_version_is_reported_without_a_release_source() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let executable = dir.path().join("agy");
        fs::write(
            &executable,
            "#!/bin/sh\n[ \"$1\" = --version ] || exit 1\nprintf '1.5.2\\n'\n",
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();

        let (sender, receiver) = mpsc::channel();
        check_agent(
            CliAgentId::Antigravity,
            &executable,
            None,
            &sender,
            &AtomicBool::new(false),
        );

        let (agent, version) = receiver.try_recv().unwrap();
        assert_eq!(agent, CliAgentId::Antigravity);
        assert_eq!(version.installed.to_string(), "1.5.2");
        assert!(version.update.is_none());
    }

    #[test]
    fn codex_update_appears_only_for_a_newer_stable_release() {
        let installed = Version::parse("0.167.0").unwrap();
        let publisher = Publisher::for_agent(CliAgentId::Codex, &installed).unwrap();
        let update = publisher
            .available_from(
                &installed,
                r#"{"tag_name":"rust-v0.168.0","draft":false,"prerelease":false}"#,
            )
            .unwrap();
        assert_eq!(update.version.to_string(), "0.168.0");
        for payload in [
            r#"{"tag_name":"rust-v0.168.0-beta.1","draft":false,"prerelease":true}"#,
            r#"{"tag_name":"rust-v0.168.0","draft":true,"prerelease":false}"#,
            r#"{"tag_name":"rust-v0.167.0","draft":false,"prerelease":false}"#,
        ] {
            assert!(publisher.available_from(&installed, payload).is_none());
        }
    }
}
