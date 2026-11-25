//! Coderune's native desktop application.

pub(crate) mod scenes;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use coderune_agents::CliAgentId;
use coderune_app::{PreviewSession, Workspace};
use coderune_avatars::BotAvatarStage;
use coderune_domain::{AgentAvailability, AppError, ContextCapture, ProfileDraft};
#[cfg(test)]
use coderune_domain::{
    AgentId, Certification, EducationEntry, EmploymentEntry, ExperienceStage, LearningPreferences,
    Link, ProjectDraft, ProjectExperience, SelfReportedSkill, SkillRating,
};
use rune::{col, text, Cx, El, FileDialogOptions, Key, KeyEvent, TextEdit, Tx};
use rune_kit::shell::is_sidebar_toggle;

use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, HEADING};

use crate::agent_detection::{AgentDetection, Check};
use crate::chat_terminal::Terminal;
use crate::chats::{Author, Chat, Chats};
use crate::forms::{
    edit, project_name, split_list, DocumentForm, ExtractedDocument, ProfileForm, ProjectForm,
};
use crate::preferences::{AgentDock, Preferences};
use crate::screens;
use crate::shell::{self, SidebarPane};
use crate::updates::Updates;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Profile,
    Projects,
    Documents,
    Subagents,
    Capture,
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
            Self::Subagents => "Subagents",
            Self::Capture => "Capture",
            Self::Avatars => "Avatars",
        }
    }

    pub fn icon(self) -> &'static rune::Icon {
        match self {
            Self::Home => &icons::HOUSE,
            Self::Profile => &icons::USER,
            Self::Projects => &icons::FOLDER,
            Self::Documents => &icons::FILES,
            Self::Subagents => &icons::ROBOT,
            Self::Capture => &icons::SCAN,
            Self::Avatars => &icons::ROBOT,
        }
    }

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Profile => "profile",
            Self::Projects => "projects",
            Self::Documents => "documents",
            Self::Subagents => "subagents",
            Self::Capture => "capture",
            Self::Avatars => "avatars",
        }
    }

    pub(crate) fn nav_key(self) -> &'static str {
        match self {
            Self::Home => "nav-home",
            Self::Profile => "nav-profile-footer",
            Self::Projects => "nav-projects",
            Self::Documents => "nav-documents",
            Self::Subagents => "nav-subagents",
            Self::Capture => "nav-capture",
            Self::Avatars => "nav-avatars",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsSection {
    General,
    Storage,
    Agents,
    Shortcuts,
}

impl SettingsSection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Storage => "Storage",
            Self::Agents => "Agents",
            Self::Shortcuts => "Shortcuts",
        }
    }
}

pub(crate) const VIEW_SHORTCUTS: [(char, View); 6] = [
    ('1', View::Home),
    ('2', View::Profile),
    ('3', View::Projects),
    ('4', View::Documents),
    ('5', View::Subagents),
    ('6', View::Capture),
];

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

pub(crate) struct ContextItem {
    pub label: &'static str,
    pub done: bool,
}

pub struct CaptureDetail {
    pub capture: ContextCapture,
    pub contents: BTreeMap<String, String>,
    pub selected_path: Option<String>,
}

pub struct CoderuneApp {
    pub(crate) workspace: Option<Workspace>,
    pub(crate) fatal: Option<String>,
    pub(crate) view: View,
    pub(crate) sidebar: SidebarPane,
    /// Whether the onboarding dialog is showing. It can be dismissed into the
    /// Profile panel, which finishes the same baseline.
    pub(crate) onboarding_visible: bool,
    pub profile_tab: ProfileTab,
    pub profile_section: Option<ProfileSection>,
    pub(crate) profile_form: ProfileForm,
    pub(crate) project_form: ProjectForm,
    /// The new-project dialog opened by picking a folder.
    pub(crate) project_dialog: bool,
    pub(crate) project_scope_expanded: bool,
    pub(crate) document_form: DocumentForm,
    pub(crate) capture_session: Option<PreviewSession>,
    pub(crate) capture_reviewed: bool,
    pub(crate) captures: Vec<ContextCapture>,
    projects_with_captures: BTreeSet<String>,
    pub(crate) capture_detail: Option<CaptureDetail>,
    pub(crate) confirm_delete_project: Option<String>,
    pub(crate) confirm_delete_capture: Option<String>,
    pub(crate) confirm_delete_all: bool,
    pub(crate) settings: Option<SettingsSection>,
    /// The font list open in Settings, by row.
    pub(crate) font_menu: Option<&'static str>,
    pub(crate) settings_feedback: Option<Result<String, AppError>>,
    preferences: Preferences,
    pub(crate) agent_detection: AgentDetection,
    pub(crate) updates: Updates,
    /// Chats on disk and the tabs open in this window.
    pub(crate) chats: Chats,
    pub(crate) chat_draft: TextEdit,
    chat_drafts: HashMap<String, TextEdit>,
    pub(crate) chat_terminals: HashMap<String, Terminal>,
    /// The family lists last handed to the text system.
    applied_fonts: Option<(String, String)>,
    pub error: Option<AppError>,
    pub notice: Option<String>,
    /// The animated avatars on screen, keyed by subagent or gallery id.
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
        let preferences = workspace
            .as_ref()
            .map(|workspace| Preferences::load(&Preferences::path(&workspace.data_path())))
            .transpose()
            .unwrap_or_else(|error| {
                eprintln!("Could not load desktop settings: {error}");
                None
            })
            .unwrap_or_default();
        let mut chat_error = None;
        let chats = workspace.as_ref().map_or_else(Chats::default, |workspace| {
            let path = Chats::path(&workspace.data_path());
            Chats::load(&path).unwrap_or_else(|error| {
                chat_error = Some(AppError::new(
                    coderune_domain::ErrorCode::UnreadablePath,
                    format!("Could not load saved chats: {error}"),
                ));
                Chats::unreadable(path)
            })
        });
        let mut app = Self {
            workspace,
            fatal,
            view: View::Home,
            sidebar: SidebarPane::default(),
            onboarding_visible: true,
            profile_tab: ProfileTab::Baseline,
            profile_section: None,
            profile_form,
            project_form,
            project_dialog: false,
            project_scope_expanded: false,
            document_form: DocumentForm::empty(),
            capture_session: None,
            capture_reviewed: false,
            captures: Vec::new(),
            projects_with_captures: BTreeSet::new(),
            capture_detail: None,
            confirm_delete_project: None,
            confirm_delete_capture: None,
            confirm_delete_all: false,
            settings: None,
            font_menu: None,
            settings_feedback: None,
            preferences,
            agent_detection: AgentDetection::default(),
            updates: Updates::new(),
            chats,
            chat_draft: TextEdit::default(),
            chat_drafts: HashMap::new(),
            chat_terminals: HashMap::new(),
            applied_fonts: None,
            error: chat_error,
            notice: None,
            avatars: BotAvatarStage::new(),
        };
        if let Some(workspace) = &app.workspace {
            match workspace.projects_with_captures() {
                Ok(projects) => app.projects_with_captures = projects,
                Err(error) => app.error = Some(error),
            }
        }
        app.refresh_captures();
        if app.active_chat().is_some() {
            app.refresh_chat_agents();
        }
        app
    }

    /// Whether the developer baseline is still missing, so the onboarding dialog
    /// stays in front of the workspace.
    pub(crate) fn onboarding_required(&self) -> bool {
        self.workspace
            .as_ref()
            .is_some_and(|workspace| !workspace.onboarding.can_start_project)
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
        self.captures.clear();
        if let Some(workspace) = &self.workspace {
            if let Some(id) = &workspace.selected_project_id {
                match workspace.list_captures(id) {
                    Ok(captures) => {
                        if captures.is_empty() {
                            self.projects_with_captures.remove(id);
                        } else {
                            self.projects_with_captures.insert(id.clone());
                        }
                        self.captures = captures;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
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
        self.show_workspace();
    }

    pub(crate) fn select_settings(&mut self, section: SettingsSection) {
        self.settings = Some(section);
        self.settings_feedback = None;
        self.font_menu = None;
        self.confirm_delete_all = false;
        if section == SettingsSection::Agents
            && matches!(self.agent_detection.check, Check::NotChecked)
        {
            self.agent_detection.refresh();
        }
    }

    pub(crate) fn close_settings(&mut self) {
        self.settings = None;
        self.settings_feedback = None;
        self.font_menu = None;
        self.confirm_delete_all = false;
    }

    pub(crate) fn dark_mode(&self) -> bool {
        self.preferences.dark_mode
    }

    pub(crate) fn toggle_dark_mode(&mut self) {
        let dark_mode = !self.dark_mode();
        self.update_preferences(|preferences| preferences.dark_mode = dark_mode);
    }

    /// What a project still needs before a chat has full context.
    pub(crate) fn context_items(&self, project_id: &str) -> Vec<ContextItem> {
        let Some(project) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.projects.iter().find(|p| p.id == project_id))
        else {
            return Vec::new();
        };
        vec![
            ContextItem {
                label: "Project folder",
                done: std::path::Path::new(&project.root_path).is_dir(),
            },
            ContextItem {
                label: "Goals",
                done: !project.goals.is_empty(),
            },
            ContextItem {
                label: "Capture scope",
                done: !project.scope.include_paths.is_empty()
                    || !project.scope.exclude_paths.is_empty(),
            },
            ContextItem {
                label: "First capture",
                done: self.projects_with_captures.contains(project_id),
            },
        ]
    }

    pub(crate) fn context_done(&self, project_id: &str) -> (u8, u8) {
        let items = self.context_items(project_id);
        let done = items.iter().filter(|item| item.done).count() as u8;
        (done, items.len() as u8)
    }

    pub(crate) fn active_chat(&self) -> Option<&Chat> {
        self.chats.active_chat()
    }

    pub(crate) fn show_chat_project(&mut self) {
        if let Some(project_id) = self.active_chat().map(|chat| chat.project_id.clone()) {
            self.load_project_form(&project_id);
            self.navigate(View::Projects);
        }
    }

    /// Creates a chat in the selected project and opens its tab.
    pub(crate) fn new_chat(&mut self, project_id: &str) {
        self.error = None;
        self.notice = None;
        if !self.has_project(project_id) {
            return;
        }
        self.stash_chat_draft();
        self.chats.new_chat(project_id);
        self.update_preferences(|preferences| preferences.set_project_expanded(project_id, true));
        self.save_chats();
        self.refresh_chat_agents();
    }

    pub(crate) fn open_chat(&mut self, id: &str) {
        let project_id = self.chats.chat(id).map(|chat| chat.project_id.clone());
        if self.chats.active.as_deref() == Some(id) {
            return;
        }
        let Some(project_id) = project_id else { return };
        self.error = None;
        self.notice = None;
        if !self.has_project(&project_id) {
            return;
        }
        self.stash_chat_draft();
        self.chats.open_chat(id);
        self.restore_chat_draft();
        self.save_chats();
        self.refresh_chat_agents();
    }

    pub(crate) fn show_workspace(&mut self) {
        if self.chats.active.is_some() {
            self.stash_chat_draft();
            self.chats.active = None;
            self.save_chats();
        }
    }

    pub(crate) fn close_chat(&mut self, id: &str) {
        let active = self.chats.active.as_deref() == Some(id);
        if active {
            self.stash_chat_draft();
        }
        self.chats.close_chat(id);
        self.chat_terminals.remove(id);
        if active {
            self.restore_chat_draft();
        }
        self.save_chats();
    }

    fn has_project(&self, project_id: &str) -> bool {
        self.workspace.as_ref().is_some_and(|workspace| {
            workspace
                .projects
                .iter()
                .any(|project| project.id == project_id)
        })
    }

    fn refresh_chat_agents(&mut self) {
        if matches!(self.agent_detection.check, Check::NotChecked) {
            self.agent_detection.refresh();
        }
    }

    fn stash_chat_draft(&mut self) {
        if let Some(id) = &self.chats.active {
            self.chat_drafts
                .insert(id.clone(), std::mem::take(&mut self.chat_draft));
        }
    }

    fn restore_chat_draft(&mut self) {
        self.chat_draft = self
            .chats
            .active
            .as_ref()
            .and_then(|id| self.chat_drafts.remove(id))
            .unwrap_or_default();
    }

    /// Sends the message to the running agent in the active tab.
    pub(crate) fn send_chat_message(&mut self) {
        let text = self.chat_draft.text.trim();
        let Some(id) = self.chats.active.clone() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let Some(terminal) = self
            .chat_terminals
            .get_mut(&id)
            .filter(|term| term.is_running())
        else {
            return;
        };
        if let Err(error) = terminal.send_line(text) {
            self.chat_agent_error("Could not send message", error);
            return;
        }
        self.error = None;
        self.chats.add_message(&id, Author::Developer, text);
        self.chat_draft = TextEdit::default();
        self.save_chats();
    }

    pub(crate) fn set_chat_agent(&mut self, agent: Option<coderune_agents::CliAgentId>) {
        let Some(id) = self.chats.active.clone() else {
            return;
        };
        if self.active_chat().is_some_and(|chat| chat.agent == agent) {
            return;
        }
        self.chat_terminals.remove(&id);
        self.chats.set_agent(&id, agent);
        self.error = None;
        self.save_chats();
    }

    pub(crate) fn start_chat_agent(&mut self) {
        let Some(chat) = self.active_chat() else {
            return;
        };
        let Some(agent) = chat.agent.filter(|agent| self.agent_enabled(*agent)) else {
            return;
        };
        let Some(executable) = self
            .agent_detection
            .results
            .iter()
            .find(|status| status.id == agent)
            .and_then(|status| status.executable.as_deref())
        else {
            return;
        };
        let Some(project) = self.workspace.as_ref().and_then(|workspace| {
            workspace
                .projects
                .iter()
                .find(|project| project.id == chat.project_id)
        }) else {
            return;
        };
        if self
            .chat_terminals
            .get(&chat.id)
            .is_some_and(Terminal::is_running)
        {
            return;
        }
        match Terminal::launch(
            executable,
            std::path::Path::new(&project.root_path),
            self.agent_detection.path.as_deref(),
        ) {
            Ok(terminal) => {
                self.chat_terminals.insert(chat.id.clone(), terminal);
                self.error = None;
            }
            Err(error) => self.chat_agent_error("Could not start agent", error),
        }
    }

    pub(crate) fn active_terminal_mut(&mut self) -> Option<&mut Terminal> {
        self.chat_terminals.get_mut(self.chats.active.as_deref()?)
    }

    pub(crate) fn stop_chat_agent(&mut self) {
        if let Some(Err(error)) = self.active_terminal_mut().map(Terminal::stop) {
            self.chat_agent_error("Could not stop agent", error);
        }
    }

    pub(crate) fn clear_chat_terminal(&mut self) {
        if let Some(id) = &self.chats.active {
            self.chat_terminals.remove(id);
        }
    }

    fn chat_agent_error(&mut self, message: &str, error: std::io::Error) {
        self.error = Some(AppError::new(
            coderune_domain::ErrorCode::AgentFailed,
            format!("{message}: {error}"),
        ));
    }

    pub(crate) fn agent_dock(&self) -> AgentDock {
        self.preferences.agent_dock
    }

    pub(crate) fn set_agent_dock(&mut self, dock: AgentDock) {
        self.update_preferences(|preferences| preferences.agent_dock = dock);
    }

    pub(crate) fn project_expanded(&self, id: &str) -> bool {
        self.preferences.project_expanded(id)
    }

    pub(crate) fn toggle_project_chats(&mut self, id: &str) {
        let expanded = !self.project_expanded(id);
        let id = id.to_string();
        self.update_preferences(move |preferences| preferences.set_project_expanded(&id, expanded));
    }

    fn save_chats(&mut self) {
        if let Err(error) = self.chats.save() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::UnreadablePath,
                format!("Could not save chats: {error}"),
            ));
        }
    }

    pub(crate) fn ui_font(&self) -> &str {
        self.preferences
            .ui_font
            .as_deref()
            .unwrap_or(crate::DEFAULT_UI_FONT)
    }

    pub(crate) fn mono_font(&self) -> &str {
        self.preferences
            .mono_font
            .as_deref()
            .unwrap_or(crate::DEFAULT_MONO_FONT)
    }

    pub(crate) fn set_ui_font(&mut self, family: String) {
        self.update_preferences(|preferences| preferences.ui_font = Some(family));
    }

    pub(crate) fn set_mono_font(&mut self, family: String) {
        self.update_preferences(|preferences| preferences.mono_font = Some(family));
    }

    /// The CSS family lists the text system should use: the chosen family
    /// first, then the shipped one so missing glyphs still resolve.
    fn font_families(&self) -> (String, String) {
        (
            format!("{}, {}", self.ui_font(), crate::DEFAULT_UI_FONT),
            format!("{}, {}", self.mono_font(), crate::DEFAULT_MONO_FONT),
        )
    }

    pub(crate) fn agent_enabled(&self, agent: CliAgentId) -> bool {
        self.preferences.agent_enabled(agent)
    }

    pub(crate) fn set_agent_enabled(&mut self, agent: CliAgentId, enabled: bool) {
        self.update_preferences(|preferences| preferences.set_agent_enabled(agent, enabled));
        if !enabled {
            self.chat_terminals.retain(|id, _| {
                self.chats
                    .chat(id)
                    .is_some_and(|chat| chat.agent != Some(agent))
            });
        }
        if self
            .document_form
            .agent
            .is_some_and(|selected| !self.agent_enabled(selected.into()))
        {
            self.document_form.agent = None;
            self.document_form.draft_preview = None;
        }
    }

    fn update_preferences(&mut self, change: impl FnOnce(&mut Preferences)) {
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        match Preferences::update(&Preferences::path(&workspace.data_path()), change) {
            Ok(next) => {
                self.preferences = next;
                self.settings_feedback = None;
            }
            Err(error) => {
                self.settings_feedback = Some(Err(AppError::new(
                    coderune_domain::ErrorCode::UnreadablePath,
                    format!("Could not save desktop settings: {error}"),
                )));
            }
        }
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

    /// Why the onboarding dialog cannot be confirmed, if it cannot. Only the two
    /// questions every project needs are required; the rest can wait in Profile.
    fn onboarding_blocker(&self) -> Option<&'static str> {
        if self.profile_form.display_name.text.trim().is_empty() {
            Some("Add a display name or pseudonym to continue.")
        } else if self.profile_form.stage.is_none() {
            Some("Choose the experience stage that fits you today.")
        } else {
            None
        }
    }

    pub(crate) fn confirm_onboarding(&mut self) {
        if let Some(message) = self.onboarding_blocker() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                message,
            ));
            self.notice = None;
            return;
        }
        self.confirm_profile();
    }

    /// Closes the onboarding dialog and leaves the same answers in the Profile
    /// panel, where the rest of the baseline can be finished.
    pub(crate) fn dismiss_onboarding(&mut self) {
        self.onboarding_visible = false;
        self.view = View::Profile;
        self.profile_tab = ProfileTab::Baseline;
        self.error = None;
        self.notice = None;
        self.show_workspace();
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
                self.onboarding_visible = false;
                self.view = View::Home;
                self.profile_tab = ProfileTab::Baseline;
                self.profile_section = None;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_profile(&mut self) {
        // Before the first baseline is confirmed, the Profile panel is the
        // onboarding fallback: the same form, with the same confirmation.
        if self.onboarding_required() {
            self.confirm_profile();
            return;
        }
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
        self.project_scope_expanded = false;
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
        self.project_dialog = false;
        self.project_scope_expanded = false;
        self.confirm_delete_project = None;
        self.error = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(None);
        }
        self.refresh_captures();
    }

    /// Opens the system folder picker. A picked folder fills a new project form
    /// and opens its quick questions; cancelling changes nothing.
    pub(crate) fn choose_project_folder(&mut self, tx: &mut Tx) {
        let directory = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.launch_directory.clone());
        tx.open_folder(
            FileDialogOptions {
                title: "Choose a project folder".to_string(),
                directory,
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, tx| match result {
                Ok(Some(path)) => {
                    s.start_new_project(&path);
                    tx.instant();
                }
                Ok(None) => {}
                Err(error) => s.error = Some(folder_picker_error(error)),
            },
        );
    }

    /// The panel's folder row. It replaces the form's root, keeping the name the
    /// developer typed unless it is still untouched.
    pub(crate) fn change_project_folder(&mut self, tx: &mut Tx) {
        let directory = self.project_form.folder();
        tx.open_folder(
            FileDialogOptions {
                title: "Choose a project folder".to_string(),
                directory,
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, tx| match result {
                Ok(Some(path)) => {
                    if s.project_form.name.text.trim().is_empty() {
                        s.project_form.name = edit(project_name(&path));
                    }
                    s.project_form.root = edit(path.display().to_string());
                    s.error = None;
                    tx.instant();
                }
                Ok(None) => {}
                Err(error) => s.error = Some(folder_picker_error(error)),
            },
        );
    }

    fn start_new_project(&mut self, root: &Path) {
        self.project_form = ProjectForm::from_folder(root);
        self.project_dialog = true;
        self.project_scope_expanded = false;
        self.confirm_delete_project = None;
        self.error = None;
        self.notice = None;
        self.reset_capture_state();
        if let Some(workspace) = self.workspace.as_mut() {
            workspace.select_project(None);
        }
        self.view = View::Projects;
        self.show_workspace();
        self.refresh_captures();
    }

    /// Closes the quick questions without creating anything: the picked folder
    /// stays in the panel, where the rest can be completed.
    pub(crate) fn close_project_dialog(&mut self) {
        self.project_dialog = false;
        self.error = None;
    }

    /// Creates the project from the dialog. Only the name and one goal are
    /// required; scope and limits take their defaults.
    pub(crate) fn create_project_from_dialog(&mut self) {
        if self.project_form.name.text.trim().is_empty() {
            self.error = Some(AppError::validation(vec!["name".to_string()]));
            return;
        }
        if split_list(&self.project_form.goals.text).is_empty() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Add at least one goal, or finish the project in the panel.",
            ));
            return;
        }
        self.save_project();
        if self.project_form.editing.is_some() {
            self.project_dialog = false;
        }
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
                for chat in self.chats.chats_for(&id) {
                    self.chat_terminals.remove(&chat.id);
                    self.chat_drafts.remove(&chat.id);
                }
                self.chats.delete_for_project(&id);
                self.projects_with_captures.remove(&id);
                self.new_project_form();
                self.save_chats();
                self.notice = Some("Project, captures, and chats deleted.".to_string());
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
        if !self.agent_enabled(agent.into()) {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "Turn on this coding agent in Settings before using it.",
            ));
            return;
        }
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        if !workspace
            .agents
            .iter()
            .any(|status| status.id == agent && status.availability == AgentAvailability::Ready)
        {
            self.document_form.draft_preview = None;
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                "This coding agent is unavailable. Recheck installed agents in Settings.",
            ));
            return;
        }
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

    /// Asks where to write the export, then writes it there. No path field: the
    /// system save dialog names the file.
    pub(crate) fn export_data(&mut self, tx: &mut Tx) {
        let directory = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.data_path())
            .and_then(|path| path.parent().map(Path::to_path_buf));
        tx.save_file(
            FileDialogOptions {
                title: "Export local context".to_string(),
                directory,
                file_name: Some("coderune-export.json".to_string()),
                ..FileDialogOptions::default()
            },
            |s: &mut CoderuneApp, result, _| match result {
                Ok(Some(path)) => s.export_data_to(&path),
                Ok(None) => {}
                Err(error) => {
                    s.settings_feedback = Some(Err(AppError::new(
                        coderune_domain::ErrorCode::UnreadablePath,
                        format!("Could not open the save dialog: {error}"),
                    )));
                }
            },
        );
    }

    fn export_data_to(&mut self, path: &Path) {
        let Some(workspace) = self.workspace.as_ref() else {
            return;
        };
        self.settings_feedback = Some(workspace.export_json().and_then(|json| {
            std::fs::write(path, json)
                .map(|()| "Local context exported.".to_string())
                .map_err(|error| {
                    AppError::new(
                        coderune_domain::ErrorCode::Validation,
                        format!("Could not write {}: {error}", path.display()),
                    )
                })
        }));
    }

    pub(crate) fn delete_all_data(&mut self) {
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.delete_all_data() {
            Ok(()) => {
                self.chat_terminals.clear();
                self.chat_drafts.clear();
                self.chat_draft = TextEdit::default();
                let clear_chats = self.chats.clear();
                self.projects_with_captures.clear();
                self.notice = clear_chats.is_ok().then(|| {
                    "All workspace data deleted. Onboarding is required again.".to_string()
                });
                self.error = None;
                self.close_settings();
                self.view = View::Home;
                self.onboarding_visible = true;
                self.reset_capture_state();
                self.confirm_delete_project = None;
                self.confirm_delete_capture = None;
                self.project_form = ProjectForm::empty(None);
                self.project_dialog = false;
                self.document_form = DocumentForm::empty();
                self.refresh_profile_form();
                self.refresh_captures();
                if let Err(error) = clear_chats {
                    self.chat_agent_error("Could not remove saved chats", error);
                }
            }
            Err(error) => self.settings_feedback = Some(Err(error)),
        }
    }

    pub(crate) fn open_folder(&mut self, path: PathBuf) {
        if let Err(error) = Workspace::open_directory(&path) {
            self.error = Some(error);
        }
    }

    pub(crate) fn open_settings_folder(&mut self, path: &std::path::Path) {
        self.settings_feedback = Workspace::open_directory(path).err().map(Err);
    }

    pub(crate) fn open_editor(&mut self, editor: &str, path: PathBuf) {
        if let Err(error) = Workspace::launch_editor(editor, &path) {
            self.error = Some(error);
        }
    }
}

/// Advances visible animations and applies completed background updates.
pub fn poll(s: &mut CoderuneApp, t: f64, tx: &mut Tx) -> bool {
    let families = s.font_families();
    if s.applied_fonts.as_ref() != Some(&families) {
        tx.set_font_families(&families.0, &families.1);
        s.applied_fonts = Some(families);
    }
    let update_changed = s.updates.poll(tx);
    let detection_changed = s.agent_detection.poll(tx);
    let mut chat_changed = false;
    for terminal in s.chat_terminals.values_mut() {
        chat_changed |= terminal.poll(tx);
    }
    if detection_changed && matches!(s.agent_detection.check, Check::Finished) {
        if let Some(workspace) = s.workspace.as_mut() {
            workspace.agents = coderune_agents::draft_statuses(&s.agent_detection.results);
        }
    }
    let avatar_changed = match s.view {
        View::Avatars => {
            screens::register_gallery(&mut s.avatars);
            s.avatars.tick(t, 1.0)
        }
        _ => {
            if !s.avatars.is_empty() {
                s.avatars.clear();
            }
            false
        }
    };
    update_changed || detection_changed || avatar_changed || chat_changed
}

fn folder_picker_error(error: rune::FileDialogError) -> AppError {
    AppError::new(
        coderune_domain::ErrorCode::UnreadablePath,
        format!("Could not open the folder picker: {error}"),
    )
}

pub fn view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    theme::with_dark(s.dark_mode(), || themed_view(s, cx))
}

fn themed_view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    let k = theme::kit();
    let inset = cx.titlebar_inset();
    let body = if let Some(message) = &s.fatal {
        fatal(message, inset)
    } else {
        shell::workspace(s, k, cx)
    };
    let base = col().w_full().h_full().fill(theme::bg()).child(body);
    let mut overlays = Vec::new();
    if let Some(section) = s.settings {
        screens::settings::dialog(s, section, k, &mut overlays);
    }
    if s.project_dialog {
        screens::projects::dialog(s, k, &mut overlays);
    }
    // Onboarding is a dialog over the workspace, not a page of its own: the rest
    // of the profile can wait in the Profile panel.
    screens::onboarding::dialog(s, k, &mut overlays);
    rune_kit::root(base, overlays)
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
            .child(text("Could not start").font(HEADING).color(theme::ink()))
            .child(
                text(message)
                    .font(BODY)
                    .color(theme::muted())
                    .wrap()
                    .max_w(520.0),
            ),
    )
    .titlebar_inset(inset)
    .toolbar_in_panel()
    .frame_inset(theme::FRAME)
    .build(theme::kit())
}

pub fn on_key(s: &mut CoderuneApp, event: &KeyEvent, tx: &mut rune::Tx) -> bool {
    match event.key {
        Key::Command('q') => {
            s.chat_terminals.clear();
            std::process::exit(0);
        }
        Key::Command(',' | '7') if s.workspace.is_some() => {
            if s.settings.is_none() {
                s.select_settings(SettingsSection::General);
                tx.instant();
            }
            return true;
        }
        _ if s.settings.is_some() => return false,
        _ => {}
    }
    if is_sidebar_toggle(event) {
        s.sidebar.toggle();
        tx.instant();
        return true;
    }
    let Key::Command(key) = event.key else {
        return false;
    };
    let Some(&(_, view)) = VIEW_SHORTCUTS.iter().find(|(shortcut, _)| *shortcut == key) else {
        return false;
    };
    s.navigate(view);
    tx.instant();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::scenes::{scene_home, scene_projects, seed_demo};
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
    fn onboarding_confirms_the_required_answers() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        assert!(state.onboarding_required());

        state.confirm_onboarding();
        assert!(state.onboarding_required());
        assert!(state.error.is_some());

        state.profile_form.display_name = edit("Ada");
        state.profile_form.stage = Some(ExperienceStage::EarlyCareer);
        state.confirm_onboarding();
        assert!(state.onboarding_required());
        assert!(state.error.is_some());

        state.profile_form.new_to_coding = true;
        state.profile_form.goals = edit("Learn Rust");
        state.confirm_onboarding();
        assert!(!state.onboarding_required());
        assert!(state.error.is_none());
    }

    #[test]
    fn the_profile_panel_finishes_onboarding_after_it_is_dismissed() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        state.dismiss_onboarding();
        assert!(!state.onboarding_visible);
        assert_eq!(state.view, View::Profile);

        state.profile_form.display_name = edit("Ada");
        state.profile_form.stage = Some(ExperienceStage::EarlyCareer);
        state.profile_form.new_to_coding = true;
        state.profile_form.goals = edit("Learn Rust");
        state.save_profile();
        assert!(!state.onboarding_required());
        assert!(state.error.is_none());
    }

    #[test]
    fn a_new_project_folder_opens_the_quick_questions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        seed_demo(&mut state);
        let folder = tempfile::tempdir().unwrap();
        state.start_new_project(folder.path());
        assert!(state.project_dialog);
        assert!(state.project_form.editing.is_none());

        // A goal is the one thing the quick questions insist on.
        state.project_form.goals = edit("");
        state.create_project_from_dialog();
        assert!(state.project_dialog);
        assert!(state.error.is_some());

        state.project_form.goals = edit("Ship the desktop app");
        state.create_project_from_dialog();
        assert!(!state.project_dialog);
        assert!(state.project_form.editing.is_some());
        assert!(state.workspace.as_ref().unwrap().projects.len() >= 2);
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
        assert!(onboarding.find("onboarding-dialog").is_some());
        assert!(onboarding.find("stage-chip-EarlyCareer").is_some());
        assert!(onboarding.find("onboarding-confirm").is_some());

        let mut workspace_state = CoderuneApp::init();
        seed_demo(&mut workspace_state);
        let mut workspace = Runtime::new(workspace_state, view, (1280.0, 860.0), env);
        workspace.frame(0.0);
        assert!(workspace.find("nav-capture").is_some());
        assert!(workspace.find("sidebar-toggle").is_none());

        let project_id = workspace
            .state
            .workspace
            .as_ref()
            .unwrap()
            .selected_project_id
            .as_ref()
            .unwrap();
        let [x, y, w, h] = workspace
            .presented(&format!("project-{project_id}"))
            .unwrap()
            .0;
        workspace.pointer_down(x + w / 2.0, y + h / 2.0);
        workspace.pointer_up(x + w / 2.0, y + h / 2.0);
        workspace.frame(0.0);
        assert!(workspace.find("t-workspace").is_some());
        assert!(workspace.find("projects").is_some());
        assert!(workspace.find("t-projects").is_none());
        assert!(workspace.find("brand").is_none());

        let position = |runtime: &Runtime<CoderuneApp>, key| runtime.presented(key).unwrap().0;
        assert!(
            position(&workspace, "nav-data-footer")[1]
                < position(&workspace, "nav-profile-footer")[1]
        );

        let [x, y, w, _] = position(&workspace, "sidebar-resize");
        workspace.pointer_down(x + w / 2.0, y + 20.0);
        workspace.pointer_move(10.0, y + 20.0);
        workspace.pointer_up(10.0, y + 20.0);
        workspace.frame(0.0);
        assert!(workspace.find("nav-projects").is_none());
        assert!(workspace.find("projects").is_some());

        let [x, y, w, _] = position(&workspace, "sidebar-resize");
        workspace.pointer_down(x + w / 2.0, y + 20.0);
        workspace.pointer_move(230.0, y + 20.0);
        workspace.pointer_up(230.0, y + 20.0);
        workspace.frame(0.0);
        assert!(workspace.find("nav-projects").is_some());

        workspace.set_viewport((1000.0, 720.0), 1.0);
        workspace.frame(0.0);
        workspace.frame(1.0 / 60.0);
        assert!(workspace.find("nav-projects").is_none());
        assert!(workspace.find("projects").is_some());
        assert!(workspace.find("t-workspace").is_some());

        let [x, y, w, _] = position(&workspace, "sidebar-resize");
        workspace.pointer_down(x + w / 2.0, y + 20.0);
        workspace.pointer_move(320.0, y + 20.0);
        workspace.pointer_up(320.0, y + 20.0);
        workspace.set_viewport((960.0, 640.0), 1.0);
        workspace.frame(0.0);
        assert_eq!(position(&workspace, "workspace-sidebar")[2], 296.0);

        workspace.set_viewport((1080.0, 860.0), 2.0);
        workspace.frame(0.0);
        workspace.frame(1.0 / 60.0);
        assert!(workspace.find("nav-projects").is_some());
        let wide_sidebar = position(&workspace, "workspace-sidebar")[2];

        workspace.set_viewport((1079.5, 860.0), 2.0);
        workspace.frame(0.0);
        workspace.frame(1.0 / 60.0);
        assert!(workspace.find("nav-projects").is_none());
        workspace.update(|state, _| state.sidebar.toggle());
        workspace.frame(0.0);
        assert!(workspace.find("nav-projects").is_some());

        workspace.set_viewport((1280.0, 860.0), 2.0);
        workspace.frame(0.0);
        workspace.frame(1.0 / 60.0);
        assert_eq!(position(&workspace, "workspace-sidebar")[2], wide_sidebar);
    }

    #[test]
    fn chats_preserve_workspace_selection_and_the_floating_panel_leaves_send_visible() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        seed_demo(&mut state);
        let first = state.project_form.editing.clone().unwrap();
        let other_folder = tempfile::tempdir().unwrap();
        state.project_form = ProjectForm::empty(Some(other_folder.path().to_path_buf()));
        state.project_form.name = edit("Second project");
        state.project_form.goals = edit("Keep its chat separate");
        state.save_project();
        let second = state.project_form.editing.clone().unwrap();
        state.load_project_form(&first);
        state.view = View::Projects;
        state.agent_detection.check = Check::Finished;
        state.new_chat(&second);
        let chat_id = state.active_chat().unwrap().id.clone();
        state.chat_draft = edit("Draft only for this chat");
        state.set_agent_dock(AgentDock::Floating);
        let mut app = Runtime::new(
            state,
            view,
            (960.0, 640.0),
            Env {
                system_dark: false,
                scale: 1.0,
            },
        );
        app.frame(0.0);
        app.frame(1.0 / 60.0);
        let [_, panel_y, _, panel_height] = app.presented("agent-panel").unwrap().0;
        let [_, send_y, _, _] = app.presented("send-message").unwrap().0;
        assert!(panel_y + panel_height < send_y);
        assert!(app.find(&format!("t-{chat_id}")).is_some());

        app.update(|state, _| state.show_workspace());
        assert_eq!(
            app.state
                .workspace
                .as_ref()
                .unwrap()
                .selected_project_id
                .as_deref(),
            Some(first.as_str())
        );
        assert_eq!(
            app.state.project_form.editing.as_deref(),
            Some(first.as_str())
        );
        app.update(|state, _| state.open_chat(&chat_id));
        assert_eq!(app.state.chat_draft.text, "Draft only for this chat");
        app.update(|state, _| state.new_chat(&first));
        assert!(app.state.chat_draft.text.is_empty());
        app.update(|state, _| state.close_chat(&chat_id));
        assert!(app.state.chats.chat(&chat_id).is_some());
        assert!(!app.state.chats.open.contains(&chat_id));
        assert!(CoderuneApp::init().chats.chat(&chat_id).is_some());
    }

    #[test]
    fn settings_opens_over_the_current_page_and_groups_storage_actions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        scene_projects(&mut state);
        let export_path = dir.path().join("export.json");
        let mut app = Runtime::new(
            state,
            view,
            (1280.0, 860.0),
            Env {
                system_dark: false,
                scale: 1.0,
            },
        );
        app.frame(0.0);

        let click = |app: &mut Runtime<CoderuneApp>, key| {
            let [x, y, w, h] = app.presented(key).unwrap().0;
            app.pointer_down(x + w / 2.0, y + h / 2.0);
            app.pointer_up(x + w / 2.0, y + h / 2.0);
            for _ in 0..24 {
                app.frame(1.0 / 60.0);
            }
        };
        assert!(app.find("settings-shortcut").is_none());
        click(&mut app, "nav-data-footer");
        assert!(app.find("settings-dialog").is_some());
        assert!(app.find("check-updates").is_some());
        assert!(app.find("projects").is_some());
        assert!(!app.state.dark_mode());
        click(&mut app, "dark-mode");
        assert!(app.state.dark_mode());
        assert!(CoderuneApp::init().dark_mode());
        click(&mut app, "settings-shortcuts");
        assert!(app.find("navigation-shortcuts").is_some());
        assert!(app.find("window-shortcuts").is_some());
        for (_, view) in VIEW_SHORTCUTS {
            assert!(app.find(view.label()).is_some());
        }

        click(&mut app, "settings-storage");
        for key in ["open-data-folder", "export-data", "delete-data"] {
            assert!(app.find(key).is_some(), "missing storage action {key}");
        }
        assert!(app.find("check-updates").is_none());

        click(&mut app, "export-data");
        for _ in 0..8 {
            app.frame(1.0 / 60.0);
        }
        app.resolve_file_dialog(Some(export_path.clone())).unwrap();
        for _ in 0..8 {
            app.frame(1.0 / 60.0);
        }
        let exported: serde_json::Value =
            serde_json::from_slice(&std::fs::read(export_path).unwrap()).unwrap();
        assert_eq!(exported["projects"][0]["name"], "Coderune");

        click(&mut app, "delete-data");
        assert!(app.find("cancel-delete-data").is_some());
        click(&mut app, "delete-data");
        assert!(app.find("confirm-delete-data").is_some());
        click(&mut app, "cancel-delete-data");
        assert!(app.find("cancel-delete-data").is_none());

        click(&mut app, "close");
        assert!(app.find("settings-dialog").is_none());
        assert!(app.find("projects").is_some());

        click(&mut app, "nav-data-footer");
        click(&mut app, "settings-storage");
        click(&mut app, "delete-data");
        click(&mut app, "confirm-delete-data");
        assert!(app.find("settings-dialog").is_none());
        assert!(app.find("stage-chip-EarlyCareer").is_some());
    }

    #[test]
    fn settings_opens_install_instructions_only_for_missing_clis() {
        use coderune_agents::{CliAgentId, CliAgentStatus};

        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CODERUNE_DATA_DIR", dir.path());
        let mut state = CoderuneApp::init();
        scene_home(&mut state);
        state.agent_detection.results = vec![
            CliAgentStatus {
                id: CliAgentId::Codex,
                name: "Codex",
                command: "codex",
                install_url: "https://learn.chatgpt.com/docs/codex/cli",
                executable: Some(PathBuf::from("/opt/homebrew/bin/codex")),
            },
            CliAgentStatus {
                id: CliAgentId::OpenCode,
                name: "OpenCode",
                command: "opencode",
                install_url: "https://opencode.ai/docs/",
                executable: None,
            },
        ];
        state.agent_detection.check = Check::Finished;
        state.document_form.agent = Some(AgentId::Codex);
        state.select_settings(SettingsSection::Agents);
        let mut app = Runtime::new(state, view, (960.0, 640.0), Env::new(false, 1.0));
        for _ in 0..24 {
            app.frame(1.0 / 60.0);
        }

        assert!(app.find("recheck-agents").is_some());
        assert!(app.find("install-codex").is_none());
        assert!(app.find("agent-switch-opencode").is_none());
        let [x, y, w, h] = app.presented("agent-switch-codex").unwrap().0;
        app.pointer_down(x + w / 2.0, y + h / 2.0);
        app.pointer_up(x + w / 2.0, y + h / 2.0);
        app.frame(1.0 / 60.0);
        assert!(!app.state.agent_enabled(CliAgentId::Codex));
        assert_eq!(app.state.document_form.agent, None);
        assert!(!CoderuneApp::init().agent_enabled(CliAgentId::Codex));

        let [x, y, w, h] = app.presented("install-opencode").unwrap().0;
        app.pointer_down(x + w / 2.0, y + h / 2.0);
        app.pointer_up(x + w / 2.0, y + h / 2.0);
        app.frame(1.0 / 60.0);
        assert_eq!(app.take_url_requests(), ["https://opencode.ai/docs/"]);
    }
}
