//! SQLite persistence for Coderune, through SeaORM entities and migrations.
//!
//! The legacy database name and location are preserved so existing local data
//! remains discoverable. JSON payloads are the source of truth for hydration;
//! normalised columns exist for ordering and filtering.

mod context;
mod entities;
mod migration;

#[cfg(test)]
mod test_support;

pub use context::{ContextReadFilter, ProfileContextSnapshot};

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Path, PathBuf};

pub use entities::*;

use crate::context::write_profile_context;
use coderune_domain::{
    now_iso, CapturePreview, ContextCapture, DeveloperProfile, DocumentKind, ExportBundle,
    ExportContent, ImportedDocument, OnboardingDraft, OnboardingState, ProfileDraft, Project,
    ProjectDraft, ValidationError,
};
use migration::Migrator;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unreadable path: {0}")]
    UnreadablePath(String),
    #[error("personal context index is not ready: {0}")]
    ContextIndex(String),
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
    conn: DatabaseConnection,
    runtime: tokio::runtime::Runtime,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentFilter {
    All,
    Unlinked,
    Project(String),
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
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let conn = runtime.block_on(Database::connect(&url))?;
        runtime.block_on(async {
            conn.execute_unprepared("PRAGMA journal_mode = WAL;")
                .await?;
            conn.execute_unprepared("PRAGMA foreign_keys = ON;").await?;
            conn.execute_unprepared("PRAGMA busy_timeout = 5000;")
                .await?;
            Migrator::up(&conn, None).await?;
            Ok::<(), sea_orm::DbErr>(())
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600);
                let _ = std::fs::set_permissions(&path, permissions);
            }
        }
        let storage = Self {
            conn,
            runtime,
            path,
        };
        storage.refresh_profile_context_if_stale()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let conn = runtime.block_on(Database::connect("sqlite::memory:"))?;
        runtime.block_on(async {
            conn.execute_unprepared("PRAGMA foreign_keys = ON;").await?;
            conn.execute_unprepared("PRAGMA busy_timeout = 5000;")
                .await?;
            Migrator::up(&conn, None).await?;
            Ok::<(), sea_orm::DbErr>(())
        })?;
        let storage = Self {
            conn,
            runtime,
            path: PathBuf::from(":memory:"),
        };
        storage.refresh_profile_context_if_stale()?;
        Ok(storage)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn run<T>(
        &self,
        work: impl Future<Output = std::result::Result<T, StorageError>>,
    ) -> Result<T> {
        self.runtime.block_on(work)
    }

    pub fn load_onboarding(&self) -> Result<OnboardingState> {
        Ok(OnboardingState::from_parts(
            self.get_profile()?,
            self.get_onboarding_draft()?,
        ))
    }

    pub fn get_profile(&self) -> Result<Option<DeveloperProfile>> {
        let model = self.run(async {
            developer_profile::Entity::find_by_id("developer")
                .one(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        model
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .transpose()
    }

    pub fn get_profile_versions(&self) -> Result<Vec<DeveloperProfile>> {
        let models = self.run(async {
            developer_profile_version::Entity::find()
                .order_by_asc(developer_profile_version::Column::Version)
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        models
            .into_iter()
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .collect()
    }

    pub fn get_onboarding_draft(&self) -> Result<Option<OnboardingDraft>> {
        let model = self.run(async {
            onboarding_draft::Entity::find_by_id("onboarding")
                .one(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        model
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .transpose()
    }

    pub fn save_onboarding_draft(&mut self, draft: ProfileDraft) -> Result<OnboardingDraft> {
        let draft = OnboardingDraft {
            id: "onboarding".to_string(),
            draft: draft.normalize(),
            updated_at: now_iso(),
        };
        let data = serde_json::to_string(&draft)?;
        let updated_at = draft.updated_at.clone();
        self.run(async {
            onboarding_draft::Entity::insert(onboarding_draft::ActiveModel {
                id: Set("onboarding".to_string()),
                data: Set(data),
                updated_at: Set(updated_at),
            })
            .on_conflict(
                OnConflict::column(onboarding_draft::Column::Id)
                    .update_columns([
                        onboarding_draft::Column::Data,
                        onboarding_draft::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(StorageError::from)
        })?;
        Ok(draft)
    }

    /// Confirm onboarding: writes profile v1 (or the next version when editing)
    /// and clears the resumable draft in one transaction.
    pub fn confirm_profile(&mut self, draft: ProfileDraft) -> Result<DeveloperProfile> {
        let previous = self.get_profile()?;
        let profile = draft.into_profile(previous.as_ref())?;
        let data = serde_json::to_string(&profile)?;
        let profile_id = profile.id.clone();
        let version = profile.version;
        let created_at = profile.created_at.clone();
        let updated_at = profile.updated_at.clone();
        let projected_at = now_iso();
        self.run(async {
            let tx = self.conn.begin().await?;
            developer_profile::Entity::insert(developer_profile::ActiveModel {
                id: Set(profile_id.clone()),
                version: Set(version as i32),
                data: Set(data.clone()),
                created_at: Set(created_at.clone()),
                updated_at: Set(updated_at),
            })
            .on_conflict(
                OnConflict::column(developer_profile::Column::Id)
                    .update_columns([
                        developer_profile::Column::Version,
                        developer_profile::Column::Data,
                        developer_profile::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&tx)
            .await?;
            developer_profile_version::Entity::insert(developer_profile_version::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                profile_id: Set(profile_id),
                version: Set(version as i32),
                data: Set(data),
                created_at: Set(created_at),
            })
            .on_conflict(
                OnConflict::columns([
                    developer_profile_version::Column::ProfileId,
                    developer_profile_version::Column::Version,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(&tx)
            .await?;
            write_profile_context(&tx, &profile, &projected_at).await?;
            onboarding_draft::Entity::delete_by_id("onboarding")
                .exec(&tx)
                .await?;
            tx.commit().await?;
            Ok(())
        })?;
        Ok(profile)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let models = self.run(async {
            project::Entity::find()
                .order_by_asc(project::Column::CreatedAt)
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        models
            .into_iter()
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .collect()
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let model = self.run(async {
            project::Entity::find_by_id(id)
                .one(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        model
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .transpose()
    }

    pub fn get_project_versions(&self) -> Result<Vec<Project>> {
        let models = self.run(async {
            project_version::Entity::find()
                .order_by_asc(project_version::Column::ProjectId)
                .order_by_asc(project_version::Column::Version)
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        models
            .into_iter()
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .collect()
    }

    pub fn save_project(&mut self, draft: ProjectDraft) -> Result<Project> {
        validate_project_root(&draft.root_path)?;
        let project = draft.into_project(None)?;
        self.write_project(&project)?;
        Ok(project)
    }

    pub fn update_project(&mut self, id: &str, draft: ProjectDraft) -> Result<Project> {
        validate_project_root(&draft.root_path)?;
        let previous = self
            .get_project(id)?
            .ok_or_else(|| StorageError::NotFound(id.to_string()))?;
        let project = draft.into_project(Some(&previous))?;
        self.write_project(&project)?;
        Ok(project)
    }

    fn write_project(&mut self, project: &Project) -> Result<()> {
        let data = serde_json::to_string(project)?;
        let id = project.id.clone();
        let version = project.version;
        let created_at = project.created_at.clone();
        let updated_at = project.updated_at.clone();
        self.run(async {
            let tx = self.conn.begin().await?;
            project::Entity::insert(project::ActiveModel {
                id: Set(id.clone()),
                version: Set(version as i32),
                data: Set(data.clone()),
                created_at: Set(created_at.clone()),
                updated_at: Set(updated_at),
            })
            .on_conflict(
                OnConflict::column(project::Column::Id)
                    .update_columns([
                        project::Column::Version,
                        project::Column::Data,
                        project::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&tx)
            .await?;
            project_version::Entity::insert(project_version::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                project_id: Set(id),
                version: Set(version as i32),
                data: Set(data),
                created_at: Set(created_at),
            })
            .on_conflict(
                OnConflict::columns([
                    project_version::Column::ProjectId,
                    project_version::Column::Version,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(&tx)
            .await?;
            tx.commit().await?;
            Ok(())
        })
    }

    pub fn delete_project(&mut self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.run(async {
            let tx = self.conn.begin().await?;
            let capture_ids = capture::Entity::find()
                .filter(capture::Column::ProjectId.eq(&id))
                .all(&tx)
                .await?
                .into_iter()
                .map(|capture| capture.id)
                .collect::<Vec<_>>();
            if !capture_ids.is_empty() {
                capture_skipped::Entity::delete_many()
                    .filter(capture_skipped::Column::CaptureId.is_in(capture_ids.clone()))
                    .exec(&tx)
                    .await?;
                capture_file::Entity::delete_many()
                    .filter(capture_file::Column::CaptureId.is_in(capture_ids.clone()))
                    .exec(&tx)
                    .await?;
                capture::Entity::delete_many()
                    .filter(capture::Column::Id.is_in(capture_ids))
                    .exec(&tx)
                    .await?;
            }
            cleanup_orphan_contents(&tx).await?;
            project_version::Entity::delete_many()
                .filter(project_version::Column::ProjectId.eq(&id))
                .exec(&tx)
                .await?;
            document::Entity::delete_many()
                .filter(document::Column::ProjectId.eq(&id))
                .exec(&tx)
                .await?;
            project::Entity::delete_by_id(&id).exec(&tx).await?;
            tx.commit().await?;
            Ok(())
        })
    }

    pub fn list_documents(&self, filter: DocumentFilter) -> Result<Vec<ImportedDocument>> {
        let models = self.run(async {
            let mut query = document::Entity::find().order_by_asc(document::Column::ImportedAt);
            query = match filter {
                DocumentFilter::All => query,
                DocumentFilter::Unlinked => query.filter(document::Column::ProjectId.is_null()),
                DocumentFilter::Project(id) => query.filter(document::Column::ProjectId.eq(id)),
            };
            query.all(&self.conn).await.map_err(StorageError::from)
        })?;
        models
            .into_iter()
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .collect()
    }

    pub fn save_document(&mut self, document: &ImportedDocument) -> Result<()> {
        let data = serde_json::to_string(document)?;
        let kind = match document.kind {
            DocumentKind::Cv => "cv",
            DocumentKind::Srs => "srs",
        };
        let id = document.id.clone();
        let project_id = document.project_id.clone();
        let name = document.name.clone();
        let path = document.source_path.clone().unwrap_or_default();
        let hash = document.sha256.clone();
        let size = document.text_bytes as i64;
        let text = document.text.clone();
        let imported_at = document.created_at.clone();
        self.run(async {
            document::Entity::insert(document::ActiveModel {
                id: Set(id),
                project_id: Set(project_id),
                kind: Set(kind.to_string()),
                name: Set(name),
                path: Set(path),
                hash: Set(hash),
                size: Set(size),
                text: Set(text),
                data: Set(data),
                imported_at: Set(imported_at),
            })
            .on_conflict(
                OnConflict::column(document::Column::Id)
                    .update_columns([
                        document::Column::ProjectId,
                        document::Column::Kind,
                        document::Column::Name,
                        document::Column::Path,
                        document::Column::Hash,
                        document::Column::Size,
                        document::Column::Text,
                        document::Column::Data,
                        document::Column::ImportedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(StorageError::from)
        })?;
        Ok(())
    }

    pub fn list_captures(&self, project_id: &str) -> Result<Vec<ContextCapture>> {
        let models = self.run(async {
            capture::Entity::find()
                .filter(capture::Column::ProjectId.eq(project_id))
                .order_by_desc(capture::Column::Version)
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        models
            .into_iter()
            .map(|model| serde_json::from_str(&model.data).map_err(StorageError::from))
            .collect()
    }

    pub fn projects_with_captures(&self) -> Result<BTreeSet<String>> {
        self.run(async {
            let ids = capture::Entity::find()
                .select_only()
                .column(capture::Column::ProjectId)
                .distinct()
                .into_tuple::<String>()
                .all(&self.conn)
                .await?;
            Ok(ids.into_iter().collect())
        })
    }

    pub fn capture_contents(&self, capture_id: &str) -> Result<BTreeMap<String, String>> {
        let files = self.run(async {
            capture_file::Entity::find()
                .filter(capture_file::Column::CaptureId.eq(capture_id))
                .order_by_asc(capture_file::Column::Path)
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        let hashes = files
            .iter()
            .map(|file| file.content_hash.clone())
            .collect::<Vec<_>>();
        if hashes.is_empty() {
            return Ok(BTreeMap::new());
        }
        let contents = self.run(async {
            content::Entity::find()
                .filter(content::Column::Hash.is_in(hashes))
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
        let text_by_hash = contents
            .into_iter()
            .map(|content| (content.hash, content.text))
            .collect::<std::collections::BTreeMap<_, _>>();
        Ok(files
            .into_iter()
            .filter_map(|file| {
                text_by_hash
                    .get(&file.content_hash)
                    .cloned()
                    .map(|text| (file.path, text))
            })
            .collect())
    }

    /// Persist a reviewed capture. The caller has already checked the preview
    /// fingerprint. An identical latest capture is returned unchanged.
    pub fn save_capture(
        &mut self,
        preview: &CapturePreview,
        contents: &BTreeMap<String, String>,
    ) -> Result<ContextCapture> {
        let latest = self.list_captures(&preview.project_id)?.into_iter().next();
        if let Some(latest) = &latest {
            if latest.fingerprint == preview.fingerprint {
                return Ok(latest.clone());
            }
        }
        let version = latest.map(|capture| capture.version).unwrap_or(0) + 1;
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
        let id = capture.id.clone();
        let project_id = capture.project_id.clone();
        let version = capture.version;
        let fingerprint = capture.fingerprint.clone();
        let created_at = capture.created_at.clone();
        self.run(async {
            let tx = self.conn.begin().await?;
            capture::ActiveModel {
                id: Set(id.clone()),
                project_id: Set(project_id),
                version: Set(version as i32),
                fingerprint: Set(fingerprint),
                data: Set(data),
                created_at: Set(created_at),
            }
            .insert(&tx)
            .await?;
            for file in &capture.files {
                if let Some(text) = contents.get(&file.path) {
                    content::Entity::insert(content::ActiveModel {
                        hash: Set(file.sha256.clone()),
                        text: Set(text.clone()),
                        size: Set(file.bytes as i64),
                    })
                    .on_conflict(
                        OnConflict::column(content::Column::Hash)
                            .update_columns([content::Column::Text, content::Column::Size])
                            .to_owned(),
                    )
                    .exec(&tx)
                    .await?;
                }
                capture_file::ActiveModel {
                    capture_id: Set(id.clone()),
                    path: Set(file.path.clone()),
                    content_hash: Set(file.sha256.clone()),
                    size: Set(file.bytes as i64),
                    category: Set(file.category.clone()),
                }
                .insert(&tx)
                .await?;
            }
            for skipped in &capture.skipped {
                capture_skipped::ActiveModel {
                    capture_id: Set(id.clone()),
                    path: Set(skipped.path.clone()),
                    reason: Set(skipped.reason.clone()),
                }
                .insert(&tx)
                .await?;
            }
            tx.commit().await?;
            Ok(())
        })?;
        Ok(capture)
    }

    pub fn delete_capture(&mut self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.run(async {
            let tx = self.conn.begin().await?;
            delete_capture_rows(&tx, &id).await?;
            cleanup_orphan_contents(&tx).await?;
            tx.commit().await?;
            Ok(())
        })
    }

    pub fn delete_all_data(&mut self) -> Result<()> {
        self.run(async {
            let tx = self.conn.begin().await?;
            capture_skipped::Entity::delete_many().exec(&tx).await?;
            capture_file::Entity::delete_many().exec(&tx).await?;
            capture::Entity::delete_many().exec(&tx).await?;
            content::Entity::delete_many().exec(&tx).await?;
            document::Entity::delete_many().exec(&tx).await?;
            project_version::Entity::delete_many().exec(&tx).await?;
            project::Entity::delete_many().exec(&tx).await?;
            profile_context_index_state::Entity::delete_many()
                .exec(&tx)
                .await?;
            profile_context_entry::Entity::delete_many()
                .exec(&tx)
                .await?;
            developer_profile_version::Entity::delete_many()
                .exec(&tx)
                .await?;
            developer_profile::Entity::delete_many().exec(&tx).await?;
            onboarding_draft::Entity::delete_many().exec(&tx).await?;
            tx.commit().await?;
            Ok(())
        })
    }

    pub fn export_bundle(&self) -> Result<ExportBundle> {
        let projects = self.list_projects()?;
        let mut captures = Vec::new();
        for project in &projects {
            captures.extend(self.list_captures(&project.id)?);
        }
        let contents = self.run(async {
            content::Entity::find()
                .all(&self.conn)
                .await
                .map_err(StorageError::from)
        })?;
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
            contents: contents
                .into_iter()
                .map(|content| ExportContent {
                    hash: content.hash,
                    text: content.text,
                    size: content.size.max(0) as u64,
                })
                .collect(),
        })
    }

    /// Resolve the first project whose real path matches `directory`.
    pub fn find_project_by_path(&self, directory: &Path) -> Result<Option<Project>> {
        let canonical = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf());
        Ok(self.list_projects()?.into_iter().find(|project| {
            Path::new(&project.root_path)
                .canonicalize()
                .map(|value| value == canonical)
                .unwrap_or_else(|_| Path::new(&project.root_path) == canonical)
        }))
    }
}

fn validate_project_root(root: &str) -> Result<()> {
    let path = Path::new(root);
    if !path.is_dir() || std::fs::read_dir(path).is_err() {
        return Err(StorageError::UnreadablePath(root.to_string()));
    }
    Ok(())
}

async fn delete_capture_rows(
    tx: &sea_orm::DatabaseTransaction,
    capture_id: &str,
) -> std::result::Result<(), sea_orm::DbErr> {
    capture_skipped::Entity::delete_many()
        .filter(capture_skipped::Column::CaptureId.eq(capture_id))
        .exec(tx)
        .await?;
    capture_file::Entity::delete_many()
        .filter(capture_file::Column::CaptureId.eq(capture_id))
        .exec(tx)
        .await?;
    capture::Entity::delete_by_id(capture_id).exec(tx).await?;
    Ok(())
}

async fn cleanup_orphan_contents(
    tx: &sea_orm::DatabaseTransaction,
) -> std::result::Result<(), sea_orm::DbErr> {
    let used = capture_file::Entity::find()
        .all(tx)
        .await?
        .into_iter()
        .map(|file| file.content_hash)
        .collect::<BTreeSet<_>>();
    let all = content::Entity::find().all(tx).await?;
    let orphaned = all
        .into_iter()
        .filter(|content| !used.contains(&content.hash))
        .map(|content| content.hash)
        .collect::<Vec<_>>();
    if !orphaned.is_empty() {
        content::Entity::delete_many()
            .filter(content::Column::Hash.is_in(orphaned))
            .exec(tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::profile_draft;
    use coderune_domain::{
        CaptureLimits, CollectionScope, ContextSharingPolicy, PersonalContextPurpose, ProjectDraft,
        SelfReportedSkill, SkillRating,
    };

    fn coding_filter(paths: &[&str]) -> ContextReadFilter {
        let policy = ContextSharingPolicy::new(PersonalContextPurpose::Coding);
        ContextReadFilter {
            profile_id: "developer".to_string(),
            sensitivities: policy.allowed_sensitivities(),
            kinds: policy.allowed_kinds(),
            field_paths: paths.iter().map(|path| (*path).to_string()).collect(),
        }
    }

    #[test]
    fn opening_twice_keeps_data_and_records_seed_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-bridge.sqlite");
        {
            let mut storage = Storage::open(&path).unwrap();
            storage.confirm_profile(profile_draft()).unwrap();
        }
        let storage = Storage::open(&path).unwrap();
        let profile = storage.get_profile().unwrap().unwrap();
        assert_eq!(profile.display_name, "Ada");
        assert_eq!(profile.version, 1);
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
            .save_project(ProjectDraft {
                name: "Demo".to_string(),
                root_path: std::env::temp_dir().display().to_string(),
                goals: vec!["Ship".to_string()],
                scope: CollectionScope::default(),
                limits: CaptureLimits::default(),
                ..Default::default()
            })
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
        let export = storage.export_bundle().unwrap();
        assert_eq!(export.contents.len(), 1);
        assert_eq!(export.contents[0].text, "main");
        storage.delete_project(&project.id).unwrap();
        assert!(storage.list_projects().unwrap().is_empty());
        assert!(storage.list_captures(&project.id).unwrap().is_empty());
        assert!(storage.load_onboarding().unwrap().required_complete);
    }

    #[test]
    fn profile_workflow_runs_end_to_end_on_a_real_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.sqlite");

        // Onboarding starts incomplete and keeps a resumable draft.
        let mut storage = Storage::open(&path).unwrap();
        assert!(!storage.load_onboarding().unwrap().can_start_project);
        storage.save_onboarding_draft(profile_draft()).unwrap();
        assert!(storage.load_onboarding().unwrap().draft.is_some());

        // Confirming creates v1, clears the draft, and unlocks project work.
        let first = storage.confirm_profile(profile_draft()).unwrap();
        assert_eq!(first.version, 1);
        let onboarding = storage.load_onboarding().unwrap();
        assert!(onboarding.draft.is_none());
        assert!(onboarding.can_start_project);

        // Editing creates v2 without changing the original creation time.
        let second = storage
            .confirm_profile(ProfileDraft {
                skills_statement: "Rust and SQLite".to_string(),
                self_reported_skills: vec![SelfReportedSkill {
                    topic: "Rust".to_string(),
                    rating: Some(SkillRating::Intermediate),
                }],
                ..profile_draft()
            })
            .unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(second.created_at, first.created_at);
        assert_eq!(
            storage
                .get_profile_versions()
                .unwrap()
                .iter()
                .map(|profile| profile.version)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        // The context projection follows the current version.
        let filter = coding_filter(&["experienceStage", "skillsStatement"]);
        let snapshot = storage.read_profile_context(&filter).unwrap();
        assert_eq!(snapshot.profile.version, 2);
        assert_eq!(snapshot.entries.len(), 2);

        // The file keeps everything after a restart.
        drop(storage);
        let mut storage = Storage::open(&path).unwrap();
        assert_eq!(storage.get_profile().unwrap().unwrap().version, 2);
        assert_eq!(storage.get_profile_versions().unwrap().len(), 2);

        // Deleting all data returns the store to onboarding.
        storage.delete_all_data().unwrap();
        assert!(storage.get_profile().unwrap().is_none());
        assert!(storage.get_profile_versions().unwrap().is_empty());
        assert!(!storage.load_onboarding().unwrap().can_start_project);
        assert!(storage.read_profile_context(&filter).is_err());

        // A fresh confirmation starts at v1 again.
        assert_eq!(storage.confirm_profile(profile_draft()).unwrap().version, 1);
    }
}
