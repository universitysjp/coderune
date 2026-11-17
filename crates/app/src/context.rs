//! Personal context retrieval.
//!
//! The application pins the current profile, applies the purpose policy, adds
//! the required baseline, and returns one bounded bundle. It never sends more
//! than the policy permits and never falls back to the whole profile.

mod render;
mod request;

use coderune_domain::{
    ContextSharingPolicy, ErrorCode, PersonalContextBundle, PersonalContextQuery,
    PERSONAL_CONTEXT_BUNDLE_SCHEMA_VERSION,
};
use coderune_storage::ContextReadFilter;

use crate::{map_storage, require_profile, AppError, AppResult, Workspace};

use render::{fit_budget, render_header, OmissionCounts};
use request::{select_entries, RequestPlan};

impl Workspace {
    /// Prepare the personal context a task may receive. This has no agent or
    /// network side effects; it only reads the current confirmed profile.
    pub fn retrieve_personal_context(
        &self,
        mut query: PersonalContextQuery,
    ) -> AppResult<PersonalContextBundle> {
        require_profile(&self.onboarding)?;
        if query.max_entries == 0 || query.max_bytes == 0 {
            return Err(AppError::new(
                ErrorCode::Validation,
                "Set a positive personal context budget.",
            ));
        }
        let policy = ContextSharingPolicy::new(query.purpose);
        let mut omissions = OmissionCounts::default();
        let plan = RequestPlan::new(&query.requested_fields, &policy, &mut omissions);
        query.requested_fields = plan.normalized();

        let profile = self.profile().ok_or_else(|| {
            AppError::new(ErrorCode::OnboardingRequired, "Complete onboarding first.")
        })?;
        let snapshot = self
            .storage
            .read_profile_context(&ContextReadFilter {
                profile_id: profile.id.clone(),
                sensitivities: policy.allowed_sensitivities(),
                kinds: policy.allowed_kinds(),
                field_paths: plan.read_paths(),
            })
            .map_err(map_storage)?;

        let selected =
            select_entries(&snapshot.entries, &policy, plan.permitted(), &mut omissions)?;
        let header = render_header(
            query.purpose.as_str(),
            snapshot.profile.version,
            ContextSharingPolicy::VERSION,
        );
        let (accepted, lines) = fit_budget(
            header,
            selected,
            &query,
            policy.baseline_fields().len(),
            &mut omissions,
        )?;

        let rendered = lines.join("\n");
        let mut bundle = PersonalContextBundle {
            schema_version: PERSONAL_CONTEXT_BUNDLE_SCHEMA_VERSION,
            profile_id: snapshot.profile.id,
            profile_version: snapshot.profile.version,
            purpose: query.purpose,
            policy_version: ContextSharingPolicy::VERSION,
            query,
            entries: accepted,
            omissions: omissions.into_vec(),
            byte_count: rendered.len(),
            rendered,
            bundle_hash: String::new(),
        };
        bundle.bundle_hash = bundle.compute_hash().map_err(|error| {
            AppError::new(
                ErrorCode::Validation,
                format!("Could not hash personal context: {error}"),
            )
        })?;
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coderune_domain::{
        EducationEntry, EmploymentEntry, ExperienceStage, LearningPreferences, Link,
        PersonalContextEntryKind, PersonalContextOmission, PersonalContextOmissionReason,
        PersonalContextPurpose, ProfileDraft, SelfReportedSkill, SkillRating,
    };

    fn workspace() -> Workspace {
        let mut workspace = Workspace::in_memory().unwrap();
        workspace
            .confirm_profile(ProfileDraft {
                display_name: "Ada Lovelace".to_string(),
                experience_stage: Some(ExperienceStage::EarlyCareer),
                skills_statement: "Rust and SQLite".to_string(),
                learning_goals: vec!["Learn error handling".to_string()],
                self_reported_skills: vec![SelfReportedSkill {
                    topic: "Rust".to_string(),
                    rating: Some(SkillRating::Intermediate),
                }],
                education: vec![EducationEntry {
                    institution: "Open University".to_string(),
                    qualification: Some("BSc".to_string()),
                    ..Default::default()
                }],
                employment: vec![EmploymentEntry {
                    employer: "Example Ltd".to_string(),
                    role: "Developer".to_string(),
                    ..Default::default()
                }],
                links: vec![Link {
                    label: "Site".to_string(),
                    url: "https://example.com".to_string(),
                }],
                learning_preferences: Some(LearningPreferences {
                    formats: vec!["Worked examples".to_string()],
                    notes: None,
                }),
                ..Default::default()
            })
            .unwrap();
        workspace
    }

    fn query(purpose: PersonalContextPurpose, requested: &[&str]) -> PersonalContextQuery {
        PersonalContextQuery {
            task: "Fix the SQLite insert".to_string(),
            purpose,
            requested_fields: requested.iter().map(|field| (*field).to_string()).collect(),
            max_entries: 16,
            max_bytes: 4096,
        }
    }

    fn kinds(bundle: &PersonalContextBundle) -> Vec<PersonalContextEntryKind> {
        bundle.entries.iter().map(|entry| entry.kind).collect()
    }

    #[test]
    fn coding_bundle_keeps_the_baseline_and_requested_skill_only() {
        let workspace = workspace();
        let bundle = workspace
            .retrieve_personal_context(query(
                PersonalContextPurpose::Coding,
                &[
                    "skills.Rust",
                    "education",
                    "employment",
                    "links",
                    "displayName",
                ],
            ))
            .unwrap();

        assert_eq!(bundle.profile_version, 1);
        assert_eq!(
            kinds(&bundle),
            vec![
                PersonalContextEntryKind::ExperienceStage,
                PersonalContextEntryKind::SkillsStatement,
                PersonalContextEntryKind::SelfReportedSkill,
            ]
        );
        assert!(bundle.rendered.contains("Rust (Intermediate)"));
        assert!(bundle.rendered.contains("source: /selfReportedSkills/0"));
        assert!(!bundle.rendered.contains("Open University"));
        assert!(!bundle.rendered.contains("Example Ltd"));
        assert!(!bundle.rendered.contains("https://example.com"));
        assert!(!bundle.rendered.contains("Ada Lovelace"));
        assert!(bundle.omissions.contains(&PersonalContextOmission {
            reason: PersonalContextOmissionReason::NotPermitted,
            count: 4,
        }));
        assert_eq!(bundle.byte_count, bundle.rendered.len());
    }

    #[test]
    fn explanation_bundle_adds_learning_context_but_not_employment() {
        let workspace = workspace();
        let bundle = workspace
            .retrieve_personal_context(query(
                PersonalContextPurpose::Explanation,
                &["learningGoals", "learningPreferences", "employment"],
            ))
            .unwrap();
        let kinds = kinds(&bundle);
        assert!(kinds.contains(&PersonalContextEntryKind::LearningGoal));
        assert!(kinds.contains(&PersonalContextEntryKind::LearningPreference));
        assert!(!kinds.contains(&PersonalContextEntryKind::Employment));
        assert!(bundle.rendered.contains("Learn error handling"));
        assert!(bundle.rendered.contains("Worked examples"));
        assert!(!bundle.rendered.contains("Example Ltd"));
    }

    #[test]
    fn verification_bundle_contains_no_personal_entries() {
        let workspace = workspace();
        let bundle = workspace
            .retrieve_personal_context(query(PersonalContextPurpose::Verification, &[]))
            .unwrap();
        assert!(bundle.entries.is_empty());
        assert!(bundle.omissions.is_empty());
        assert_eq!(
            bundle.rendered,
            "Personal context | verification | profile v1 | policy v1"
        );
        assert!(!bundle.bundle_hash.is_empty());
    }

    #[test]
    fn unknown_and_missing_fields_are_reported_without_a_profile_fallback() {
        let workspace = workspace();
        let bundle = workspace
            .retrieve_personal_context(query(
                PersonalContextPurpose::Coding,
                &["employment_level", "skill:go"],
            ))
            .unwrap();
        assert_eq!(
            kinds(&bundle),
            vec![
                PersonalContextEntryKind::ExperienceStage,
                PersonalContextEntryKind::SkillsStatement,
            ]
        );
        assert!(bundle.omissions.contains(&PersonalContextOmission {
            reason: PersonalContextOmissionReason::UnknownField,
            count: 1,
        }));
        assert!(bundle.omissions.contains(&PersonalContextOmission {
            reason: PersonalContextOmissionReason::NoMatch,
            count: 1,
        }));
    }

    #[test]
    fn a_tiny_budget_rejects_the_baseline_instead_of_truncating_it() {
        let workspace = workspace();
        let mut request = query(PersonalContextPurpose::Coding, &[]);
        request.max_bytes = 10;
        let error = workspace.retrieve_personal_context(request).unwrap_err();
        assert_eq!(error.code, ErrorCode::Validation);
    }

    #[test]
    fn repeated_requests_hash_identically_and_profile_edits_change_the_snapshot() {
        let mut workspace = workspace();
        let request = query(PersonalContextPurpose::Coding, &["skill:rust"]);
        let first = workspace
            .retrieve_personal_context(request.clone())
            .unwrap();
        let second = workspace
            .retrieve_personal_context(request.clone())
            .unwrap();
        assert_eq!(first.bundle_hash, second.bundle_hash);

        workspace
            .confirm_profile(ProfileDraft {
                display_name: "Ada Lovelace".to_string(),
                experience_stage: Some(ExperienceStage::MidCareer),
                skills_statement: "Rust, SQLite, and async".to_string(),
                learning_goals: vec!["Learn error handling".to_string()],
                ..Default::default()
            })
            .unwrap();
        let third = workspace.retrieve_personal_context(request).unwrap();
        assert_eq!(third.profile_version, 2);
        assert_ne!(first.bundle_hash, third.bundle_hash);
    }
}
