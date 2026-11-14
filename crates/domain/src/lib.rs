//! Coderune domain model.
//!
//! Glossary terms are the source of truth: developer profile, onboarding,
//! developer baseline, project context, repository context, context capture.
//! Persistence and transport shapes are deliberately kept here so the storage,
//! collector, agent, and UI crates share one vocabulary.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn trimmed(value: &str) -> String {
    value.trim().to_string()
}

pub fn non_blank(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExperienceStage {
    NewToCoding,
    EarlyCareer,
    MidCareer,
    Experienced,
}

impl ExperienceStage {
    pub const ALL: [Self; 4] = [
        Self::NewToCoding,
        Self::EarlyCareer,
        Self::MidCareer,
        Self::Experienced,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NewToCoding => "New to coding",
            Self::EarlyCareer => "Early career",
            Self::MidCareer => "Mid career",
            Self::Experienced => "Experienced",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillRating {
    Beginner,
    Intermediate,
    Advanced,
}

impl SkillRating {
    pub fn label(self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FieldSource {
    Developer,
    Document {
        #[serde(rename = "documentId")]
        document_id: String,
        #[serde(rename = "documentName")]
        document_name: String,
        reference: Option<String>,
    },
}

impl Default for FieldSource {
    fn default() -> Self {
        Self::Developer
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfReportedSkill {
    pub topic: String,
    pub rating: Option<SkillRating>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EducationEntry {
    pub institution: String,
    pub qualification: Option<String>,
    pub period: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmploymentEntry {
    pub employer: String,
    pub role: String,
    pub period: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectExperience {
    pub name: String,
    pub role: Option<String>,
    pub period: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    pub issuer: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningPreferences {
    #[serde(default)]
    pub formats: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeveloperProfile {
    pub id: String,
    pub version: u32,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "experienceStage")]
    pub experience_stage: ExperienceStage,
    #[serde(rename = "newToCoding")]
    pub new_to_coding: bool,
    #[serde(rename = "skillsStatement")]
    pub skills_statement: String,
    #[serde(rename = "learningGoals")]
    pub learning_goals: Vec<String>,
    #[serde(rename = "selfReportedSkills", default)]
    pub self_reported_skills: Vec<SelfReportedSkill>,
    #[serde(default)]
    pub education: Vec<EducationEntry>,
    #[serde(default)]
    pub employment: Vec<EmploymentEntry>,
    #[serde(default)]
    pub projects: Vec<ProjectExperience>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(rename = "learningPreferences")]
    pub learning_preferences: Option<LearningPreferences>,
    #[serde(rename = "fieldSources", default)]
    pub field_sources: std::collections::BTreeMap<String, FieldSource>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

pub const NEW_TO_CODING_STATEMENT: &str = "New to coding; no skills listed yet.";
pub const REQUIRED_PROFILE_FIELDS: [&str; 4] = [
    "display name",
    "experience stage",
    "initial skills statement",
    "learning goal",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileDraft {
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(rename = "experienceStage")]
    pub experience_stage: Option<ExperienceStage>,
    #[serde(rename = "newToCoding", default)]
    pub new_to_coding: bool,
    #[serde(rename = "skillsStatement", default)]
    pub skills_statement: String,
    #[serde(rename = "learningGoals", default)]
    pub learning_goals: Vec<String>,
    #[serde(rename = "selfReportedSkills", default)]
    pub self_reported_skills: Vec<SelfReportedSkill>,
    #[serde(default)]
    pub education: Vec<EducationEntry>,
    #[serde(default)]
    pub employment: Vec<EmploymentEntry>,
    #[serde(default)]
    pub projects: Vec<ProjectExperience>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(rename = "learningPreferences")]
    pub learning_preferences: Option<LearningPreferences>,
}

impl ProfileDraft {
    pub fn from_profile(profile: &DeveloperProfile) -> Self {
        Self {
            display_name: profile.display_name.clone(),
            experience_stage: Some(profile.experience_stage),
            new_to_coding: profile.new_to_coding,
            skills_statement: profile.skills_statement.clone(),
            learning_goals: profile.learning_goals.clone(),
            self_reported_skills: profile.self_reported_skills.clone(),
            education: profile.education.clone(),
            employment: profile.employment.clone(),
            projects: profile.projects.clone(),
            certifications: profile.certifications.clone(),
            links: profile.links.clone(),
            learning_preferences: profile.learning_preferences.clone(),
        }
    }

    /// Returns missing required-field labels in canonical order.
    pub fn missing(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if self.display_name.trim().is_empty() {
            missing.push("display name".to_string());
        }
        if self.experience_stage.is_none() {
            missing.push("experience stage".to_string());
        }
        if !self.new_to_coding && self.skills_statement.trim().is_empty() {
            missing.push("initial skills statement".to_string());
        }
        if non_blank(self.learning_goals.clone()).is_empty() {
            missing.push("learning goal".to_string());
        }
        missing
    }

    pub fn normalize(mut self) -> Self {
        self.display_name = trimmed(&self.display_name);
        self.skills_statement = trimmed(&self.skills_statement);
        if self.new_to_coding && self.skills_statement.is_empty() {
            self.skills_statement = NEW_TO_CODING_STATEMENT.to_string();
        }
        self.learning_goals = non_blank(self.learning_goals);
        self.self_reported_skills = self
            .self_reported_skills
            .into_iter()
            .filter_map(|mut skill| {
                skill.topic = trimmed(&skill.topic);
                (!skill.topic.is_empty()).then_some(skill)
            })
            .collect();
        self.links = self
            .links
            .into_iter()
            .filter_map(|mut link| {
                link.label = trimmed(&link.label);
                link.url = trimmed(&link.url);
                (!link.label.is_empty() && !link.url.is_empty()).then_some(link)
            })
            .collect();
        self
    }

    pub fn into_profile(self, previous: Option<&DeveloperProfile>) -> Result<DeveloperProfile, ValidationError> {
        let missing = self.missing();
        if !missing.is_empty() {
            return Err(ValidationError { details: missing });
        }
        let draft = self.normalize();
        let now = now_iso();
        let version = previous.map(|profile| profile.version + 1).unwrap_or(1);
        Ok(DeveloperProfile {
            id: "developer".to_string(),
            version,
            display_name: draft.display_name,
            experience_stage: draft.experience_stage.expect("validated experience stage"),
            new_to_coding: draft.new_to_coding,
            skills_statement: draft.skills_statement,
            learning_goals: draft.learning_goals,
            self_reported_skills: draft.self_reported_skills,
            education: draft.education,
            employment: draft.employment,
            projects: draft.projects,
            certifications: draft.certifications,
            links: draft.links,
            learning_preferences: draft.learning_preferences,
            field_sources: previous
                .map(|profile| profile.field_sources.clone())
                .unwrap_or_default(),
            created_at: previous
                .map(|profile| profile.created_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnboardingDraft {
    pub id: String,
    #[serde(flatten)]
    pub draft: ProfileDraft,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnboardingState {
    #[serde(rename = "requiredComplete")]
    pub required_complete: bool,
    #[serde(rename = "canStartProject")]
    pub can_start_project: bool,
    pub missing: Vec<String>,
    pub profile: Option<DeveloperProfile>,
    pub draft: Option<OnboardingDraft>,
}

impl OnboardingState {
    pub fn from_parts(profile: Option<DeveloperProfile>, draft: Option<OnboardingDraft>) -> Self {
        let missing = profile
            .as_ref()
            .map(|_| Vec::new())
            .unwrap_or_else(|| draft.as_ref().map(|draft| draft.draft.missing()).unwrap_or_else(|| {
                REQUIRED_PROFILE_FIELDS.iter().map(|field| (*field).to_string()).collect()
            }));
        Self {
            required_complete: profile.is_some(),
            can_start_project: profile.is_some(),
            missing,
            profile,
            draft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentKind {
    Cv,
    Srs,
}

impl DocumentKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cv => "CV",
            Self::Srs => "SRS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedDocument {
    pub id: String,
    pub kind: DocumentKind,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    pub name: String,
    #[serde(rename = "sourcePath")]
    pub source_path: Option<String>,
    pub sha256: String,
    pub text: String,
    #[serde(rename = "textBytes")]
    pub text_bytes: usize,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionScope {
    #[serde(rename = "includePaths", default)]
    pub include_paths: Vec<String>,
    #[serde(rename = "excludePaths", default)]
    pub exclude_paths: Vec<String>,
    #[serde(rename = "respectGitignore", default = "default_true")]
    pub respect_gitignore: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CollectionScope {
    fn default() -> Self {
        Self {
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            respect_gitignore: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureLimits {
    #[serde(rename = "perFileBytes")]
    pub per_file_bytes: u64,
    #[serde(rename = "perCaptureBytes")]
    pub per_capture_bytes: u64,
}

impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            per_file_bytes: 1024 * 1024,
            per_capture_bytes: 20 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectStatement {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub provenance: FieldSource,
}

impl ProjectStatement {
    pub fn developer(text: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            text: text.into(),
            provenance: FieldSource::Developer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub version: u32,
    pub name: String,
    #[serde(rename = "rootPath")]
    pub root_path: String,
    pub goals: Vec<ProjectStatement>,
    pub purpose: Option<ProjectStatement>,
    #[serde(rename = "businessLogic")]
    pub business_logic: Option<ProjectStatement>,
    #[serde(default)]
    pub rules: Vec<ProjectStatement>,
    #[serde(default)]
    pub constraints: Vec<ProjectStatement>,
    #[serde(default)]
    pub decisions: Vec<ProjectStatement>,
    #[serde(rename = "acceptanceCriteria", default)]
    pub acceptance_criteria: Vec<ProjectStatement>,
    #[serde(default)]
    pub knowledge: Vec<ProjectStatement>,
    pub scope: CollectionScope,
    pub limits: CaptureLimits,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDraft {
    pub name: String,
    #[serde(rename = "rootPath", default)]
    pub root_path: String,
    #[serde(default)]
    pub goals: Vec<String>,
    pub purpose: Option<String>,
    #[serde(rename = "businessLogic")]
    pub business_logic: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub decisions: Vec<String>,
    #[serde(rename = "acceptanceCriteria", default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub knowledge: Vec<String>,
    pub scope: CollectionScope,
    pub limits: CaptureLimits,
}

impl ProjectDraft {
    pub fn from_project(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            root_path: project.root_path.clone(),
            goals: project.goals.iter().map(|s| s.text.clone()).collect(),
            purpose: project.purpose.as_ref().map(|s| s.text.clone()),
            business_logic: project.business_logic.as_ref().map(|s| s.text.clone()),
            rules: project.rules.iter().map(|s| s.text.clone()).collect(),
            constraints: project.constraints.iter().map(|s| s.text.clone()).collect(),
            decisions: project.decisions.iter().map(|s| s.text.clone()).collect(),
            acceptance_criteria: project.acceptance_criteria.iter().map(|s| s.text.clone()).collect(),
            knowledge: project.knowledge.iter().map(|s| s.text.clone()).collect(),
            scope: project.scope.clone(),
            limits: project.limits.clone(),
        }
    }

    pub fn missing(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if self.name.trim().is_empty() {
            missing.push("project name".to_string());
        }
        if self.root_path.trim().is_empty() {
            missing.push("root folder".to_string());
        }
        if non_blank(self.goals.clone()).is_empty() {
            missing.push("goal".to_string());
        }
        missing
    }

    /// Create a versioned project. Unchanged statement text preserves its id
    /// and provenance; changed text becomes developer-authored.
    pub fn into_project(
        self,
        previous: Option<&Project>,
        validate_root: impl FnOnce(&str) -> bool,
    ) -> Result<Project, ValidationError> {
        let mut details = self.missing();
        if !self.root_path.trim().is_empty() && !validate_root(&self.root_path) {
            details.push("root folder must be a readable directory".to_string());
        }
        if !details.is_empty() {
            return Err(ValidationError { details });
        }
        let now = now_iso();
        let text = |value: Option<String>| value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
        let statements = |values: Vec<String>, existing: &[ProjectStatement]| {
            non_blank(values)
                .into_iter()
                .map(|value| {
                    existing
                        .iter()
                        .find(|statement| statement.text == value)
                        .cloned()
                        .unwrap_or_else(|| ProjectStatement::developer(value))
                })
                .collect::<Vec<_>>()
        };
        let previous_statements = previous.map(|p| p.goals.as_slice()).unwrap_or(&[]);
        let previous_rules = previous.map(|p| p.rules.as_slice()).unwrap_or(&[]);
        let previous_constraints = previous.map(|p| p.constraints.as_slice()).unwrap_or(&[]);
        let previous_decisions = previous.map(|p| p.decisions.as_slice()).unwrap_or(&[]);
        let previous_acceptance = previous.map(|p| p.acceptance_criteria.as_slice()).unwrap_or(&[]);
        let previous_knowledge = previous.map(|p| p.knowledge.as_slice()).unwrap_or(&[]);
        let one = |value: Option<String>, existing: Option<&ProjectStatement>| {
            text(value).map(|value| {
                existing
                    .filter(|statement| statement.text == value)
                    .cloned()
                    .unwrap_or_else(|| ProjectStatement::developer(value))
            })
        };
        let defaults = CaptureLimits::default();
        Ok(Project {
            id: previous.map(|p| p.id.clone()).unwrap_or_else(new_id),
            version: previous.map(|p| p.version + 1).unwrap_or(1),
            name: self.name.trim().to_string(),
            root_path: self.root_path.trim().to_string(),
            goals: statements(self.goals, previous_statements),
            purpose: one(self.purpose, previous.and_then(|p| p.purpose.as_ref())),
            business_logic: one(self.business_logic, previous.and_then(|p| p.business_logic.as_ref())),
            rules: statements(self.rules, previous_rules),
            constraints: statements(self.constraints, previous_constraints),
            decisions: statements(self.decisions, previous_decisions),
            acceptance_criteria: statements(self.acceptance_criteria, previous_acceptance),
            knowledge: statements(self.knowledge, previous_knowledge),
            scope: CollectionScope {
                include_paths: non_blank(self.scope.include_paths),
                exclude_paths: non_blank(self.scope.exclude_paths),
                respect_gitignore: self.scope.respect_gitignore,
            },
            limits: CaptureLimits {
                per_file_bytes: if self.limits.per_file_bytes > 0 {
                    self.limits.per_file_bytes
                } else {
                    defaults.per_file_bytes
                },
                per_capture_bytes: if self.limits.per_capture_bytes > 0 {
                    self.limits.per_capture_bytes
                } else {
                    defaults.per_capture_bytes
                },
            },
            created_at: previous.map(|p| p.created_at.clone()).unwrap_or_else(|| now.clone()),
            updated_at: now,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitState {
    pub branch: Option<String>,
    pub head: Option<String>,
    pub dirty: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturePreview {
    pub project_id: String,
    #[serde(rename = "projectVersion")]
    pub project_version: u32,
    #[serde(rename = "profileVersion")]
    pub profile_version: u32,
    pub fingerprint: String,
    pub files: Vec<CapturedFile>,
    #[serde(default)]
    pub skipped: Vec<SkippedFile>,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    pub git: Option<GitState>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextCapture {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub version: u32,
    pub fingerprint: String,
    #[serde(rename = "profileVersion")]
    pub profile_version: u32,
    #[serde(rename = "projectVersion")]
    pub project_version: u32,
    pub files: Vec<CapturedFile>,
    #[serde(default)]
    pub skipped: Vec<SkippedFile>,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    pub git: Option<GitState>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBundle {
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub application: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub profile: Option<DeveloperProfile>,
    #[serde(rename = "profileVersions")]
    pub profile_versions: Vec<DeveloperProfile>,
    pub projects: Vec<Project>,
    #[serde(rename = "projectVersions")]
    pub project_versions: Vec<Project>,
    pub documents: Vec<ImportedDocument>,
    pub captures: Vec<ContextCapture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    Codex,
    ClaudeCode,
    OpenCode,
    Antigravity,
}

impl AgentId {
    pub const ALL: [Self; 4] = [
        Self::Codex,
        Self::ClaudeCode,
        Self::OpenCode,
        Self::Antigravity,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeCode => "Claude Code",
            Self::OpenCode => "OpenCode",
            Self::Antigravity => "Antigravity",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude",
            Self::OpenCode => "opencode",
            Self::Antigravity => "agy",
        }
    }

    pub fn setup_url(self) -> &'static str {
        match self {
            Self::Codex => "https://learn.chatgpt.com/docs/app-server",
            Self::ClaudeCode => "https://code.claude.com/docs/en/headless",
            Self::OpenCode => "https://docs.opencode.ai/docs/acp/",
            Self::Antigravity => "https://antigravity.google/docs/cli/headless/",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAvailability {
    NotInstalled,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatus {
    pub id: AgentId,
    pub availability: AgentAvailability,
    pub detail: String,
    #[serde(rename = "setupUrl")]
    pub setup_url: String,
    pub verified: bool,
    pub capabilities: Vec<String>,
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDraft {
    pub kind: DocumentKind,
    pub agent: AgentId,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    Validation,
    OnboardingRequired,
    NotFound,
    UnreadablePath,
    UnsupportedDocument,
    UnreadableDocument,
    ImageOnlyDocument,
    DocumentTooLarge,
    CaptureStale,
    CaptureLimit,
    PreviewExpired,
    ConfirmationRequired,
    AgentUnavailable,
    AgentInputTooLarge,
    AgentCancelled,
    AgentTimeout,
    AgentInvalidOutput,
    AgentFailed,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::OnboardingRequired => "onboarding-required",
            Self::NotFound => "not-found",
            Self::UnreadablePath => "unreadable-path",
            Self::UnsupportedDocument => "unsupported-document",
            Self::UnreadableDocument => "unreadable-document",
            Self::ImageOnlyDocument => "image-only-document",
            Self::DocumentTooLarge => "document-too-large",
            Self::CaptureStale => "capture-stale",
            Self::CaptureLimit => "capture-limit",
            Self::PreviewExpired => "preview-expired",
            Self::ConfirmationRequired => "confirmation-required",
            Self::AgentUnavailable => "agent-unavailable",
            Self::AgentInputTooLarge => "agent-input-too-large",
            Self::AgentCancelled => "agent-cancelled",
            Self::AgentTimeout => "agent-timeout",
            Self::AgentInvalidOutput => "agent-invalid-output",
            Self::AgentFailed => "agent-failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default)]
    pub details: Vec<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn with_details(code: ErrorCode, message: impl Into<String>, details: Vec<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details,
        }
    }

    pub fn validation(details: Vec<String>) -> Self {
        let message = if details.is_empty() {
            "The input is invalid.".to_string()
        } else {
            format!("Missing or invalid: {}.", details.join(", "))
        };
        Self::with_details(ErrorCode::Validation, message, details)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub details: Vec<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details.join(", "))
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for AppError {
    fn from(value: ValidationError) -> Self {
        AppError::validation(value.details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_draft() -> ProfileDraft {
        ProfileDraft {
            display_name: "  Ada  ".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust and TypeScript".to_string(),
            learning_goals: vec!["Ship a native app".to_string(), "  ".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn profile_draft_requires_skills_unless_new_to_coding() {
        let mut draft = valid_draft();
        draft.new_to_coding = true;
        draft.skills_statement.clear();
        assert!(draft.missing().is_empty());
        let profile = draft.into_profile(None).unwrap();
        assert_eq!(profile.skills_statement, NEW_TO_CODING_STATEMENT);
        assert_eq!(profile.learning_goals.len(), 1);
        assert_eq!(profile.version, 1);
    }

    #[test]
    fn profile_versions_increment_and_keep_created_at() {
        let first = valid_draft().into_profile(None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = valid_draft().into_profile(Some(&first)).unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(second.created_at, first.created_at);
    }

    #[test]
    fn project_preserves_unchanged_statement_provenance() {
        let root = std::env::temp_dir();
        let first = ProjectDraft {
            name: "Demo".to_string(),
            root_path: root.display().to_string(),
            goals: vec!["Ship it".to_string()],
            ..Default::default()
        }
        .into_project(None, |_| true)
        .unwrap();
        let source = FieldSource::Document {
            document_id: "doc".to_string(),
            document_name: "srs.md".to_string(),
            reference: None,
        };
        let mut previous = first.clone();
        previous.goals[0].provenance = source.clone();
        let second = ProjectDraft::from_project(&previous)
            .into_project(Some(&previous), |_| true)
            .unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(second.goals[0].id, previous.goals[0].id);
        assert_eq!(second.goals[0].provenance, source);
    }
}
