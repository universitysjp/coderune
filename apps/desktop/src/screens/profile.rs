//! The profile screen: the required baseline and the optional details that
//! onboarding intentionally leaves for later.

use crate::app::{CoderuneApp, ProfileSection, ProfileTab};
use crate::widgets::*;
use coderune_ui::components as ui;
use coderune_ui::theme::{self, BODY, BODY_SM, HEADING_XS, LABEL_SM};
use rune::{col, row, text, El};
use rune_kit::{Accordion, AccordionItem, Kit, SegmentedControl};

pub(crate) fn profile(s: &CoderuneApp, k: &Kit) -> El {
    let mut body = ui::page("profile-page").child(ui::page_header(
        "Profile",
        "One developer profile shared across every project. Required answers stay short; optional detail can be added over time.",
        vec![],
    ));

    let selected = match s.profile_tab {
        ProfileTab::Baseline => 0,
        ProfileTab::Details => 1,
    };
    body = body.child(
        SegmentedControl::new(&["Baseline", "Complete profile"], Some(selected))
            .label("Profile view")
            .width(Some(300.0))
            .on_pick(|s: &mut CoderuneApp, index, _| {
                s.profile_tab = if index == 0 {
                    ProfileTab::Baseline
                } else {
                    ProfileTab::Details
                };
                s.error = None;
                s.notice = None;
            })
            .build(k),
    );

    if let Some(error) = &s.error {
        body = body.child(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        body = body.child(ui::alert(None, notice));
    }

    match s.profile_tab {
        ProfileTab::Baseline => body.child(baseline(s, k)),
        ProfileTab::Details => body.child(details(s, k)),
    }
}

/// The first save confirms onboarding; later saves create a new version.
fn save_label(s: &CoderuneApp) -> &'static str {
    if s.onboarding_required() {
        "Confirm baseline"
    } else {
        "Save new version"
    }
}

fn baseline(s: &CoderuneApp, k: &Kit) -> El {
    let mut body = col().w_full().gap(ui::SECTION_GAP);
    if let Some(profile) = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile())
    {
        body = body.child(ui::section(
            k,
            "profile-baseline",
            "Current baseline",
            [col()
                .w_full()
                .gap(12.0)
                .py(ui::ROW_PAD)
                .child(ui::subtle(
                    "Edits create a new immutable version. Existing captures keep their snapshot.",
                ))
                .children(vec![
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
                        "{} self-reported skill(s) · {} education · {} employment · {} link(s)",
                        profile.self_reported_skills.len(),
                        profile.education.len(),
                        profile.employment.len(),
                        profile.links.len()
                    )),
                ])],
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
            Some("Separate several goals with commas."),
            text_area(
                &s.profile_form.goals,
                "What do you want to learn?",
                "Learning goals",
                3,
                |s| &mut s.profile_form.goals,
                k,
            ),
        ),
        row()
            .gap(8.0)
            .items_center()
            .child(option_chip(
                "profile-new-to-coding".to_string(),
                "I am new to coding",
                s.profile_form.new_to_coding,
                |s: &mut CoderuneApp| s.profile_form.new_to_coding = !s.profile_form.new_to_coding,
            ))
            .child(primary(save_label(s), |s| s.save_profile(), k)),
    ];
    if s.profile_form.details_count() > 0 {
        form_children.push(ui::tiny(&format!(
            "{} optional section(s) hold content. Edit them under Complete profile.",
            s.profile_form.details_count()
        )));
    }
    body.child(ui::section(
        k,
        "profile-required",
        "Edit required fields",
        [col()
            .w_full()
            .gap(12.0)
            .py(ui::ROW_PAD)
            .child(ui::subtle(
                "Self-reported skills are never presented as verified mastery.",
            ))
            .children(form_children)],
    ))
}

fn details(s: &CoderuneApp, k: &Kit) -> El {
    let mut accordion = Accordion::new();
    for section in ProfileSection::ALL {
        let open = s.profile_section == Some(section);
        accordion = accordion.item(
            AccordionItem::new(section.title())
                .expanded(open)
                .on_toggle(move |s: &mut CoderuneApp| s.toggle_profile_section(section))
                .child(section_body(s, section, k)),
        );
    }
    col().w_full().gap(ui::SECTION_GAP).child(ui::section_with(
        k,
        "profile-details",
        "Complete your profile",
        primary("Save profile", |s| s.save_profile(), k),
        [col()
            .w_full()
            .gap(10.0)
            .py(ui::ROW_PAD)
            .child(ui::subtle(
                "Optional. Each entry makes future project context more useful, but none is required to start.",
            ))
            .child(accordion.build(k))],
    ))
}

fn section_body(s: &CoderuneApp, section: ProfileSection, k: &Kit) -> El {
    let body = match section {
        ProfileSection::Skills => text_area(
            &s.profile_form.self_reported_skills,
            "Rust | intermediate\nTypeScript | advanced",
            "Self-reported skills",
            4,
            |s| &mut s.profile_form.self_reported_skills,
            k,
        ),
        ProfileSection::Education => text_area(
            &s.profile_form.education,
            "University of Moratuwa | BSc Computer Science | 2021–2025",
            "Education",
            4,
            |s| &mut s.profile_form.education,
            k,
        ),
        ProfileSection::Employment => text_area(
            &s.profile_form.employment,
            "Acme | Software Engineer | 2024–present | Platform team",
            "Employment",
            4,
            |s| &mut s.profile_form.employment,
            k,
        ),
        ProfileSection::Projects => text_area(
            &s.profile_form.project_experience,
            "Coderune | Maintainer | 2026 | Local context for coding agents",
            "Project experience",
            4,
            |s| &mut s.profile_form.project_experience,
            k,
        ),
        ProfileSection::Certifications => text_area(
            &s.profile_form.certifications,
            "AWS Solutions Architect | Amazon | 2025",
            "Certifications",
            3,
            |s| &mut s.profile_form.certifications,
            k,
        ),
        ProfileSection::Links => text_area(
            &s.profile_form.links,
            "Portfolio | https://example.com",
            "Links",
            3,
            |s| &mut s.profile_form.links,
            k,
        ),
        ProfileSection::Preferences => ui::field(
            "Formats",
            Some("Comma-separated, for example: examples, short notes, pair work."),
            text_input(
                &s.profile_form.learning_formats,
                "Examples, short notes",
                "Learning formats",
                |s| &mut s.profile_form.learning_formats,
                k,
            ),
        )
        .child(ui::field(
            "Notes",
            Some("Anything a future session should know about how you learn."),
            text_area(
                &s.profile_form.learning_notes,
                "I learn best from a small example I can run.",
                "Learning notes",
                3,
                |s| &mut s.profile_form.learning_notes,
                k,
            ),
        )),
    };
    col().w_full().padding(16.0).gap(8.0).child(body).child(
        text(section.hint())
            .font(LABEL_SM)
            .color(theme::faint())
            .wrap(),
    )
}
