//! Coderune's native desktop application.

pub(crate) mod actions;
pub(crate) mod root;
pub(crate) mod scenes;

pub(crate) use root::{on_key, poll, view};

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use coderune_app::{PreviewSession, Workspace};
use coderune_avatars::BotAvatarStage;
#[cfg(test)]
use coderune_domain::{AgentId, ExperienceStage};
use coderune_domain::{AppError, ContextCapture};
use rune::TextEdit;

use coderune_ui::icons;

use crate::agent_detection::AgentDetection;
use crate::chat_terminal::Terminal;
use crate::chats::Chats;
use crate::forms::{DocumentForm, ProfileForm, ProjectForm};
use crate::preferences::Preferences;
use crate::shell::SidebarPane;
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

    pub(crate) fn navigate(&mut self, view: View) {
        self.view = view;
        self.error = None;
        self.notice = None;
        self.confirm_delete_project = None;
        self.confirm_delete_capture = None;
        self.show_workspace();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_detection::Check;
    use crate::app::scenes::{scene_home, scene_projects, seed_demo};
    use crate::forms::edit;
    use crate::preferences::AgentDock;
    use rune::{Env, Runtime};

    /// Tests that point the app at a temporary data folder must not overlap.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
