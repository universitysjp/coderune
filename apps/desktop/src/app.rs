//! Coderune's native desktop application.

use std::collections::BTreeMap;
use std::path::PathBuf;

use coderune_app::{PreviewSession, Workspace};
use coderune_avatars::{BotAvatarStage, BotAvatarState, BotAvatarType};
use coderune_domain::{
    AgentAvailability, AgentId, AppError, Certification, ContextCapture, DocumentKind,
    EducationEntry, EmploymentEntry, ExperienceStage, LearningPreferences, Link, ProfileDraft,
    ProjectDraft, ProjectExperience, SelfReportedSkill, SkillRating,
};
use rune::{col, row, text, Cx, El, Key, KeyEvent, ScrollSpec, TextEdit, Tx};
use rune_kit::shell::{PaneState, Sidebar, SplitView, Toolbar};
use rune_kit::Kit;

use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};

use crate::{screens, updates::Updates};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Profile,
    Projects,
    Documents,
    Agents,
    Capture,
    Data,
    /// The avatar gallery: a development scene, reachable with `--scene avatars`.
    Avatars,
}

impl View {
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Profile => "Profile",
            Self::Projects => "Projects",
            Self::Documents => "Documents",
            Self::Agents => "Agents",
            Self::Capture => "Capture",
            Self::Data => "Data",
            Self::Avatars => "Avatars",
        }
    }

    pub fn icon(self) -> &'static rune::Icon {
        match self {
            Self::Home => &icons::HOUSE,
            Self::Profile => &icons::USER,
            Self::Projects => &icons::FOLDER,
            Self::Documents => &icons::FILES,
            Self::Agents => &icons::ROBOT,
            Self::Capture => &icons::SCAN,
            Self::Data => &icons::GEAR,
            Self::Avatars => &icons::ROBOT,
        }
    }

    pub const NAV: [Self; 6] = [
        Self::Home,
        Self::Profile,
        Self::Projects,
        Self::Documents,
        Self::Agents,
        Self::Capture,
    ];
}

/// The three short onboarding questions. Optional profile detail lives in the
/// app's Profile screen, so onboarding never becomes a form with everything on
/// one page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    About,
    StartingPoint,
    Review,
}

impl OnboardingStep {
    pub const ALL: [Self; 3] = [Self::About, Self::StartingPoint, Self::Review];

    pub fn index(self) -> usize {
        match self {
            Self::About => 0,
            Self::StartingPoint => 1,
            Self::Review => 2,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::About => "About you",
            Self::StartingPoint => "Your starting point",
            Self::Review => "Review your baseline",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::About => "Two quick questions. Your name and where you are starting from.",
            Self::StartingPoint => {
                "Two more. Say what you can do today and what you want to learn. New to coding is a valid answer."
            }
            Self::Review => {
                "Check the baseline Coderune will share with your projects. You can change it later."
            }
        }
    }

    pub fn previous(self) -> Option<Self> {
        match self {
            Self::About => None,
            Self::StartingPoint => Some(Self::About),
            Self::Review => Some(Self::StartingPoint),
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::About => Some(Self::StartingPoint),
            Self::StartingPoint => Some(Self::Review),
            Self::Review => None,
        }
    }
}

/// Profile's two views: the required baseline and the optional details that
/// live inside the app after onboarding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileTab {
    Baseline,
    Details,
}

/// The optional profile sections. One open at a time keeps the details view
/// scannable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSection {
    Skills,
    Education,
    Employment,
    Projects,
    Certifications,
    Links,
    Preferences,
}

impl ProfileSection {
    pub const ALL: [Self; 7] = [
        Self::Skills,
        Self::Education,
        Self::Employment,
        Self::Projects,
        Self::Certifications,
        Self::Links,
        Self::Preferences,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Skills => "Self-reported skills",
            Self::Education => "Education",
            Self::Employment => "Employment",
            Self::Projects => "Project experience",
            Self::Certifications => "Certifications",
            Self::Links => "Links",
            Self::Preferences => "Learning preferences",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Skills => "One skill per line: topic | beginner, intermediate, or advanced.",
            Self::Education => "One per line: institution | qualification | period | details.",
            Self::Employment => "One per line: employer | role | period | details.",
            Self::Projects => "One per line: name | role | period | details.",
            Self::Certifications => "One per line: name | issuer | year.",
            Self::Links => "One per line: label | https://example.com.",
            Self::Preferences => "How you like to learn, and any notes for later sessions.",
        }
    }
}

fn edit(value: impl Into<String>) -> TextEdit {
    let mut edit = TextEdit::default();
    edit.text = value.into();
    edit.anchor = edit.text.len();
    edit.focus = edit.text.len();
    edit
}

pub struct ProfileForm {
    pub display_name: TextEdit,
    pub skills: TextEdit,
    pub goals: TextEdit,
    pub stage: Option<ExperienceStage>,
    pub new_to_coding: bool,
    pub self_reported_skills: TextEdit,
    pub education: TextEdit,
    pub employment: TextEdit,
    pub project_experience: TextEdit,
    pub certifications: TextEdit,
    pub links: TextEdit,
    pub learning_formats: TextEdit,
    pub learning_notes: TextEdit,
}

impl ProfileForm {
    pub(crate) fn from_value(
        profile: Option<&coderune_domain::DeveloperProfile>,
        draft: Option<&coderune_domain::OnboardingDraft>,
    ) -> Self {
        if let Some(profile) = profile {
            Self::from_draft(&ProfileDraft::from_profile(profile))
        } else if let Some(draft) = draft {
            Self::from_draft(&draft.draft)
        } else {
            Self::from_draft(&ProfileDraft::default())
        }
    }

    fn from_draft(draft: &ProfileDraft) -> Self {
        Self {
            display_name: edit(draft.display_name.clone()),
            skills: edit(draft.skills_statement.clone()),
            goals: edit(draft.learning_goals.join(", ")),
            stage: draft.experience_stage,
            new_to_coding: draft.new_to_coding,
            self_reported_skills: edit(format_skills(&draft.self_reported_skills)),
            education: edit(format_education(&draft.education)),
            employment: edit(format_employment(&draft.employment)),
            project_experience: edit(format_project_experience(&draft.projects)),
            certifications: edit(format_certifications(&draft.certifications)),
            links: edit(format_links(&draft.links)),
            learning_formats: edit(
                draft
                    .learning_preferences
                    .as_ref()
                    .map(|preferences| preferences.formats.join(", "))
                    .unwrap_or_default(),
            ),
            learning_notes: edit(
                draft
                    .learning_preferences
                    .as_ref()
                    .and_then(|preferences| preferences.notes.clone())
                    .unwrap_or_default(),
            ),
        }
    }

    /// Preserve fields the form does not edit (CV provenance) and overwrite the
    /// visible ones, including the optional sections completed inside the app.
    pub(crate) fn to_draft(&self, base: ProfileDraft) -> ProfileDraft {
        let mut draft = base;
        draft.display_name = self.display_name.text.trim().to_string();
        draft.experience_stage = self.stage;
        draft.new_to_coding = self.new_to_coding;
        draft.skills_statement = self.skills.text.trim().to_string();
        draft.learning_goals = split_list(&self.goals.text);
        draft.self_reported_skills = parse_skills(&self.self_reported_skills.text);
        draft.education = parse_education(&self.education.text);
        draft.employment = parse_employment(&self.employment.text);
        draft.projects = parse_project_experience(&self.project_experience.text);
        draft.certifications = parse_certifications(&self.certifications.text);
        draft.links = parse_links(&self.links.text);
        let formats = split_list(&self.learning_formats.text);
        let notes = self.learning_notes.text.trim().to_string();
        draft.learning_preferences = if formats.is_empty() && notes.is_empty() {
            None
        } else {
            Some(LearningPreferences {
                formats,
                notes: (!notes.is_empty()).then_some(notes),
            })
        };
        draft
    }

    /// Whether the optional sections hold anything at all.
    pub(crate) fn details_count(&self) -> usize {
        [
            &self.self_reported_skills,
            &self.education,
            &self.employment,
            &self.project_experience,
            &self.certifications,
            &self.links,
            &self.learning_formats,
            &self.learning_notes,
        ]
        .into_iter()
        .filter(|field| !field.text.trim().is_empty())
        .count()
    }
}

pub struct ProjectForm {
    pub name: TextEdit,
    pub root: TextEdit,
    pub goals: TextEdit,
    pub include_paths: TextEdit,
    pub exclude_paths: TextEdit,
    pub respect_gitignore: bool,
    pub per_file_kib: TextEdit,
    pub per_capture_mib: TextEdit,
    pub editing: Option<String>,
}

impl ProjectForm {
    pub(crate) fn empty(initial_root: Option<PathBuf>) -> Self {
        Self {
            name: edit(""),
            root: edit(
                initial_root
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
            ),
            goals: edit(""),
            include_paths: edit(""),
            exclude_paths: edit(""),
            respect_gitignore: true,
            per_file_kib: edit("1024"),
            per_capture_mib: edit("20"),
            editing: None,
        }
    }

    pub(crate) fn from_project(project: &coderune_domain::Project) -> Self {
        Self {
            name: edit(project.name.clone()),
            root: edit(project.root_path.clone()),
            goals: edit(
                project
                    .goals
                    .iter()
                    .map(|goal| goal.text.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            include_paths: edit(project.scope.include_paths.join(", ")),
            exclude_paths: edit(project.scope.exclude_paths.join(", ")),
            respect_gitignore: project.scope.respect_gitignore,
            per_file_kib: edit((project.limits.per_file_bytes / 1024).to_string()),
            per_capture_mib: edit((project.limits.per_capture_bytes / (1024 * 1024)).to_string()),
            editing: Some(project.id.clone()),
        }
    }

    /// Preserve project fields the form does not edit (purpose, business logic, rules,
    /// constraints, decisions, acceptance criteria, knowledge) and overwrite visible ones.
    pub(crate) fn to_draft(
        &self,
        previous: Option<&coderune_domain::Project>,
    ) -> Result<ProjectDraft, AppError> {
        let parse = |value: &str, multiplier: u64, field: &str| {
            value
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .and_then(|value| value.checked_mul(multiplier))
                .ok_or_else(|| AppError::validation(vec![field.to_string()]))
        };
        let mut draft = previous.map(ProjectDraft::from_project).unwrap_or_default();
        draft.name = self.name.text.trim().to_string();
        draft.root_path = self.root.text.trim().to_string();
        draft.goals = split_list(&self.goals.text);
        draft.scope = coderune_domain::CollectionScope {
            include_paths: split_list(&self.include_paths.text),
            exclude_paths: split_list(&self.exclude_paths.text),
            respect_gitignore: self.respect_gitignore,
        };
        draft.limits = coderune_domain::CaptureLimits {
            per_file_bytes: parse(&self.per_file_kib.text, 1024, "per-file limit")?,
            per_capture_bytes: parse(&self.per_capture_mib.text, 1024 * 1024, "per-capture limit")?,
        };
        Ok(draft)
    }
}

#[derive(Clone)]
pub struct ExtractedDocument {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) bytes: usize,
    pub(crate) source_path: Option<String>,
}

pub struct DocumentForm {
    pub path: TextEdit,
    pub kind: DocumentKind,
    pub project_id: Option<String>,
    pub extracted: Option<ExtractedDocument>,
    pub selected_document: Option<String>,
    pub agent: Option<AgentId>,
    pub draft_preview: Option<coderune_agents::DraftSharingPreview>,
}

impl DocumentForm {
    pub(crate) fn empty() -> Self {
        Self {
            path: edit(""),
            kind: DocumentKind::Cv,
            project_id: None,
            extracted: None,
            selected_document: None,
            agent: None,
            draft_preview: None,
        }
    }
}

pub struct CaptureDetail {
    pub capture: ContextCapture,
    pub contents: BTreeMap<String, String>,
    pub selected_path: Option<String>,
}

pub struct CoderuneApp {
    pub(crate) workspace: Option<Workspace>,
    pub(crate) fatal: Option<String>,
    pub view: View,
    pub(crate) sidebar: PaneState,
    pub onboarding_step: OnboardingStep,
    pub profile_tab: ProfileTab,
    pub profile_section: Option<ProfileSection>,
    pub(crate) profile_form: ProfileForm,
    pub(crate) project_form: ProjectForm,
    pub(crate) document_form: DocumentForm,
    pub(crate) capture_session: Option<PreviewSession>,
    pub(crate) capture_reviewed: bool,
    pub(crate) captures: Vec<ContextCapture>,
    pub(crate) capture_detail: Option<CaptureDetail>,
    pub(crate) confirm_delete_project: Option<String>,
    pub(crate) confirm_delete_capture: Option<String>,
    pub(crate) confirm_delete_all: bool,
    pub(crate) export_path: TextEdit,
    pub(crate) updates: Updates,
    pub error: Option<AppError>,
    pub notice: Option<String>,
    /// The animated avatars on screen, keyed by agent or gallery id.
    pub(crate) avatars: BotAvatarStage,
}

impl CoderuneApp {
    pub fn init() -> Self {
        let mut fatal = None;
        let mut workspace = match Workspace::open_default() {
            Ok(workspace) => Some(workspace),
            Err(error) => {
                fatal = Some(format!("Could not open local storage: {error}"));
                None
            }
        };
        let launch = std::env::args()
            .nth(1)
            .filter(|argument| !argument.starts_with("--"))
            .map(PathBuf::from);
        if let (Some(workspace), Some(directory)) = (workspace.as_mut(), launch) {
            if let Err(error) = workspace.adopt_launch_directory(directory) {
                fatal = Some(format!("Could not inspect the launch directory: {error}"));
            }
        }
        let profile_form = match workspace.as_ref() {
            Some(workspace) => ProfileForm::from_value(
                workspace.onboarding.profile.as_ref(),
                workspace.onboarding.draft.as_ref(),
            ),
            None => ProfileForm::from_value(None, None),
        };
        let project_form = match workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project())
        {
            Some(project) => ProjectForm::from_project(project),
            None => ProjectForm::empty(
                workspace
                    .as_ref()
                    .and_then(|workspace| workspace.launch_directory.clone()),
            ),
        };
        let export_path = edit(
            std::env::var("HOME")
                .map(|home| {
                    PathBuf::from(home)
                        .join("coderune-export.json")
                        .display()
                        .to_string()
                })
                .unwrap_or_else(|_| "coderune-export.json".to_string()),
        );
        let mut app = Self {
            workspace,
            fatal,
            view: View::Home,
            sidebar: PaneState::new(theme::SIDEBAR).range(200.0, 320.0),
            onboarding_step: OnboardingStep::About,
            profile_tab: ProfileTab::Baseline,
            profile_section: None,
            profile_form,
            project_form,
            document_form: DocumentForm::empty(),
            capture_session: None,
            capture_reviewed: false,
            captures: Vec::new(),
            capture_detail: None,
            confirm_delete_project: None,
            confirm_delete_capture: None,
            confirm_delete_all: false,
            export_path,
            updates: Updates::new(),
            error: None,
            notice: None,
            avatars: BotAvatarStage::new(),
        };
        app.refresh_captures();
        app.onboarding_step = app.initial_onboarding_step();
        app
    }

    /// Resume onboarding at the first step whose required answers are missing.
    fn initial_onboarding_step(&self) -> OnboardingStep {
        let Some(workspace) = self.workspace.as_ref() else {
            return OnboardingStep::About;
        };
        let Some(draft) = workspace.onboarding.draft.as_ref() else {
            return OnboardingStep::About;
        };
        let draft = &draft.draft;
        if draft.display_name.trim().is_empty() || draft.experience_stage.is_none() {
            OnboardingStep::About
        } else if (!draft.new_to_coding && draft.skills_statement.trim().is_empty())
            || draft
                .learning_goals
                .iter()
                .all(|goal| goal.trim().is_empty())
        {
            OnboardingStep::StartingPoint
        } else {
            OnboardingStep::Review
        }
    }

    fn profile_base(&self) -> ProfileDraft {
        self.workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .onboarding
                    .profile
                    .as_ref()
                    .map(ProfileDraft::from_profile)
                    .or_else(|| {
                        workspace
                            .onboarding
                            .draft
                            .as_ref()
                            .map(|draft| draft.draft.clone())
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    fn reset_capture_state(&mut self) {
        self.capture_session = None;
        self.capture_reviewed = false;
        self.capture_detail = None;
        self.confirm_delete_capture = None;
    }

    fn refresh_captures(&mut self) {
        let project_id = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project_id.clone());
        let result = project_id.as_deref().and_then(|id| {
            self.workspace
                .as_ref()
                .map(|workspace| workspace.list_captures(id))
        });
        match result {
            Some(Ok(captures)) => self.captures = captures,
            Some(Err(error)) => {
                self.captures.clear();
                self.error = Some(error);
            }
            None => self.captures.clear(),
        }
    }

    pub(crate) fn refresh_profile_form(&mut self) {
        if let Some(workspace) = self.workspace.as_ref() {
            self.profile_form = ProfileForm::from_value(
                workspace.onboarding.profile.as_ref(),
                workspace.onboarding.draft.as_ref(),
            );
        }
    }

    pub(crate) fn navigate(&mut self, view: View) {
        self.view = view;
        self.error = None;
        self.notice = None;
        self.confirm_delete_project = None;
        self.confirm_delete_capture = None;
        self.confirm_delete_all = false;
    }

    pub(crate) fn save_draft(&mut self) {
        if self.persist_draft() {
            self.notice = Some("Progress saved. You can close and resume.".to_string());
        }
    }

    /// Save the current form as the resumable draft. Returns whether it stuck.
    fn persist_draft(&mut self) -> bool {
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return false;
        };
        match workspace.save_onboarding_draft(draft) {
            Ok(()) => {
                self.error = None;
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    /// Why the current onboarding step cannot continue, if it cannot.
    fn onboarding_blocker(&self) -> Option<&'static str> {
        match self.onboarding_step {
            OnboardingStep::About => {
                if self.profile_form.display_name.text.trim().is_empty() {
                    Some("Add a display name or pseudonym to continue.")
                } else if self.profile_form.stage.is_none() {
                    Some("Choose the experience stage that fits you today.")
                } else {
                    None
                }
            }
            OnboardingStep::StartingPoint => {
                if !self.profile_form.new_to_coding
                    && self.profile_form.skills.text.trim().is_empty()
                {
                    Some("Say what you can do today, or mark that you are new to coding.")
                } else if split_list(&self.profile_form.goals.text).is_empty() {
                    Some("Add at least one learning goal to continue.")
                } else {
                    None
                }
            }
            OnboardingStep::Review => None,
        }
    }

    pub(crate) fn onboarding_next(&mut self) {
        if self.onboarding_step == OnboardingStep::Review {
            self.confirm_profile();
            return;
        }
        if let Some(message) = self.onboarding_blocker() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                message,
            ));
            self.notice = None;
            return;
        }
        if !self.persist_draft() {
            return;
        }
        if let Some(next) = self.onboarding_step.next() {
            self.onboarding_step = next;
            self.error = None;
            self.notice = None;
        }
    }

    pub(crate) fn onboarding_back(&mut self) {
        if let Some(previous) = self.onboarding_step.previous() {
            self.onboarding_step = previous;
            self.error = None;
            self.notice = None;
        }
    }

    pub(crate) fn open_profile_details(&mut self) {
        self.view = View::Profile;
        self.profile_tab = ProfileTab::Details;
        self.error = None;
        self.notice = None;
    }

    pub(crate) fn toggle_profile_section(&mut self, section: ProfileSection) {
        self.profile_section = (self.profile_section != Some(section)).then_some(section);
    }

    pub(crate) fn confirm_profile(&mut self) {
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.confirm_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!(
                    "Developer baseline v{} confirmed. Add more to your profile whenever you want.",
                    profile.version
                ));
                self.error = None;
                self.view = View::Home;
                self.profile_tab = ProfileTab::Baseline;
                self.profile_section = None;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_profile(&mut self) {
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.update_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!("Profile v{} saved.", profile.version));
                self.error = None;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn load_project_form(&mut self, id: &str) {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.projects.iter().find(|project| project.id == id))
            .cloned()
        else {
            return;
        };
        self.project_form = ProjectForm::from_project(&project);
        self.confirm_delete_project = None;
        self.error = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(Some(project.id));
        }
        self.refresh_captures();
    }

    pub(crate) fn new_project_form(&mut self) {
        let initial_root = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        self.project_form = ProjectForm::empty(initial_root);
        self.confirm_delete_project = None;
        self.error = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(None);
        }
        self.refresh_captures();
    }

    pub(crate) fn save_project(&mut self) {
        let previous = self.project_form.editing.as_deref().and_then(|id| {
            self.workspace
                .as_ref()
                .and_then(|workspace| workspace.projects.iter().find(|project| project.id == id))
                .cloned()
        });
        let draft = match self.project_form.to_draft(previous.as_ref()) {
            Ok(draft) => draft,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        let result = match self.project_form.editing.as_deref() {
            Some(id) => workspace.update_project(id, draft),
            None => workspace.create_project(draft),
        };
        match result {
            Ok(project) => {
                self.notice = Some(format!(
                    "Project {} saved at version {}.",
                    project.name, project.version
                ));
                self.error = None;
                self.project_form = ProjectForm::from_project(&project);
                self.confirm_delete_project = None;
                self.reset_capture_state();
                self.refresh_captures();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn delete_project(&mut self) {
        let Some(id) = self.confirm_delete_project.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_project(&id) {
            Ok(()) => {
                self.notice = Some("Project and its captures deleted.".to_string());
                self.error = None;
                self.new_project_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn extract_document(&mut self) {
        let path = self.document_form.path.text.trim().to_string();
        if path.is_empty() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadableDocument,
                "Choose a document path first.",
            ));
            return;
        }
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.extract_document(PathBuf::from(&path), 20 * 1024 * 1024) {
            Ok((name, text, bytes)) => {
                self.document_form.extracted = Some(ExtractedDocument {
                    name,
                    text,
                    bytes,
                    source_path: Some(path),
                });
                self.error = None;
                self.notice = Some("Text extracted locally. Review it before saving.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_document_import(&mut self) {
        let Some(extracted) = self.document_form.extracted.take() else {
            return;
        };
        let result = self.workspace.as_mut().map(|workspace| {
            workspace.save_document(
                self.document_form.kind,
                self.document_form.project_id.clone(),
                &extracted.name,
                extracted.source_path.as_deref(),
                &extracted.text,
                true,
            )
        });
        match result {
            Some(Ok(document)) => {
                self.notice = Some(format!(
                    "Saved {} import {}.",
                    document.kind.label(),
                    document.name
                ));
                self.error = None;
                self.document_form.draft_preview = None;
                self.document_form.selected_document = Some(document.id);
            }
            Some(Err(error)) => {
                self.document_form.extracted = Some(extracted);
                self.error = Some(error);
            }
            None => self.document_form.extracted = Some(extracted),
        }
    }

    pub(crate) fn prepare_draft(&mut self) {
        let (Some(agent), Some(document_id)) = (
            self.document_form.agent,
            self.document_form.selected_document.clone(),
        ) else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Select an imported document and a coding agent first.",
            ));
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.prepare_draft(agent, &document_id) {
            Ok(preview) => {
                self.document_form.draft_preview = Some(preview);
                self.error = None;
                self.notice =
                    Some("Sharing preview prepared. No agent process has started.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn preview_capture(&mut self) {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project())
            .cloned()
        else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::NotFound,
                "Create or select a project before capturing context.",
            ));
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.preview_capture(&project.id, project.scope.clone(), project.limits.clone())
        {
            Ok(session) => {
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some(
                    "Preview collected. Review the exact file list before saving.".to_string(),
                );
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn refresh_capture(&mut self) {
        let Some(project_id) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project_id.clone())
        else {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::NotFound,
                "Create or select a project before refreshing context.",
            ));
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.refresh_capture(&project_id) {
            Ok((latest, session)) => {
                let unchanged = latest.is_some();
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some(if unchanged {
                    "Repository unchanged. The latest capture already matches this state."
                        .to_string()
                } else {
                    "Changes found. A fresh preview is ready for review.".to_string()
                });
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_capture(&mut self) {
        let Some(session) = self.capture_session.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.save_capture(&session, self.capture_reviewed) {
            Ok(capture) => {
                self.notice = Some(format!("Context capture v{} saved.", capture.version));
                self.error = None;
                self.capture_reviewed = false;
                self.capture_session = None;
                self.refresh_captures();
            }
            Err(error) => {
                if error.code == coderune_domain::ErrorCode::CaptureStale {
                    self.capture_reviewed = false;
                }
                self.error = Some(error);
            }
        }
    }

    pub(crate) fn inspect_capture(&mut self, id: &str) {
        let Some(capture) = self
            .captures
            .iter()
            .find(|capture| capture.id == id)
            .cloned()
        else {
            return;
        };
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.capture_contents(&capture.id) {
            Ok(contents) => {
                let selected_path = capture.files.first().map(|file| file.path.clone());
                self.capture_detail = Some(CaptureDetail {
                    capture,
                    contents,
                    selected_path,
                });
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn delete_capture(&mut self, id: &str) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_capture(id) {
            Ok(()) => {
                self.notice = Some("Context capture deleted.".to_string());
                self.error = None;
                if self
                    .capture_detail
                    .as_ref()
                    .is_some_and(|detail| detail.capture.id == id)
                {
                    self.capture_detail = None;
                }
                self.confirm_delete_capture = None;
                self.refresh_captures();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn refresh_agents(&mut self) {
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.agents = coderune_agents::detect_all();
        }
        self.notice = Some("Agent detection refreshed.".to_string());
    }

    pub(crate) fn export_data(&mut self) {
        let path = PathBuf::from(self.export_path.text.trim());
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match workspace.export_json() {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => {
                    self.notice = Some(format!("Exported local context to {}.", path.display()));
                    self.error = None;
                }
                Err(error) => {
                    self.error = Some(AppError::new(
                        coderune_domain::ErrorCode::Validation,
                        format!("Could not write {}: {error}", path.display()),
                    ));
                }
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn delete_all_data(&mut self) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_all_data() {
            Ok(()) => {
                self.notice =
                    Some("All application data deleted. Onboarding is required again.".to_string());
                self.error = None;
                self.confirm_delete_all = false;
                self.view = View::Home;
                self.reset_capture_state();
                self.confirm_delete_project = None;
                self.confirm_delete_capture = None;
                self.confirm_delete_all = false;
                self.project_form = ProjectForm::empty(None);
                self.document_form = DocumentForm::empty();
                self.refresh_profile_form();
                self.refresh_captures();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn open_folder(&mut self, path: PathBuf) {
        if let Err(error) = Workspace::open_directory(&path) {
            self.error = Some(error);
        }
    }

    pub(crate) fn open_editor(&mut self, editor: &str, path: PathBuf) {
        if let Err(error) = Workspace::launch_editor(editor, &path) {
            self.error = Some(error);
        }
    }
}

/// Seed a developer baseline and one project so headless scenes can show the workspace.
pub fn seed_demo(s: &mut CoderuneApp) {
    let needs_profile = s
        .workspace
        .as_ref()
        .map(|workspace| !workspace.onboarding.can_start_project)
        .unwrap_or(true);
    if needs_profile {
        s.profile_form.display_name = edit("Ada");
        s.profile_form.stage = Some(ExperienceStage::EarlyCareer);
        s.profile_form.skills = edit("Rust and TypeScript");
        s.profile_form.goals = edit("Ship the native desktop app, Learn Rune");
        s.confirm_profile();
    }
    let needs_project = s
        .workspace
        .as_ref()
        .map(|workspace| workspace.projects.is_empty())
        .unwrap_or(true);
    if needs_project {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        s.project_form = ProjectForm::empty(Some(root));
        s.project_form.name = edit("Coderune");
        s.project_form.goals = edit("Ship the native desktop app");
        s.save_project();
    }
}

pub fn scene_home(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Home;
}
pub fn scene_profile(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Profile;
}
pub fn scene_projects(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Projects;
}
pub fn scene_documents(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Documents;
}
pub fn scene_agents(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Agents;
}
pub fn scene_capture(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Capture;
}
pub fn scene_capture_preview(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Capture;
    s.preview_capture();
}
pub fn scene_data(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Data;
}

/// The avatar gallery, a development scene reached with `--scene avatars`.
pub fn scene_avatars(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Avatars;
}

/// The body an agent wears.
pub fn avatar_kind(id: AgentId) -> BotAvatarType {
    match id {
        AgentId::Codex => BotAvatarType::Mech,
        AgentId::ClaudeCode => BotAvatarType::Clover,
        AgentId::OpenCode => BotAvatarType::Square,
        AgentId::Antigravity => BotAvatarType::Star,
    }
}

/// What an agent's avatar is doing: working while the agent is ready, asleep
/// otherwise.
pub fn avatar_state(availability: AgentAvailability) -> BotAvatarState {
    if availability == AgentAvailability::Ready {
        BotAvatarState::Working
    } else {
        BotAvatarState::Sleeping
    }
}

/// Advances visible animations and applies completed background updates.
pub fn poll(s: &mut CoderuneApp, t: f64, tx: &mut Tx) -> bool {
    let update_changed = s.updates.poll(tx);
    let avatar_changed = match s.view {
        View::Agents => {
            let wanted: Vec<(String, BotAvatarType, BotAvatarState)> = s
                .workspace
                .as_ref()
                .map(|workspace| {
                    workspace
                        .agents
                        .iter()
                        .map(|status| {
                            (
                                status.id.label().to_string(),
                                avatar_kind(status.id),
                                avatar_state(status.availability),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            for (id, kind, state) in wanted {
                s.avatars.set(&id, kind, state);
            }
            s.avatars.tick(t, 1.0)
        }
        View::Avatars => {
            for kind in BotAvatarType::ALL {
                for state in BotAvatarState::ALL {
                    s.avatars
                        .set(&screens::gallery_id(kind, state), kind, state);
                }
            }
            s.avatars.tick(t, 1.0)
        }
        _ => {
            if !s.avatars.is_empty() {
                s.avatars.clear();
            }
            false
        }
    };
    update_changed || avatar_changed
}

/// Onboarding's second step, with the first step's answers filled in.
pub fn scene_onboarding_start(s: &mut CoderuneApp) {
    s.profile_form.display_name = edit("Ada");
    s.profile_form.stage = Some(ExperienceStage::EarlyCareer);
    s.onboarding_step = OnboardingStep::StartingPoint;
}

/// Onboarding's review step, without confirming the baseline.
pub fn scene_onboarding_review(s: &mut CoderuneApp) {
    scene_onboarding_start(s);
    s.profile_form.skills = edit("TypeScript and a little Rust");
    s.profile_form.goals = edit("Ship the native desktop app, Learn Rune");
    s.onboarding_step = OnboardingStep::Review;
}

/// Profile's optional-details view with one section open.
pub fn scene_profile_details(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Profile;
    s.profile_tab = ProfileTab::Details;
    s.profile_section = Some(ProfileSection::Skills);
    s.profile_form.self_reported_skills = edit("Rust | intermediate\nTypeScript | advanced");
    s.profile_form.education = edit("University of Moratuwa | BSc Computer Science | 2021-2025");
}

fn opt_field(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Split a details text area into trimmed `|`-separated columns.
fn rows(value: &str) -> Vec<Vec<String>> {
    value
        .lines()
        .map(|line| {
            line.split('|')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .filter(|row: &Vec<String>| row.first().is_some_and(|first| !first.is_empty()))
        .collect()
}

fn join_fields(fields: &[Option<&str>]) -> String {
    fields
        .iter()
        .map(|field| field.unwrap_or("").trim())
        .collect::<Vec<_>>()
        .join(" | ")
        .trim_end_matches(" | ")
        .to_string()
}

fn format_skills(skills: &[SelfReportedSkill]) -> String {
    skills
        .iter()
        .map(|skill| match skill.rating {
            Some(rating) => format!("{} | {}", skill.topic, rating.label().to_lowercase()),
            None => skill.topic.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_skills(value: &str) -> Vec<SelfReportedSkill> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let topic = row.first()?.trim().to_string();
            let rating = row
                .get(1)
                .and_then(|rating| match rating.to_lowercase().as_str() {
                    "beginner" => Some(SkillRating::Beginner),
                    "intermediate" => Some(SkillRating::Intermediate),
                    "advanced" => Some(SkillRating::Advanced),
                    _ => None,
                });
            Some(SelfReportedSkill { topic, rating })
        })
        .collect()
}

fn format_education(entries: &[EducationEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.institution.as_str()),
                entry.qualification.as_deref(),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_education(value: &str) -> Vec<EducationEntry> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let institution = row.first()?.clone();
            Some(EducationEntry {
                institution,
                qualification: row.get(1).and_then(|value| opt_field(value)),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_employment(entries: &[EmploymentEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.employer.as_str()),
                Some(entry.role.as_str()),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_employment(value: &str) -> Vec<EmploymentEntry> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let employer = row.first()?.clone();
            Some(EmploymentEntry {
                employer,
                role: row.get(1).cloned().unwrap_or_default(),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_project_experience(entries: &[ProjectExperience]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.name.as_str()),
                entry.role.as_deref(),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_project_experience(value: &str) -> Vec<ProjectExperience> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let name = row.first()?.clone();
            Some(ProjectExperience {
                name,
                role: row.get(1).and_then(|value| opt_field(value)),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_certifications(entries: &[Certification]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.name.as_str()),
                entry.issuer.as_deref(),
                entry.year.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_certifications(value: &str) -> Vec<Certification> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let name = row.first()?.clone();
            Some(Certification {
                name,
                issuer: row.get(1).and_then(|value| opt_field(value)),
                year: row.get(2).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_links(entries: &[Link]) -> String {
    entries
        .iter()
        .map(|entry| join_fields(&[Some(entry.label.as_str()), Some(entry.url.as_str())]))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_links(value: &str) -> Vec<Link> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let label = row.first()?.clone();
            let url = row.get(1).cloned().unwrap_or_default();
            (!url.is_empty()).then_some(Link { label, url })
        })
        .collect()
}

pub(crate) fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n', ';'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub fn view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    let k = theme::kit();
    let inset = cx.titlebar_inset();
    let body = if let Some(message) = &s.fatal {
        fatal(message, inset)
    } else if s
        .workspace
        .as_ref()
        .is_some_and(|workspace| workspace.onboarding.can_start_project)
    {
        workspace(s, k, cx)
    } else {
        screens::onboarding(s, k, inset)
    };
    col().w_full().h_full().fill(theme::bg()).child(body)
}

fn fatal(message: &str, inset: f32) -> El {
    use rune_kit::shell::Page;

    Page::new(
        col()
            .w_full()
            .h_full()
            .center()
            .gap(12.0)
            .padding(32.0)
            .child(
                text("Coderune could not start")
                    .font(HEADING)
                    .color(theme::ink()),
            )
            .child(
                text(message)
                    .font(BODY)
                    .color(theme::muted())
                    .wrap()
                    .max_w(520.0),
            ),
    )
    .title("Coderune")
    .titlebar_inset(inset)
    .toolbar_in_panel()
    .frame_inset(theme::FRAME)
    .build(theme::kit())
}

fn workspace(s: &CoderuneApp, k: &Kit, cx: &mut Cx<CoderuneApp>) -> El {
    let content = match s.view {
        View::Home => screens::home(s, k),
        View::Profile => screens::profile(s, k),
        View::Projects => screens::projects(s, k),
        View::Documents => screens::documents(s, k),
        View::Agents => screens::agents(s, k),
        View::Capture => screens::capture(s, k),
        View::Data => screens::data(s, k),
        View::Avatars => screens::avatar_gallery(s),
    };
    let main = col()
        .w_full()
        .h_full()
        .fill(theme::panel())
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .scroll_edge_fade(12.0)
        .child(
            col()
                .w_full()
                .px(theme::GUTTER)
                .py(24.0)
                .items_center()
                .child(
                    col()
                        .w_full()
                        .max_w(theme::CONTAINER)
                        .gap(24.0)
                        .child(content),
                ),
        );
    let mut toolbar =
        Toolbar::new().leading(text(s.view.label()).font(LABEL).color(theme::muted()));
    if let Some(project) = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.selected_project())
    {
        toolbar = toolbar.trailing(ui::badge(&project.name, k)).trailing(
            text(&project.root_path)
                .font(CODE_SM)
                .color(theme::faint())
                .truncate()
                .max_w(320.0),
        );
    }
    SplitView::new(main)
        .sidebar(
            &s.sidebar,
            |s: &mut CoderuneApp| &mut s.sidebar,
            sidebar(s, k),
        )
        .toolbar(toolbar)
        .titlebar_inset(cx.titlebar_inset())
        .toolbar_in_panel()
        .frame_inset(theme::FRAME)
        .reduced_motion(cx.is_reduced_motion())
        .build(k)
}

fn sidebar(s: &CoderuneApp, k: &Kit) -> El {
    let brand = row()
        .key("brand")
        .h(32.0)
        .px(8.0)
        .gap(10.0)
        .items_center()
        .child(
            row()
                .size(22.0, 22.0)
                .center()
                .radius(5.0)
                .fill(theme::ink())
                .child(text("C").font(LABEL).color(theme::inverse())),
        )
        .child(text("Coderune").font(HEADING_XS).color(theme::ink()));
    let mut nav = Sidebar::new()
        .label("Workspace")
        .inset(8.0)
        .header(brand)
        .child(rune::el().key("nav-gap").h(8.0).shrink());
    for view in View::NAV {
        nav = nav.child(ui::nav_item(
            nav_key(view),
            view.label(),
            view.icon(),
            s.view == view,
            move |s: &mut CoderuneApp| s.navigate(view),
        ));
    }
    let profile_label = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile())
        .map_or("No baseline".to_string(), |profile| {
            profile.display_name.clone()
        });
    let baseline = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile())
        .map(|profile| format!("Baseline v{}", profile.version))
        .unwrap_or_else(|| "Developer baseline".to_string());
    let footer = col()
        .key("sidebar-footer-content")
        .w_full()
        .gap(2.0)
        .pt(8.0)
        .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
        .child(
            row()
                .key("nav-profile-footer")
                .hit()
                .focusable(true)
                .role("button")
                .label(format!("Profile, {profile_label}"))
                .h(40.0)
                .w_full()
                .items_center()
                .gap(10.0)
                .px(8.0)
                .radius(6.0)
                .hover(theme::hover())
                .on_press(|s: &mut CoderuneApp| s.navigate(View::Profile))
                .child(
                    row()
                        .size(22.0, 22.0)
                        .center()
                        .radius(11.0)
                        .fill(theme::code())
                        .child(
                            text(profile_label.chars().next().unwrap_or('?').to_string())
                                .font(LABEL)
                                .color(theme::ink())
                                .a11y_hidden(),
                        )
                        .a11y_hidden(),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(&profile_label)
                                .font(LABEL)
                                .color(theme::ink())
                                .truncate()
                                .a11y_hidden(),
                        )
                        .child(
                            text(&baseline)
                                .font(LABEL_SM)
                                .color(theme::faint())
                                .truncate()
                                .a11y_hidden(),
                        ),
                ),
        )
        .child(ui::nav_item(
            "nav-data-footer",
            "Data",
            &icons::GEAR,
            s.view == View::Data,
            |s: &mut CoderuneApp| s.navigate(View::Data),
        ));
    nav.footer(footer).build(k)
}

fn nav_key(view: View) -> &'static str {
    match view {
        View::Home => "nav-home",
        View::Profile => "nav-profile",
        View::Projects => "nav-projects",
        View::Documents => "nav-documents",
        View::Agents => "nav-agents",
        View::Capture => "nav-capture",
        View::Data => "nav-data",
        View::Avatars => "nav-avatars",
    }
}

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

pub fn on_key(s: &mut CoderuneApp, event: &KeyEvent, _: &mut rune::Tx) -> bool {
    match event.key {
        Key::Command('q') => {
            std::process::exit(0);
        }
        Key::Command('1') => s.navigate(View::Home),
        Key::Command('2') => s.navigate(View::Profile),
        Key::Command('3') => s.navigate(View::Projects),
        Key::Command('4') => s.navigate(View::Documents),
        Key::Command('5') => s.navigate(View::Agents),
        Key::Command('6') => s.navigate(View::Capture),
        Key::Command('7') => s.navigate(View::Data),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    /// Tests that point the app at a temporary data folder must not overlap.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn profile_form_keeps_fields_it_does_not_edit() {
        let mut draft = ProfileDraft {
            display_name: "Ada".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust".to_string(),
            learning_goals: vec!["Ship".to_string()],
            self_reported_skills: vec![coderune_domain::SelfReportedSkill {
                topic: "Rust".to_string(),
                rating: Some(coderune_domain::SkillRating::Advanced),
            }],
            links: vec![coderune_domain::Link {
                label: "Site".to_string(),
                url: "https://example.com".to_string(),
            }],
            ..Default::default()
        };
        draft.display_name = "Ada".to_string();
        let profile = draft.into_profile(None).unwrap();
        let form = ProfileForm::from_value(Some(&profile), None);
        let round_trip = form.to_draft(ProfileDraft::from_profile(&profile));
        assert_eq!(round_trip.self_reported_skills.len(), 1);
        assert_eq!(round_trip.links.len(), 1);
    }

    #[test]
    fn project_form_keeps_fields_it_does_not_edit() {
        let root = std::env::temp_dir();
        let project = ProjectDraft {
            name: "Demo".to_string(),
            root_path: root.display().to_string(),
            goals: vec!["Ship".to_string()],
            purpose: Some("Purpose".to_string()),
            rules: vec!["Rule".to_string()],
            ..Default::default()
        }
        .into_project(None)
        .unwrap();
        let form = ProjectForm::from_project(&project);
        let round_trip = form.to_draft(Some(&project)).unwrap();
        assert_eq!(round_trip.purpose.as_deref(), Some("Purpose"));
        assert_eq!(round_trip.rules, vec!["Rule"]);
    }

    #[test]
    fn invalid_capture_limit_is_rejected() {
        let mut form = ProjectForm::empty(None);
        form.per_file_kib = edit("not-a-number");
        assert!(form.to_draft(None).is_err());
    }

    #[test]
    fn optional_profile_details_round_trip() {
        let draft = ProfileDraft {
            display_name: "Ada".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust".to_string(),
            learning_goals: vec!["Ship".to_string()],
            self_reported_skills: vec![SelfReportedSkill {
                topic: "Rust".to_string(),
                rating: Some(SkillRating::Advanced),
            }],
            education: vec![EducationEntry {
                institution: "University".to_string(),
                qualification: Some("BSc".to_string()),
                period: Some("2021-2025".to_string()),
                details: None,
            }],
            employment: vec![EmploymentEntry {
                employer: "Acme".to_string(),
                role: "Engineer".to_string(),
                period: None,
                details: None,
            }],
            projects: vec![ProjectExperience {
                name: "Coderune".to_string(),
                role: None,
                period: None,
                details: None,
            }],
            certifications: vec![Certification {
                name: "AWS".to_string(),
                issuer: Some("Amazon".to_string()),
                year: None,
            }],
            links: vec![Link {
                label: "Site".to_string(),
                url: "https://example.com".to_string(),
            }],
            learning_preferences: Some(LearningPreferences {
                formats: vec!["examples".to_string()],
                notes: Some("Small runnable examples".to_string()),
            }),
            ..Default::default()
        };
        let form = ProfileForm::from_draft(&draft);
        let round_trip = form.to_draft(ProfileDraft::default());
        assert_eq!(round_trip.self_reported_skills, draft.self_reported_skills);
        assert_eq!(round_trip.education, draft.education);
        assert_eq!(round_trip.employment, draft.employment);
        assert_eq!(round_trip.projects, draft.projects);
        assert_eq!(round_trip.certifications, draft.certifications);
        assert_eq!(round_trip.links, draft.links);
        assert_eq!(round_trip.learning_preferences, draft.learning_preferences);
    }

    #[test]
    fn onboarding_blocks_a_step_until_its_two_answers_exist() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        assert_eq!(state.onboarding_step, OnboardingStep::About);

        state.onboarding_next();
        assert_eq!(state.onboarding_step, OnboardingStep::About);
        assert!(state.error.is_some());

        state.profile_form.display_name = edit("Ada");
        state.profile_form.stage = Some(ExperienceStage::EarlyCareer);
        state.onboarding_next();
        assert_eq!(state.onboarding_step, OnboardingStep::StartingPoint);
        assert!(state.error.is_none());

        state.onboarding_next();
        assert_eq!(state.onboarding_step, OnboardingStep::StartingPoint);
        assert!(state.error.is_some());

        state.profile_form.new_to_coding = true;
        state.profile_form.goals = edit("Learn Rust");
        state.onboarding_next();
        assert_eq!(state.onboarding_step, OnboardingStep::Review);
        state.onboarding_back();
        assert_eq!(state.onboarding_step, OnboardingStep::StartingPoint);
    }

    #[test]
    fn onboarding_and_workspace_render_headlessly() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let env = Env {
            system_dark: false,
            scale: 1.0,
        };

        let mut onboarding = Runtime::new(CoderuneApp::init(), view, (1280.0, 860.0), env);
        onboarding.frame(0.0);
        assert!(onboarding.find("stage-chip-EarlyCareer").is_some());

        let mut workspace_state = CoderuneApp::init();
        seed_demo(&mut workspace_state);
        let mut workspace = Runtime::new(workspace_state, view, (1280.0, 860.0), env);
        workspace.frame(0.0);
        assert!(workspace.find("nav-capture").is_some());
    }
}
