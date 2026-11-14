//! Local repository collection.
//!
//! Collection is explicit, read-only, local-only, and deterministic. It never
//! executes project tooling and never follows symlinks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use coderune_domain::{
    now_iso, CaptureLimits, CapturePreview, CapturedFile, CollectionScope, GitState, SkippedFile,
};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub const DEFAULT_PER_FILE_BYTES: u64 = 1024 * 1024;
pub const DEFAULT_PER_CAPTURE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum CollectError {
    #[error("{0} is not a readable directory")]
    UnreadableRoot(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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

pub fn validate_root(root: &str) -> bool {
    let path = Path::new(root);
    path.is_dir() && std::fs::read_dir(path).is_ok()
}

pub fn collect(request: &CollectRequest) -> Result<CollectedRepository, CollectError> {
    if !validate_root(&request.root.display().to_string()) {
        return Err(CollectError::UnreadableRoot(request.root.display().to_string()));
    }
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
        .filter_entry(|entry| !protected_directory(entry.path(), &root))
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
        if let Some(reason) = excludes.as_ref().and_then(|set| {
            set.is_match(&relative_path).then_some("excluded")
        }) {
            push_skip(&mut skipped, relative_path, reason);
            continue;
        }
        if let Some(gitignore) = gitignore.as_ref() {
            if gitignore.is_ignored(&relative_path) {
                push_skip(&mut skipped, relative_path, "ignored");
                continue;
            }
        }
        if !allowed_file(&relative_path) {
            push_skip(&mut skipped, relative_path, "unsupported-type");
            continue;
        }
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
            category: category(&relative_path).to_string(),
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.dedup_by(|left, right| left.path == right.path);

    let git = git_state(&root);
    let repository_fingerprint = repository_fingerprint(&files, git.as_ref(), &request.scope, &request.limits);
    let fingerprint = capture_fingerprint(
        &repository_fingerprint,
        request.profile_version,
        &request.profile_updated_at,
        request.project_version,
        &request.project_updated_at,
    );
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

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn repository_fingerprint(
    files: &[CapturedFile],
    git: Option<&GitState>,
    scope: &CollectionScope,
    limits: &CaptureLimits,
) -> String {
    #[derive(serde::Serialize)]
    struct Payload<'a> {
        files: Vec<(&'a str, &'a str)>,
        git: Option<(&'a Option<String>, &'a Option<String>, bool)>,
        scope: &'a CollectionScope,
        limits: &'a CaptureLimits,
    }
    let payload = Payload {
        files: files.iter().map(|file| (file.path.as_str(), file.sha256.as_str())).collect(),
        git: git.map(|git| (&git.head, &git.branch, git.dirty)),
        scope,
        limits,
    };
    sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
}

pub fn capture_fingerprint(
    repository: &str,
    profile_version: u32,
    profile_updated_at: &str,
    project_version: u32,
    project_updated_at: &str,
) -> String {
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
    sha256_hex(
        &serde_json::to_vec(&Payload {
            repository,
            profile_version,
            profile_updated_at,
            project_version,
            project_updated_at,
        })
        .unwrap_or_default(),
    )
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
        let include = include.trim().trim_start_matches("./").trim_end_matches('/');
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

fn protected_directory(path: &Path, root: &Path) -> bool {
    let relative = relative(root, path);
    if relative.is_empty() {
        return false;
    }
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    PROTECTED_DIRECTORIES.contains(&name)
}

fn protected_reason(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    if name == ".env" || name.starts_with(".env.") || matches!(name, ".npmrc" | ".pypirc" | ".netrc") {
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
    if BINARY_EXTENSIONS.iter().any(|extension| lower.ends_with(extension)) {
        return Some("binary");
    }
    if GENERATED_DIRECTORIES.iter().any(|directory| lower.split('/').any(|part| part == *directory)) {
        return Some("generated-output");
    }
    if DEPENDENCY_DIRECTORIES.iter().any(|directory| lower.split('/').any(|part| part == *directory)) {
        return Some("dependency-directory");
    }
    None
}

fn allowed_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    if lower == "agents.md" || lower.starts_with("readme") {
        return true;
    }
    if MANIFESTS.contains(&lower.as_str()) || lower.starts_with("requirements") && lower.ends_with(".txt") {
        return true;
    }
    if CONFIGURATION_EXACT.contains(&lower.as_str()) {
        return true;
    }
    if CONFIGURATION_PREFIXES.iter().any(|prefix| lower.starts_with(prefix)) {
        return true;
    }
    SOURCE_EXTENSIONS
        .iter()
        .chain(DOCUMENT_EXTENSIONS.iter())
        .any(|extension| lower.ends_with(extension))
}

fn category(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    if lower == "agents.md" {
        return "agent-instructions";
    }
    if lower.starts_with("readme") {
        return "readme";
    }
    if MANIFESTS.contains(&lower.as_str()) || lower.starts_with("requirements") && lower.ends_with(".txt") {
        return "manifest";
    }
    if CONFIGURATION_EXACT.contains(&lower.as_str())
        || CONFIGURATION_PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
    {
        return "configuration";
    }
    if is_test_path(path) {
        return "test";
    }
    if SOURCE_EXTENSIONS.iter().any(|extension| lower.ends_with(extension)) {
        return "source";
    }
    if DOCUMENT_EXTENSIONS.iter().any(|extension| lower.ends_with(extension)) {
        return "documentation";
    }
    "other"
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    lower.split('/').any(|part| part == "__tests__" || part == "test" || part == "tests")
        || name.contains(".test.")
        || name.contains(".spec.")
        || (name.starts_with("test_") && name.ends_with(".py"))
        || name.ends_with("_test.py")
}

fn git_state(root: &Path) -> Option<GitState> {
    let inside = Command::new("git")
        .args(["-C", &root.display().to_string(), "rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()?;
    if !inside.status.success() || !String::from_utf8_lossy(&inside.stdout).trim().starts_with("true") {
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

const SOURCE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".py"];
const DOCUMENT_EXTENSIONS: &[&str] = &[".md", ".markdown", ".txt"];

const PROTECTED_DIRECTORIES: &[&str] = &[
    ".git", ".hg", ".svn", ".venv", "venv", "env", "node_modules", "__pycache__",
    ".pytest_cache", ".mypy_cache", ".ruff_cache", ".turbo", ".next", ".nuxt", ".cache",
    "dist", "build", "out", "coverage", "vendor", "target",
];

const DEPENDENCY_DIRECTORIES: &[&str] = &[
    ".git", ".hg", ".svn", ".venv", "venv", "env", "node_modules", "__pycache__",
    ".pytest_cache", ".mypy_cache", ".ruff_cache", ".turbo", ".next", ".nuxt", ".cache",
    "dist", "build", "out", "coverage", "vendor", "target",
];

const GENERATED_DIRECTORIES: &[&str] = &[
    "dist", "build", "out", "coverage", ".next", ".turbo",
];

const BINARY_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico", ".bmp", ".tiff", ".mp3", ".mp4",
    ".mov", ".avi", ".wav", ".pdf", ".docx", ".doc", ".xlsx", ".zip", ".gz", ".tar",
    ".rar", ".7z", ".exe", ".dll", ".so", ".dylib", ".bin", ".woff", ".woff2", ".ttf",
    ".otf", ".sqlite", ".db",
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
        let paths: Vec<_> = collected.preview.files.iter().map(|file| file.path.as_str()).collect();
        assert_eq!(paths, vec!["README.md", "package.json", "src/main.test.ts", "src/main.ts"]);
        assert_eq!(collected.preview.files[2].category, "test");
        let skipped: Vec<_> = collected.preview.skipped.iter().map(|file| (file.path.as_str(), file.reason.as_str())).collect();
        assert!(skipped.contains(&(".env", "environment-file")));
        assert!(skipped.contains(&("node_modules/pkg/index.js", "dependency-directory")) || collected.preview.skipped.iter().all(|s| s.path != "node_modules/pkg/index.js"));
        assert!(skipped.contains(&("logo.png", "binary")));
    }

    #[test]
    fn fingerprint_changes_with_content_but_not_skip_order() {
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
        let ignore = RootIgnore::load(dir.path(), true).expect("ignore file loads");
        assert!(ignore.is_ignored("src/main.ts"));
        let collected = collect(&request(dir.path())).unwrap();
        assert!(collected.preview.files.iter().all(|file| file.path != "src/main.ts"));
    }
}
