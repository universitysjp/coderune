//! The window's root view, keyboard shortcuts, and frame poll.

use coderune_ui::theme::{self, BODY, HEADING};
use rune::{col, text, Cx, El, Key, KeyEvent, Tx};
use rune_kit::shell::is_sidebar_toggle;

use crate::agent_detection::Check;
use crate::app::{CoderuneApp, SettingsSection, View, VIEW_SHORTCUTS};
use crate::screens;
use crate::shell;

/// Advances visible animations and applies completed background updates.
pub fn poll(s: &mut CoderuneApp, t: f64, tx: &mut Tx) -> bool {
    let families = s.font_families();
    if s.applied_fonts.as_ref() != Some(&families) {
        tx.set_font_families(&families.0, &families.1);
        s.applied_fonts = Some(families);
    }
    let update_changed = s.updates.poll(tx);
    let detection_changed = s.agent_detection.poll(tx);
    let chat_changed = s.poll_chats(tx);
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

pub fn view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    theme::with_dark(s.dark_mode(), || themed_view(s, cx))
}

fn themed_view(s: &CoderuneApp, cx: &mut Cx<CoderuneApp>) -> El {
    let k = theme::kit();
    let inset = cx.titlebar_inset();
    let mut overlays = Vec::new();
    let body = if let Some(message) = &s.fatal {
        fatal(message, inset)
    } else {
        shell::workspace(s, k, cx, &mut overlays)
    };
    let base = col().w_full().h_full().fill(theme::bg()).child(body);
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
            tx.quit();
            return true;
        }
        Key::Command(',' | '7') if s.workspace.is_some() => {
            if s.settings.is_none() {
                s.select_settings(SettingsSection::Preferences);
                tx.instant();
            }
            return true;
        }
        _ if s.settings.is_some() => {
            if event.key == Key::Escape {
                s.close_settings();
                tx.instant();
                return true;
            }
            return false;
        }
        _ => {}
    }
    let terminal_shortcut = event.is_command('j')
        || (event.mods.ctrl
            && matches!(&event.key, Key::Text(key) if key.eq_ignore_ascii_case("j")));
    if terminal_shortcut && !event.mods.alt && !event.mods.shift && s.workspace.is_some() {
        s.open_terminal();
        tx.instant();
        return true;
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
