//! Onboarding as one dialog over the workspace.
//!
//! The dialog asks for the two answers every project needs — a display name and
//! an experience stage. The starting-point questions are there too, but leaving
//! them empty is fine: education, employment, links and the rest live in the
//! Profile panel, exactly where the project form leaves its optional fields.

use crate::app::CoderuneApp;
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, BODY_SM, LABEL_SM};
use rune::{col, el, row, text, El, ScrollSpec};
use rune_kit::Kit;

pub(crate) fn dialog(s: &CoderuneApp, k: &Kit, overlays: &mut Vec<El>) {
    if !s.onboarding_visible || !s.onboarding_required() {
        return;
    }
    let mut body = col()
        .w_full()
        .gap(16.0)
        .child(ui::field(
            "Display name or pseudonym",
            Some("This labels your baseline. A pseudonym is fine."),
            text_input(
                &s.profile_form.display_name,
                "Your name or pseudonym",
                "Display name",
                |s| &mut s.profile_form.display_name,
                k,
            )
            .key("onboarding-name"),
        ))
        .child(ui::field(
            "Where are you starting from?",
            Some("Self-reported. Coderune never treats it as verified mastery."),
            stage_chips(s),
        ))
        .child(ui::field(
            "What can you do today?",
            Some("A short, honest statement. If you are new, mark it below instead."),
            text_area(
                &s.profile_form.skills,
                "For example: I write TypeScript for small web apps.",
                "Initial skills statement",
                2,
                |s| &mut s.profile_form.skills,
                k,
            ),
        ))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .items_center()
                .child(option_chip(
                    "onboarding-new-to-coding".to_string(),
                    "I am new to coding",
                    s.profile_form.new_to_coding,
                    |s: &mut CoderuneApp| {
                        s.profile_form.new_to_coding = !s.profile_form.new_to_coding
                    },
                ))
                .child(
                    text("No skills need inventing.")
                        .font(BODY_SM)
                        .color(theme::muted()),
                ),
        )
        .child(ui::field(
            "What do you want to learn?",
            Some("At least one goal. Separate several goals with commas."),
            text_area(
                &s.profile_form.goals,
                "For example: Ship a native desktop app, learn Rust",
                "Learning goals",
                2,
                |s| &mut s.profile_form.goals,
                k,
            ),
        ))
        .child(
            text(
                "Not finished? Later in Profile opens the same answers, and education, employment, links and the rest are there too.",
            )
            .font(LABEL_SM)
            .color(theme::faint())
            .wrap(),
        );
    body = body.children(status_alerts(s));

    let actions = row()
        .w_full()
        .gap(8.0)
        .items_center()
        .px(20.0)
        .py(14.0)
        .child(ghost("Save draft", |s| s.save_draft(), k).key("onboarding-save-draft"))
        .child(ghost("Later in Profile", |s| s.dismiss_onboarding(), k).key("onboarding-later"))
        .child(el().grow(1.0))
        .child(
            primary("Confirm baseline", |s| s.confirm_onboarding(), k).key("onboarding-confirm"),
        );

    let panel = col()
        .w_full()
        .gap(16.0)
        .padding(20.0)
        .child(text("Welcome to Coderune").font(theme::HEADING).color(theme::ink()))
        .child(ui::subtle(
            "Two answers are enough to start. Coderune keeps one developer profile shared across your projects.",
        ))
        .child(body);

    let dialog = ui::surface_frame(k, theme::surface(), k.radius.xl)
        .key("onboarding-dialog")
        .role("dialog")
        .label("Developer baseline")
        .w_full()
        .max_w(560.0)
        .h(560.0)
        .child(
            col()
                .w_full()
                .h_full()
                .min_h(0.0)
                .child(
                    col()
                        .key("onboarding-body")
                        .w_full()
                        .grow(1.0)
                        .min_h(0.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .scroll_edge_fade(16.0)
                        .child(panel),
                )
                .child(actions),
        )
        .modal(true, CoderuneApp::dismiss_onboarding)
        .token_motion(k.motion.overlay);
    let layer = |key| el().key(key).absolute(0.0, 0.0).w_full().h_full();
    overlays.extend([
        layer("onboarding-scrim")
            .fill(k.c.scrim)
            .a11y_hidden()
            .token_motion(k.motion.overlay),
        layer("onboarding-layer")
            .padding(16.0)
            .center()
            .child(dialog),
    ]);
}
