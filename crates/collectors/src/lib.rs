//! Local repository collection.
//!
//! Collection is explicit, read-only, local-only, and deterministic. It never
//! executes project tooling and never follows symlinks.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use coderune_domain::{
    now_iso, CaptureLimits, CapturePreview, CapturedFile, CollectionScope, GitState, SkippedFile,
};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, thiserror::Error)]
pub enum CollectError {
    #[error("{0} is not a readable directory")]
    UnreadableRoot(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not serialize capture fingerprint: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("this build cannot extract text from that document type")]
    UnsupportedDocument,
    #[error("the document exceeds the {0} byte limit")]
    DocumentTooLarge(u64),
    #[error("the document could not be read")]
    UnreadableDocument,
    #[error("the document contains no extractable text")]
    ImageOnlyDocument,
}

#[derive(Debug, Clone)]
pub struct CollectRequest {
    pub root: PathBuf,
    pub scope: CollectionScope,
    pub limits: CaptureLimits,
    pub project_id: String,
    pub project_version: u32,
    pub project_updated_at: String,
    pub profile_version: u32,
    pub profile_updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CollectedRepository {
    pub preview: CapturePreview,
    pub contents: BTreeMap<String, String>,
}

pub fn validate_root(root: &Path) -> bool {
    root.is_dir() && std::fs::read_dir(root).is_ok()
}

pub fn collect(request: &CollectRequest) -> Result<CollectedRepository, CollectError> {
    if !validate_root(&request.root) {
        return Err(CollectError::UnreadableRoot(
            request.root.display().to_string(),
        ));
    }
    // A readable root can still fail canonicalization through an unreadable
    // ancestor. Keep collecting from the validated path rather than guessing
    // that the directory itself is unusable.
    let root = request
        .root
        .canonicalize()
        .unwrap_or_else(|_| request.root.clone());
    let excludes = build_excludes(&request.scope.exclude_paths);
    let gitignore = RootIgnore::load(&root, request.scope.respect_gitignore);
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut contents = BTreeMap::new();
    let mut total_bytes = 0u64;
    let mut seen = std::collections::BTreeSet::new();

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| !protected_directory(entry, &root))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                if let Some(path) = error.path() {
                    push_skip(&mut skipped, relative(&root, path), "not-readable");
                }
                continue;
            }
        };
        if entry.depth() == 0 {
            continue;
        }
        let path = entry.path();
        let relative_path = relative(&root, path);
        if !in_scope(&relative_path, &request.scope.include_paths) {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                push_skip(&mut skipped, relative_path, "not-readable");
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            push_skip(&mut skipped, relative_path, "symbolic-link");
            continue;
        }
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        if let Some(reason) = protected_reason(&relative_path) {
            push_skip(&mut skipped, relative_path, reason);
            continue;
        }
        if let Some(reason) = excludes
            .as_ref()
            .and_then(|set| set.is_match(&relative_path).then_some("excluded"))
        {
            push_skip(&mut skipped, relative_path, reason);
            continue;
        }
        if let Some(gitignore) = gitignore.as_ref() {
            if gitignore.is_ignored(&relative_path) {
                push_skip(&mut skipped, relative_path, "ignored");
                continue;
            }
        }
        let Some(file_category) = classify(&relative_path) else {
            push_skip(&mut skipped, relative_path, "unsupported-type");
            continue;
        };
        if !seen.insert(relative_path.clone()) {
            continue;
        }
        if metadata.len() > request.limits.per_file_bytes {
            push_skip(&mut skipped, relative_path, "file-too-large");
            continue;
        }
        if total_bytes.saturating_add(metadata.len()) > request.limits.per_capture_bytes {
            push_skip(&mut skipped, relative_path, "capture-limit");
            continue;
        }
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => {
                push_skip(&mut skipped, relative_path, "not-readable");
                continue;
            }
        };
        if bytes.iter().take(8_000).any(|byte| *byte == 0) {
            push_skip(&mut skipped, relative_path, "binary");
            continue;
        }
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                push_skip(&mut skipped, relative_path, "binary");
                continue;
            }
        };
        let sha256 = sha256_hex(text.as_bytes());
        let bytes_len = text.len() as u64;
        total_bytes += bytes_len;
        contents.insert(relative_path.clone(), text);
        files.push(CapturedFile {
            path: relative_path.clone(),
            sha256,
            bytes: bytes_len,
            category: file_category.to_string(),
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.dedup_by(|left, right| left.path == right.path);

    let git = git_state(&root);
    let repository_fingerprint =
        repository_fingerprint(&files, git.as_ref(), &request.scope, &request.limits)?;
    let fingerprint = capture_fingerprint(
        &repository_fingerprint,
        request.profile_version,
        &request.profile_updated_at,
        request.project_version,
        &request.project_updated_at,
    )?;
    Ok(CollectedRepository {
        preview: CapturePreview {
            project_id: request.project_id.clone(),
            project_version: request.project_version,
            profile_version: request.profile_version,
            fingerprint,
            files,
            skipped,
            total_bytes,
            git,
            created_at: now_iso(),
        },
        contents,
    })
}

/// Read the text of a local CV or SRS. PDF and DOCX parsing happens here, in
/// the collectors crate: no project code runs, no network is used, and the
/// original file is never copied into application storage.
pub fn extract_document_text(
    path: &Path,
    max_bytes: u64,
) -> Result<(String, String, usize), CollectError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "md" | "markdown" | "txt" | "pdf" | "docx"
    ) {
        return Err(CollectError::UnsupportedDocument);
    }
    let metadata = std::fs::metadata(path).map_err(|_| CollectError::UnreadableDocument)?;
    if !metadata.is_file() {
        return Err(CollectError::UnreadableDocument);
    }
    if metadata.len() > max_bytes {
        return Err(CollectError::DocumentTooLarge(max_bytes));
    }
    let text = match extension.as_str() {
        "md" | "markdown" | "txt" => {
            std::fs::read_to_string(path).map_err(|_| CollectError::UnreadableDocument)?
        }
        "docx" => {
            let bytes = std::fs::read(path).map_err(|_| CollectError::UnreadableDocument)?;
            docx_text(&bytes, max_bytes)?
        }
        "pdf" => {
            let bytes = std::fs::read(path).map_err(|_| CollectError::UnreadableDocument)?;
            pdf_extract::extract_text_from_mem(&bytes)
                .map_err(|_| CollectError::UnreadableDocument)?
        }
        _ => unreachable!(),
    };
    if text.trim().is_empty() {
        return Err(CollectError::ImageOnlyDocument);
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document")
        .to_string();
    let bytes = text.len();
    Ok((name, text, bytes))
}

/// Word stores visible text in `word/document.xml`; paragraphs are `w:p`
/// elements and their runs are `w:t`. A bounded read keeps a compressed
/// archive from expanding into an unbounded string.
fn docx_text(bytes: &[u8], max_bytes: u64) -> Result<String, CollectError> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| CollectError::UnreadableDocument)?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|_| CollectError::UnreadableDocument)?;
    let mut xml = String::new();
    document
        .by_ref()
        .take(max_bytes.saturating_add(1))
        .read_to_string(&mut xml)
        .map_err(|_| CollectError::UnreadableDocument)?;
    if xml.len() as u64 > max_bytes {
        return Err(CollectError::DocumentTooLarge(max_bytes));
    }
    let mut text = String::new();
    let mut rest = xml.as_str();
    loop {
        let next_run = rest.find("<w:t");
        let next_paragraph = rest.find("</w:p>");
        let next_break = rest.find("<w:br");
        let next_tab = rest.find("<w:tab");
        let here = [next_run, next_paragraph, next_break, next_tab]
            .into_iter()
            .flatten()
            .min();
        let Some(at) = here else {
            break;
        };
        if Some(at) == next_paragraph {
            text.push('\n');
            rest = &rest[at + "</w:p>".len()..];
            continue;
        }
        if Some(at) == next_break || Some(at) == next_tab {
            text.push(if Some(at) == next_break { '\n' } else { '\t' });
            let close = if Some(at) == next_break {
                "<w:br"
            } else {
                "<w:tab"
            };
            rest = &rest[at + close.len()..];
            if let Some(slash) = rest.find('>') {
                rest = &rest[slash + 1..];
            }
            continue;
        }
        rest = &rest[at..];
        let Some(open_end) = rest.find('>') else {
            break;
        };
        rest = &rest[open_end + 1..];
        let Some(end) = rest.find("</w:t>") else {
            break;
        };
        text.push_str(&decode_xml_entities(&rest[..end]));
        rest = &rest[end + "</w:t>".len()..];
    }
    Ok(text)
}

fn decode_xml_entities(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        text.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find(';') else {
            text.push_str(rest);
            return text;
        };
        let entity = &rest[1..end];
        match entity {
            "amp" => text.push('&'),
            "lt" => text.push('<'),
            "gt" => text.push('>'),
            "quot" => text.push('"'),
            "apos" => text.push('\''),
            _ => {
                let code = entity
                    .strip_prefix("#x")
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()));
                match code.and_then(char::from_u32) {
                    Some(character) => text.push(character),
                    None => text.push_str(&rest[..=end]),
                }
            }
        }
        rest = &rest[end + 1..];
    }
    text.push_str(rest);
    text
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn repository_fingerprint(
    files: &[CapturedFile],
    git: Option<&GitState>,
    scope: &CollectionScope,
    limits: &CaptureLimits,
) -> Result<String, CollectError> {
    #[derive(serde::Serialize)]
    struct Payload<'a> {
        files: Vec<(&'a str, &'a str)>,
        git: Option<(&'a Option<String>, &'a Option<String>, bool)>,
        scope: &'a CollectionScope,
        limits: &'a CaptureLimits,
    }
    let payload = Payload {
        files: files
            .iter()
            .map(|file| (file.path.as_str(), file.sha256.as_str()))
            .collect(),
        git: git.map(|git| (&git.head, &git.branch, git.dirty)),
        scope,
        limits,
    };
    Ok(sha256_hex(&serde_json::to_vec(&payload)?))
}

pub fn capture_fingerprint(
    repository: &str,
    profile_version: u32,
    profile_updated_at: &str,
    project_version: u32,
    project_updated_at: &str,
) -> Result<String, CollectError> {
    #[derive(serde::Serialize)]
    struct Payload<'a> {
        repository: &'a str,
        #[serde(rename = "profileVersion")]
        profile_version: u32,
        #[serde(rename = "profileUpdatedAt")]
        profile_updated_at: &'a str,
        #[serde(rename = "projectVersion")]
        project_version: u32,
        #[serde(rename = "projectUpdatedAt")]
        project_updated_at: &'a str,
    }
    Ok(sha256_hex(&serde_json::to_vec(&Payload {
        repository,
        profile_version,
        profile_updated_at,
        project_version,
        project_updated_at,
    })?))
}

fn push_skip(skipped: &mut Vec<SkippedFile>, path: String, reason: &str) {
    skipped.push(SkippedFile {
        path,
        reason: reason.to_string(),
    });
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn in_scope(path: &str, includes: &[String]) -> bool {
    if includes.is_empty() {
        return true;
    }
    includes.iter().any(|include| {
        let include = include
            .trim()
            .trim_start_matches("./")
            .trim_end_matches('/');
        path == include || path.starts_with(&format!("{include}/"))
    })
}

fn build_excludes(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = pattern.trim().trim_start_matches("./");
        if pattern.is_empty() {
            continue;
        }
        for candidate in [
            pattern.to_string(),
            format!("{pattern}/**"),
            format!("**/{pattern}"),
            format!("**/{pattern}/**"),
        ] {
            if let Ok(glob) = Glob::new(&candidate) {
                builder.add(glob);
            }
        }
    }
    builder.build().ok()
}

/// Only the root `.gitignore` is read. Rules are applied in order and `!`
/// negations un-ignore a later match, mirroring the increment's documented
/// behaviour without loading nested files.
struct RootIgnore {
    rules: Vec<(bool, GlobSet)>,
}

impl RootIgnore {
    fn load(root: &Path, respect: bool) -> Option<Self> {
        if !respect {
            return None;
        }
        let text = std::fs::read_to_string(root.join(".gitignore")).ok()?;
        let mut rules = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (ignore, pattern) = match line.strip_prefix('!') {
                Some(rest) => (false, rest.trim()),
                None => (true, line),
            };
            let pattern = pattern.trim_start_matches('/');
            if pattern.is_empty() {
                continue;
            }
            let mut builder = GlobSetBuilder::new();
            for candidate in [
                pattern.to_string(),
                format!("{pattern}/**"),
                format!("**/{pattern}"),
                format!("**/{pattern}/**"),
            ] {
                if let Ok(glob) = Glob::new(&candidate) {
                    builder.add(glob);
                }
            }
            if let Ok(set) = builder.build() {
                rules.push((ignore, set));
            }
        }
        Some(Self { rules })
    }

    fn is_ignored(&self, path: &str) -> bool {
        let mut ignored = false;
        for (ignore, set) in &self.rules {
            if set.is_match(path) {
                ignored = *ignore;
            }
        }
        ignored
    }
}

fn protected_directory(entry: &DirEntry, root: &Path) -> bool {
    if !entry.file_type().is_dir() || relative(root, entry.path()).is_empty() {
        return false;
    }
    let name = entry.file_name().to_str().unwrap_or_default();
    PROTECTED_DIRECTORIES.contains(&name)
}

fn protected_reason(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    if name == ".env"
        || name.starts_with(".env.")
        || matches!(name, ".npmrc" | ".pypirc" | ".netrc")
    {
        return Some("environment-file");
    }
    if matches!(name, "id_rsa" | "id_ed25519" | "id_ecdsa")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".p12")
        || lower.ends_with(".pfx")
    {
        return Some("private-key");
    }
    if BINARY_EXTENSIONS
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        return Some("binary");
    }
    None
}

fn classify(path: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    if lower == "agents.md" {
        return Some("agent-instructions");
    }
    if lower.starts_with("readme") {
        return Some("readme");
    }
    if MANIFESTS.contains(&lower.as_str())
        || lower.starts_with("requirements") && lower.ends_with(".txt")
    {
        return Some("manifest");
    }
    if CONFIGURATION_EXACT.contains(&lower.as_str())
        || CONFIGURATION_PREFIXES
            .iter()
            .any(|prefix| lower.starts_with(prefix))
    {
        return Some("configuration");
    }
    if is_test_path(path) {
        return Some("test");
    }
    if SOURCE_EXTENSIONS
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        return Some("source");
    }
    if DOCUMENT_EXTENSIONS
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        return Some("documentation");
    }
    None
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    lower
        .split('/')
        .any(|part| part == "__tests__" || part == "test" || part == "tests")
        || name.contains(".test.")
        || name.contains(".spec.")
        || (name.starts_with("test_") && name.ends_with(".py"))
        || name.ends_with("_test.py")
}

fn git_state(root: &Path) -> Option<GitState> {
    let inside = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()?;
    if !inside.status.success()
        || !String::from_utf8_lossy(&inside.stdout)
            .trim()
            .starts_with("true")
    {
        return None;
    }
    let head = git_value(root, &["rev-parse", "HEAD"]);
    let branch = git_value(root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let status = git_value(root, &["status", "--porcelain=v1"]).unwrap_or_default();
    let dirty = !status.trim().is_empty();
    let summary = if dirty {
        format!("{} changed path(s)", status.lines().count())
    } else {
        "Clean working tree".to_string()
    };
    Some(GitState {
        branch,
        head,
        dirty,
        summary,
    })
}

fn git_value(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

const MANIFESTS: &[&str] = &[
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
    "pyproject.toml",
    "poetry.lock",
    "pipfile",
    "pipfile.lock",
    "uv.lock",
    "setup.py",
    "setup.cfg",
    "requirements.txt",
];

const CONFIGURATION_EXACT: &[&str] = &[
    "tsconfig.json",
    "jsconfig.json",
    "turbo.json",
    "nx.json",
    "pytest.ini",
    "tox.ini",
    "mypy.ini",
    "ruff.toml",
    "pylintrc",
    ".editorconfig",
    ".python-version",
    ".nvmrc",
    ".tool-versions",
];

const CONFIGURATION_PREFIXES: &[&str] = &[
    "tsconfig.",
    "vite.config.",
    "vitest.config.",
    "jest.config.",
    "eslint.config.",
    ".eslintrc",
    ".prettierrc",
];

const SOURCE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".py",
];
const DOCUMENT_EXTENSIONS: &[&str] = &[".md", ".markdown", ".txt"];

const PROTECTED_DIRECTORIES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "env",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".turbo",
    ".next",
    ".nuxt",
    ".cache",
    "dist",
    "build",
    "out",
    "coverage",
    "vendor",
    "target",
];

const BINARY_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico", ".bmp", ".tiff", ".mp3", ".mp4", ".mov",
    ".avi", ".wav", ".pdf", ".docx", ".doc", ".xlsx", ".zip", ".gz", ".tar", ".rar", ".7z", ".exe",
    ".dll", ".so", ".dylib", ".bin", ".woff", ".woff2", ".ttf", ".otf", ".sqlite", ".db",
];

#[cfg(test)]
mod tests {
    use super::*;
    use coderune_domain::{CaptureLimits, CollectionScope};

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        std::fs::write(dir.path().join("README.md"), "# Demo").unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        std::fs::write(dir.path().join("src/main.ts"), "console.log('hi')").unwrap();
        std::fs::write(dir.path().join("src/main.test.ts"), "test('x', () => {})").unwrap();
        std::fs::write(dir.path().join(".env"), "SECRET=1").unwrap();
        std::fs::write(dir.path().join("node_modules/pkg/index.js"), "ignored").unwrap();
        std::fs::write(dir.path().join("logo.png"), [0u8, 1, 2]).unwrap();
        dir
    }

    fn request(root: &Path) -> CollectRequest {
        CollectRequest {
            root: root.to_path_buf(),
            scope: CollectionScope::default(),
            limits: CaptureLimits::default(),
            project_id: "project".to_string(),
            project_version: 1,
            project_updated_at: "2026-01-01T00:00:00.000Z".to_string(),
            profile_version: 1,
            profile_updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn collects_allowed_files_and_explains_skips() {
        let dir = fixture();
        let collected = collect(&request(dir.path())).unwrap();
        let paths: Vec<_> = collected
            .preview
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(
            paths,
            vec![
                "README.md",
                "package.json",
                "src/main.test.ts",
                "src/main.ts"
            ]
        );
        assert_eq!(collected.preview.files[2].category, "test");
        let skipped: Vec<_> = collected
            .preview
            .skipped
            .iter()
            .map(|file| (file.path.as_str(), file.reason.as_str()))
            .collect();
        assert!(skipped.contains(&(".env", "environment-file")));
        assert!(skipped.contains(&("logo.png", "binary")));
        assert!(skipped
            .iter()
            .all(|(path, _)| !path.starts_with("node_modules/")));
    }

    #[test]
    fn fingerprint_changes_with_content() {
        let dir = fixture();
        let first = collect(&request(dir.path())).unwrap();
        std::fs::write(dir.path().join("src/main.ts"), "console.log('changed')").unwrap();
        let second = collect(&request(dir.path())).unwrap();
        assert_ne!(first.preview.fingerprint, second.preview.fingerprint);
    }

    #[test]
    fn capture_limit_skips_remaining_files() {
        let dir = fixture();
        let mut request = request(dir.path());
        request.limits.per_capture_bytes = 8;
        let collected = collect(&request).unwrap();
        assert!(collected.preview.total_bytes <= 8);
        assert!(collected
            .preview
            .skipped
            .iter()
            .any(|file| file.reason == "capture-limit"));
    }

    #[test]
    fn respects_root_gitignore() {
        let dir = fixture();
        std::fs::write(dir.path().join(".gitignore"), "src/main.ts\n").unwrap();
        let collected = collect(&request(dir.path())).unwrap();
        assert!(collected
            .preview
            .files
            .iter()
            .all(|file| file.path != "src/main.ts"));
    }

    #[test]
    fn docx_extraction_keeps_paragraphs_and_entities() {
        use std::io::Write;

        let dir = fixture();
        let path = dir.path().join("cv.docx");
        let xml = r#"<?xml version="1.0"?><w:document><w:body>
            <w:p><w:r><w:t>Ada Lovelace</w:t></w:r></w:p>
            <w:p><w:r><w:t>Rust &amp; TypeScript</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let file = std::fs::File::create(&path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file(
                "word/document.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(xml.as_bytes()).unwrap();
        archive.finish().unwrap();

        let (name, text, bytes) = extract_document_text(&path, 1024 * 1024).unwrap();
        assert_eq!(name, "cv.docx");
        assert!(text.contains("Ada Lovelace"));
        assert!(text.contains("Rust & TypeScript"));
        assert!(text.contains('\n'));
        assert_eq!(bytes, text.len());
    }

    #[test]
    fn unsupported_and_oversized_documents_are_explained() {
        let dir = fixture();
        let path = dir.path().join("cv.rtf");
        std::fs::write(&path, "{\\rtf1 Ada}").unwrap();
        assert!(matches!(
            extract_document_text(&path, 1024),
            Err(CollectError::UnsupportedDocument)
        ));
        let text = dir.path().join("cv.txt");
        std::fs::write(&text, "Ada Lovelace").unwrap();
        assert!(matches!(
            extract_document_text(&text, 4),
            Err(CollectError::DocumentTooLarge(4))
        ));
    }
}
