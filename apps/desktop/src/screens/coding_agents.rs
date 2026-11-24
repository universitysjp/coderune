//! Installed and available coding-agent CLIs in Settings.

use std::sync::{Arc, LazyLock};

use coderune_agents::{CliAgentId, CliAgentStatus};
use coderune_ui::components::{self as ui, PushButtonExt};
use coderune_ui::theme;
use rune::{col, el, row, text, El, ImageCache, SurfaceFit};
use rune_kit::{Button, Kit, Switch};

use crate::agent_detection::Check;
use crate::app::CoderuneApp;

pub(super) fn coding_agents(s: &CoderuneApp, k: &Kit) -> El {
    let detection = &s.agent_detection;
    let checking = matches!(detection.check, Check::Checking(_));
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
                    .disabled(checking)
                    .on_press(|s: &mut CoderuneApp| s.agent_detection.refresh())
                    .build_push(k)
                    .key("recheck-agents"),
            ),
    );
    if checking {
        body = body.child(ui::subtle("Checking for coding agents…"));
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
        body = body.child(
            col()
                .w_full()
                .children(installed.map(|agent| installed_agent(s, agent, k))),
        );
    } else {
        body = body.child(ui::subtle("No coding agents found."));
    }
    if missing.clone().next().is_some() {
        body = body.child(heading("Not installed")).child(
            col()
                .w_full()
                .children(missing.map(|agent| missing_agent(agent, k))),
        );
    }
    body
}

fn heading(label: &str) -> El {
    text(label).font(theme::HEADING_XS).color(theme::ink())
}

fn installed_agent(s: &CoderuneApp, agent: &CliAgentStatus, k: &Kit) -> El {
    let id = agent.id;
    let enabled = s.agent_enabled(id);
    let toggle = Switch::new(enabled)
        .label(format!("Use {}", agent.name))
        .hide_label()
        .on_toggle(move |s: &mut CoderuneApp| s.set_agent_enabled(id, !s.agent_enabled(id)))
        .build(k)
        .min_h(28.0)
        .key(format!("agent-switch-{}", agent.command));
    agent_row(
        agent,
        Some(if enabled { "Ready" } else { "Off" }),
        toggle,
        k,
    )
}

fn missing_agent(agent: &CliAgentStatus, k: &Kit) -> El {
    let url = agent.install_url;
    let install = Button::new("Install from website")
        .on_press_tx(move |s: &mut CoderuneApp, tx| {
            if let Err(error) = tx.open_url(url) {
                s.settings_feedback = Some(Err(coderune_domain::AppError::new(
                    coderune_domain::ErrorCode::Validation,
                    format!("Could not open the installation link: {error}"),
                )));
            }
        })
        .build_small_push(k)
        .key(format!("install-{}", agent.command))
        .role("link")
        .label(format!("Install {} from website", agent.name));
    agent_row(agent, None, install, k)
}

fn agent_row(agent: &CliAgentStatus, status: Option<&str>, action: El, k: &Kit) -> El {
    let mut name = row().items_center().min_w(0.0).gap(8.0).child(
        text(agent.name)
            .font(theme::LABEL)
            .color(theme::ink())
            .truncate(),
    );
    if let Some(status) = status {
        name = name.child(ui::tiny(status));
    }
    let entry = row()
        .w_full()
        .h(48.0)
        .gap(12.0)
        .items_center()
        .child(logo(agent, k.dark))
        .child(name.grow(1.0))
        .child(action);
    col()
        .w_full()
        .child(entry)
        .child(el().w_full().h(1.0).fill(theme::row_line()))
}

fn logo(agent: &CliAgentStatus, dark: bool) -> El {
    use CliAgentId::*;
    static IMAGES: LazyLock<ImageCache> = LazyLock::new(|| ImageCache::new(Arc::new(|| {})));
    const SIZE: f32 = 20.0;
    let svg: Option<&'static [u8]> = match (agent.id, dark) {
        (ClaudeCode, _) => Some(include_bytes!("../../assets/agent-logos/claude-code.svg")),
        (Codex, false) => Some(include_bytes!("../../assets/agent-logos/codex_light.svg")),
        (Codex, true) => Some(include_bytes!("../../assets/agent-logos/codex_dark.svg")),
        (Grok, false) => Some(include_bytes!("../../assets/agent-logos/grok-light.svg")),
        (Grok, true) => Some(include_bytes!("../../assets/agent-logos/grok-dark.svg")),
        (OpenCode, false) => Some(include_bytes!("../../assets/agent-logos/opencode.svg")),
        (OpenCode, true) => Some(include_bytes!("../../assets/agent-logos/opencode-dark.svg")),
        (Pi, false) => Some(include_bytes!("../../assets/agent-logos/pi-light.svg")),
        (Pi, true) => Some(include_bytes!("../../assets/agent-logos/pi-dark.svg")),
        (Amp, _) => Some(include_bytes!("../../assets/agent-logos/amp.svg")),
        (Cursor, false) => Some(include_bytes!("../../assets/agent-logos/cursor_light.svg")),
        (Cursor, true) => Some(include_bytes!("../../assets/agent-logos/cursor_dark.svg")),
        (Antigravity, _) => Some(include_bytes!("../../assets/agent-logos/antigravity.svg")),
        (Kiro | Muse | Devin | Fx | Jcode, _) => None,
    };
    let mark = if let Some(svg) = svg {
        let image = IMAGES.svg(svg, SIZE, 3.0);
        el().size(SIZE, SIZE)
            .pixels_fit(image.pixels(), SurfaceFit::Fit)
    } else {
        text(&agent.name[..1])
            .font(theme::LABEL)
            .color(theme::muted())
    };
    row()
        .size(32.0, 32.0)
        .radius(7.0)
        .border(1.0, theme::row_line())
        .center()
        .a11y_hidden()
        .child(mark)
}
