//! Shared fixtures for storage tests.

use coderune_domain::{ExperienceStage, ProfileDraft};

pub(crate) fn profile_draft() -> ProfileDraft {
    ProfileDraft {
        display_name: "Ada".to_string(),
        experience_stage: Some(ExperienceStage::EarlyCareer),
        skills_statement: "Rust".to_string(),
        learning_goals: vec!["Ship".to_string()],
        ..Default::default()
    }
}
