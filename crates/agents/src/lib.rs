//! External coding-agent detection and draft requests.
//!
//! Coderune never owns sign-in, providers, models, or credentials. Agents are
//! detected on `PATH` and invoked only for an explicitly confirmed draft job.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use coderune_domain::{
    AgentAvailability, AgentDraft, AgentId, AgentStatus, AppError, DocumentKind, ErrorCode,
    ImportedDocument,
};

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub id: AgentId,
    pub command: &'static str,
    pub max_input_bytes: usize,
    pub capabilities: &'static [&'static str],
    pub restrictions: &'static [&'static str],
}

pub const DEFINITIONS: &[AgentDefinition] = &[
    AgentDefinition {
        id: AgentId::Codex,
        command: "codex",
        max_input_bytes: 20 * 1024 * 1024,
        capabilities: &["structured output", "streaming", "cancellation"],
        restrictions: &["read-only sandbox", "fresh temporary working directory"],
    },
    AgentDefinition {
        id: AgentId::ClaudeCode,
        command: "claude",
        max_input_bytes: 10 * 1024 * 1024,
        capabilities: &["structured output", "cancellation"],
        restrictions: &[
            "non-interactive",
            "tools denied",
            "fresh temporary working directory",
        ],
    },
    AgentDefinition {
        id: AgentId::OpenCode,
        command: "opencode",
        max_input_bytes: 20 * 1024 * 1024,
        capabilities: &["streaming", "cancellation", "ACP v1"],
        restrictions: &[
            "ACP permissions rejected",
            "fresh temporary working directory",
        ],
    },
    AgentDefinition {
        id: AgentId::Antigravity,
        command: "agy",
        max_input_bytes: 20 * 1024 * 1024,
        capabilities: &["structured output", "streaming", "cancellation"],
        restrictions: &["sandbox mode", "fresh temporary working directory"],
    },
];

pub fn definition(id: AgentId) -> &'static AgentDefinition {
    match id {
        AgentId::Codex => &DEFINITIONS[0],
        AgentId::ClaudeCode => &DEFINITIONS[1],
        AgentId::OpenCode => &DEFINITIONS[2],
        AgentId::Antigravity => &DEFINITIONS[3],
    }
}

/// Detect one agent with an injectable lookup so detection is testable.
pub fn detect_with(id: AgentId, lookup: &impl Fn(&str) -> Option<PathBuf>) -> AgentStatus {
    let definition = definition(id);
    let found = lookup(definition.command);
    let availability = if found.is_some() {
        AgentAvailability::Ready
    } else {
        AgentAvailability::NotInstalled
    };
    let detail = match &found {
        Some(path) => format!(
            "Found at {}. Sign-in, provider, and model choice remain with this tool.",
            path.display()
        ),
        None => format!(
            "Not found on PATH. Install `{}` or open its setup documentation.",
            definition.command
        ),
    };
    AgentStatus {
        id,
        availability,
        detail,
        setup_url: id.setup_url().to_string(),
        verified: false,
        capabilities: definition
            .capabilities
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        restrictions: definition
            .restrictions
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

pub fn detect_all() -> Vec<AgentStatus> {
    AgentId::ALL
        .iter()
        .map(|id| detect_with(*id, &which))
        .collect()
}

pub fn detect_all_with(lookup: impl Fn(&str) -> Option<PathBuf>) -> Vec<AgentStatus> {
    AgentId::ALL
        .iter()
        .map(|id| detect_with(*id, &lookup))
        .collect()
}

pub fn which(command: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(command);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate = directory.join(format!("{command}.exe"));
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DraftSharingPreview {
    pub agent: AgentId,
    pub kind: DocumentKind,
    pub text: String,
    #[serde(rename = "estimatedBytes")]
    pub estimated_bytes: usize,
    #[serde(rename = "maxInputBytes")]
    pub max_input_bytes: usize,
    pub fits: bool,
}

pub fn prepare_preview(
    agent: AgentId,
    kind: DocumentKind,
    document: &ImportedDocument,
    project_name: Option<&str>,
) -> DraftSharingPreview {
    let text = build_request(kind, document, project_name);
    let estimated_bytes = text.len();
    let max_input_bytes = definition(agent).max_input_bytes;
    DraftSharingPreview {
        agent,
        kind,
        text,
        estimated_bytes,
        max_input_bytes,
        fits: estimated_bytes <= max_input_bytes,
    }
}

pub fn build_request(
    kind: DocumentKind,
    document: &ImportedDocument,
    project_name: Option<&str>,
) -> String {
    let kind_name = match kind {
        DocumentKind::Cv => "CV",
        DocumentKind::Srs => "SRS",
    };
    let project = project_name
        .map(|name| format!("Project: {name}\n"))
        .unwrap_or_default();
    let schema = match kind {
        DocumentKind::Cv => {
            r#"{"kind":"cv","displayName"?:string,"experienceStage"?:"new-to-coding"|"early-career"|"mid-career"|"experienced","newToCoding"?:boolean,"skillsStatement"?:string,"learningGoals"?:string[],"selfReportedSkills"?:{"topic":string,"rating"?:"beginner"|"intermediate"|"advanced"|null}[],"education"?:..., "employment"?:..., "projects"?:..., "certifications"?:..., "links"?:..., "learningPreferences"?:...,"sources":{"field":string,"reference"?:string}[],"warnings":string[]}"#
        }
        DocumentKind::Srs => {
            r#"{"kind":"srs","purpose"?:string,"businessLogic"?:string,"goals"?:string[],"rules"?:string[],"constraints"?:string[],"decisions"?:string[],"acceptanceCriteria"?:string[],"knowledge"?:string[],"sources":{"field":string,"reference"?:string}[],"warnings":string[]}"#
        }
    };
    format!(
        "You are extracting a reviewed {kind_name} draft for Coderune.\n\
         Use only facts present in the source text. Do not invent experience, skills, goals, or requirements.\n\
         Return one JSON object only. Do not wrap it in prose or a markdown fence.\n\
         Omit absent optional fields. Include source references for extracted fields.\n\
         The developer will review and confirm every field before it is saved.\n\
         Document: {name}\nKind: {kind_name}\nSHA-256: {hash}\n{project}\n\
         Required shape: {schema}\n\n--- SOURCE TEXT ---\n{text}",
        name = document.name,
        hash = document.sha256,
        text = document.text,
    )
}

/// Request a CV/SRS draft from Claude Code's documented headless mode. Other
/// adapters stay at detection-only until their protocol work is demonstrated.
pub fn request_draft(preview: &DraftSharingPreview) -> std::result::Result<AgentDraft, AppError> {
    if !preview.fits {
        return Err(AppError::new(
            ErrorCode::AgentInputTooLarge,
            "The selected text exceeds this agent's input limit.",
        ));
    }
    match preview.agent {
        AgentId::ClaudeCode => run_claude(preview),
        _ => Err(AppError::new(
            ErrorCode::AgentUnavailable,
            "This agent adapter is not implemented in the native build yet. Use manual entry or Claude Code.",
        )),
    }
}

fn run_claude(preview: &DraftSharingPreview) -> std::result::Result<AgentDraft, AppError> {
    let mut child = Command::new("claude")
        .args([
            "--print",
            "--output-format",
            "json",
            "--permission-mode",
            "dontAsk",
            "--disallowedTools",
            "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| {
            AppError::new(
                ErrorCode::AgentUnavailable,
                "Claude Code could not be started. Install it or sign in with its own CLI.",
            )
        })?;
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(preview.text.as_bytes());
    }
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                return Err(AppError::new(
                    ErrorCode::AgentTimeout,
                    "The agent did not answer within 120 seconds.",
                ));
            }
            Err(_) => {
                return Err(AppError::new(
                    ErrorCode::AgentFailed,
                    "The agent process failed.",
                ));
            }
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|_| AppError::new(ErrorCode::AgentFailed, "The agent process failed."))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        let code = if stderr.contains("login")
            || stderr.contains("sign in")
            || stderr.contains("auth")
            || stderr.contains("unauthorized")
        {
            ErrorCode::AgentUnavailable
        } else {
            ErrorCode::AgentFailed
        };
        return Err(AppError::new(
            code,
            "The agent returned an error. Manual entry remains available.",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|_| {
        AppError::new(
            ErrorCode::AgentInvalidOutput,
            "The agent output was not valid JSON.",
        )
    })?;
    let value = envelope
        .get("structured_output")
        .or_else(|| envelope.get("result"))
        .cloned()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::AgentInvalidOutput,
                "The agent returned no draft.",
            )
        })?;
    validate_draft(preview.kind, value).map(|payload| AgentDraft {
        kind: preview.kind,
        agent: preview.agent,
        payload,
    })
}

pub fn validate_draft(
    kind: DocumentKind,
    value: serde_json::Value,
) -> std::result::Result<serde_json::Value, AppError> {
    let object = value.as_object().ok_or_else(|| {
        AppError::new(
            ErrorCode::AgentInvalidOutput,
            "The draft must be a JSON object.",
        )
    })?;
    let expected = match kind {
        DocumentKind::Cv => "cv",
        DocumentKind::Srs => "srs",
    };
    if object.get("kind").and_then(|value| value.as_str()) != Some(expected) {
        return Err(AppError::new(
            ErrorCode::AgentInvalidOutput,
            format!("The draft kind must be `{expected}`."),
        ));
    }
    let allowed: &[&str] = match kind {
        DocumentKind::Cv => &[
            "kind",
            "displayName",
            "experienceStage",
            "newToCoding",
            "skillsStatement",
            "learningGoals",
            "selfReportedSkills",
            "education",
            "employment",
            "projects",
            "certifications",
            "links",
            "learningPreferences",
            "sources",
            "warnings",
        ],
        DocumentKind::Srs => &[
            "kind",
            "purpose",
            "businessLogic",
            "goals",
            "rules",
            "constraints",
            "decisions",
            "acceptanceCriteria",
            "knowledge",
            "sources",
            "warnings",
        ],
    };
    if let Some(unknown) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(AppError::new(
            ErrorCode::AgentInvalidOutput,
            format!("The draft contains unsupported field `{unknown}`."),
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use coderune_domain::now_iso;

    fn document(kind: DocumentKind) -> ImportedDocument {
        ImportedDocument {
            id: "doc".to_string(),
            kind,
            project_id: None,
            name: "cv.md".to_string(),
            source_path: None,
            sha256: "abc".to_string(),
            text: "Ada built a Rust app.".to_string(),
            text_bytes: 23,
            created_at: now_iso(),
        }
    }

    #[test]
    fn detection_uses_injected_lookup() {
        let statuses = detect_all_with(|command| {
            (command == "claude").then(|| PathBuf::from("/usr/local/bin/claude"))
        });
        assert_eq!(statuses.len(), 4);
        assert_eq!(statuses[1].availability, AgentAvailability::Ready);
        assert_eq!(statuses[0].availability, AgentAvailability::NotInstalled);
        assert!(statuses.iter().all(|status| !status.verified));
    }

    #[test]
    fn preview_reports_fit_and_exact_bytes() {
        let preview = prepare_preview(
            AgentId::ClaudeCode,
            DocumentKind::Cv,
            &document(DocumentKind::Cv),
            None,
        );
        assert!(preview.fits);
        assert!(preview.text.contains("Ada built a Rust app."));
        assert!(preview.text.contains("\"kind\":\"cv\""));
    }

    #[test]
    fn draft_validation_rejects_unknown_keys_and_wrong_kind() {
        assert!(validate_draft(DocumentKind::Cv, serde_json::json!({"kind":"srs"})).is_err());
        assert!(
            validate_draft(DocumentKind::Cv, serde_json::json!({"kind":"cv","extra":1})).is_err()
        );
        assert!(validate_draft(
            DocumentKind::Cv,
            serde_json::json!({"kind":"cv","displayName":"Ada"})
        )
        .is_ok());
    }
}
