//! Personal context contracts, sharing policy, and profile projection.
//!
//! The complete developer profile stays authoritative. These entries are a
//! derived, source-addressable view used for bounded retrieval.

use serde::{Deserialize, Serialize};

use crate::{
    non_blank, Certification, DeveloperProfile, EducationEntry, EmploymentEntry, FieldSource,
    LearningPreferences, ProjectExperience,
};

/// The operation that asks for personal context. A purpose decides which
/// profile entries may be selected; it does not decide whether code is correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersonalContextPurpose {
    Coding,
    Verification,
    Explanation,
    Quiz,
}

impl PersonalContextPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Coding => "coding",
            Self::Verification => "verification",
            Self::Explanation => "explanation",
            Self::Quiz => "quiz",
        }
    }
}

/// How sensitive a stored profile entry is. Sensitivity describes the data;
/// the sharing policy decides which purposes may select it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContextSensitivity {
    Technical,
    Personal,
    Restricted,
}

impl ContextSensitivity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Technical => "technical",
            Self::Personal => "personal",
            Self::Restricted => "restricted",
        }
    }
}

impl std::str::FromStr for ContextSensitivity {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "technical" => Ok(Self::Technical),
            "personal" => Ok(Self::Personal),
            "restricted" => Ok(Self::Restricted),
            _ => Err(()),
        }
    }
}

/// A controlled kind of searchable developer-profile entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersonalContextEntryKind {
    DisplayName,
    ExperienceStage,
    NewToCoding,
    SkillsStatement,
    LearningGoal,
    SelfReportedSkill,
    Education,
    Employment,
    ProjectExperience,
    Certification,
    Link,
    LearningPreference,
}

impl PersonalContextEntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DisplayName => "display-name",
            Self::ExperienceStage => "experience-stage",
            Self::NewToCoding => "new-to-coding",
            Self::SkillsStatement => "skills-statement",
            Self::LearningGoal => "learning-goal",
            Self::SelfReportedSkill => "self-reported-skill",
            Self::Education => "education",
            Self::Employment => "employment",
            Self::ProjectExperience => "project-experience",
            Self::Certification => "certification",
            Self::Link => "link",
            Self::LearningPreference => "learning-preference",
        }
    }

    pub fn sensitivity(self) -> ContextSensitivity {
        match self {
            Self::SkillsStatement | Self::SelfReportedSkill | Self::ProjectExperience => {
                ContextSensitivity::Technical
            }
            _ => ContextSensitivity::Personal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DisplayName => "Display name",
            Self::ExperienceStage => "Experience stage",
            Self::NewToCoding => "New to coding",
            Self::SkillsStatement => "Skills",
            Self::LearningGoal => "Learning goal",
            Self::SelfReportedSkill => "Skill",
            Self::Education => "Education",
            Self::Employment => "Employment",
            Self::ProjectExperience => "Project",
            Self::Certification => "Certification",
            Self::Link => "Link",
            Self::LearningPreference => "Learning preferences",
        }
    }
}

impl std::str::FromStr for PersonalContextEntryKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "display-name" => Ok(Self::DisplayName),
            "experience-stage" => Ok(Self::ExperienceStage),
            "new-to-coding" => Ok(Self::NewToCoding),
            "skills-statement" => Ok(Self::SkillsStatement),
            "learning-goal" => Ok(Self::LearningGoal),
            "self-reported-skill" => Ok(Self::SelfReportedSkill),
            "education" => Ok(Self::Education),
            "employment" => Ok(Self::Employment),
            "project-experience" => Ok(Self::ProjectExperience),
            "certification" => Ok(Self::Certification),
            "link" => Ok(Self::Link),
            "learning-preference" => Ok(Self::LearningPreference),
            _ => Err(()),
        }
    }
}

pub const PROFILE_CONTEXT_PROJECTION_VERSION: u32 = 1;
pub const CONTEXT_SHARING_POLICY_VERSION: u32 = 1;
pub const PERSONAL_CONTEXT_BUNDLE_SCHEMA_VERSION: u32 = 1;

/// Versioned rules that decide which entries each purpose may select.
///
/// Baseline fields are always required. Candidate kinds are the only additional
/// entries a purpose may select. A requested field never expands these rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextSharingPolicy {
    purpose: PersonalContextPurpose,
}

const CODING_BASELINE_FIELDS: [&str; 2] = ["experienceStage", "skillsStatement"];
const NO_BASELINE_FIELDS: [&str; 0] = [];
const EXPLANATION_CANDIDATE_KINDS: [PersonalContextEntryKind; 4] = [
    PersonalContextEntryKind::SelfReportedSkill,
    PersonalContextEntryKind::ProjectExperience,
    PersonalContextEntryKind::LearningGoal,
    PersonalContextEntryKind::LearningPreference,
];

impl ContextSharingPolicy {
    pub const VERSION: u32 = CONTEXT_SHARING_POLICY_VERSION;

    pub fn new(purpose: PersonalContextPurpose) -> Self {
        Self { purpose }
    }

    pub fn purpose(self) -> PersonalContextPurpose {
        self.purpose
    }

    pub fn baseline_fields(self) -> &'static [&'static str] {
        match self.purpose {
            PersonalContextPurpose::Coding
            | PersonalContextPurpose::Explanation
            | PersonalContextPurpose::Quiz => &CODING_BASELINE_FIELDS,
            PersonalContextPurpose::Verification => &NO_BASELINE_FIELDS,
        }
    }

    pub fn baseline_kinds(self) -> &'static [PersonalContextEntryKind] {
        match self.purpose {
            PersonalContextPurpose::Coding
            | PersonalContextPurpose::Explanation
            | PersonalContextPurpose::Quiz => &[
                PersonalContextEntryKind::ExperienceStage,
                PersonalContextEntryKind::SkillsStatement,
            ],
            PersonalContextPurpose::Verification => &[],
        }
    }

    pub fn candidate_kinds(self) -> &'static [PersonalContextEntryKind] {
        match self.purpose {
            PersonalContextPurpose::Coding => &[
                PersonalContextEntryKind::SelfReportedSkill,
                PersonalContextEntryKind::ProjectExperience,
            ],
            PersonalContextPurpose::Explanation | PersonalContextPurpose::Quiz => {
                &EXPLANATION_CANDIDATE_KINDS
            }
            PersonalContextPurpose::Verification => &[],
        }
    }

    pub fn allowed_kinds(self) -> Vec<PersonalContextEntryKind> {
        let mut kinds = self.baseline_kinds().to_vec();
        for kind in self.candidate_kinds() {
            if !kinds.contains(kind) {
                kinds.push(*kind);
            }
        }
        kinds
    }

    pub fn allowed_sensitivities(self) -> Vec<ContextSensitivity> {
        let mut sensitivities = Vec::new();
        for kind in self.allowed_kinds() {
            let sensitivity = kind.sensitivity();
            if !sensitivities.contains(&sensitivity) {
                sensitivities.push(sensitivity);
            }
        }
        sensitivities
    }

    /// Unknown paths are not permitted. Known collection paths map to their
    /// entry kind, and skill paths map to self-reported skills.
    pub fn permits_field(self, field: &str) -> bool {
        if self.baseline_fields().contains(&field) {
            return true;
        }
        let kind = if field.starts_with("skill:") {
            Some(PersonalContextEntryKind::SelfReportedSkill)
        } else {
            match field {
                "displayName" => Some(PersonalContextEntryKind::DisplayName),
                "newToCoding" => Some(PersonalContextEntryKind::NewToCoding),
                "learningGoals" => Some(PersonalContextEntryKind::LearningGoal),
                "education" => Some(PersonalContextEntryKind::Education),
                "employment" => Some(PersonalContextEntryKind::Employment),
                "projects" => Some(PersonalContextEntryKind::ProjectExperience),
                "certifications" => Some(PersonalContextEntryKind::Certification),
                "links" => Some(PersonalContextEntryKind::Link),
                "learningPreferences" => Some(PersonalContextEntryKind::LearningPreference),
                _ => None,
            }
        };
        kind.is_some_and(|kind| self.candidate_kinds().contains(&kind))
    }
}

/// A request for a bounded personal context bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalContextQuery {
    pub task: String,
    pub purpose: PersonalContextPurpose,
    #[serde(default)]
    pub requested_fields: Vec<String>,
    pub max_entries: usize,
    pub max_bytes: usize,
}

/// One source-addressable entry derived from a profile version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalContextEntry {
    pub kind: PersonalContextEntryKind,
    pub field_path: String,
    pub source_path: String,
    pub sensitivity: ContextSensitivity,
    pub search_text: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub field_source: Option<FieldSource>,
    pub content_hash: String,
}

/// Why an entry or requested field did not enter the bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersonalContextOmissionReason {
    UnknownField,
    NotPermitted,
    NoMatch,
    OverBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalContextOmission {
    pub reason: PersonalContextOmissionReason,
    pub count: usize,
}

/// An immutable, purpose-filtered, bounded selection of personal context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalContextBundle {
    pub schema_version: u32,
    pub profile_id: String,
    pub profile_version: u32,
    pub purpose: PersonalContextPurpose,
    pub policy_version: u32,
    pub query: PersonalContextQuery,
    pub entries: Vec<PersonalContextEntry>,
    pub omissions: Vec<PersonalContextOmission>,
    pub rendered: String,
    pub byte_count: usize,
    pub bundle_hash: String,
}

impl PersonalContextBundle {
    /// Hash the canonical bundle core with SHA-256. The hash itself and all
    /// timestamps are excluded. Entry order is already deterministic.
    pub fn compute_hash(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Core<'a> {
            schema_version: u32,
            profile_id: &'a str,
            profile_version: u32,
            purpose: PersonalContextPurpose,
            policy_version: u32,
            query: &'a PersonalContextQuery,
            entries: &'a [PersonalContextEntry],
            omissions: &'a [PersonalContextOmission],
            rendered: &'a str,
            byte_count: usize,
        }
        let core = Core {
            schema_version: self.schema_version,
            profile_id: &self.profile_id,
            profile_version: self.profile_version,
            purpose: self.purpose,
            policy_version: self.policy_version,
            query: &self.query,
            entries: &self.entries,
            omissions: &self.omissions,
            rendered: &self.rendered,
            byte_count: self.byte_count,
        };
        Ok(sha256_hex(&serde_json::to_vec(&core)?))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

/// Collapse whitespace and lower-case a technology topic for logical lookup.
pub fn normalize_context_topic(topic: &str) -> String {
    topic
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Whether a requested field names a profile field Coderune knows about.
/// Unknown paths are reported as unknown rather than silently returning data.
pub fn is_known_context_field(field: &str) -> bool {
    matches!(
        field,
        "displayName"
            | "experienceStage"
            | "newToCoding"
            | "skillsStatement"
            | "learningGoals"
            | "education"
            | "employment"
            | "projects"
            | "certifications"
            | "links"
            | "learningPreferences"
    ) || field.starts_with("skill:")
}

/// Normalize a requested field path. `skills.rust` and `skill:Rust` both
/// resolve to `skill:rust`. Other paths are taken as written after trimming.
pub fn normalize_context_field(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    if let Some(rest) = lower.strip_prefix("skills.") {
        format!("skill:{}", normalize_context_topic(rest))
    } else if let Some(rest) = lower.strip_prefix("skill:") {
        format!("skill:{}", normalize_context_topic(rest))
    } else {
        trimmed.to_string()
    }
}

impl DeveloperProfile {
    /// Project every non-empty profile field into source-addressable entries.
    /// The projection is derived data; the profile JSON remains authoritative.
    pub fn context_entries(&self) -> Result<Vec<PersonalContextEntry>, serde_json::Error> {
        let mut entries = Vec::new();
        let source = |key: &str| self.field_sources.get(key).cloned();

        push_context_entry(
            &mut entries,
            PersonalContextEntryKind::DisplayName,
            "displayName",
            "/displayName",
            &self.display_name,
            serde_json::Value::String(self.display_name.clone()),
            source("displayName"),
        )?;
        push_context_entry(
            &mut entries,
            PersonalContextEntryKind::ExperienceStage,
            "experienceStage",
            "/experienceStage",
            self.experience_stage.label(),
            serde_json::to_value(self.experience_stage)?,
            source("experienceStage"),
        )?;
        if self.new_to_coding {
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::NewToCoding,
                "newToCoding",
                "/newToCoding",
                "New to coding",
                serde_json::Value::Bool(true),
                source("newToCoding"),
            )?;
        }
        push_context_entry(
            &mut entries,
            PersonalContextEntryKind::SkillsStatement,
            "skillsStatement",
            "/skillsStatement",
            &self.skills_statement,
            serde_json::Value::String(self.skills_statement.clone()),
            source("skillsStatement"),
        )?;

        for (index, goal) in self.learning_goals.iter().enumerate() {
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::LearningGoal,
                "learningGoals",
                &format!("/learningGoals/{index}"),
                goal,
                serde_json::Value::String(goal.clone()),
                source("learningGoals"),
            )?;
        }

        for (index, skill) in self.self_reported_skills.iter().enumerate() {
            let field_path = format!("skill:{}", normalize_context_topic(&skill.topic));
            let text = match skill.rating {
                Some(rating) => format!("{} ({})", skill.topic, rating.label()),
                None => skill.topic.clone(),
            };
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::SelfReportedSkill,
                &field_path,
                &format!("/selfReportedSkills/{index}"),
                &text,
                serde_json::to_value(skill)?,
                source("selfReportedSkills"),
            )?;
        }

        for (index, entry) in self.education.iter().enumerate() {
            let text = format_education(entry);
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::Education,
                "education",
                &format!("/education/{index}"),
                &text,
                serde_json::to_value(entry)?,
                source("education"),
            )?;
        }

        for (index, entry) in self.employment.iter().enumerate() {
            let text = format_employment(entry);
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::Employment,
                "employment",
                &format!("/employment/{index}"),
                &text,
                serde_json::to_value(entry)?,
                source("employment"),
            )?;
        }

        for (index, entry) in self.projects.iter().enumerate() {
            let text = format_project_experience(entry);
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::ProjectExperience,
                "projects",
                &format!("/projects/{index}"),
                &text,
                serde_json::to_value(entry)?,
                source("projects"),
            )?;
        }

        for (index, entry) in self.certifications.iter().enumerate() {
            let text = format_certification(entry);
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::Certification,
                "certifications",
                &format!("/certifications/{index}"),
                &text,
                serde_json::to_value(entry)?,
                source("certifications"),
            )?;
        }

        for (index, link) in self.links.iter().enumerate() {
            let text = format!("{}: {}", link.label, link.url);
            push_context_entry(
                &mut entries,
                PersonalContextEntryKind::Link,
                "links",
                &format!("/links/{index}"),
                &text,
                serde_json::to_value(link)?,
                source("links"),
            )?;
        }

        if let Some(preferences) = &self.learning_preferences {
            let text = format_learning_preferences(preferences);
            if !text.is_empty() {
                push_context_entry(
                    &mut entries,
                    PersonalContextEntryKind::LearningPreference,
                    "learningPreferences",
                    "/learningPreferences",
                    &text,
                    serde_json::to_value(preferences)?,
                    source("learningPreferences"),
                )?;
            }
        }

        Ok(entries)
    }
}

fn push_context_entry(
    entries: &mut Vec<PersonalContextEntry>,
    kind: PersonalContextEntryKind,
    field_path: &str,
    source_path: &str,
    text: &str,
    payload: serde_json::Value,
    field_source: Option<FieldSource>,
) -> Result<(), serde_json::Error> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let content_hash = sha256_hex(&serde_json::to_vec(&payload)?);
    entries.push(PersonalContextEntry {
        kind,
        field_path: field_path.to_string(),
        source_path: source_path.to_string(),
        sensitivity: kind.sensitivity(),
        search_text: text.to_string(),
        payload,
        field_source,
        content_hash,
    });
    Ok(())
}

fn format_education(entry: &EducationEntry) -> String {
    let mut text = entry.institution.trim().to_string();
    if let Some(qualification) = non_empty(entry.qualification.as_deref()) {
        text.push_str(" — ");
        text.push_str(qualification);
    }
    if let Some(period) = non_empty(entry.period.as_deref()) {
        text.push_str(" (");
        text.push_str(period);
        text.push(')');
    }
    if let Some(details) = non_empty(entry.details.as_deref()) {
        text.push_str(". ");
        text.push_str(details);
    }
    text
}

fn format_employment(entry: &EmploymentEntry) -> String {
    let mut text = format!("{} at {}", entry.role.trim(), entry.employer.trim());
    if let Some(period) = non_empty(entry.period.as_deref()) {
        text.push_str(" (");
        text.push_str(period);
        text.push(')');
    }
    if let Some(details) = non_empty(entry.details.as_deref()) {
        text.push_str(". ");
        text.push_str(details);
    }
    text
}

fn format_project_experience(entry: &ProjectExperience) -> String {
    let mut text = entry.name.trim().to_string();
    if let Some(role) = non_empty(entry.role.as_deref()) {
        text.push_str(" — ");
        text.push_str(role);
    }
    if let Some(period) = non_empty(entry.period.as_deref()) {
        text.push_str(" (");
        text.push_str(period);
        text.push(')');
    }
    if let Some(details) = non_empty(entry.details.as_deref()) {
        text.push_str(". ");
        text.push_str(details);
    }
    text
}

fn format_certification(entry: &Certification) -> String {
    let mut text = entry.name.trim().to_string();
    let issuer = non_empty(entry.issuer.as_deref());
    let year = non_empty(entry.year.as_deref());
    match (issuer, year) {
        (Some(issuer), Some(year)) => {
            text.push_str(" — ");
            text.push_str(issuer);
            text.push_str(", ");
            text.push_str(year);
        }
        (Some(issuer), None) => {
            text.push_str(" — ");
            text.push_str(issuer);
        }
        (None, Some(year)) => {
            text.push_str(" (");
            text.push_str(year);
            text.push(')');
        }
        (None, None) => {}
    }
    text
}

fn format_learning_preferences(preferences: &LearningPreferences) -> String {
    let mut parts = Vec::new();
    let formats = non_blank(preferences.formats.clone());
    if !formats.is_empty() {
        parts.push(format!("Formats: {}", formats.join(", ")));
    }
    if let Some(notes) = non_empty(preferences.notes.as_deref()) {
        parts.push(format!("Notes: {notes}"));
    }
    parts.join(". ")
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExperienceStage, Link, ProfileDraft, SelfReportedSkill, SkillRating};

    fn rich_profile() -> DeveloperProfile {
        ProfileDraft {
            display_name: "Ada".to_string(),
            experience_stage: Some(ExperienceStage::EarlyCareer),
            skills_statement: "Rust and SQLite".to_string(),
            learning_goals: vec!["Learn error handling".to_string()],
            self_reported_skills: vec![
                SelfReportedSkill {
                    topic: "Rust".to_string(),
                    rating: Some(SkillRating::Intermediate),
                },
                SelfReportedSkill {
                    topic: "Type  Script".to_string(),
                    rating: None,
                },
            ],
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
                notes: Some("Short steps".to_string()),
            }),
            ..Default::default()
        }
        .into_profile(None)
        .unwrap()
    }

    #[test]
    fn projection_addresses_skills_by_normalized_topic() {
        let entries = rich_profile().context_entries().unwrap();
        let skills = entries
            .iter()
            .filter(|entry| entry.kind == PersonalContextEntryKind::SelfReportedSkill)
            .collect::<Vec<_>>();
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].field_path, "skill:rust");
        assert_eq!(skills[0].source_path, "/selfReportedSkills/0");
        assert_eq!(skills[0].sensitivity, ContextSensitivity::Technical);
        assert_eq!(skills[1].field_path, "skill:type script");
        assert!(skills[1].search_text.contains("Type  Script"));
    }

    #[test]
    fn requested_fields_normalize_skill_aliases() {
        assert_eq!(normalize_context_field(" skills.Rust "), "skill:rust");
        assert_eq!(
            normalize_context_field("skill:Type Script"),
            "skill:type script"
        );
        assert_eq!(
            normalize_context_field("learningPreferences"),
            "learningPreferences"
        );
    }

    #[test]
    fn coding_policy_denies_identity_and_history_but_keeps_technical_context() {
        let policy = ContextSharingPolicy::new(PersonalContextPurpose::Coding);
        assert!(policy.permits_field("experienceStage"));
        assert!(policy.permits_field("skillsStatement"));
        assert!(policy.permits_field("skill:rust"));
        assert!(policy.permits_field("projects"));
        assert!(!policy.permits_field("displayName"));
        assert!(!policy.permits_field("education"));
        assert!(!policy.permits_field("employment"));
        assert!(!policy.permits_field("links"));

        let explanation = ContextSharingPolicy::new(PersonalContextPurpose::Explanation);
        assert!(explanation.permits_field("learningGoals"));
        assert!(explanation.permits_field("learningPreferences"));
        assert!(!explanation.permits_field("employment"));

        let verification = ContextSharingPolicy::new(PersonalContextPurpose::Verification);
        assert!(verification.baseline_fields().is_empty());
        assert!(!verification.permits_field("skillsStatement"));
    }

    #[test]
    fn sha256_hex_is_lowercase_and_fixed_width() {
        let hex = sha256_hex(b"");
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)));
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn bundle_hash_is_stable_and_changes_with_content() {
        let mut bundle = PersonalContextBundle {
            schema_version: PERSONAL_CONTEXT_BUNDLE_SCHEMA_VERSION,
            profile_id: "developer".to_string(),
            profile_version: 1,
            purpose: PersonalContextPurpose::Coding,
            policy_version: CONTEXT_SHARING_POLICY_VERSION,
            query: PersonalContextQuery {
                task: "Build".to_string(),
                purpose: PersonalContextPurpose::Coding,
                requested_fields: vec!["skill:rust".to_string()],
                max_entries: 8,
                max_bytes: 4096,
            },
            entries: Vec::new(),
            omissions: Vec::new(),
            rendered: "Personal context".to_string(),
            byte_count: 16,
            bundle_hash: String::new(),
        };
        let first = bundle.compute_hash().unwrap();
        bundle.bundle_hash = first.clone();
        assert_eq!(bundle.compute_hash().unwrap(), first);
        bundle.rendered.push('.');
        assert_ne!(bundle.compute_hash().unwrap(), first);
    }
}
