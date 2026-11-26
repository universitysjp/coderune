//! Discover installed coding-agent executables without starting the agents.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use coderune_domain::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CliAgentId {
    ClaudeCode,
    Codex,
    Grok,
    Kiro,
    OpenCode,
    Pi,
    Muse,
    Amp,
    Devin,
    Cursor,
    Fx,
    Jcode,
    Antigravity,
}

use CliAgentId::*;

impl CliAgentId {
    /// The user-facing name shared by discovery and saved conversations.
    pub fn label(self) -> &'static str {
        CLI_AGENTS
            .iter()
            .find(|(id, _, _, _)| *id == self)
            .map_or("Coding agent", |(_, name, _, _)| name)
    }
}

impl From<AgentId> for CliAgentId {
    fn from(id: AgentId) -> Self {
        match id {
            AgentId::Codex => Codex,
            AgentId::ClaudeCode => ClaudeCode,
            AgentId::OpenCode => OpenCode,
            AgentId::Antigravity => Antigravity,
        }
    }
}

const CLI_AGENTS: &[(CliAgentId, &str, &str, &str)] = &[
    (
        ClaudeCode,
        "Claude Code",
        "claude",
        "https://code.claude.com/docs/en/setup",
    ),
    (
        Codex,
        "Codex",
        "codex",
        "https://learn.chatgpt.com/docs/codex/cli",
    ),
    (Grok, "Grok Build", "grok", "https://x.ai/build"),
    (Kiro, "Kiro", "kiro-cli", "https://kiro.dev/docs/cli/"),
    (
        OpenCode,
        "OpenCode",
        "opencode",
        "https://opencode.ai/docs/",
    ),
    (Pi, "Pi", "pi", "https://pi.dev/"),
    (
        Muse,
        "Muse Code",
        "muse",
        "https://dev.meta.ai/docs/muse-code",
    ),
    (Amp, "Amp", "amp", "https://ampcode.com/docs/cli"),
    (Devin, "Devin", "devin", "https://docs.devin.ai/cli"),
    (
        Cursor,
        "Cursor",
        "cursor-agent",
        "https://cursor.com/docs/cli/installation",
    ),
    (
        Fx,
        "fx",
        "fx",
        "https://fx.sh/docs/getting-started/installation",
    ),
    (
        Jcode,
        "jcode",
        "jcode",
        "https://www.j-code.net/docs/get-started",
    ),
    (
        Antigravity,
        "Antigravity CLI",
        "agy",
        "https://antigravity.google/docs/cli-getting-started",
    ),
];

/// Whether a known coding agent's CLI was found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliAgentStatus {
    pub id: CliAgentId,
    pub name: &'static str,
    pub command: &'static str,
    pub install_url: &'static str,
    pub executable: Option<PathBuf>,
}

/// Re-read the user's shell PATH and check for each known executable.
///
/// Shell startup can take a few seconds, so call from a background worker.
pub fn detect_cli_agents() -> Vec<CliAgentStatus> {
    detect_cli_agents_with_path().0
}

/// Detect coding agents and supply the same PATH to their child processes.
pub fn detect_cli_agents_with_path() -> (Vec<CliAgentStatus>, Option<OsString>) {
    let path = SearchPath::discover();
    (detect_cli_agents_on(&path), path.joined().ok())
}

fn detect_cli_agents_on(path: &SearchPath) -> Vec<CliAgentStatus> {
    CLI_AGENTS
        .iter()
        .map(|&(id, name, command, install_url)| {
            let executable = match id {
                Cursor => path
                    .find(command)
                    .or_else(|| path.find_matching("agent", is_cursor_alias)),
                Fx => path.find_matching(command, is_fx_agent),
                _ => path.find(command),
            };
            CliAgentStatus {
                id,
                name,
                command,
                install_url,
                executable,
            }
        })
        .collect()
}

fn is_fx_agent(path: &Path) -> bool {
    use std::io::Read;

    // The JSON viewer also installs `fx`. The coding agent embeds the fx.sh
    // address for its own updates. Identify it without running either tool.
    const MARKER: &[u8] = b"https://fx.sh";
    const MAX_BINARY_BYTES: u64 = 16 * 1024 * 1024;
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut bytes = Vec::new();
    file.take(MAX_BINARY_BYTES).read_to_end(&mut bytes).is_ok()
        && bytes.windows(MARKER.len()).any(|part| part == MARKER)
}

fn is_cursor_alias(alias: &Path) -> bool {
    // Grok also installs an `agent` launcher. Only accept the alternate
    // command when its symlink points to a Cursor installation.
    let Ok(target) = alias.canonicalize() else {
        return false;
    };
    target
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .any(|name| {
            name.eq_ignore_ascii_case("cursor-agent")
                || name.eq_ignore_ascii_case("cursor-agent.exe")
        })
}

pub(super) struct SearchPath(Vec<PathBuf>);

impl SearchPath {
    /// Search the inherited PATH and conventional installation directories.
    pub(super) fn inherited() -> Self {
        Self::new(None)
    }

    /// Include the interactive login shell's PATH, even for Finder launches.
    pub(super) fn discover() -> Self {
        Self::new(login_shell_path())
    }

    fn new(shell_path: Option<OsString>) -> Self {
        let inherited = env::var_os("PATH");
        let mut dirs = Vec::new();
        for path in [shell_path, inherited].into_iter().flatten() {
            for dir in env::split_paths(&path) {
                add_dir(&mut dirs, dir);
            }
        }

        if let Some(home) = env::home_dir() {
            for dir in [
                ".local/bin",
                ".claude/local",
                ".opencode/bin",
                ".bun/bin",
                ".npm-global/bin",
                ".local/share/mise/shims",
                ".asdf/shims",
                ".volta/bin",
            ] {
                add_dir(&mut dirs, home.join(dir));
            }
            #[cfg(windows)]
            add_dir(&mut dirs, home.join("scoop/shims"));
        }

        #[cfg(unix)]
        for dir in [
            "/opt/homebrew/bin",
            "/home/linuxbrew/.linuxbrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ] {
            add_dir(&mut dirs, PathBuf::from(dir));
        }
        #[cfg(windows)]
        {
            if let Some(app_data) = env::var_os("APPDATA") {
                add_dir(&mut dirs, PathBuf::from(app_data).join("npm"));
            }
            if let Some(app_data) = env::var_os("LOCALAPPDATA") {
                let local = PathBuf::from(app_data);
                for dir in ["Microsoft/WinGet/Links", "agy/bin"] {
                    add_dir(&mut dirs, local.join(dir));
                }
            }
        }
        Self(dirs)
    }

    pub(super) fn find(&self, command: &str) -> Option<PathBuf> {
        self.find_matching(command, |_| true)
    }

    fn find_matching(
        &self,
        command: &str,
        matches_agent: impl Fn(&Path) -> bool,
    ) -> Option<PathBuf> {
        #[cfg(windows)]
        let extensions = windows_extensions();
        for directory in &self.0 {
            let candidate = directory.join(command);
            // npm installs an extensionless Unix shim next to the Windows launcher.
            if (!cfg!(windows) || candidate.extension().is_some())
                && is_executable(&candidate)
                && matches_agent(&candidate)
            {
                return Some(candidate);
            }
            #[cfg(windows)]
            for extension in &extensions {
                let candidate = directory.join(format!("{command}{extension}"));
                if is_executable(&candidate) && matches_agent(&candidate) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    pub(super) fn joined(&self) -> Result<OsString, env::JoinPathsError> {
        env::join_paths(&self.0)
    }
}

fn add_dir(dirs: &mut Vec<PathBuf>, dir: PathBuf) {
    // Empty and relative PATH entries would search the active project.
    if dir.is_absolute() && !dirs.contains(&dir) {
        dirs.push(dir);
    }
}

#[cfg(windows)]
fn windows_extensions() -> Vec<String> {
    env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
        .split(';')
        .filter(|extension| extension.starts_with('.'))
        .map(str::to_ascii_lowercase)
        .collect()
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(not(unix))]
fn login_shell_path() -> Option<OsString> {
    None
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[test]
    fn fx_discovery_skips_the_json_viewer_and_finds_the_coding_agent() {
        let base = env::temp_dir().join(format!("coderune-cli-fx-{}", std::process::id()));
        let viewer = base.join("viewer");
        let agent = base.join("agent");
        for (directory, bytes) in [
            (&viewer, b"github.com/antonmedv/fx".as_slice()),
            (&agent, b"https://fx.sh".as_slice()),
        ] {
            fs::create_dir_all(directory).unwrap();
            let executable = directory.join("fx");
            fs::write(&executable, bytes).unwrap();
            fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let find_fx = |dirs| {
            detect_cli_agents_on(&SearchPath(dirs))
                .into_iter()
                .find(|status| status.id == Fx)
                .unwrap()
                .executable
        };
        let missing = find_fx(vec![viewer.clone()]);
        let found = find_fx(vec![viewer, agent.clone()]);
        fs::remove_dir_all(base).unwrap();
        assert_eq!(missing, None);
        assert_eq!(found, Some(agent.join("fx")));
    }

    #[test]
    fn cursor_alias_is_accepted_only_when_it_resolves_to_cursor() {
        let base = env::temp_dir().join(format!("coderune-cli-alias-{}", std::process::id()));
        fs::create_dir(&base).unwrap();
        let search = SearchPath(vec![base.clone()]);
        let cursor_path = || {
            detect_cli_agents_on(&search)
                .into_iter()
                .find(|agent| agent.id == Cursor)
                .unwrap()
                .executable
        };
        let alias = base.join("agent");

        let grok = base.join("grok");
        fs::write(&grok, "").unwrap();
        fs::set_permissions(&grok, fs::Permissions::from_mode(0o755)).unwrap();
        symlink(&grok, &alias).unwrap();
        let grok_alias = cursor_path();

        fs::remove_file(&alias).unwrap();
        let cursor = base.join("cursor-agent/versions/agent");
        fs::create_dir_all(cursor.parent().unwrap()).unwrap();
        fs::write(&cursor, "").unwrap();
        fs::set_permissions(&cursor, fs::Permissions::from_mode(0o755)).unwrap();
        symlink(cursor, &alias).unwrap();
        let cursor_alias = cursor_path();
        fs::remove_dir_all(base).unwrap();

        assert_eq!(grok_alias, None);
        assert_eq!(cursor_alias, Some(alias));
    }
}

#[cfg(unix)]
fn login_shell_path() -> Option<OsString> {
    use std::io::{ErrorKind, Read};
    use std::os::fd::OwnedFd;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::net::UnixStream;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const MARKER: &[u8] = b"\0CODERUNE_PATH\0";
    const OUTPUT_LIMIT: u64 = 128 * 1024;
    let default_shell = if cfg!(target_os = "macos") {
        "/bin/zsh"
    } else {
        "/bin/sh"
    };
    let shell = env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(default_shell));
    // A shell startup script can spawn a child that keeps stdout open.
    // Use a nonblocking reader so the deadline covers both process and output.
    let (mut reader, writer) = UnixStream::pair().ok()?;
    reader.set_nonblocking(true).ok()?;
    let mut command = Command::new(shell);
    command
        .args([
            "-ilc",
            "printf '\\000CODERUNE_PATH\\000'; exec /usr/bin/printenv PATH",
        ])
        .env("DISABLE_AUTO_UPDATE", "true")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(writer)));
    if let Some(home) = env::home_dir().filter(|home| home.is_absolute()) {
        command.current_dir(home);
    }
    let mut child = command.spawn().ok()?;
    drop(command);

    let deadline = Instant::now() + Duration::from_secs(4);
    let mut output = Vec::new();
    loop {
        let status = child.try_wait();
        let remaining = OUTPUT_LIMIT + 1 - output.len() as u64;
        let read = (&mut reader).take(remaining).read_to_end(&mut output);
        let bad_read = read.is_err_and(|error| error.kind() != ErrorKind::WouldBlock);
        if bad_read || output.len() as u64 > OUTPUT_LIMIT {
            break;
        }
        match status {
            Ok(Some(status)) if status.success() => {
                let marker = output
                    .windows(MARKER.len())
                    .rposition(|part| part == MARKER)?;
                let path = &output[marker + MARKER.len()..];
                return Some(OsString::from_vec(path.strip_suffix(b"\n")?.to_vec()));
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            _ => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}
