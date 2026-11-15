//! The step-by-step onboarding screen.
//!
//! Onboarding asks two questions at a time: who you are, then where you are
//! starting from. Everything optional lives in Profile after the baseline is
//! confirmed.

use crate::app::{split_list, CoderuneApp, OnboardingStep};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, BODY_SM, CODE_SM, HEADING_XS, LABEL, LABEL_SM};
use coderune_ui::WhenExt;
use rune::{col, row, text, El, ScrollSpec};
use rune_kit::shell::Page;
use rune_kit::Kit;

pub(crate) fn onboarding(s: &CoderuneApp, k: &Kit, inset: f32) -> El {
    let step = s.onboarding_step;
    let body = match step {
        OnboardingStep::About => about_step(s, k),
        OnboardingStep::StartingPoint => starting_point_step(s, k),
        OnboardingStep::Review => review_step(s, k),
    };
    let mut actions = row().w_full().gap(8.0).items_center();
    if step.previous().is_some() {
        actions = actions.child(ghost("Back", |s| s.onboarding_back(), k));
    }
    actions = actions
        .child(rune::el().grow(1.0))
        .child(ghost("Save draft", |s| s.save_draft(), k))
        .child(primary(
            if step == OnboardingStep::Review {
                "Confirm baseline"
            } else {
                "Continue"
            },
            |s| s.onboarding_next(),
            k,
        ));

    let mut content = col()
        .w_full()
        .max_w(640.0)
        .gap(20.0)
        .child(
            col()
                .gap(6.0)
                .child(text(step.title()).font(theme::HEADING).color(theme::ink()))
                .child(
                    text(step.description())
                        .font(BODY_SM)
                        .color(theme::muted())
                        .wrap(),
                ),
        )
        .child(steps(s, k))
        .child(body)
        .child(actions)
        .child(
            text("Local only. Nothing leaves this computer unless you start an agent draft.")
                .font(LABEL_SM)
                .color(theme::faint())
                .wrap(),
        );
    if let Some(error) = &s.error {
        content = content.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        content = content.child(ui::alert(None, notice));
    }

    Page::new(
        col()
            .w_full()
            .h_full()
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .padding(32.0)
            .items_center()
            .child(content),
    )
    .title("Coderune")
    .titlebar_inset(inset)
    .toolbar_in_panel()
    .frame_inset(theme::FRAME)
    .build(k)
}

/// The three-step progress line. It is a map, not a control: Back and Continue
/// are the only ways to move, so a step with missing answers cannot be skipped.
fn steps(s: &CoderuneApp, k: &Kit) -> El {
    let mut row_ = row().w_full().gap(8.0).items_center();
    for (index, step) in OnboardingStep::ALL.into_iter().enumerate() {
        if index > 0 {
            row_ = row_.child(rune::el().h(1.0).grow(1.0).fill(theme::row_line()));
        }
        let current = step == s.onboarding_step;
        let done = step.index() < s.onboarding_step.index();
        row_ = row_.child(
            row()
                .key(format!("onboarding-step-{}", step.index()))
                .h(26.0)
                .px(8.0)
                .gap(6.0)
                .items_center()
                .radius(6.0)
                .when(current, |this| this.fill(theme::accent_soft()))
                .child(
                    row()
                        .size(16.0, 16.0)
                        .center()
                        .radius(8.0)
                        .fill(if current || done {
                            theme::ink()
                        } else {
                            theme::code()
                        })
                        .child(text((index + 1).to_string()).font(LABEL_SM).color(
                            if current || done {
                                theme::inverse()
                            } else {
                                theme::faint()
                            },
                        )),
                )
                .child(text(step.title()).font(LABEL_SM).color(if current {
                    theme::ink()
                } else if done {
                    theme::muted()
                } else {
                    theme::faint()
                })),
        );
    }
    let _ = k;
    row_
}

fn about_step(s: &CoderuneApp, k: &Kit) -> El {
    ui::card(
        None,
        None,
        vec![
            ui::field(
                "Display name or pseudonym",
                Some("This is how your baseline is labelled. A pseudonym is fine."),
                text_input(
                    &s.profile_form.display_name,
                    "Your name or pseudonym",
                    "Display name",
                    |s| &mut s.profile_form.display_name,
                    k,
                ),
            ),
            ui::field(
                "Where are you starting from?",
                Some("Self-reported. Coderune never treats it as verified mastery."),
                stage_chips(s),
            ),
        ],
    )
}

fn starting_point_step(s: &CoderuneApp, k: &Kit) -> El {
    let mut children = vec![ui::field(
        "What can you do today?",
        Some("A short, honest statement. If you are new, mark it below instead."),
        text_area(
            &s.profile_form.skills,
            "For example: I write TypeScript for small web apps.",
            "Initial skills statement",
            3,
            |s| &mut s.profile_form.skills,
            k,
        ),
    )];
    children.push(
        row()
            .gap(8.0)
            .items_center()
            .child(option_chip(
                "onboarding-new-to-coding".to_string(),
                "I am new to coding",
                s.profile_form.new_to_coding,
                |s: &mut CoderuneApp| s.profile_form.new_to_coding = !s.profile_form.new_to_coding,
            ))
            .child(
                text("No skills need inventing. This records a starting point, not a test.")
                    .font(BODY_SM)
                    .color(theme::muted())
                    .wrap(),
            ),
    );
    children.push(ui::field(
        "What do you want to learn?",
        Some("At least one goal. Separate several goals with commas."),
        text_area(
            &s.profile_form.goals,
            "For example: Ship a native desktop app, learn Rust",
            "Learning goals",
            3,
            |s| &mut s.profile_form.goals,
            k,
        ),
    ));
    ui::card(None, None, children)
}

fn review_step(s: &CoderuneApp, k: &Kit) -> El {
    let name = s.profile_form.display_name.text.trim();
    let stage = s
        .profile_form
        .stage
        .map(|stage| stage.label())
        .unwrap_or("Not chosen");
    let skills = if s.profile_form.new_to_coding {
        "New to coding".to_string()
    } else {
        let value = s.profile_form.skills.text.trim();
        if value.is_empty() {
            "Not set".to_string()
        } else {
            value.to_string()
        }
    };
    let goals = split_list(&s.profile_form.goals.text);
    let mut goal_rows = col().gap(4.0);
    if goals.is_empty() {
        goal_rows = goal_rows.child(ui::subtle("No learning goals yet."));
    } else {
        for goal in &goals {
            goal_rows = goal_rows.child(
                row()
                    .gap(8.0)
                    .items_center()
                    .child(rune::el().size(4.0, 4.0).radius(2.0).fill(theme::accent()))
                    .child(text(goal).font(CODE_SM).color(theme::ink()).wrap()),
            );
        }
    }
    ui::card(
        Some("Your developer baseline"),
        Some("You can edit this later. A save creates a new profile version."),
        vec![
            summary_row("Name", if name.is_empty() { "Not set" } else { name }),
            summary_row("Experience", stage),
            summary_row("Starting point", &skills),
            ui::field("Learning goals", None, goal_rows),
            row()
                .w_full()
                .gap(8.0)
                .items_center()
                .child(ui::badge("later", k))
                .child(
                    text(
                        "Add education, employment, project experience, certifications, and links from Profile when you are ready.",
                    )
                    .font(BODY_SM)
                    .color(theme::muted())
                    .wrap()
                    .grow(1.0)
                    .min_w(0.0),
                ),
        ],
    )
}

fn summary_row(label: &str, value: &str) -> El {
    row()
        .w_full()
        .gap(16.0)
        .items_start()
        .child(
            text(label)
                .font(LABEL)
                .color(theme::muted())
                .w(112.0)
                .shrink(),
        )
        .child(text(value).font(HEADING_XS).color(theme::ink()).wrap())
}
