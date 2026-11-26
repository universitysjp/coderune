//! Installed and available coding-agent CLIs in Settings.

use std::sync::{Arc, LazyLock};

use coderune_agents::chat::ChatProvider;
use coderune_agents::{CliAgentId, CliAgentStatus};
use coderune_ui::components::{self as ui, PushButtonExt};
use coderune_ui::theme;
use rune::{col, el, row, text, El, ImageCache, SurfaceFit};
use rune_kit::{Button, Kit, Switch};

use crate::agent_detection::Check;
use crate::app::CoderuneApp;
use crate::widgets::small_secondary_tx;

pub(super) fn coding_agents(s: &CoderuneApp, k: &Kit) -> El {
    let detection = &s.agent_detection;
    let checking = matches!(detection.check, Check::Checking(_));
    let checking_versions = detection.checking_versions();
    let installed = detection
        .results
        .iter()
        .filter(|agent| agent.executable.is_some());
    let missing = detection
        .results
        .iter()
        .filter(|agent| agent.executable.is_none());
    let mut body = col().w_full().gap(12.0).child(
        row()
            .w_full()
            .items_center()
            .justify_between()
            .child(heading("Installed agents"))
            .child(
                Button::new("Check again")
                    .disabled(checking || checking_versions)
                    .on_press(|s: &mut CoderuneApp| s.agent_detection.refresh())
                    .build_push(k)
                    .key("recheck-agents"),
            ),
    );
    if checking {
        body = body.child(ui::subtle("Checking for coding agents…"));
    } else if checking_versions {
        body = body.child(ui::subtle("Checking versions and updates…"));
    }
    if matches!(detection.check, Check::Failed) {
        body = body.child(ui::alert(
            None,
            "Could not check for coding agents. Try again.",
        ));
    }
    if detection.results.is_empty() {
        return body;
    }
    if installed.clone().next().is_some() {
        body = body.child(ui::settings_card(
            k,
            "installed-list",
            installed.map(|agent| installed_agent(s, agent, k)),
        ));
    } else {
        body = body.child(ui::subtle("No coding agents found."));
    }
    if missing.clone().next().is_some() {
        body = body
            .child(heading("Not installed"))
            .child(ui::settings_card(
                k,
                "missing-list",
                missing.map(|agent| missing_agent(agent, k)),
            ));
    }
    body
}

fn heading(label: &str) -> El {
    text(label).font(theme::HEADING_XS).color(theme::ink())
}

fn installed_agent(s: &CoderuneApp, agent: &CliAgentStatus, k: &Kit) -> El {
    let id = agent.id;
    let enabled = s.agent_enabled(id);
    let status = if !enabled {
        "Off"
    } else if ChatProvider::for_agent(id).is_some() {
        "Ready"
    } else {
        "CLI only"
    };
    let version = s.agent_detection.version(id);
    let detail = version.map(|info| format!("Version {}", info.installed));
    let toggle = Switch::new(enabled)
        .label(format!("Use {}", agent.name))
        .hide_label()
        .on_toggle(move |s: &mut CoderuneApp| s.set_agent_enabled(id, !s.agent_enabled(id)))
        .build(theme::switch_kit(k.dark))
        .min_h(28.0)
        .key(format!("agent-switch-{}", agent.command));
    let mut actions = row().items_center().gap(8.0);
    if let Some(update) = version.and_then(|info| info.update.as_ref()) {
        actions = actions.child(agent_link(
            &format!("{} available", update.version),
            format!("Update instructions for {} {}", agent.name, update.version),
            format!("update-{}", agent.command),
            update.instructions,
            k,
        ));
    }
    agent_row(
        agent,
        Some(status),
        detail.as_deref(),
        actions.child(toggle),
        k,
    )
}

fn missing_agent(agent: &CliAgentStatus, k: &Kit) -> El {
    let install = agent_link(
        "Install from website",
        format!("Install {} from website", agent.name),
        format!("install-{}", agent.command),
        agent.install_url,
        k,
    );
    agent_row(agent, None, None, install, k)
}

fn agent_link(title: &str, label: String, key: String, url: &'static str, k: &Kit) -> El {
    small_secondary_tx(
        title,
        move |s, tx| {
            if let Err(error) = tx.open_url(url) {
                s.settings_feedback = Some(Err(coderune_domain::AppError::new(
                    coderune_domain::ErrorCode::Validation,
                    format!("Could not open the agent website: {error}"),
                )));
            }
        },
        k,
    )
    .key(key)
    .role("link")
    .label(label)
}

fn agent_row(
    agent: &CliAgentStatus,
    status: Option<&str>,
    detail: Option<&str>,
    action: El,
    k: &Kit,
) -> El {
    let mut name = row().items_center().min_w(0.0).gap(8.0).child(
        text(agent.name)
            .font(theme::LABEL)
            .color(theme::ink())
            .truncate(),
    );
    if let Some(status) = status {
        name = name.child(ui::tiny(status));
    }
    let mut description = col().grow(1.0).min_w(0.0).gap(2.0).child(name);
    if let Some(detail) = detail {
        description = description.child(ui::tiny(detail).truncate());
    }
    row()
        .w_full()
        .h(48.0)
        .gap(12.0)
        .items_center()
        .child(logo(agent.id, k.dark))
        .child(description)
        .child(action)
}

pub(crate) fn logo(agent: CliAgentId, dark: bool) -> El {
    row()
        .size(32.0, 32.0)
        .radius(7.0)
        .border(1.0, theme::row_line())
        .center()
        .a11y_hidden()
        .child(agent_mark(agent, dark, 20.0))
}

pub(crate) fn compact_logo(agent: CliAgentId, dark: bool) -> El {
    agent_mark(agent, dark, 16.0).a11y_hidden()
}

fn agent_mark(agent: CliAgentId, dark: bool, size: f32) -> El {
    use CliAgentId::*;
    static IMAGES: LazyLock<ImageCache> = LazyLock::new(|| ImageCache::new(Arc::new(|| {})));
    let svg: &'static [u8] = match (agent, dark) {
        (ClaudeCode, _) => include_bytes!("../../assets/agent-logos/claude-code.svg"),
        (Codex, false) => include_bytes!("../../assets/agent-logos/codex_light.svg"),
        (Codex, true) => include_bytes!("../../assets/agent-logos/codex_dark.svg"),
        (Grok, false) => include_bytes!("../../assets/agent-logos/grok-light.svg"),
        (Grok, true) => include_bytes!("../../assets/agent-logos/grok-dark.svg"),
        (OpenCode, false) => include_bytes!("../../assets/agent-logos/opencode.svg"),
        (OpenCode, true) => include_bytes!("../../assets/agent-logos/opencode-dark.svg"),
        (Pi, false) => include_bytes!("../../assets/agent-logos/pi-light.svg"),
        (Pi, true) => include_bytes!("../../assets/agent-logos/pi-dark.svg"),
        (Amp, _) => include_bytes!("../../assets/agent-logos/amp.svg"),
        (Cursor, false) => include_bytes!("../../assets/agent-logos/cursor_light.svg"),
        (Cursor, true) => include_bytes!("../../assets/agent-logos/cursor_dark.svg"),
        (Antigravity, _) => include_bytes!("../../assets/agent-logos/antigravity.svg"),
        (Kiro, false) => include_bytes!("../../assets/agent-logos/kiro.svg"),
        (Kiro, true) => include_bytes!("../../assets/agent-logos/kiro-dark.svg"),
        (Muse, _) => include_bytes!("../../assets/agent-logos/muse.svg"),
        (Devin, _) => include_bytes!("../../assets/agent-logos/devin.svg"),
        (Fx, false) => include_bytes!("../../assets/agent-logos/fx.svg"),
        (Fx, true) => include_bytes!("../../assets/agent-logos/fx-dark.svg"),
        (Jcode, false) => include_bytes!("../../assets/agent-logos/jcode.svg"),
        (Jcode, true) => include_bytes!("../../assets/agent-logos/jcode-dark.svg"),
    };
    let image = IMAGES.svg(svg, size, 3.0);
    el().size(size, size)
        .pixels_fit(image.pixels(), SurfaceFit::Fit)
}
