//! Application service layer.
//!
//! The UI talks to this crate only. It composes storage, collection, document
//! extraction, agent detection, and OS integration, and converts infrastructure
//! failures into the domain error taxonomy.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use coderune_agents::{self as agents, DraftSharingPreview};
use coderune_collectors::{self as collectors, CollectRequest, CollectedRepository};
use coderune_domain::{
    now_iso, AgentStatus, AppError, CaptureLimits, CollectionScope, ContextCapture,
    DeveloperProfile, DocumentKind, ErrorCode, ExportBundle, ImportedDocument, OnboardingState,
    ProfileDraft, Project, ProjectDraft,
};
use coderune_storage::{DocumentFilter, Storage, StorageError};

pub type AppResult<T> = std::result::Result<T, AppError>;

pub struct Workspace {
    storage: Storage,
    pub onboarding: OnboardingState,
    pub projects: Vec<Project>,
    pub documents: Vec<ImportedDocument>,
    pub agents: Vec<AgentStatus>,
    pub selected_project_id: Option<String>,
    pub launch_directory: Option<PathBuf>,
    pub notice: Option<String>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub collected: CollectedRepository,
    pub id: String,
}

impl Workspace {
    pub fn open_default() -> AppResult<Self> {
        Self::from_storage(Storage::open_default().map_err(map_storage)?)
    }

    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        Self::from_storage(Storage::open(path).map_err(map_storage)?)
    }

    pub fn in_memory() -> AppResult<Self> {
        Self::from_storage(Storage::in_memory().map_err(map_storage)?)
    }

    fn from_storage(storage: Storage) -> AppResult<Self> {
        let mut workspace = Self {
            storage,
            onboarding: OnboardingState::from_parts(None, None),
            projects: Vec::new(),
            documents: Vec::new(),
            agents: agents::detect_all(),
            selected_project_id: None,
            launch_directory: None,
            notice: None,
            error: None,
        };
        workspace.reload()?;
        Ok(workspace)
    }

    pub fn reload(&mut self) -> AppResult<()> {
        self.onboarding = self.storage.load_onboarding().map_err(map_storage)?;
        self.projects = self.storage.list_projects().map_err(map_storage)?;
        self.documents = self
            .storage
            .list_documents(DocumentFilter::All)
            .map_err(map_storage)?;
        self.agents = agents::detect_all();
        if self.selected_project_id.is_none() {
            self.selected_project_id = self.projects.first().map(|project| project.id.clone());
        }
        Ok(())
    }

    pub fn data_path(&self) -> PathBuf {
        self.storage.path().to_path_buf()
    }

    pub fn profile(&self) -> Option<&DeveloperProfile> {
        self.onboarding.profile.as_ref()
    }

    pub fn selected_project(&self) -> Option<&Project> {
        let id = self.selected_project_id.as_deref()?;
        self.projects.iter().find(|project| project.id == id)
    }

    pub fn select_project(&mut self, id: Option<String>) {
        self.selected_project_id = id;
    }

    pub fn save_onboarding_draft(&mut self, draft: ProfileDraft) -> AppResult<()> {
        self.storage
            .save_onboarding_draft(draft)
            .map_err(map_storage)?;
        self.onboarding = self.storage.load_onboarding().map_err(map_storage)?;
        Ok(())
    }

    pub fn confirm_profile(&mut self, draft: ProfileDraft) -> AppResult<DeveloperProfile> {
        let profile = self.storage.confirm_profile(draft).map_err(map_storage)?;
        self.onboarding = self.storage.load_onboarding().map_err(map_storage)?;
        Ok(profile)
    }

    pub fn update_profile(&mut self, draft: ProfileDraft) -> AppResult<DeveloperProfile> {
        require_profile(&self.onboarding)?;
        self.confirm_profile(draft)
    }

    pub fn create_project(&mut self, draft: ProjectDraft) -> AppResult<Project> {
        require_profile(&self.onboarding)?;
        let root = draft.root_path.clone();
        let is_dir = collectors::validate_root(&root);
        if !is_dir {
            return Err(AppError::new(
                ErrorCode::UnreadablePath,
                "Choose an existing readable project directory.",
            ));
        }
        let project = self
            .storage
            .save_project(draft, true)
            .map_err(map_storage)?;
        self.selected_project_id = Some(project.id.clone());
        self.reload()?;
        Ok(project)
    }

    pub fn update_project(&mut self, id: &str, draft: ProjectDraft) -> AppResult<Project> {
        require_profile(&self.onboarding)?;
        let root = draft.root_path.clone();
        let is_dir = collectors::validate_root(&root);
        if !is_dir {
            return Err(AppError::new(
                ErrorCode::UnreadablePath,
                "Choose an existing readable project directory.",
            ));
        }
        let project = self
            .storage
            .update_project(id, draft, true)
            .map_err(map_storage)?;
        self.reload()?;
        Ok(project)
    }

    pub fn delete_project(&mut self, id: &str) -> AppResult<()> {
        require_profile(&self.onboarding)?;
        self.storage.delete_project(id).map_err(map_storage)?;
        if self.selected_project_id.as_deref() == Some(id) {
            self.selected_project_id = None;
        }
        self.reload()?;
        Ok(())
    }

    pub fn list_captures(&self, project_id: &str) -> AppResult<Vec<ContextCapture>> {
        self.storage.list_captures(project_id).map_err(map_storage)
    }

    pub fn capture_contents(&self, capture_id: &str) -> AppResult<BTreeMap<String, String>> {
        self.storage
            .capture_contents(capture_id)
            .map_err(map_storage)
    }

    pub fn preview_capture(
        &self,
        project_id: &str,
        scope: CollectionScope,
        limits: CaptureLimits,
    ) -> AppResult<PreviewSession> {
        require_profile(&self.onboarding)?;
        let profile = self.profile().ok_or_else(|| {
            AppError::new(ErrorCode::OnboardingRequired, "Complete onboarding first.")
        })?;
        let project = self
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Select a project first."))?;
        let request = CollectRequest {
            root: PathBuf::from(&project.root_path),
            scope,
            limits,
            project_id: project.id.clone(),
            project_version: project.version,
            project_updated_at: project.updated_at.clone(),
            profile_version: profile.version,
            profile_updated_at: profile.updated_at.clone(),
        };
        let collected = collectors::collect(&request).map_err(|error| match error {
            collectors::CollectError::UnreadableRoot(path) => AppError::new(
                ErrorCode::UnreadablePath,
                format!("{path} is not readable."),
            ),
            other => AppError::new(ErrorCode::UnreadablePath, other.to_string()),
        })?;
        Ok(PreviewSession {
            collected,
            id: coderune_domain::new_id(),
        })
    }

    /// Re-collect before saving so a changed source cannot be saved from a stale
    /// preview. The caller must pass the explicit confirmation flag.
    pub fn save_capture(
        &mut self,
        session: &PreviewSession,
        confirmed: bool,
    ) -> AppResult<ContextCapture> {
        if !confirmed {
            return Err(AppError::new(
                ErrorCode::ConfirmationRequired,
                "Review the exact file list before saving this capture.",
            ));
        }
        let preview = &session.collected.preview;
        let project = self
            .projects
            .iter()
            .find(|project| project.id == preview.project_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "The project no longer exists."))?;
        let profile = self.profile().ok_or_else(|| {
            AppError::new(ErrorCode::OnboardingRequired, "Complete onboarding first.")
        })?;
        let fresh = collectors::collect(&CollectRequest {
            root: PathBuf::from(&project.root_path),
            scope: project.scope.clone(),
            limits: project.limits.clone(),
            project_id: project.id.clone(),
            project_version: project.version,
            project_updated_at: project.updated_at.clone(),
            profile_version: profile.version,
            profile_updated_at: profile.updated_at.clone(),
        })
        .map_err(|error| AppError::new(ErrorCode::CaptureStale, error.to_string()))?;
        if fresh.preview.fingerprint != preview.fingerprint {
            return Err(AppError::new(
                ErrorCode::CaptureStale,
                "The repository changed after review. Preview it again before saving.",
            ));
        }
        let capture = self
            .storage
            .save_capture(preview, &session.collected.contents)
            .map_err(map_storage)?;
        self.notice = Some(format!("Saved context capture v{}.", capture.version));
        Ok(capture)
    }

    pub fn refresh_capture(
        &self,
        project_id: &str,
    ) -> AppResult<(Option<ContextCapture>, PreviewSession)> {
        let project = self
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Select a project first."))?;
        let session =
            self.preview_capture(project_id, project.scope.clone(), project.limits.clone())?;
        let latest = self.list_captures(project_id)?.into_iter().next();
        if latest
            .as_ref()
            .is_some_and(|capture| capture.fingerprint == session.collected.preview.fingerprint)
        {
            return Ok((latest, session));
        }
        Ok((None, session))
    }

    pub fn delete_capture(&mut self, id: &str) -> AppResult<()> {
        require_profile(&self.onboarding)?;
        self.storage.delete_capture(id).map_err(map_storage)?;
        Ok(())
    }

    pub fn list_documents(&self, filter: DocumentFilter) -> AppResult<Vec<ImportedDocument>> {
        self.storage.list_documents(filter).map_err(map_storage)
    }

    /// Read a selected CV/SRS file. PDF and DOCX extraction is not yet ported;
    /// the UI falls back to manual entry for those formats.
    pub fn extract_document(
        &self,
        path: impl AsRef<Path>,
        max_bytes: u64,
    ) -> AppResult<(String, String, usize)> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !matches!(extension.as_str(), "md" | "markdown" | "txt") {
            return Err(AppError::new(
                ErrorCode::UnsupportedDocument,
                "This build extracts Markdown and plain text. Use manual entry for PDF or DOCX.",
            ));
        }
        let metadata = std::fs::metadata(path).map_err(|_| {
            AppError::new(
                ErrorCode::UnreadableDocument,
                "The document could not be read.",
            )
        })?;
        if metadata.len() > max_bytes {
            return Err(AppError::new(
                ErrorCode::DocumentTooLarge,
                format!("The document exceeds the {max_bytes} byte limit."),
            ));
        }
        let text = std::fs::read_to_string(path).map_err(|_| {
            AppError::new(
                ErrorCode::UnreadableDocument,
                "The document is not valid UTF-8 text.",
            )
        })?;
        if text.trim().is_empty() {
            return Err(AppError::new(
                ErrorCode::ImageOnlyDocument,
                "The document contains no extractable text.",
            ));
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("document")
            .to_string();
        let bytes = text.len();
        Ok((name, text, bytes))
    }

    pub fn save_document(
        &mut self,
        kind: DocumentKind,
        project_id: Option<String>,
        name: String,
        source_path: Option<String>,
        text: String,
        confirmed: bool,
    ) -> AppResult<ImportedDocument> {
        require_profile(&self.onboarding)?;
        if !confirmed {
            return Err(AppError::new(
                ErrorCode::ConfirmationRequired,
                "Review the extracted text before saving the import.",
            ));
        }
        if text.trim().is_empty() {
            return Err(AppError::new(
                ErrorCode::UnreadableDocument,
                "The text is empty.",
            ));
        }
        if kind == DocumentKind::Srs && project_id.is_none() {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Choose the project this SRS belongs to.",
            ));
        }
        let document = ImportedDocument {
            id: coderune_domain::new_id(),
            kind,
            project_id,
            name,
            source_path,
            sha256: collectors::sha256_hex(text.as_bytes()),
            text_bytes: text.len(),
            text,
            created_at: now_iso(),
        };
        self.storage.save_document(&document).map_err(map_storage)?;
        self.documents = self
            .storage
            .list_documents(DocumentFilter::All)
            .map_err(map_storage)?;
        Ok(document)
    }

    pub fn prepare_draft(
        &self,
        agent: coderune_domain::AgentId,
        document_id: &str,
    ) -> AppResult<DraftSharingPreview> {
        let document = self
            .documents
            .iter()
            .find(|document| document.id == document_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Select an imported document."))?;
        let project_name = document
            .project_id
            .as_deref()
            .and_then(|id| self.projects.iter().find(|project| project.id == id))
            .map(|project| project.name.as_str());
        Ok(agents::prepare_preview(
            agent,
            document.kind,
            document,
            project_name,
        ))
    }

    pub fn export_bundle(&self) -> AppResult<ExportBundle> {
        self.storage.export_bundle().map_err(map_storage)
    }

    pub fn export_json(&self) -> AppResult<String> {
        serde_json::to_string_pretty(&self.export_bundle()?).map_err(|error| {
            AppError::new(
                ErrorCode::Validation,
                format!("Could not serialize export: {error}"),
            )
        })
    }

    pub fn delete_all_data(&mut self) -> AppResult<()> {
        self.storage.delete_all_data().map_err(map_storage)?;
        self.selected_project_id = None;
        self.reload()?;
        Ok(())
    }

    pub fn adopt_launch_directory(&mut self, directory: PathBuf) -> AppResult<Option<Project>> {
        self.launch_directory = Some(directory.clone());
        if let Some(project) = self
            .storage
            .find_project_by_path(&directory)
            .map_err(map_storage)?
        {
            self.selected_project_id = Some(project.id.clone());
            return Ok(Some(project));
        }
        Ok(None)
    }

    /// Open a directory in the platform file manager. Returns an error instead
    /// of panicking when no opener is available.
    pub fn open_directory(path: &Path) -> AppResult<()> {
        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = std::process::Command::new("open");
            command.arg(path);
            command
        };
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = std::process::Command::new("explorer");
            command.arg(path);
            command
        };
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = {
            let mut command = std::process::Command::new("xdg-open");
            command.arg(path);
            command
        };
        command.spawn().map(|_| ()).map_err(|error| {
            AppError::new(
                ErrorCode::UnreadablePath,
                format!("Could not open the folder: {error}"),
            )
        })
    }

    /// Launch an installed editor against a project root. No shell is used.
    pub fn launch_editor(editor: &str, path: &Path) -> AppResult<()> {
        let allowed = ["code", "cursor", "antigravity", "zed"];
        if !allowed.contains(&editor) {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Choose a supported editor.",
            ));
        }
        std::process::Command::new(editor)
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|error| {
                AppError::new(
                    ErrorCode::UnreadablePath,
                    format!("Could not launch {editor}: {error}"),
                )
            })
    }
}

fn require_profile(onboarding: &OnboardingState) -> AppResult<()> {
    if onboarding.can_start_project {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::OnboardingRequired,
            "Complete developer onboarding before project work.",
        ))
    }
}

fn map_storage(error: StorageError) -> AppError {
    match error {
        StorageError::Validation(validation) => AppError::validation(validation.details),
        StorageError::NotFound(message) => AppError::new(ErrorCode::NotFound, message),
        other => AppError::new(ErrorCode::Validation, other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coderune_domain::{CaptureLimits, CollectionScope, ExperienceStage, ProjectDraft};

    fn workspace() -> Workspace {
        Workspace::in_memory().unwrap()
    }

    fn profile() -> ProfileDraft {
        ProfileDraft {
            display_name: "Ada".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust".to_string(),
            learning_goals: vec!["Ship".to_string()],
            ..Default::default()
        }
    }

    fn project_draft(root: &Path) -> ProjectDraft {
        ProjectDraft {
            name: "Demo".to_string(),
            root_path: root.display().to_string(),
            goals: vec!["Ship".to_string()],
            scope: CollectionScope::default(),
            limits: CaptureLimits::default(),
            ..Default::default()
        }
    }

    #[test]
    fn project_work_requires_onboarding() {
        let mut workspace = workspace();
        let root = tempfile::tempdir().unwrap();
        let error = workspace
            .create_project(project_draft(root.path()))
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::OnboardingRequired);
        workspace.confirm_profile(profile()).unwrap();
        assert!(workspace.create_project(project_draft(root.path())).is_ok());
    }

    #[test]
    fn preview_then_save_requires_confirmation_and_detects_changes() {
        let mut workspace = workspace();
        workspace.confirm_profile(profile()).unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("README.md"), "# Demo").unwrap();
        std::fs::write(root.path().join("main.ts"), "export {}").unwrap();
        let project = workspace
            .create_project(project_draft(root.path()))
            .unwrap();
        let session = workspace
            .preview_capture(&project.id, project.scope.clone(), project.limits.clone())
            .unwrap();
        assert_eq!(session.collected.preview.files.len(), 2);
        let error = workspace.save_capture(&session, false).unwrap_err();
        assert_eq!(error.code, ErrorCode::ConfirmationRequired);
        let capture = workspace.save_capture(&session, true).unwrap();
        assert_eq!(capture.version, 1);
        std::fs::write(root.path().join("main.ts"), "export const x = 1").unwrap();
        let error = workspace.save_capture(&session, true).unwrap_err();
        assert_eq!(error.code, ErrorCode::CaptureStale);
    }

    #[test]
    fn launch_directory_selects_an_existing_project_by_real_path() {
        let mut workspace = workspace();
        workspace.confirm_profile(profile()).unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("README.md"), "# Demo").unwrap();
        let project = workspace
            .create_project(project_draft(root.path()))
            .unwrap();
        let found = workspace
            .adopt_launch_directory(root.path().to_path_buf())
            .unwrap()
            .expect("existing project should be selected");
        assert_eq!(found.id, project.id);
        assert_eq!(
            workspace.selected_project_id.as_deref(),
            Some(project.id.as_str())
        );
        let other = tempfile::tempdir().unwrap();
        assert!(workspace
            .adopt_launch_directory(other.path().to_path_buf())
            .unwrap()
            .is_none());
    }

    #[test]
    fn project_scope_and_limits_round_trip() {
        let mut workspace = workspace();
        workspace.confirm_profile(profile()).unwrap();
        let root = tempfile::tempdir().unwrap();
        let mut draft = project_draft(root.path());
        draft.scope = CollectionScope {
            include_paths: vec!["src".to_string(), "docs".to_string()],
            exclude_paths: vec!["vendor".to_string()],
            respect_gitignore: false,
        };
        draft.limits = CaptureLimits {
            per_file_bytes: 2048,
            per_capture_bytes: 4096,
        };
        let project = workspace.create_project(draft).unwrap();
        let reloaded = workspace
            .projects
            .iter()
            .find(|item| item.id == project.id)
            .unwrap();
        assert_eq!(reloaded.scope.include_paths, vec!["src", "docs"]);
        assert_eq!(reloaded.scope.exclude_paths, vec!["vendor"]);
        assert!(!reloaded.scope.respect_gitignore);
        assert_eq!(reloaded.limits.per_file_bytes, 2048);
        assert_eq!(reloaded.limits.per_capture_bytes, 4096);
    }

    #[test]
    fn document_import_is_reviewed_and_can_prepare_an_agent_preview() {
        let mut workspace = workspace();
        workspace.confirm_profile(profile()).unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("cv.md");
        std::fs::write(&path, "# Ada\n\nRust and TypeScript.").unwrap();
        let (name, text, bytes) = workspace.extract_document(&path, 1024).unwrap();
        assert_eq!(name, "cv.md");
        assert_eq!(bytes, text.len());
        let document = workspace
            .save_document(
                DocumentKind::Cv,
                None,
                name,
                Some(path.display().to_string()),
                text,
                true,
            )
            .unwrap();
        let preview = workspace
            .prepare_draft(coderune_domain::AgentId::ClaudeCode, &document.id)
            .unwrap();
        assert!(preview.fits);
        assert!(preview.text.contains("Rust and TypeScript."));
        assert!(!preview.text.is_empty());
    }

    #[test]
    fn export_and_delete_all_return_to_onboarding() {
        let mut workspace = workspace();
        workspace.confirm_profile(profile()).unwrap();
        let root = tempfile::tempdir().unwrap();
        workspace
            .create_project(project_draft(root.path()))
            .unwrap();
        let json = workspace.export_json().unwrap();
        assert!(json.contains("\"application\": \"Coderune\""));
        workspace.delete_all_data().unwrap();
        assert!(!workspace.onboarding.can_start_project);
        assert!(workspace.projects.is_empty());
    }
}
