//! Coderune's native desktop application.

use std::collections::BTreeMap;
use std::path::PathBuf;

use coderune_app::{PreviewSession, Workspace};
use coderune_domain::{
    AgentId, AppError, ContextCapture, DocumentKind, ExperienceStage, ProfileDraft, ProjectDraft,
};
use rune::{col, row, text, Cx, El, Key, KeyEvent, ScrollSpec, TextEdit};
use rune_kit::Kit;

use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};

use crate::screens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Profile,
    Projects,
    Documents,
    Agents,
    Capture,
    Data,
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
}

impl ProfileForm {
    pub(crate) fn from_value(
        profile: Option<&coderune_domain::DeveloperProfile>,
        draft: Option<&coderune_domain::OnboardingDraft>,
    ) -> Self {
        if let Some(profile) = profile {
            Self {
                display_name: edit(profile.display_name.clone()),
                skills: edit(profile.skills_statement.clone()),
                goals: edit(profile.learning_goals.join(", ")),
                stage: Some(profile.experience_stage),
                new_to_coding: profile.new_to_coding,
            }
        } else if let Some(draft) = draft {
            Self {
                display_name: edit(draft.draft.display_name.clone()),
                skills: edit(draft.draft.skills_statement.clone()),
                goals: edit(draft.draft.learning_goals.join(", ")),
                stage: draft.draft.experience_stage,
                new_to_coding: draft.draft.new_to_coding,
            }
        } else {
            Self {
                display_name: edit(""),
                skills: edit(""),
                goals: edit(""),
                stage: None,
                new_to_coding: false,
            }
        }
    }

    pub(crate) fn to_draft(&self) -> ProfileDraft {
        ProfileDraft {
            display_name: self.display_name.text.trim().to_string(),
            experience_stage: self.stage,
            new_to_coding: self.new_to_coding,
            skills_statement: self.skills.text.trim().to_string(),
            learning_goals: split_list(&self.goals.text),
            ..Default::default()
        }
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
    pub(crate) profile_form: ProfileForm,
    pub(crate) project_form: ProjectForm,
    pub(crate) document_form: DocumentForm,
    pub(crate) capture_session: Option<PreviewSession>,
    pub(crate) capture_reviewed: bool,
    pub(crate) capture_detail: Option<CaptureDetail>,
    pub(crate) confirm_delete_project: Option<String>,
    pub(crate) confirm_delete_capture: Option<String>,
    pub(crate) confirm_delete_all: bool,
    pub(crate) export_path: TextEdit,
    pub error: Option<AppError>,
    pub notice: Option<String>,
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
        let launch = std::env::args().nth(1).map(PathBuf::from);
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
        Self {
            workspace,
            fatal,
            view: View::Home,
            profile_form,
            project_form,
            document_form: DocumentForm::empty(),
            capture_session: None,
            capture_reviewed: false,
            capture_detail: None,
            confirm_delete_project: None,
            confirm_delete_capture: None,
            confirm_delete_all: false,
            export_path,
            error: None,
            notice: None,
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
    }

    pub(crate) fn save_draft(&mut self) {
        let draft = self.profile_form.to_draft();
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.save_onboarding_draft(draft) {
            Ok(()) => {
                self.notice = Some("Onboarding draft saved. You can close and resume.".to_string());
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn confirm_profile(&mut self) {
        let draft = self.profile_form.to_draft();
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.confirm_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!(
                    "Developer baseline v{} confirmed.",
                    profile.version
                ));
                self.error = None;
                self.view = View::Home;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_profile(&mut self) {
        let draft = self.profile_form.to_draft();
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
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(Some(project.id));
        }
    }

    pub(crate) fn new_project_form(&mut self) {
        let initial_root = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        self.project_form = ProjectForm::empty(initial_root);
        self.confirm_delete_project = None;
        self.error = None;
    }

    pub(crate) fn save_project(&mut self) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        let draft = ProjectDraft {
            name: self.project_form.name.text.trim().to_string(),
            root_path: self.project_form.root.text.trim().to_string(),
            goals: split_list(&self.project_form.goals.text),
            scope: coderune_domain::CollectionScope {
                include_paths: split_list(&self.project_form.include_paths.text),
                exclude_paths: split_list(&self.project_form.exclude_paths.text),
                respect_gitignore: self.project_form.respect_gitignore,
            },
            limits: coderune_domain::CaptureLimits {
                per_file_bytes: self
                    .project_form
                    .per_file_kib
                    .text
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .unwrap_or(1024)
                    * 1024,
                per_capture_bytes: self
                    .project_form
                    .per_capture_mib
                    .text
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .unwrap_or(20)
                    * 1024
                    * 1024,
            },
            ..Default::default()
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
        let Some(extracted) = self.document_form.extracted.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.save_document(
            self.document_form.kind,
            self.document_form.project_id.clone(),
            extracted.name.clone(),
            extracted.source_path.clone(),
            extracted.text.clone(),
            true,
        ) {
            Ok(document) => {
                self.notice = Some(format!(
                    "Saved {} import {}.",
                    document.kind.label(),
                    document.name
                ));
                self.error = None;
                self.document_form.extracted = None;
                self.document_form.selected_document = Some(document.id);
            }
            Err(error) => self.error = Some(error),
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
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn inspect_capture(&mut self, id: &str) {
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        let Some(project_id) = workspace.selected_project_id.clone() else {
            return;
        };
        let Ok(captures) = workspace.list_captures(&project_id) else {
            return;
        };
        let Some(capture) = captures.into_iter().find(|capture| capture.id == id) else {
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
                self.capture_session = None;
                self.capture_detail = None;
                self.refresh_profile_form();
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
        s.profile_form.goals = edit("Ship the native desktop app, Learn GPUI");
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

fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n', ';'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub fn view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    let k = theme::kit();
    let _ = cx;
    let body = if let Some(message) = &s.fatal {
        fatal(message)
    } else if s
        .workspace
        .as_ref()
        .is_some_and(|workspace| workspace.onboarding.can_start_project)
    {
        workspace(s, &k)
    } else {
        screens::onboarding(s, &k)
    };
    col().w_full().h_full().fill(theme::bg()).child(body)
}

fn fatal(message: &str) -> El {
    col()
        .w_full()
        .h_full()
        .center()
        .gap(12.0)
        .fill(theme::bg())
        .child(
            text("Coderune could not start")
                .font(HEADING)
                .color(theme::ink()),
        )
        .child(text(message).font(BODY).color(theme::muted()).wrap())
}

fn workspace(s: &CoderuneApp, k: &Kit) -> El {
    let content = match s.view {
        View::Home => screens::home(s, k),
        View::Profile => screens::profile(s, k),
        View::Projects => screens::projects(s, k),
        View::Documents => screens::documents(s, k),
        View::Agents => screens::agents(s, k),
        View::Capture => screens::capture(s, k),
        View::Data => screens::data(s, k),
    };
    row()
        .w_full()
        .h_full()
        .fill(theme::bg())
        .child(sidebar(s, k))
        .child(
            col().grow(1.0).h_full().min_h(0.0).child(topbar(s)).child(
                col()
                    .grow(1.0)
                    .min_h(0.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .padding(theme::GUTTER)
                    .items_center()
                    .child(
                        col()
                            .w_full()
                            .max_w(theme::CONTAINER)
                            .gap(24.0)
                            .child(content),
                    ),
            ),
        )
}

fn sidebar(s: &CoderuneApp, k: &Kit) -> El {
    let mut nav = Vec::new();
    for view in View::NAV {
        nav.push(ui::nav_item(
            match view {
                View::Home => "nav-home",
                View::Profile => "nav-profile",
                View::Projects => "nav-projects",
                View::Documents => "nav-documents",
                View::Agents => "nav-agents",
                View::Capture => "nav-capture",
                View::Data => "nav-data",
            },
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
        .map(|profile| profile.display_name.clone())
        .unwrap_or_else(|| "No baseline".to_string());
    let baseline = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile())
        .map(|profile| format!("Baseline v{}", profile.version))
        .unwrap_or_else(|| "Developer baseline".to_string());
    col()
        .w(theme::SIDEBAR)
        .h_full()
        .shrink()
        .padding(16.0)
        .gap(4.0)
        .fill(theme::panel())
        .border_sides([0.0, 1.0, 0.0, 0.0], theme::line())
        .child(
            row()
                .gap(12.0)
                .items_center()
                .mb(16.0)
                .child(
                    row()
                        .size(28.0, 28.0)
                        .center()
                        .radius(6.0)
                        .fill(theme::ink())
                        .child(text("C").font(LABEL).color(theme::inverse())),
                )
                .child(text("Coderune").font(HEADING_XS).color(theme::ink()))
                .child(ui::badge("native", k)),
        )
        .children(nav)
        .child(rune::el().grow(1.0))
        .child(ui::nav_item(
            "nav-data-footer",
            "Data",
            &icons::GEAR,
            s.view == View::Data,
            |s: &mut CoderuneApp| s.navigate(View::Data),
        ))
        .child(
            col()
                .gap(4.0)
                .mt(8.0)
                .pt(12.0)
                .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
                .child(text(&profile_label).font(LABEL).color(theme::ink()))
                .child(text(&baseline).font(LABEL_SM).color(theme::faint())),
        )
}

fn topbar(s: &CoderuneApp) -> El {
    let project = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.selected_project());
    let mut right = row().gap(8.0).items_center();
    if let Some(project) = project {
        right = right
            .child(ui::badge(&project.name, &theme::kit()))
            .child(text(&project.root_path).font(CODE_SM).color(theme::faint()));
    }
    row()
        .w_full()
        .h(theme::TOPBAR)
        .px(24.0)
        .items_center()
        .fill(theme::panel())
        .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line())
        .child(text(s.view.label()).font(LABEL).color(theme::muted()))
        .child(rune::el().grow(1.0))
        .child(right)
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

    #[test]
    fn onboarding_and_workspace_render_headlessly() {
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
