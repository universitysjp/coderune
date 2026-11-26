//! Headless fixture scenes for `--scene` renders and desktop verification.
//!
//! Scenes only seed or reshape `CoderuneApp`; they never read or write storage
//! outside the fixture helpers the app already uses.

use std::path::PathBuf;

use coderune_agents::{
    chat::{Effort, Model},
    CliAgentId, CliAgentStatus,
};
use coderune_domain::ExperienceStage;

use crate::agent_detection::Check;
use crate::app::{CoderuneApp, ProfileSection, ProfileTab, SettingsSection, View};
use crate::chats::{Author, Chats};
use crate::forms::{edit, ProjectForm};

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
pub fn scene_subagents(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Subagents;
}
pub fn scene_capture(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Capture;
}

/// The quick questions shown after a folder is picked.
pub fn scene_new_project(s: &mut CoderuneApp) {
    seed_demo(s);
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    s.start_new_project(&root);
}
pub fn scene_capture_preview(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Capture;
    s.preview_capture();
}
pub fn scene_settings(s: &mut CoderuneApp) {
    seed_demo(s);
    s.select_settings(SettingsSection::General);
}

pub fn scene_storage(s: &mut CoderuneApp) {
    seed_demo(s);
    s.select_settings(SettingsSection::Storage);
}

pub fn scene_shortcuts(s: &mut CoderuneApp) {
    seed_demo(s);
    s.select_settings(SettingsSection::Shortcuts);
}

pub fn scene_settings_agents(s: &mut CoderuneApp) {
    seed_demo(s);
    s.agent_detection.results = coderune_agents::detect_cli_agents();
    s.agent_detection.check = Check::Finished;
    s.select_settings(SettingsSection::Agents);
}

pub fn scene_chat(s: &mut CoderuneApp) {
    seed_demo(s);
    let project = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.selected_project_id.clone());
    if let Some(project) = project {
        s.chats = Chats::default();
        let chat = s.chats.new_chat(&project);
        let agent = CliAgentId::Codex;
        s.preferences.set_project_expanded(&project, true);
        s.agent_detection.results = vec![CliAgentStatus {
            id: agent,
            name: agent.label(),
            command: "codex",
            install_url: "",
            executable: Some(PathBuf::from("fixtures/codex")),
        }];
        s.agent_detection.check = Check::Finished;
        s.chats.set_agent(&chat, Some(agent));
        s.seed_chat_catalog(vec![Model {
            id: "example-model".into(),
            label: "Example model".into(),
            efforts: vec![Effort {
                id: "medium".into(),
                label: "Medium".into(),
            }],
            is_default: true,
        }]);
        s.chats.add_message(
            &chat,
            Author::Developer,
            "Which parts of this project should the agent read first?",
        );
        s.chats.add_message(
            &chat,
            Author::Agent(agent),
            "## Start here\n\n- Read the project goals and `AGENTS.md`\n- Open the app entry point\n- Run the tests for the area you plan to change\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```",
        );
    }
}

pub fn scene_chat_dark(s: &mut CoderuneApp) {
    scene_chat(s);
    s.preferences.dark_mode = true;
}

pub fn scene_settings_archived(s: &mut CoderuneApp) {
    scene_chat(s);
    if let Some(chat) = s.active_chat().map(|chat| chat.id.clone()) {
        s.archive_chat(&chat);
    }
    s.select_settings(SettingsSection::Archived);
}

pub fn scene_settings_fonts(s: &mut CoderuneApp) {
    scene_settings(s);
    s.font_menu = Some("ui-font");
}

pub fn scene_home_dark(s: &mut CoderuneApp) {
    scene_home(s);
    s.preferences.dark_mode = true;
}

pub fn scene_settings_dark(s: &mut CoderuneApp) {
    scene_settings(s);
    s.preferences.dark_mode = true;
}

pub fn scene_settings_agents_dark(s: &mut CoderuneApp) {
    scene_settings_agents(s);
    s.preferences.dark_mode = true;
}

/// The avatar gallery, a development scene reached with `--scene avatars`.
pub fn scene_avatars(s: &mut CoderuneApp) {
    seed_demo(s);
    s.view = View::Avatars;
}

/// Onboarding with only the two required answers filled in.
pub fn scene_onboarding_start(s: &mut CoderuneApp) {
    s.profile_form.display_name = edit("Ada");
    s.profile_form.stage = Some(ExperienceStage::EarlyCareer);
}

/// Onboarding with every answer filled in, dismissed into the Profile panel so
/// the same baseline can be finished there.
pub fn scene_onboarding_review(s: &mut CoderuneApp) {
    scene_onboarding_start(s);
    s.profile_form.skills = edit("TypeScript and a little Rust");
    s.profile_form.goals = edit("Ship the native desktop app, Learn Rune");
    s.dismiss_onboarding();
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
