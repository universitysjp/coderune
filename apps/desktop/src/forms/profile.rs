//! The developer profile form and its line-oriented detail codecs.

use coderune_domain::{
    Certification, EducationEntry, EmploymentEntry, ExperienceStage, LearningPreferences, Link,
    ProfileDraft, ProjectExperience, SelfReportedSkill, SkillRating,
};
use rune::TextEdit;

use super::{edit, split_list};

pub(crate) struct ProfileForm {
    pub display_name: TextEdit,
    pub skills: TextEdit,
    pub goals: TextEdit,
    pub stage: Option<ExperienceStage>,
    pub new_to_coding: bool,
    pub self_reported_skills: TextEdit,
    pub education: TextEdit,
    pub employment: TextEdit,
    pub project_experience: TextEdit,
    pub certifications: TextEdit,
    pub links: TextEdit,
    pub learning_formats: TextEdit,
    pub learning_notes: TextEdit,
}

impl ProfileForm {
    pub(crate) fn from_value(
        profile: Option<&coderune_domain::DeveloperProfile>,
        draft: Option<&coderune_domain::OnboardingDraft>,
    ) -> Self {
        if let Some(profile) = profile {
            Self::from_draft(&ProfileDraft::from_profile(profile))
        } else if let Some(draft) = draft {
            Self::from_draft(&draft.draft)
        } else {
            Self::from_draft(&ProfileDraft::default())
        }
    }

    pub(crate) fn from_draft(draft: &ProfileDraft) -> Self {
        Self {
            display_name: edit(draft.display_name.clone()),
            skills: edit(draft.skills_statement.clone()),
            goals: edit(draft.learning_goals.join(", ")),
            stage: draft.experience_stage,
            new_to_coding: draft.new_to_coding,
            self_reported_skills: edit(format_skills(&draft.self_reported_skills)),
            education: edit(format_education(&draft.education)),
            employment: edit(format_employment(&draft.employment)),
            project_experience: edit(format_project_experience(&draft.projects)),
            certifications: edit(format_certifications(&draft.certifications)),
            links: edit(format_links(&draft.links)),
            learning_formats: edit(
                draft
                    .learning_preferences
                    .as_ref()
                    .map(|preferences| preferences.formats.join(", "))
                    .unwrap_or_default(),
            ),
            learning_notes: edit(
                draft
                    .learning_preferences
                    .as_ref()
                    .and_then(|preferences| preferences.notes.clone())
                    .unwrap_or_default(),
            ),
        }
    }

    /// Preserve fields the form does not edit (CV provenance) and overwrite the
    /// visible ones, including the optional sections completed inside the app.
    pub(crate) fn to_draft(&self, base: ProfileDraft) -> ProfileDraft {
        let mut draft = base;
        draft.display_name = self.display_name.text.trim().to_string();
        draft.experience_stage = self.stage;
        draft.new_to_coding = self.new_to_coding;
        draft.skills_statement = self.skills.text.trim().to_string();
        draft.learning_goals = split_list(&self.goals.text);
        draft.self_reported_skills = parse_skills(&self.self_reported_skills.text);
        draft.education = parse_education(&self.education.text);
        draft.employment = parse_employment(&self.employment.text);
        draft.projects = parse_project_experience(&self.project_experience.text);
        draft.certifications = parse_certifications(&self.certifications.text);
        draft.links = parse_links(&self.links.text);
        let formats = split_list(&self.learning_formats.text);
        let notes = self.learning_notes.text.trim().to_string();
        draft.learning_preferences = if formats.is_empty() && notes.is_empty() {
            None
        } else {
            Some(LearningPreferences {
                formats,
                notes: (!notes.is_empty()).then_some(notes),
            })
        };
        draft
    }

    /// Whether the optional sections hold anything at all.
    pub(crate) fn details_count(&self) -> usize {
        [
            &self.self_reported_skills,
            &self.education,
            &self.employment,
            &self.project_experience,
            &self.certifications,
            &self.links,
            &self.learning_formats,
            &self.learning_notes,
        ]
        .into_iter()
        .filter(|field| !field.text.trim().is_empty())
        .count()
    }
}

fn opt_field(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Split a details text area into trimmed `|`-separated columns.
fn rows(value: &str) -> Vec<Vec<String>> {
    value
        .lines()
        .map(|line| {
            line.split('|')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .filter(|row: &Vec<String>| row.first().is_some_and(|first| !first.is_empty()))
        .collect()
}

fn join_fields(fields: &[Option<&str>]) -> String {
    fields
        .iter()
        .map(|field| field.unwrap_or("").trim())
        .collect::<Vec<_>>()
        .join(" | ")
        .trim_end_matches(" | ")
        .to_string()
}

fn format_skills(skills: &[SelfReportedSkill]) -> String {
    skills
        .iter()
        .map(|skill| match skill.rating {
            Some(rating) => format!("{} | {}", skill.topic, rating.label().to_lowercase()),
            None => skill.topic.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_skills(value: &str) -> Vec<SelfReportedSkill> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let topic = row.first()?.trim().to_string();
            let rating = row
                .get(1)
                .and_then(|rating| match rating.to_lowercase().as_str() {
                    "beginner" => Some(SkillRating::Beginner),
                    "intermediate" => Some(SkillRating::Intermediate),
                    "advanced" => Some(SkillRating::Advanced),
                    _ => None,
                });
            Some(SelfReportedSkill { topic, rating })
        })
        .collect()
}

fn format_education(entries: &[EducationEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.institution.as_str()),
                entry.qualification.as_deref(),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_education(value: &str) -> Vec<EducationEntry> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let institution = row.first()?.clone();
            Some(EducationEntry {
                institution,
                qualification: row.get(1).and_then(|value| opt_field(value)),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_employment(entries: &[EmploymentEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.employer.as_str()),
                Some(entry.role.as_str()),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_employment(value: &str) -> Vec<EmploymentEntry> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let employer = row.first()?.clone();
            Some(EmploymentEntry {
                employer,
                role: row.get(1).cloned().unwrap_or_default(),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_project_experience(entries: &[ProjectExperience]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.name.as_str()),
                entry.role.as_deref(),
                entry.period.as_deref(),
                entry.details.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_project_experience(value: &str) -> Vec<ProjectExperience> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let name = row.first()?.clone();
            Some(ProjectExperience {
                name,
                role: row.get(1).and_then(|value| opt_field(value)),
                period: row.get(2).and_then(|value| opt_field(value)),
                details: row.get(3).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_certifications(entries: &[Certification]) -> String {
    entries
        .iter()
        .map(|entry| {
            join_fields(&[
                Some(entry.name.as_str()),
                entry.issuer.as_deref(),
                entry.year.as_deref(),
            ])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_certifications(value: &str) -> Vec<Certification> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let name = row.first()?.clone();
            Some(Certification {
                name,
                issuer: row.get(1).and_then(|value| opt_field(value)),
                year: row.get(2).and_then(|value| opt_field(value)),
            })
        })
        .collect()
}

fn format_links(entries: &[Link]) -> String {
    entries
        .iter()
        .map(|entry| join_fields(&[Some(entry.label.as_str()), Some(entry.url.as_str())]))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_links(value: &str) -> Vec<Link> {
    rows(value)
        .into_iter()
        .filter_map(|row| {
            let label = row.first()?.clone();
            let url = row.get(1).cloned().unwrap_or_default();
            (!url.is_empty()).then_some(Link { label, url })
        })
        .collect()
}
