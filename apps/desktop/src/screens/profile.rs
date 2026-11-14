//! The profile screen.

#![allow(unused_imports)]

use crate::app::{CoderuneApp, View};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::icons;
use coderune_ui::theme::{self, BODY, BODY_SM, CODE_SM, HEADING, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, Cx, El, ScrollSpec};
use rune_kit::{Badge, Button, ButtonVariant, ControlSize, Kit, TextInput};

pub(crate) fn profile(s: &CoderuneApp, k: &Kit) -> El {
    let current = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile());
    let mut body = col().w_full().gap(24.0).child(ui::page_header(
        "Profile",
        "The reusable developer profile shared across every project.",
        vec![],
    ));
    if let Some(profile) = current {
        body = body.child(ui::card(
            Some("Current baseline"),
            Some("Edits create a new immutable version. Existing captures keep their snapshot."),
            vec![
                row()
                    .gap(8.0)
                    .items_center()
                    .child(
                        text(&profile.display_name)
                            .font(HEADING_XS)
                            .color(theme::ink()),
                    )
                    .child(ui::badge(&format!("v{}", profile.version), k))
                    .child(ui::badge(profile.experience_stage.label(), k)),
                ui::field(
                    "Skills",
                    None,
                    text(&profile.skills_statement)
                        .font(BODY)
                        .color(theme::muted())
                        .wrap(),
                ),
                ui::field(
                    "Learning goals",
                    None,
                    col()
                        .gap(4.0)
                        .children(profile.learning_goals.iter().map(|goal| {
                            text(format!("· {goal}"))
                                .font(BODY_SM)
                                .color(theme::muted())
                        })),
                ),
                ui::tiny(&format!(
                    "{} self-reported skill(s) · {} education entr(ies) · {} employment entr(ies)",
                    profile.self_reported_skills.len(),
                    profile.education.len(),
                    profile.employment.len()
                )),
            ],
        ));
    }
    let mut form_children = vec![
        ui::field(
            "Display name",
            Some("A name or pseudonym."),
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
            Some("Self-reported baseline."),
            stage_chips(s),
        ),
        ui::field(
            "Initial skills statement",
            Some("New to coding is a valid answer."),
            text_input(
                &s.profile_form.skills,
                "What can you do today?",
                "Initial skills statement",
                |s| &mut s.profile_form.skills,
                k,
            ),
        ),
        ui::field(
            "Learning goals",
            Some("Separate several goals with commas."),
            text_input(
                &s.profile_form.goals,
                "What do you want to learn?",
                "Learning goals",
                |s| &mut s.profile_form.goals,
                k,
            ),
        ),
    ];
    if let Some(error) = &s.error {
        form_children.push(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        form_children.push(ui::alert(None, notice));
    }
    form_children.push(
        row()
            .gap(8.0)
            .child(option_chip(
                "profile-new-to-coding".to_string(),
                "I am new to coding",
                s.profile_form.new_to_coding,
                |s: &mut CoderuneApp| s.profile_form.new_to_coding = !s.profile_form.new_to_coding,
            ))
            .child(primary("Save new version", |s| s.save_profile(), k)),
    );
    body.child(ui::card(
        Some("Edit profile"),
        Some("Required fields stay required. Self-reported skills are never presented as verified mastery."),
        form_children,
    ))
}
