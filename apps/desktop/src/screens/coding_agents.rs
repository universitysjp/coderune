//! Installed and available coding-agent CLIs in Settings.

use std::sync::{Arc, LazyLock};

use coderune_agents::{CliAgentId, CliAgentStatus};
use coderune_ui::{components as ui, theme};
use rune::{col, el, row, text, El, ImageCache, SurfaceFit};
use rune_kit::{Button, Kit};

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
                    .build(k)
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
                .children(installed.map(|agent| agent_row(agent, k))),
        );
    } else {
        body = body.child(ui::subtle("No coding agents found."));
    }
    if missing.clone().next().is_some() {
        body = body.child(heading("Not installed")).child(
            col()
                .w_full()
                .children(missing.map(|agent| agent_row(agent, k))),
        );
    }
    body
}

fn heading(label: &str) -> El {
    text(label).font(theme::HEADING_XS).color(theme::ink())
}

fn agent_row(agent: &CliAgentStatus, k: &Kit) -> El {
    let detail = agent.executable.as_ref().map_or_else(
        || agent.command.to_string(),
        |path| path.display().to_string(),
    );
    let mut entry = row()
        .w_full()
        .h(48.0)
        .gap(12.0)
        .items_center()
        .child(logo(agent))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(2.0)
                .child(text(agent.name).font(theme::LABEL).color(theme::ink()))
                .child(
                    text(detail)
                        .font(theme::CODE_SM)
                        .color(theme::faint())
                        .truncate(),
                ),
        );
    if agent.executable.is_none() {
        let url = agent.install_url;
        entry = entry.child(
            Button::new("Install from website")
                .on_press_tx(move |s: &mut CoderuneApp, tx| {
                    if let Err(error) = tx.open_url(url) {
                        s.settings_feedback = Some(Err(coderune_domain::AppError::new(
                            coderune_domain::ErrorCode::Validation,
                            format!("Could not open the installation link: {error}"),
                        )));
                    }
                })
                .build(k)
                .key(format!("install-{}", agent.command))
                .role("link")
                .label(format!("Install {} from website", agent.name)),
        );
    }
    col()
        .w_full()
        .child(entry)
        .child(el().w_full().h(1.0).fill(theme::row_line()))
}

fn logo(agent: &CliAgentStatus) -> El {
    use CliAgentId::*;
    static IMAGES: LazyLock<ImageCache> = LazyLock::new(|| ImageCache::new(Arc::new(|| {})));
    const SIZE: f32 = 20.0;
    let svg: Option<&'static [u8]> = match agent.id {
        ClaudeCode => Some(include_bytes!("../../assets/agent-logos/claude-code.svg")),
        Codex => Some(include_bytes!("../../assets/agent-logos/codex_light.svg")),
        Grok => Some(include_bytes!("../../assets/agent-logos/grok-light.svg")),
        OpenCode => Some(include_bytes!("../../assets/agent-logos/opencode.svg")),
        Pi => Some(include_bytes!("../../assets/agent-logos/pi-light.svg")),
        Amp => Some(include_bytes!("../../assets/agent-logos/amp.svg")),
        Cursor => Some(include_bytes!("../../assets/agent-logos/cursor_light.svg")),
        Antigravity => Some(include_bytes!("../../assets/agent-logos/antigravity.svg")),
        Kiro | Muse | Devin | Fx | Jcode => None,
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
