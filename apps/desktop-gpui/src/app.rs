use std::path::PathBuf;

use coderune_app::Workspace;
use coderune_domain::{AppError, DocumentKind, ExperienceStage, ProfileDraft};
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

pub struct CoderuneApp {
    workspace: Option<Workspace>,
    fatal: Option<String>,
    pub view: View,
    profile_form: ProfileForm,
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
        Self {
            workspace,
            fatal,
            view: View::Home,
            profile_form,
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
            View::Profile => self.render_section_placeholder("Profile", "Inspect and edit the reusable developer profile.").into_any_element(),
            View::Projects => self.render_section_placeholder("Projects", "Create, select, edit, and delete project context.").into_any_element(),
            View::Documents => self.render_section_placeholder("Documents", "Import CV and SRS source documents for review.").into_any_element(),
            View::Agents => self.render_section_placeholder("Agents", "Coding agents detected on this machine.").into_any_element(),
            View::Capture => self.render_section_placeholder("Capture", "Preview, review, and save repository context.").into_any_element(),
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
