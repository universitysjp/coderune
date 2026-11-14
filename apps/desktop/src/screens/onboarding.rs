//! The onboarding screen.

#![allow(unused_imports)]

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, BODY_SM, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, Cx, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit, TextInput};

pub(crate) fn onboarding(s: &CoderuneApp, k: &Kit) -> El {
    let mut form_children = vec![
        ui::field(
            "Display name",
            Some("A name or pseudonym. This is the start of your developer baseline."),
            text_input(
                &s.profile_form.display_name,
                "Your name or pseudonym",
                "Display name",
                |s| &mut s.profile_form.display_name,
                k,
            ),
        ),
        ui::field(
            "Experience stage",
            Some("Self-reported. It is never treated as verified mastery."),
            stage_chips(s),
        ),
        ui::field(
            "Initial skills statement",
            Some("A short honest statement. New to coding is a valid answer."),
            text_area(
                &s.profile_form.skills,
                "What can you do today?",
                "Initial skills statement",
                3,
                |s| &mut s.profile_form.skills,
                k,
            ),
        ),
        ui::field(
            "Learning goals",
            Some("At least one. Separate several goals with commas."),
            text_area(
                &s.profile_form.goals,
                "What do you want to learn?",
                "Learning goals",
                3,
                |s| &mut s.profile_form.goals,
                k,
            ),
        ),
        option_chip(
            "new-to-coding".to_string(),
            "I am new to coding",
            s.profile_form.new_to_coding,
            |s: &mut CoderuneApp| s.profile_form.new_to_coding = !s.profile_form.new_to_coding,
        ),
    ];
    if let Some(error) = &s.error {
        form_children.push(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        form_children.push(ui::alert(None, notice));
    }
    let actions = row()
        .gap(8.0)
        .child(ghost("Save draft", |s| s.save_draft(), k))
        .child(primary("Confirm baseline", |s| s.confirm_profile(), k));

    col()
        .w_full()
        .h_full()
        .scroll(ScrollSpec { stick_bottom: false })
        .fill(theme::bg())
        .padding(32.0)
        .items_center()
        .child(
            col()
                .w_full()
                .max_w(720.0)
                .gap(24.0)
                .child(
                    row()
                        .gap(12.0)
                        .items_center()
                        .child(
                            row()
                                .size(32.0, 32.0)
                                .center()
                                .radius(6.0)
                                .fill(theme::ink())
                                .child(text("C").font(HEADING_XS).color(theme::inverse())),
                        )
                        .child(text("Coderune").font(HEADING_XS).color(theme::ink())),
                )
                .child(ui::page_header(
                    "Set up your developer baseline",
                    "Coderune keeps local, reviewable context about you and your projects. Start with a short baseline. You can edit it later.",
                    vec![],
                ))
                .child(ui::card(None, None, form_children))
                .child(actions),
        )
}
