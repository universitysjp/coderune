//! SQLite persistence for Coderune.
//!
//! The legacy database name and location are preserved so existing local data
//! remains discoverable after the native rewrite. JSON payloads are the source
//! of truth for hydration; normalised columns exist for ordering and queries.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use coderune_domain::{
    now_iso, CapturePreview, ContextCapture, DeveloperProfile, DocumentKind, ExportBundle,
    ImportedDocument, OnboardingDraft, OnboardingState, ProfileDraft, Project, ProjectDraft,
    ValidationError,
};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),
    #[error("unsupported database version {0}; expected 4")]
    UnsupportedVersion(i64),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Resolve the data directory. `CODERUNE_DATA_DIR` is the explicit test and
/// sandbox override; otherwise the legacy `Context Bridge` directory is used.
pub fn default_data_dir() -> PathBuf {
    if let Ok(value) = std::env::var("CODERUNE_DATA_DIR") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    if let Some(dirs) = directories::ProjectDirs::from("", "", "Context Bridge") {
        return dirs.data_dir().to_path_buf();
    }
    std::env::temp_dir().join("context-bridge")
}

pub fn default_database_path() -> PathBuf {
    default_data_dir().join("context-bridge.sqlite")
}

pub struct Storage {
    conn: Connection,
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFilter {
    All,
    Unlinked,
    Project(&'static str),
}

impl Storage {
    pub fn open_default() -> Result<Self> {
        Self::open(default_database_path())
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        Self::from_connection(conn, path)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn, PathBuf::from(":memory:"))
    }

    fn from_connection(conn: Connection, path: PathBuf) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        let mut storage = Self { conn, path };
        storage.migrate()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&storage.path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600);
                let _ = std::fs::set_permissions(&storage.path, permissions);
            }
        }
        Ok(storage)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )?;
        let applied: std::collections::BTreeSet<i64> = {
            let mut statement = self.conn.prepare("SELECT version FROM schema_migrations")?;
            let rows = statement.query_map([], |row| row.get(0))?;
            rows.collect::<std::result::Result<_, _>>()?
        };
        for (version, sql) in MIGRATIONS {
            if applied.contains(&version) {
                continue;
            }
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![version, now_iso()],
            )?;
            tx.commit()?;
        }
        let current: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| row.get(0))?;
        if current != 4 {
            return Err(StorageError::UnsupportedVersion(current));
        }
        Ok(())
    }

    pub fn load_onboarding(&self) -> Result<OnboardingState> {
        Ok(OnboardingState::from_parts(self.get_profile()?, self.get_onboarding_draft()?))
    }

    pub fn get_profile(&self) -> Result<Option<DeveloperProfile>> {
        self.conn
            .query_row("SELECT data FROM developer_profiles WHERE id = 'developer'", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(|data| serde_json::from_str(&data).map_err(StorageError::from))
            .transpose()
    }

    pub fn get_profile_versions(&self) -> Result<Vec<DeveloperProfile>> {
        let mut statement = self
            .conn
            .prepare("SELECT data FROM developer_profile_versions ORDER BY version ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn get_onboarding_draft(&self) -> Result<Option<OnboardingDraft>> {
        self.conn
            .query_row("SELECT data FROM onboarding_draft WHERE id = 'onboarding'", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(|data| serde_json::from_str(&data).map_err(StorageError::from))
            .transpose()
    }

    pub fn save_onboarding_draft(&mut self, draft: ProfileDraft) -> Result<OnboardingDraft> {
        let draft = OnboardingDraft {
            id: "onboarding".to_string(),
            draft: draft.normalize(),
            updated_at: now_iso(),
        };
        let data = serde_json::to_string(&draft)?;
        self.conn.execute(
            "INSERT INTO onboarding_draft(id, data, updated_at) VALUES ('onboarding', ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            params![data, draft.updated_at],
        )?;
        Ok(draft)
    }

    /// Confirm onboarding: writes profile v1 (or the next version when editing)
    /// and clears the resumable draft in one transaction.
    pub fn confirm_profile(&mut self, draft: ProfileDraft) -> Result<DeveloperProfile> {
        let previous = self.get_profile()?;
        let profile = draft.into_profile(previous.as_ref())?;
        let data = serde_json::to_string(&profile)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO developer_profiles(id, version, data, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                version = excluded.version, data = excluded.data, updated_at = excluded.updated_at",
            params![profile.id, profile.version, data, profile.created_at, profile.updated_at],
        )?;
        tx.execute(
            "INSERT INTO developer_profile_versions(profile_id, version, data, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![profile.id, profile.version, data, profile.created_at],
        )?;
        tx.execute("DELETE FROM onboarding_draft WHERE id = 'onboarding'", [])?;
        tx.commit()?;
        Ok(profile)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut statement = self
            .conn
            .prepare("SELECT data FROM projects ORDER BY created_at ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        self.conn
            .query_row("SELECT data FROM projects WHERE id = ?1", params![id], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(|data| serde_json::from_str(&data).map_err(StorageError::from))
            .transpose()
    }

    pub fn get_project_versions(&self) -> Result<Vec<Project>> {
        let mut statement = self
            .conn
            .prepare("SELECT data FROM project_versions ORDER BY project_id ASC, version ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn save_project(
        &mut self,
        draft: ProjectDraft,
        root_is_directory: bool,
    ) -> Result<Project> {
        let project = draft.into_project(None, |_| root_is_directory)?;
        self.write_project(&project)?;
        Ok(project)
    }

    pub fn update_project(
        &mut self,
        id: &str,
        draft: ProjectDraft,
        root_is_directory: bool,
    ) -> Result<Project> {
        let previous = self
            .get_project(id)?
            .ok_or_else(|| StorageError::NotFound(id.to_string()))?;
        let project = draft.into_project(Some(&previous), |_| root_is_directory)?;
        self.write_project(&project)?;
        Ok(project)
    }

    fn write_project(&mut self, project: &Project) -> Result<()> {
        let data = serde_json::to_string(project)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO projects(id, version, data, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                version = excluded.version, data = excluded.data, updated_at = excluded.updated_at",
            params![project.id, project.version, data, project.created_at, project.updated_at],
        )?;
        tx.execute(
            "INSERT INTO project_versions(project_id, version, data, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![project.id, project.version, data, project.created_at],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_project(&mut self, id: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        let capture_ids: Vec<String> = {
            let mut statement = tx.prepare("SELECT id FROM captures WHERE project_id = ?1")?;
            let rows = statement.query_map(params![id], |row| row.get(0))?;
            rows.collect::<std::result::Result<_, _>>()?
        };
        for capture_id in capture_ids {
            delete_capture_rows(&tx, &capture_id)?;
        }
        tx.execute("DELETE FROM project_versions WHERE project_id = ?1", params![id])?;
        tx.execute("DELETE FROM documents WHERE project_id = ?1", params![id])?;
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        cleanup_orphan_contents(&tx)?;
        tx.commit()?;
        Ok(())
    }

    pub fn list_documents(&self, filter: DocumentFilter) -> Result<Vec<ImportedDocument>> {
        let (sql, value): (&str, Option<String>) = match filter {
            DocumentFilter::All => ("SELECT data FROM documents ORDER BY imported_at ASC", None),
            DocumentFilter::Unlinked => (
                "SELECT data FROM documents WHERE project_id IS NULL ORDER BY imported_at ASC",
                None,
            ),
            DocumentFilter::Project(id) => (
                "SELECT data FROM documents WHERE project_id = ?1 ORDER BY imported_at ASC",
                Some(id.to_string()),
            ),
        };
        let mut statement = self.conn.prepare(sql)?;
        let rows = match value {
            Some(id) => statement.query_map(params![id], |row| row.get::<_, String>(0))?.collect::<std::result::Result<Vec<_>, _>>()?,
            None => statement.query_map([], |row| row.get::<_, String>(0))?.collect::<std::result::Result<Vec<_>, _>>()?,
        };
        rows.into_iter()
            .map(|data| serde_json::from_str(&data).map_err(StorageError::from))
            .collect()
    }

    pub fn save_document(&mut self, document: &ImportedDocument) -> Result<()> {
        let data = serde_json::to_string(document)?;
        self.conn.execute(
            "INSERT INTO documents(id, project_id, kind, name, path, hash, size, text, data, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                document.id,
                document.project_id,
                match document.kind {
                    DocumentKind::Cv => "cv",
                    DocumentKind::Srs => "srs",
                },
                document.name,
                document.source_path.clone().unwrap_or_default(),
                document.sha256,
                document.text_bytes as i64,
                document.text,
                data,
                document.created_at
            ],
        )?;
        Ok(())
    }

    pub fn list_captures(&self, project_id: &str) -> Result<Vec<ContextCapture>> {
        let mut statement = self.conn.prepare(
            "SELECT data FROM captures WHERE project_id = ?1 ORDER BY version DESC",
        )?;
        let rows = statement.query_map(params![project_id], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn get_capture(&self, id: &str) -> Result<Option<ContextCapture>> {
        self.conn
            .query_row("SELECT data FROM captures WHERE id = ?1", params![id], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .map(|data| serde_json::from_str(&data).map_err(StorageError::from))
            .transpose()
    }

    pub fn capture_contents(&self, capture_id: &str) -> Result<BTreeMap<String, String>> {
        let mut statement = self.conn.prepare(
            "SELECT f.path, c.text FROM capture_files f
             JOIN contents c ON c.hash = f.content_hash
             WHERE f.capture_id = ?1 ORDER BY f.path ASC",
        )?;
        let rows = statement.query_map(params![capture_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut contents = BTreeMap::new();
        for row in rows {
            let (path, text) = row?;
            contents.insert(path, text);
        }
        Ok(contents)
    }

    /// Persist a reviewed capture. The caller has already checked the preview
    /// fingerprint. An identical latest capture is returned unchanged.
    pub fn save_capture(
        &mut self,
        preview: &CapturePreview,
        contents: &BTreeMap<String, String>,
    ) -> Result<ContextCapture> {
        if let Some(latest) = self.list_captures(&preview.project_id)?.into_iter().next() {
            if latest.fingerprint == preview.fingerprint {
                return Ok(latest);
            }
        }
        let version = self
            .list_captures(&preview.project_id)?
            .into_iter()
            .map(|capture| capture.version)
            .max()
            .unwrap_or(0)
            + 1;
        let capture = ContextCapture {
            id: coderune_domain::new_id(),
            project_id: preview.project_id.clone(),
            version,
            fingerprint: preview.fingerprint.clone(),
            profile_version: preview.profile_version,
            project_version: preview.project_version,
            files: preview.files.clone(),
            skipped: preview.skipped.clone(),
            total_bytes: preview.total_bytes,
            git: preview.git.clone(),
            created_at: now_iso(),
        };
        let data = serde_json::to_string(&capture)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO captures(id, project_id, version, fingerprint, data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![capture.id, capture.project_id, capture.version, capture.fingerprint, data, capture.created_at],
        )?;
        for file in &capture.files {
            if let Some(text) = contents.get(&file.path) {
                tx.execute(
                    "INSERT INTO contents(hash, text, size) VALUES (?1, ?2, ?3)
                     ON CONFLICT(hash) DO NOTHING",
                    params![file.sha256, text, file.bytes as i64],
                )?;
            }
            tx.execute(
                "INSERT INTO capture_files(capture_id, path, content_hash, size, category)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![capture.id, file.path, file.sha256, file.bytes as i64, file.category],
            )?;
        }
        for skipped in &capture.skipped {
            tx.execute(
                "INSERT INTO capture_skipped(capture_id, path, reason) VALUES (?1, ?2, ?3)",
                params![capture.id, skipped.path, skipped.reason],
            )?;
        }
        tx.commit()?;
        Ok(capture)
    }

    pub fn delete_capture(&mut self, id: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        delete_capture_rows(&tx, id)?;
        cleanup_orphan_contents(&tx)?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_all_data(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        for table in [
            "capture_skipped",
            "capture_files",
            "captures",
            "contents",
            "documents",
            "project_versions",
            "projects",
            "developer_profile_versions",
            "developer_profiles",
            "onboarding_draft",
        ] {
            tx.execute(&format!("DELETE FROM {table}"), [])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn export_bundle(&self) -> Result<ExportBundle> {
        let projects = self.list_projects()?;
        let mut captures = Vec::new();
        for project in &projects {
            captures.extend(self.list_captures(&project.id)?);
        }
        Ok(ExportBundle {
            exported_at: now_iso(),
            application: "Coderune".to_string(),
            schema_version: 1,
            profile: self.get_profile()?,
            profile_versions: self.get_profile_versions()?,
            projects,
            project_versions: self.get_project_versions()?,
            documents: self.list_documents(DocumentFilter::All)?,
            captures,
        })
    }

    /// Resolve the first project whose real path matches `directory`.
    pub fn find_project_by_path(&self, directory: &Path) -> Result<Option<Project>> {
        let canonical = directory.canonicalize().unwrap_or_else(|_| directory.to_path_buf());
        Ok(self.list_projects()?.into_iter().find(|project| {
            Path::new(&project.root_path)
                .canonicalize()
                .map(|value| value == canonical)
                .unwrap_or_else(|_| Path::new(&project.root_path) == canonical)
        }))
    }
}

fn delete_capture_rows(tx: &rusqlite::Transaction<'_>, capture_id: &str) -> Result<()> {
    tx.execute("DELETE FROM capture_skipped WHERE capture_id = ?1", params![capture_id])?;
    tx.execute("DELETE FROM capture_files WHERE capture_id = ?1", params![capture_id])?;
    tx.execute("DELETE FROM captures WHERE id = ?1", params![capture_id])?;
    Ok(())
}

fn cleanup_orphan_contents(tx: &rusqlite::Transaction<'_>) -> Result<()> {
    tx.execute(
        "DELETE FROM contents WHERE hash NOT IN (SELECT DISTINCT content_hash FROM capture_files)",
        [],
    )?;
    Ok(())
}

const MIGRATIONS: &[(i64, &str)] = &[
    (
        1,
        "CREATE TABLE IF NOT EXISTS developer_profiles (
            id TEXT PRIMARY KEY,
            version INTEGER NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS developer_profile_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(profile_id, version)
        );
        CREATE TABLE IF NOT EXISTS onboarding_draft (
            id TEXT PRIMARY KEY CHECK (id = 'onboarding'),
            data TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
    (
        2,
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            version INTEGER NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS project_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(project_id, version)
        );",
    ),
    (
        3,
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            project_id TEXT,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            hash TEXT NOT NULL,
            size INTEGER NOT NULL,
            text TEXT NOT NULL,
            data TEXT NOT NULL,
            imported_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS documents_project_id ON documents(project_id);
        CREATE INDEX IF NOT EXISTS documents_hash ON documents(hash);",
    ),
    (
        4,
        "CREATE TABLE IF NOT EXISTS contents (
            hash TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            size INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS captures (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            fingerprint TEXT NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(project_id, version)
        );
        CREATE INDEX IF NOT EXISTS captures_project_id ON captures(project_id);
        CREATE TABLE IF NOT EXISTS capture_files (
            capture_id TEXT NOT NULL,
            path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            size INTEGER NOT NULL,
            category TEXT NOT NULL,
            PRIMARY KEY (capture_id, path)
        );
        CREATE TABLE IF NOT EXISTS capture_skipped (
            capture_id TEXT NOT NULL,
            path TEXT NOT NULL,
            reason TEXT NOT NULL,
            PRIMARY KEY (capture_id, path)
        );",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use coderune_domain::{CaptureLimits, CollectionScope, ExperienceStage, ProjectDraft};

    fn profile_draft() -> ProfileDraft {
        ProfileDraft {
            display_name: "Ada".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust".to_string(),
            learning_goals: vec!["Ship".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn onboarding_is_gated_and_versions_profiles() {
        let mut storage = Storage::in_memory().unwrap();
        assert!(!storage.load_onboarding().unwrap().can_start_project);
        storage.save_onboarding_draft(profile_draft()).unwrap();
        let state = storage.load_onboarding().unwrap();
        assert!(state.draft.is_some());
        assert!(!state.can_start_project);
        let first = storage.confirm_profile(profile_draft()).unwrap();
        assert_eq!(first.version, 1);
        assert!(storage.load_onboarding().unwrap().draft.is_none());
        let second = storage.confirm_profile(profile_draft()).unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(storage.get_profile_versions().unwrap().len(), 2);
    }

    #[test]
    fn captures_are_versioned_deduplicated_and_deleted() {
        let mut storage = Storage::in_memory().unwrap();
        let profile = storage.confirm_profile(profile_draft()).unwrap();
        let project = storage
            .save_project(
                ProjectDraft {
                    name: "Demo".to_string(),
                    root_path: std::env::temp_dir().display().to_string(),
                    goals: vec!["Ship".to_string()],
                    scope: CollectionScope::default(),
                    limits: CaptureLimits::default(),
                    ..Default::default()
                },
                true,
            )
            .unwrap();
        let preview = CapturePreview {
            project_id: project.id.clone(),
            project_version: project.version,
            profile_version: profile.version,
            fingerprint: "abc".to_string(),
            files: vec![coderune_domain::CapturedFile {
                path: "src/main.rs".to_string(),
                sha256: "hash".to_string(),
                bytes: 4,
                category: "source".to_string(),
            }],
            skipped: Vec::new(),
            total_bytes: 4,
            git: None,
            created_at: now_iso(),
        };
        let mut contents = BTreeMap::new();
        contents.insert("src/main.rs".to_string(), "main".to_string());
        let first = storage.save_capture(&preview, &contents).unwrap();
        assert_eq!(first.version, 1);
        let duplicate = storage.save_capture(&preview, &contents).unwrap();
        assert_eq!(duplicate.id, first.id);
        assert_eq!(storage.capture_contents(&first.id).unwrap().len(), 1);
        storage.delete_project(&project.id).unwrap();
        assert!(storage.list_projects().unwrap().is_empty());
        assert!(storage.list_captures(&project.id).unwrap().is_empty());
        assert!(storage.load_onboarding().unwrap().required_complete);
    }
}
