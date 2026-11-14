use std::collections::BTreeMap;
use std::path::PathBuf;

use coderune_app::{PreviewSession, Workspace};
use coderune_agents::DraftSharingPreview;
use coderune_domain::{
    AgentAvailability, AgentId, AppError, ContextCapture, DocumentKind, ExperienceStage,
    ProfileDraft, ProjectDraft,
};
use coderune_ui::components::{Button, ButtonVariant, TextInputState};
use coderune_ui::theme::{self, layout, radius, space, TextRole, Tone, TypeStyled};
use coderune_ui::{
    alert, badge, card, choice_chip, field, nav_item, page_header, section_title, text,
};
use gpui::{
    div, prelude::FluentBuilder, px, AppContext, Context, Div, Entity, FocusHandle,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled,
    Window,
};

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

    pub fn icon(self) -> &'static str {
        match self {
            Self::Home => "icons/house.svg",
            Self::Profile => "icons/user-circle.svg",
            Self::Projects => "icons/folder-simple.svg",
            Self::Documents => "icons/files.svg",
            Self::Agents => "icons/robot.svg",
            Self::Capture => "icons/scan.svg",
            Self::Data => "icons/gear.svg",
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

pub struct ProfileForm {
    pub display_name: Entity<TextInputState>,
    pub skills: Entity<TextInputState>,
    pub goals: Entity<TextInputState>,
    pub stage: Option<ExperienceStage>,
    pub new_to_coding: bool,
}

impl ProfileForm {
    fn empty(cx: &mut Context<CoderuneApp>) -> Self {
        Self {
            display_name: cx.new(|cx| TextInputState::single_line("Your name or pseudonym", cx)),
            skills: cx.new(|cx| TextInputState::single_line("What can you do today?", cx)),
            goals: cx.new(|cx| TextInputState::single_line("What do you want to learn? (comma separated)", cx)),
            stage: None,
            new_to_coding: false,
        }
    }

    fn from_value(
        profile: Option<&coderune_domain::DeveloperProfile>,
        draft: Option<&coderune_domain::OnboardingDraft>,
        cx: &mut Context<CoderuneApp>,
    ) -> Self {
        let mut form = Self::empty(cx);
        if let Some(profile) = profile {
            form.display_name = cx.new(|cx| TextInputState::new_with_value(profile.display_name.clone(), "Your name or pseudonym", cx));
            form.skills = cx.new(|cx| TextInputState::new_with_value(profile.skills_statement.clone(), "What can you do today?", cx));
            form.goals = cx.new(|cx| TextInputState::new_with_value(profile.learning_goals.join(", "), "What do you want to learn? (comma separated)", cx));
            form.stage = Some(profile.experience_stage);
            form.new_to_coding = profile.new_to_coding;
        } else if let Some(draft) = draft {
            if !draft.draft.display_name.is_empty() {
                form.display_name = cx.new(|cx| TextInputState::new_with_value(draft.draft.display_name.clone(), "Your name or pseudonym", cx));
            }
            if !draft.draft.skills_statement.is_empty() {
                form.skills = cx.new(|cx| TextInputState::new_with_value(draft.draft.skills_statement.clone(), "What can you do today?", cx));
            }
            if !draft.draft.learning_goals.is_empty() {
                form.goals = cx.new(|cx| TextInputState::new_with_value(draft.draft.learning_goals.join(", "), "What do you want to learn? (comma separated)", cx));
            }
            form.stage = draft.draft.experience_stage;
            form.new_to_coding = draft.draft.new_to_coding;
        }
        form
    }

    fn to_draft(&self, cx: &Context<CoderuneApp>) -> ProfileDraft {
        ProfileDraft {
            display_name: self.display_name.read(cx).text(),
            experience_stage: self.stage,
            new_to_coding: self.new_to_coding,
            skills_statement: self.skills.read(cx).text(),
            learning_goals: split_list(&self.goals.read(cx).text()),
            ..Default::default()
        }
    }
}

pub struct ProjectForm {
    pub name: Entity<TextInputState>,
    pub root: Entity<TextInputState>,
    pub goals: Entity<TextInputState>,
    pub editing: Option<String>,
}

impl ProjectForm {
    fn empty(initial_root: Option<PathBuf>, cx: &mut Context<CoderuneApp>) -> Self {
        let root = initial_root
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        Self {
            name: cx.new(|cx| TextInputState::single_line("Project name", cx)),
            root: cx.new(|cx| TextInputState::new_with_value(root, "Absolute path to the project folder", cx)),
            goals: cx.new(|cx| TextInputState::single_line("At least one goal, comma separated", cx)),
            editing: None,
        }
    }

    fn from_project(project: &coderune_domain::Project, cx: &mut Context<CoderuneApp>) -> Self {
        Self {
            name: cx.new(|cx| TextInputState::new_with_value(project.name.clone(), "Project name", cx)),
            root: cx.new(|cx| TextInputState::new_with_value(project.root_path.clone(), "Absolute path to the project folder", cx)),
            goals: cx.new(|cx| TextInputState::new_with_value(project.goals.iter().map(|goal| goal.text.clone()).collect::<Vec<_>>().join(", "), "At least one goal, comma separated", cx)),
            editing: Some(project.id.clone()),
        }
    }
}

#[derive(Clone)]
pub struct ExtractedDocument {
    name: String,
    text: String,
    bytes: usize,
    source_path: Option<String>,
}

pub struct DocumentForm {
    pub path: Entity<TextInputState>,
    pub kind: DocumentKind,
    pub project_id: Option<String>,
    pub extracted: Option<ExtractedDocument>,
    pub selected_document: Option<String>,
    pub agent: Option<AgentId>,
    pub draft_preview: Option<DraftSharingPreview>,
}

impl DocumentForm {
    fn empty(cx: &mut Context<CoderuneApp>) -> Self {
        Self {
            path: cx.new(|cx| TextInputState::single_line("Absolute path to a Markdown or text file", cx)),
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
    workspace: Option<Workspace>,
    fatal: Option<String>,
    pub view: View,
    profile_form: ProfileForm,
    project_form: ProjectForm,
    document_form: DocumentForm,
    capture_session: Option<PreviewSession>,
    capture_reviewed: bool,
    capture_detail: Option<CaptureDetail>,
    confirm_delete_capture: Option<String>,
    confirm_delete_project: Option<String>,
    pub error: Option<AppError>,
    pub notice: Option<String>,
    focus: FocusHandle,
}

impl CoderuneApp {
    pub fn new(launch: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let mut fatal = None;
        let mut workspace = match Workspace::open_default() {
            Ok(workspace) => Some(workspace),
            Err(error) => {
                fatal = Some(format!("Could not open local storage: {error}"));
                None
            }
        };
        if let (Some(workspace), Some(directory)) = (workspace.as_mut(), launch) {
            if let Err(error) = workspace.adopt_launch_directory(directory) {
                fatal = Some(format!("Could not inspect the launch directory: {error}"));
            }
        }
        let profile_form = match workspace.as_ref() {
            Some(workspace) => ProfileForm::from_value(
                workspace.onboarding.profile.as_ref(),
                workspace.onboarding.draft.as_ref(),
                cx,
            ),
            None => ProfileForm::empty(cx),
        };
        let initial_root = workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        let project_form = match workspace.as_ref().and_then(|workspace| workspace.selected_project()) {
            Some(project) => ProjectForm::from_project(project, cx),
            None => ProjectForm::empty(initial_root, cx),
        };
        let document_form = DocumentForm::empty(cx);
        Self {
            workspace,
            fatal,
            view: View::Home,
            profile_form,
            project_form,
            document_form,
            capture_session: None,
            capture_reviewed: false,
            capture_detail: None,
            confirm_delete_capture: None,
            confirm_delete_project: None,
            error: None,
            notice: None,
            focus: cx.focus_handle(),
        }
    }

    fn refresh_profile_form(&mut self, cx: &mut Context<Self>) {
        if let Some(workspace) = self.workspace.as_ref() {
            self.profile_form = ProfileForm::from_value(
                workspace.onboarding.profile.as_ref(),
                workspace.onboarding.draft.as_ref(),
                cx,
            );
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.error = None;
        if let Some(workspace) = self.workspace.as_mut() {
            if let Err(error) = workspace.reload() {
                self.error = Some(error);
            }
        }
        self.refresh_profile_form(cx);
    }

    fn preview_capture(&mut self, cx: &mut Context<Self>) {
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
        let Some(workspace) = self.workspace.as_ref() else { return };
        match workspace.preview_capture(&project.id, project.scope.clone(), project.limits.clone()) {
            Ok(session) => {
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some("Preview collected. Review the exact file list before saving.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn refresh_capture(&mut self, cx: &mut Context<Self>) {
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
        let Some(workspace) = self.workspace.as_ref() else { return };
        match workspace.refresh_capture(&project_id) {
            Ok((latest, session)) => {
                let unchanged = latest.is_some();
                self.capture_session = Some(session);
                self.capture_reviewed = false;
                self.capture_detail = None;
                self.error = None;
                self.notice = Some(if unchanged {
                    "Repository unchanged. The latest capture already matches this state.".to_string()
                } else {
                    "Changes found. A fresh preview is ready for review.".to_string()
                });
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn save_capture(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.capture_session.clone() else { return };
        let Some(workspace) = self.workspace.as_mut() else { return };
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

    fn inspect_capture(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.as_ref() else { return };
        let Some(project_id) = workspace.selected_project_id.clone() else { return };
        let Ok(captures) = workspace.list_captures(&project_id) else { return };
        let Some(capture) = captures.into_iter().find(|capture| capture.id == id) else { return };
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

    fn delete_capture(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.delete_capture(id) {
            Ok(()) => {
                self.notice = Some("Context capture deleted.".to_string());
                self.error = None;
                if self.capture_detail.as_ref().is_some_and(|detail| detail.capture.id == id) {
                    self.capture_detail = None;
                }
                self.confirm_delete_capture = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn extract_document(&mut self, cx: &mut Context<Self>) {
        let path = self.document_form.path.read(cx).text();
        if path.trim().is_empty() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadableDocument,
                "Choose a document path first.",
            ));
            return;
        }
        let Some(workspace) = self.workspace.as_ref() else { return };
        match workspace.extract_document(std::path::PathBuf::from(&path), 20 * 1024 * 1024) {
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

    fn save_document_import(&mut self, cx: &mut Context<Self>) {
        let Some(extracted) = self.document_form.extracted.clone() else {
            return;
        };
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.save_document(
            self.document_form.kind,
            self.document_form.project_id.clone(),
            extracted.name.clone(),
            extracted.source_path.clone(),
            extracted.text.clone(),
            true,
        ) {
            Ok(document) => {
                self.notice = Some(format!("Saved {} import {}.", document.kind.label(), document.name));
                self.error = None;
                self.document_form.extracted = None;
                self.document_form.selected_document = Some(document.id);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn prepare_draft(&mut self, cx: &mut Context<Self>) {
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
        let Some(workspace) = self.workspace.as_ref() else { return };
        match workspace.prepare_draft(agent, &document_id) {
            Ok(preview) => {
                self.document_form.draft_preview = Some(preview);
                self.error = None;
                self.notice = Some("Sharing preview prepared. No agent process has started.".to_string());
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn load_project_form(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.projects.iter().find(|project| project.id == id))
            .cloned()
        else {
            return;
        };
        self.project_form = ProjectForm::from_project(&project, cx);
        self.confirm_delete_project = None;
        self.error = None;
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(Some(project.id));
        }
    }

    fn new_project_form(&mut self, cx: &mut Context<Self>) {
        let initial_root = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        self.project_form = ProjectForm::empty(initial_root, cx);
        self.confirm_delete_project = None;
        self.error = None;
    }

    fn save_project(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.as_mut() else { return };
        let existing = self
            .project_form
            .editing
            .as_deref()
            .and_then(|id| workspace.projects.iter().find(|project| project.id == id))
            .cloned();
        let draft = ProjectDraft {
            name: self.project_form.name.read(cx).text(),
            root_path: self.project_form.root.read(cx).text(),
            goals: split_list(&self.project_form.goals.read(cx).text()),
            scope: existing
                .as_ref()
                .map(|project| project.scope.clone())
                .unwrap_or_default(),
            limits: existing
                .as_ref()
                .map(|project| project.limits.clone())
                .unwrap_or_default(),
            ..Default::default()
        };
        let result = match self.project_form.editing.as_deref() {
            Some(id) => workspace.update_project(id, draft),
            None => workspace.create_project(draft),
        };
        match result {
            Ok(project) => {
                self.notice = Some(format!("Project {} saved at version {}.", project.name, project.version));
                self.error = None;
                self.project_form = ProjectForm::from_project(&project, cx);
                self.confirm_delete_project = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn delete_project(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.confirm_delete_project.clone() else { return };
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.delete_project(&id) {
            Ok(()) => {
                self.notice = Some("Project and its captures deleted.".to_string());
                self.error = None;
                self.new_project_form(cx);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn save_profile(&mut self, cx: &mut Context<Self>) {
        let draft = self.profile_form.to_draft(cx);
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.update_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!("Profile v{} saved.", profile.version));
                self.error = None;
                self.refresh_profile_form(cx);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn save_draft(&mut self, cx: &mut Context<Self>) {
        let draft = self.profile_form.to_draft(cx);
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.save_onboarding_draft(draft) {
            Ok(()) => {
                self.notice = Some("Onboarding draft saved. You can close and resume.".to_string());
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn confirm_profile(&mut self, cx: &mut Context<Self>) {
        let draft = self.profile_form.to_draft(cx);
        let Some(workspace) = self.workspace.as_mut() else { return };
        match workspace.confirm_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!("Developer baseline v{} confirmed.", profile.version));
                self.error = None;
                self.view = View::Home;
                self.refresh_profile_form(cx);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn navigate(&mut self, view: View, cx: &mut Context<Self>) {
        self.view = view;
        self.error = None;
        cx.notify();
    }

    fn render_fatal(&self, message: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap(px(space::S3))
            .bg(theme::bg())
            .child(text("Coderune could not start", TextRole::Heading, Tone::Ink))
            .child(text(message, TextRole::Body, Tone::Muted))
    }

    fn render_onboarding(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let stage_row = ExperienceStage::ALL
            .iter()
            .map(|stage| {
                choice_chip(
                    SharedString::from(format!("stage-{:?}", stage)),
                    stage.label(),
                    self.profile_form.stage == Some(*stage),
                    cx.listener(move |this, _, _, cx| {
                        this.profile_form.stage = Some(*stage);
                        cx.notify();
                    }),
                )
            })
            .collect::<Vec<_>>();

        let mut form_card = card(
            None,
            None,
            div()
                .flex()
                .flex_col()
                .gap(px(space::S5))
                .child(field(
                    "Display name",
                    Some("A name or pseudonym. This is the start of your developer baseline."),
                    self.profile_form.display_name.clone(),
                ))
                .child(field(
                    "Experience stage",
                    Some("Self-reported. It is never treated as verified mastery."),
                    div().flex().flex_wrap().gap(px(space::S2)).children(stage_row),
                ))
                .child(field(
                    "Initial skills statement",
                    Some("A short honest statement. New to coding is a valid answer."),
                    self.profile_form.skills.clone(),
                ))
                .child(field(
                    "Learning goals",
                    Some("At least one. Separate several goals with commas."),
                    self.profile_form.goals.clone(),
                ))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(space::S2))
                        .child(choice_chip(
                            "new-to-coding",
                            "I am new to coding",
                            self.profile_form.new_to_coding,
                            cx.listener(|this, _, _, cx| {
                                this.profile_form.new_to_coding = !this.profile_form.new_to_coding;
                                cx.notify();
                            }),
                        ))
                        .child(text(
                            "This is recorded as a self-reported baseline, not a verified skill level.",
                            TextRole::BodySm,
                            Tone::Muted,
                        )),
                ),
        );
        if let Some(error) = &self.error {
            form_card = form_card.child(alert(Some(error.code.as_str()), &error.message));
        }

        div()
            .id("onboarding-scroll")
            .flex()
            .flex_col()
            .items_center()
            .size_full()
            .overflow_y_scroll()
            .bg(theme::bg())
            .p(px(space::S8))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(px(720.0))
                    .gap(px(space::S6))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(space::S3))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(32.0))
                                    .rounded(px(radius::SOFT))
                                    .bg(theme::ink())
                                    .type_role(TextRole::HeadingXs)
                                    .text_color(theme::inverse())
                                    .child("C"),
                            )
                            .child(text("Coderune", TextRole::HeadingXs, Tone::Ink)),
                    )
                    .child(page_header(
                        "Set up your developer baseline",
                        "Coderune keeps local, reviewable context about you and your projects. Start with a short baseline. You can edit it later.",
                        div(),
                    ))
                    .child(form_card)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(space::S2))
                            .child(Button::new("save-draft", "Save draft", ButtonVariant::Ghost).on_click(
                                cx.listener(|this, _, _, cx| this.save_draft(cx)),
                            ))
                            .child(
                                Button::new("confirm-baseline", "Confirm baseline", ButtonVariant::Primary)
                                    .on_click(cx.listener(|this, _, _, cx| this.confirm_profile(cx))),
                            ),
                    ),
            )
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let nav = View::NAV
            .iter()
            .map(|view| {
                let view = *view;
                nav_item(
                    SharedString::from(format!("nav-{:?}", view)),
                    view.label(),
                    view.icon(),
                    self.view == view,
                    cx.listener(move |this, _, _, cx| this.navigate(view, cx)),
                )
            })
            .collect::<Vec<_>>();
        let profile_label = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.profile())
            .map(|profile| profile.display_name.clone())
            .unwrap_or_else(|| "No baseline".to_string());
        div()
            .flex()
            .flex_col()
            .w(px(layout::SIDEBAR))
            .h_full()
            .flex_shrink_0()
            .p(px(space::S4))
            .gap(px(space::S1))
            .bg(theme::panel())
            .border_r_1()
            .border_color(theme::line())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(space::S3))
                    .mb(px(space::S4))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.0))
                            .rounded(px(radius::SOFT))
                            .bg(theme::ink())
                            .type_role(TextRole::Label)
                            .text_color(theme::inverse())
                            .child("C"),
                    )
                    .child(text("Coderune", TextRole::HeadingXs, Tone::Ink))
                    .child(badge("native")),
            )
            .children(nav)
            .child(div().flex_1())
            .child(nav_item(
                "nav-data",
                "Data",
                View::Data.icon(),
                self.view == View::Data,
                cx.listener(|this, _, _, cx| this.navigate(View::Data, cx)),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S1))
                    .mt(px(space::S2))
                    .pt(px(space::S3))
                    .border_t_1()
                    .border_color(theme::row_line())
                    .child(text(&profile_label, TextRole::Label, Tone::Ink))
                    .child(text(
                        self.workspace
                            .as_ref()
                            .and_then(|workspace| workspace.profile())
                            .map(|profile| format!("Baseline v{}", profile.version))
                            .unwrap_or_else(|| "Developer baseline".to_string()),
                        TextRole::LabelSm,
                        Tone::Faint,
                    )),
            )
    }

    fn render_topbar(&self) -> Div {
        let project = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.selected_project());
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(layout::TOPBAR))
            .w_full()
            .px(px(space::S6))
            .border_b_1()
            .border_color(theme::row_line())
            .bg(theme::panel())
            .child(text(self.view.label(), TextRole::Label, Tone::Muted))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(space::S2))
                    .when_some(project, |this, project| {
                        this.child(badge(&project.name)).child(text(
                            &project.root_path,
                            TextRole::CodeSm,
                            Tone::Faint,
                        ))
                    }),
            )
    }

    fn render_home(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.as_ref();
        let profile = workspace.and_then(|workspace| workspace.profile());
        let projects = workspace.map(|workspace| workspace.projects.len()).unwrap_or(0);
        let captures = workspace
            .and_then(|workspace| workspace.selected_project())
            .and_then(|project| workspace.and_then(|workspace| workspace.list_captures(&project.id).ok()))
            .map(|captures| captures.len())
            .unwrap_or(0);

        let mut body = div().flex().flex_col().gap(px(space::S6));
        body = body.child(page_header(
            "Workspace",
            "Local context about you and the projects you work on. Nothing leaves this machine unless you start an explicit agent draft.",
            Button::new("go-projects", "Open projects", ButtonVariant::Secondary)
                .on_click(cx.listener(|this, _, _, cx| this.navigate(View::Projects, cx))),
        ));
        body = body.child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(space::S4))
                .child(div().flex_1().min_w(px(240.0)).child(card(
                    Some("Developer baseline"),
                    Some("Starting description used for later progress."),
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::S2))
                        .child(text(
                            profile.map(|profile| profile.display_name.as_str()).unwrap_or("Not set"),
                            TextRole::HeadingXs,
                            Tone::Ink,
                        ))
                        .child(text(
                            profile
                                .map(|profile| format!("v{} · {}", profile.version, profile.experience_stage.label()))
                                .unwrap_or_else(|| "Complete onboarding first".to_string()),
                            TextRole::BodySm,
                            Tone::Muted,
                        )),
                )))
                .child(div().flex_1().min_w(px(240.0)).child(card(
                    Some("Projects"),
                    Some("Project context and repository scope."),
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::S2))
                        .child(text(projects.to_string(), TextRole::Heading, Tone::Ink))
                        .child(text(
                            if projects == 1 { "project configured" } else { "projects configured" },
                            TextRole::BodySm,
                            Tone::Muted,
                        )),
                )))
                .child(div().flex_1().min_w(px(240.0)).child(card(
                    Some("Context captures"),
                    Some("Reviewed repository snapshots for the selected project."),
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(space::S2))
                        .child(text(captures.to_string(), TextRole::Heading, Tone::Ink))
                        .child(text("saved captures", TextRole::BodySm, Tone::Muted)),
                ))),
        );

        if let Some(project) = workspace.and_then(|workspace| workspace.selected_project()) {
            let editor_row = div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(space::S2))
                .child(Button::new("home-open-folder", "Open folder", ButtonVariant::Secondary).on_click({
                    let path = PathBuf::from(&project.root_path);
                    cx.listener(move |this, _, _, _| {
                        if let Err(error) = Workspace::open_directory(&path) {
                            this.error = Some(error);
                        }
                    })
                }))
                .child(Button::new("home-launch-code", "Open in VS Code", ButtonVariant::Ghost).on_click({
                    let path = PathBuf::from(&project.root_path);
                    cx.listener(move |this, _, _, _| {
                        if let Err(error) = Workspace::launch_editor("code", &path) {
                            this.error = Some(error);
                        }
                    })
                }));
            body = body.child(card(
                Some(&project.name),
                Some("Active project context."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(text(&project.root_path, TextRole::Code, Tone::Muted))
                    .child(text(
                        format!("{} goal(s) · version {}", project.goals.len(), project.version),
                        TextRole::BodySm,
                        Tone::Faint,
                    ))
                    .child(editor_row),
            ));
        } else {
            body = body.child(card(
                Some("No project yet"),
                Some("Create a project to capture repository context."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(text(
                        "Choose a folder, name the project, and add at least one goal.",
                        TextRole::Body,
                        Tone::Muted,
                    ))
                    .child(
                        Button::new("home-create-project", "Create project", ButtonVariant::Primary)
                            .on_click(cx.listener(|this, _, _, cx| this.navigate(View::Projects, cx))),
                    ),
            ));
        }

        if let Some(notice) = &self.notice {
            body = body.child(alert(None, notice));
        }
        if let Some(error) = &self.error {
            body = body.child(alert(Some(error.code.as_str()), &error.message));
        }
        body
    }

    fn render_profile(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let profile = self.workspace.as_ref().and_then(|workspace| workspace.profile());
        let stage_row = ExperienceStage::ALL
            .iter()
            .map(|stage| {
                choice_chip(
                    SharedString::from(format!("profile-stage-{:?}", stage)),
                    stage.label(),
                    self.profile_form.stage == Some(*stage),
                    cx.listener(move |this, _, _, cx| {
                        this.profile_form.stage = Some(*stage);
                        cx.notify();
                    }),
                )
            })
            .collect::<Vec<_>>();
        let current = profile.map(|profile| {
            card(
                Some("Current baseline"),
                Some("Edits create a new immutable version. Existing captures keep their snapshot."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(space::S2))
                            .child(text(&profile.display_name, TextRole::HeadingXs, Tone::Ink))
                            .child(badge(&format!("v{}", profile.version)))
                            .child(badge(profile.experience_stage.label())),
                    )
                    .child(field("Skills", None, text(&profile.skills_statement, TextRole::Body, Tone::Muted)))
                    .child(field(
                        "Learning goals",
                        None,
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(space::S1))
                            .children(profile.learning_goals.iter().map(|goal| {
                                text(format!("· {goal}"), TextRole::BodySm, Tone::Muted)
                            })),
                    ))
                    .child(text(
                        format!(
                            "{} self-reported skill(s) · {} education entr(ies) · {} employment entr(ies)",
                            profile.self_reported_skills.len(),
                            profile.education.len(),
                            profile.employment.len()
                        ),
                        TextRole::BodySm,
                        Tone::Faint,
                    )),
            )
        });
        let mut form = card(
            Some("Edit profile"),
            Some("Required fields stay required. Self-reported skills are never presented as verified mastery."),
            div()
                .flex()
                .flex_col()
                .gap(px(space::S5))
                .child(field("Display name", Some("A name or pseudonym."), self.profile_form.display_name.clone()))
                .child(field(
                    "Experience stage",
                    Some("Self-reported baseline."),
                    div().flex().flex_wrap().gap(px(space::S2)).children(stage_row),
                ))
                .child(field(
                    "Initial skills statement",
                    Some("New to coding is a valid answer."),
                    self.profile_form.skills.clone(),
                ))
                .child(field(
                    "Learning goals",
                    Some("Separate several goals with commas."),
                    self.profile_form.goals.clone(),
                ))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(px(space::S2))
                        .child(choice_chip(
                            "profile-new-to-coding",
                            "I am new to coding",
                            self.profile_form.new_to_coding,
                            cx.listener(|this, _, _, cx| {
                                this.profile_form.new_to_coding = !this.profile_form.new_to_coding;
                                cx.notify();
                            }),
                        ))
                        .child(
                            Button::new("save-profile", "Save new version", ButtonVariant::Primary)
                                .on_click(cx.listener(|this, _, _, cx| this.save_profile(cx))),
                        ),
                ),
        );
        if let Some(error) = &self.error {
            form = form.child(alert(Some(error.code.as_str()), &error.message));
        }
        if let Some(notice) = &self.notice {
            form = form.child(alert(None, notice));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(space::S6))
            .child(page_header(
                "Profile",
                "The reusable developer profile shared across every project.",
                div(),
            ))
            .when_some(current, |this, current| this.child(current))
            .child(form)
    }

    fn render_projects(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.as_ref();
        let projects = workspace.map(|workspace| workspace.projects.clone()).unwrap_or_default();
        let selected = workspace.and_then(|workspace| workspace.selected_project_id.clone());
        let rows = projects.iter().map(|project| {
            let active = selected.as_deref() == Some(project.id.as_str());
            let id = project.id.clone();
            div()
                .id(SharedString::from(format!("project-{}", project.id)))
                .flex()
                .flex_col()
                .gap(px(space::S1))
                .w_full()
                .p(px(space::S3))
                .rounded(px(radius::INNER))
                .cursor(gpui::CursorStyle::PointingHand)
                .when(active, |this| this.bg(theme::accent_soft()))
                .when(!active, |this| this.hover(|style| style.bg(theme::hover())))
                .on_click(cx.listener(move |this, _, _, cx| this.load_project_form(&id, cx)))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(space::S2))
                        .child(text(&project.name, TextRole::Label, Tone::Ink))
                        .child(badge(&format!("v{}", project.version))),
                )
                .child(text(&project.root_path, TextRole::CodeSm, Tone::Faint))
        }).collect::<Vec<_>>();

        let editing = self.project_form.editing.is_some();
        let confirm_delete = self.confirm_delete_project.is_some();
        let mut form = card(
            Some(if editing { "Edit project" } else { "New project" }),
            Some("A project needs a name, a readable root folder, and at least one goal."),
            div()
                .flex()
                .flex_col()
                .gap(px(space::S5))
                .child(field("Project name", None, self.project_form.name.clone()))
                .child(field("Root folder", Some("Absolute path. The folder must exist and be readable."), self.project_form.root.clone()))
                .child(field("Goals", Some("At least one. Separate several goals with commas."), self.project_form.goals.clone()))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(space::S2))
                        .child(
                            Button::new(
                                "save-project",
                                if editing { "Save new version" } else { "Create project" },
                                ButtonVariant::Primary,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.save_project(cx))),
                        )
                        .when(editing, |this| {
                            this.child(
                                Button::new(
                                    "delete-project",
                                    if confirm_delete { "Confirm delete" } else { "Delete project" },
                                    ButtonVariant::Destructive,
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if this.confirm_delete_project.is_some() {
                                        this.delete_project(cx);
                                    } else {
                                        this.confirm_delete_project = this.project_form.editing.clone();
                                        cx.notify();
                                    }
                                })),
                            )
                        })
                        .when(!editing, |this| {
                            this.child(
                                Button::new("blank-project", "Clear", ButtonVariant::Ghost)
                                    .on_click(cx.listener(|this, _, _, cx| this.new_project_form(cx))),
                            )
                        }),
                ),
        );
        if confirm_delete {
            form = form.child(alert(
                None,
                "Deleting a project also deletes its captures and project-linked documents. Click Confirm delete to proceed.",
            ));
        }
        if let Some(error) = &self.error {
            form = form.child(alert(Some(error.code.as_str()), &error.message));
        }
        if let Some(notice) = &self.notice {
            form = form.child(alert(None, notice));
        }

        div()
            .flex()
            .flex_col()
            .gap(px(space::S6))
            .child(page_header(
                "Projects",
                "Project context is stored locally and reused on later captures.",
                Button::new("new-project", "New project", ButtonVariant::Secondary)
                    .on_click(cx.listener(|this, _, _, cx| this.new_project_form(cx))),
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(space::S4))
                    .items_start()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(space::S1))
                            .w(px(300.0))
                            .flex_shrink_0()
                            .child(section_title("Configured", "Select a project to edit."))
                            .when(rows.is_empty(), |this| {
                                this.child(card(None, None, text(
                                    "No projects yet. Create one on the right.",
                                    TextRole::BodySm,
                                    Tone::Muted,
                                )))
                            })
                            .children(rows),
                    )
                    .child(div().flex_1().min_w(px(360.0)).child(form)),
            )
    }

    fn render_documents(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.as_ref();
        let documents = workspace.map(|workspace| workspace.documents.clone()).unwrap_or_default();
        let projects = workspace.map(|workspace| workspace.projects.clone()).unwrap_or_default();
        let agents = workspace.map(|workspace| workspace.agents.clone()).unwrap_or_default();

        let kind_chips = [DocumentKind::Cv, DocumentKind::Srs]
            .into_iter()
            .map(|kind| {
                choice_chip(
                    SharedString::from(format!("doc-kind-{:?}", kind)),
                    kind.label(),
                    self.document_form.kind == kind,
                    cx.listener(move |this, _, _, cx| {
                        this.document_form.kind = kind;
                        this.document_form.draft_preview = None;
                        cx.notify();
                    }),
                )
            })
            .collect::<Vec<_>>();

        let project_chips = projects
            .iter()
            .map(|project| {
                choice_chip(
                    SharedString::from(format!("doc-project-{}", project.id)),
                    &project.name,
                    self.document_form.project_id.as_deref() == Some(project.id.as_str()),
                    cx.listener({
                        let id = project.id.clone();
                        move |this, _, _, cx| {
                            this.document_form.project_id = Some(id.clone());
                            this.document_form.draft_preview = None;
                            cx.notify();
                        }
                    }),
                )
            })
            .collect::<Vec<_>>();

        let mut import_card = card(
            Some("Import a document"),
            Some("Extraction is local. The original file is not copied into application storage."),
            div()
                .flex()
                .flex_col()
                .gap(px(space::S4))
                .child(field(
                    "Document path",
                    Some("Markdown and plain text are supported in this build. Use manual entry for PDF or DOCX."),
                    self.document_form.path.clone(),
                ))
                .child(field("Kind", None, div().flex().flex_row().gap(px(space::S2)).children(kind_chips)))
                .when(self.document_form.kind == DocumentKind::Srs, |this| {
                    this.child(field(
                        "Project",
                        Some("An SRS belongs to one project."),
                        div().flex().flex_wrap().gap(px(space::S2)).children(project_chips),
                    ))
                })
                .child(
                    Button::new("extract-document", "Extract text", ButtonVariant::Secondary)
                        .on_click(cx.listener(|this, _, _, cx| this.extract_document(cx))),
                ),
        );
        if let Some(extracted) = &self.document_form.extracted {
            import_card = import_card.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .pt(px(space::S3))
                    .border_t_1()
                    .border_color(theme::row_line())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(space::S2))
                            .child(text(&extracted.name, TextRole::Label, Tone::Ink))
                            .child(badge(&format!("{} bytes", extracted.bytes)))
                    )
                    .child(
                        div()
                            .id("extracted-text")
                            .max_h(px(layout::FIELD_MAX_HEIGHT))
                            .overflow_y_scroll()
                            .bg(theme::code())
                            .rounded(px(radius::SOFT))
                            .p(px(space::S3))
                            .type_role(TextRole::Code)
                            .text_color(theme::ink())
                            .child(extracted.text.clone()),
                    )
                    .child(
                        Button::new("save-document", "Save reviewed import", ButtonVariant::Primary)
                            .on_click(cx.listener(|this, _, _, cx| this.save_document_import(cx))),
                    ),
            );
        }

        let document_rows = documents
            .iter()
            .map(|document| {
                let selected = self.document_form.selected_document.as_deref() == Some(document.id.as_str());
                let id = document.id.clone();
                let project_name = document
                    .project_id
                    .as_deref()
                    .and_then(|id| projects.iter().find(|project| project.id == id))
                    .map(|project| project.name.as_str())
                    .unwrap_or("No project");
                div()
                    .id(SharedString::from(format!("document-{}", document.id)))
                    .flex()
                    .flex_col()
                    .gap(px(space::S1))
                    .w_full()
                    .p(px(space::S3))
                    .rounded(px(radius::INNER))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .when(selected, |this| this.bg(theme::accent_soft()))
                    .when(!selected, |this| this.hover(|style| style.bg(theme::hover())))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.document_form.selected_document = Some(id.clone());
                        this.document_form.draft_preview = None;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(space::S2))
                            .child(text(&document.name, TextRole::Label, Tone::Ink))
                            .child(badge(document.kind.label()))
                            .child(badge(&format!("{} bytes", document.text_bytes))),
                    )
                    .child(text(
                        format!("{project_name} · {}", &document.sha256[..8.min(document.sha256.len())]),
                        TextRole::CodeSm,
                        Tone::Faint,
                    ))
            })
            .collect::<Vec<_>>();

        let selected_document = self
            .document_form
            .selected_document
            .as_deref()
            .and_then(|id| documents.iter().find(|document| document.id == id))
            .cloned();

        let agent_chips = agents
            .iter()
            .map(|status| {
                let id = status.id;
                let enabled = status.availability == AgentAvailability::Ready;
                choice_chip(
                    SharedString::from(format!("draft-agent-{:?}", status.id)),
                    status.id.label(),
                    self.document_form.agent == Some(status.id),
                    cx.listener(move |this, _, _, cx| {
                        if enabled {
                            this.document_form.agent = Some(id);
                            this.document_form.draft_preview = None;
                            cx.notify();
                        }
                    }),
                )
            })
            .collect::<Vec<_>>();

        let mut draft_card = card(
            Some("Agent-assisted draft"),
            Some("Coderune shares only the exact text you preview. Sign-in, providers, and models stay with the agent."),
            div()
                .flex()
                .flex_col()
                .gap(px(space::S3))
                .when_some(selected_document.clone(), |this, document| {
                    this.child(text(
                        format!("Selected: {} ({})", document.name, document.kind.label()),
                        TextRole::BodySm,
                        Tone::Muted,
                    ))
                })
                .when(selected_document.is_none(), |this| {
                    this.child(text(
                        "Select an imported document from the list below.",
                        TextRole::BodySm,
                        Tone::Muted,
                    ))
                })
                .child(div().flex().flex_wrap().gap(px(space::S2)).children(agent_chips))
                .child(
                    Button::new("prepare-draft", "Prepare sharing preview", ButtonVariant::Secondary)
                        .disabled(selected_document.is_none())
                        .on_click(cx.listener(|this, _, _, cx| this.prepare_draft(cx))),
                ),
        );
        if let Some(preview) = &self.document_form.draft_preview {
            draft_card = draft_card.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S2))
                    .child(text(
                        format!(
                            "{} bytes · limit {} bytes · {}",
                            preview.estimated_bytes,
                            preview.max_input_bytes,
                            if preview.fits { "fits" } else { "too large" }
                        ),
                        TextRole::BodySm,
                        if preview.fits { Tone::Muted } else { Tone::Danger },
                    ))
                    .child(
                        div()
                            .id("draft-preview")
                            .max_h(px(layout::FIELD_MAX_HEIGHT))
                            .overflow_y_scroll()
                            .bg(theme::code())
                            .rounded(px(radius::SOFT))
                            .p(px(space::S3))
                            .type_role(TextRole::Code)
                            .text_color(theme::ink())
                            .child(preview.text.clone()),
                    )
                    .child(text(
                        "Draft execution is enabled for Claude Code in the service layer. Review the exact text before requesting.",
                        TextRole::BodySm,
                        Tone::Faint,
                    )),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap(px(space::S6))
            .child(page_header(
                "Documents",
                "Import CV and SRS source documents, review the extracted text, and prepare agent drafts.",
                div(),
            ))
            .child(import_card)
            .child(card(
                Some("Imported documents"),
                Some("Approved text retained locally. Select one to prepare a draft."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S1))
                    .when(document_rows.is_empty(), |this| {
                        this.child(text("No imported documents yet.", TextRole::BodySm, Tone::Muted))
                    })
                    .children(document_rows),
            ))
            .child(draft_card)
    }

    fn render_capture(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.as_ref();
        let project = workspace.and_then(|workspace| workspace.selected_project()).cloned();
        let captures = project
            .as_ref()
            .and_then(|project| workspace.and_then(|workspace| workspace.list_captures(&project.id).ok()))
            .unwrap_or_default();

        let mut body = div().flex().flex_col().gap(px(space::S6));
        body = body.child(page_header(
            "Capture",
            "Collect repository context only on explicit request. Nothing is saved until you review and confirm the exact file list.",
            div()
                .flex()
                .flex_row()
                .gap(px(space::S2))
                .child(
                    Button::new("preview-capture", "Preview capture", ButtonVariant::Primary)
                        .disabled(project.is_none())
                        .on_click(cx.listener(|this, _, _, cx| this.preview_capture(cx))),
                )
                .child(
                    Button::new("refresh-capture", "Refresh", ButtonVariant::Secondary)
                        .disabled(project.is_none())
                        .on_click(cx.listener(|this, _, _, cx| this.refresh_capture(cx))),
                ),
        ));

        if let Some(project) = &project {
            body = body.child(card(
                Some(&project.name),
                Some("Effective scope and limits for the next preview."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S2))
                    .child(text(&project.root_path, TextRole::Code, Tone::Muted))
                    .child(text(
                        format!(
                            "Includes: {} · Excludes: {} · .gitignore: {} · per file: {} KiB · per capture: {} MiB",
                            if project.scope.include_paths.is_empty() { "whole repository".to_string() } else { project.scope.include_paths.join(", ") },
                            if project.scope.exclude_paths.is_empty() { "none".to_string() } else { project.scope.exclude_paths.join(", ") },
                            if project.scope.respect_gitignore { "yes" } else { "no" },
                            project.limits.per_file_bytes / 1024,
                            project.limits.per_capture_bytes / (1024 * 1024),
                        ),
                        TextRole::BodySm,
                        Tone::Muted,
                    )),
            ));
        }

        if let Some(session) = &self.capture_session {
            let preview = &session.collected.preview;
            let file_rows = preview
                .files
                .iter()
                .take(200)
                .map(|file| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(space::S2))
                        .py(px(space::S1))
                        .border_b_1()
                        .border_color(theme::row_line())
                        .child(text(&file.path, TextRole::CodeSm, Tone::Ink))
                        .child(badge(&file.category))
                        .child(text(format!("{} B", file.bytes), TextRole::BodySm, Tone::Faint))
                })
                .collect::<Vec<_>>();
            let skipped_rows = preview
                .skipped
                .iter()
                .take(100)
                .map(|file| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(space::S2))
                        .py(px(space::S1))
                        .child(text(&file.path, TextRole::CodeSm, Tone::Muted))
                        .child(badge(&file.reason))
                })
                .collect::<Vec<_>>();
            let mut review = card(
                Some("Review preview"),
                Some("Saving re-collects the repository. If anything changed, you must preview again."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(space::S2))
                            .child(badge(&format!("{} files", preview.files.len())))
                            .child(badge(&format!("{} bytes", preview.total_bytes)))
                            .child(badge(&format!("{} skipped", preview.skipped.len())))
                            .when_some(preview.git.as_ref(), |this, git| {
                                this.child(badge(if git.dirty { "Dirty work tree" } else { "Clean work tree" }))
                                    .child(text(
                                        format!(
                                            "{} · {}",
                                            git.branch.clone().unwrap_or_else(|| "detached".to_string()),
                                            git.summary
                                        ),
                                        TextRole::BodySm,
                                        Tone::Muted,
                                    ))
                            }),
                    )
                    .child(text(&preview.fingerprint, TextRole::CodeSm, Tone::Faint))
                    .child(
                        div()
                            .id("capture-files")
                            .flex()
                            .flex_col()
                            .max_h(px(280.0))
                            .overflow_y_scroll()
                            .children(file_rows),
                    )
                    .when(!skipped_rows.is_empty(), |this| {
                        this.child(section_title("Skipped", "Every skipped path is shown with its reason."))
                            .child(
                                div()
                                    .id("capture-skipped")
                                    .flex()
                                    .flex_col()
                                    .max_h(px(160.0))
                                    .overflow_y_scroll()
                                    .children(skipped_rows),
                            )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(space::S2))
                            .child(choice_chip(
                                "capture-reviewed",
                                "I reviewed this exact file list",
                                self.capture_reviewed,
                                cx.listener(|this, _, _, cx| {
                                    this.capture_reviewed = !this.capture_reviewed;
                                    cx.notify();
                                }),
                            ))
                            .child(
                                Button::new("save-capture", "Save capture", ButtonVariant::Primary)
                                    .disabled(!self.capture_reviewed)
                                    .on_click(cx.listener(|this, _, _, cx| this.save_capture(cx))),
                            ),
                    ),
            );
            if let Some(error) = &self.error {
                review = review.child(alert(Some(error.code.as_str()), &error.message));
            }
            if let Some(notice) = &self.notice {
                review = review.child(alert(None, notice));
            }
            body = body.child(review);
        } else {
            if let Some(error) = &self.error {
                body = body.child(alert(Some(error.code.as_str()), &error.message));
            }
            if let Some(notice) = &self.notice {
                body = body.child(alert(None, notice));
            }
        }

        let capture_rows = captures
            .iter()
            .map(|capture| {
                let id = capture.id.clone();
                let confirming = self.confirm_delete_capture.as_deref() == Some(id.as_str());
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(px(space::S2))
                    .w_full()
                    .p(px(space::S3))
                    .border_b_1()
                    .border_color(theme::row_line())
                    .child(text(&format!("v{}", capture.version), TextRole::Label, Tone::Ink))
                    .child(badge(&format!("{} files", capture.files.len())))
                    .child(badge(&format!("{} bytes", capture.total_bytes)))
                    .child(text(&capture.created_at, TextRole::BodySm, Tone::Faint))
                    .child(div().flex_1())
                    .child(
                        Button::new(SharedString::from(format!("inspect-{}", capture.id)), "Inspect", ButtonVariant::Ghost)
                            .size(coderune_ui::ButtonSize::Sm)
                            .on_click(cx.listener(move |this, _, _, cx| this.inspect_capture(&id, cx))),
                    )
                    .child(
                        Button::new(SharedString::from(format!("delete-capture-{}", capture.id)), if confirming { "Confirm" } else { "Delete" }, ButtonVariant::Destructive)
                            .size(coderune_ui::ButtonSize::Sm)
                            .on_click(cx.listener({
                                let id = capture.id.clone();
                                move |this, _, _, cx| {
                                    if this.confirm_delete_capture.as_deref() == Some(id.as_str()) {
                                        this.delete_capture(&id, cx);
                                    } else {
                                        this.confirm_delete_capture = Some(id.clone());
                                        cx.notify();
                                    }
                                }
                            })),
                    )
            })
            .collect::<Vec<_>>();

        body = body.child(card(
            Some("Capture history"),
            Some("Older captures remain immutable when the profile or project changes."),
            div()
                .flex()
                .flex_col()
                .when(capture_rows.is_empty(), |this| {
                    this.child(text("No captures for this project yet.", TextRole::BodySm, Tone::Muted))
                })
                .children(capture_rows),
        ));

        if let Some(detail) = &self.capture_detail {
            let selected_content = detail
                .selected_path
                .as_ref()
                .and_then(|path| detail.contents.get(path).cloned())
                .unwrap_or_default();
            let file_buttons = detail
                .capture
                .files
                .iter()
                .take(200)
                .map(|file| {
                    let path = file.path.clone();
                    let selected = detail.selected_path.as_deref() == Some(path.as_str());
                    div()
                        .id(SharedString::from(format!("detail-file-{}", file.path)))
                        .cursor(gpui::CursorStyle::PointingHand)
                        .px(px(space::S2))
                        .py(px(space::S1))
                        .rounded(px(radius::INNER))
                        .when(selected, |this| this.bg(theme::accent_soft()))
                        .when(!selected, |this| this.hover(|style| style.bg(theme::hover())))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(detail) = this.capture_detail.as_mut() {
                                detail.selected_path = Some(path.clone());
                            }
                            cx.notify();
                        }))
                        .child(text(&file.path, TextRole::CodeSm, Tone::Ink))
                })
                .collect::<Vec<_>>();
            body = body.child(card(
                Some(&format!("Capture v{}", detail.capture.version)),
                Some("Saved content is read back from the local content store by hash."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(
                        div()
                            .id("detail-files")
                            .flex()
                            .flex_col()
                            .max_h(px(200.0))
                            .overflow_y_scroll()
                            .children(file_buttons),
                    )
                    .child(
                        div()
                            .id("detail-content")
                            .max_h(px(layout::FIELD_MAX_HEIGHT))
                            .overflow_y_scroll()
                            .bg(theme::code())
                            .rounded(px(radius::SOFT))
                            .p(px(space::S3))
                            .type_role(TextRole::Code)
                            .text_color(theme::ink())
                            .child(if selected_content.is_empty() {
                                "No stored content for this path.".to_string()
                            } else {
                                selected_content
                            }),
                    ),
            ));
        }

        body
    }

    fn refresh_agents(&mut self, cx: &mut Context<Self>) {
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.agents = coderune_agents::detect_all();
        }
        self.notice = Some("Agent detection refreshed.".to_string());
        cx.notify();
    }

    fn render_agents(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let agents = self.workspace.as_ref().map(|workspace| workspace.agents.clone()).unwrap_or_default();
        let cards = agents.iter().map(|status| {
            let ready = status.availability == AgentAvailability::Ready;
            let capabilities = status
                .capabilities
                .iter()
                .map(|capability| badge(capability))
                .collect::<Vec<_>>();
            card(
                Some(status.id.label()),
                Some("Optional. Coderune never owns this tool's sign-in, provider, or model choice."),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S3))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(space::S2))
                            .child(badge(if ready { "Ready" } else { "Not installed" }))
                            .child(badge(if status.verified { "Verified" } else { "Unverified" }))
                            .children(capabilities),
                    )
                    .child(text(&status.detail, TextRole::BodySm, Tone::Muted))
                    .child(text(
                        format!("Restrictions: {}", status.restrictions.join("; ")),
                        TextRole::BodySm,
                        Tone::Faint,
                    ))
                    .child(text(&status.setup_url, TextRole::CodeSm, Tone::Muted)),
            )
        }).collect::<Vec<_>>();

        div()
            .flex()
            .flex_col()
            .gap(px(space::S6))
            .child(page_header(
                "Agents",
                "Coding agents detected on this machine. Compatibility is reported honestly and is not claimed until demonstrated.",
                Button::new("refresh-agents", "Refresh detection", ButtonVariant::Secondary)
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_agents(cx))),
            ))
            .child(alert(
                None,
                "Each agent keeps its own sign-in, provider, and model configuration. Coderune does not read or copy credential files. Manual entry always remains available.",
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(space::S4))
                    .children(cards),
            )
    }

    fn render_section_placeholder(&self, title: &str, description: &str) -> impl IntoElement {
        card(
            Some(title),
            Some(description),
            text(
                "This section is being ported from the GPUIX shell. The native service layer is ready.",
                TextRole::Body,
                Tone::Muted,
            ),
        )
    }

    fn render_workspace(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.view {
            View::Home => self.render_home(cx).into_any_element(),
            View::Profile => self.render_profile(cx).into_any_element(),
            View::Projects => self.render_projects(cx).into_any_element(),
            View::Documents => self.render_documents(cx).into_any_element(),
            View::Agents => self.render_agents(cx).into_any_element(),
            View::Capture => self.render_capture(cx).into_any_element(),
            View::Data => self.render_section_placeholder("Data", "Export local context or delete application data.").into_any_element(),
        };
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme::bg())
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .child(self.render_topbar())
                    .child(
                        div()
                            .id("main-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .bg(theme::bg())
                            .p(px(space::S8))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .max_w(px(layout::CONTAINER))
                                    .mx_auto()
                                    .child(content),
                            ),
                    ),
            )
    }
}

impl Render for CoderuneApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let can_start = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.onboarding.can_start_project)
            .unwrap_or(false);
        let body = if let Some(message) = &self.fatal {
            self.render_fatal(message).into_any_element()
        } else if can_start {
            self.render_workspace(cx).into_any_element()
        } else {
            self.render_onboarding(cx).into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::ink())
            .font_family(theme::FONT_SANS)
            .type_role(TextRole::Body)
            .track_focus(&self.focus)
            .child(body)
    }
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n', ';'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}
