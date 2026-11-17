//! Request classification and entry selection.

use std::collections::BTreeSet;

use coderune_domain::{
    is_known_context_field, normalize_context_field, ContextSharingPolicy, ErrorCode,
    PersonalContextEntry,
};

use super::render::OmissionCounts;
use crate::{AppError, AppResult};

/// Normalised request fields split by what the purpose policy allows.
pub(super) struct RequestPlan {
    normalized: Vec<String>,
    permitted: Vec<String>,
    read_paths: Vec<String>,
}

impl RequestPlan {
    pub(super) fn new(
        fields: &[String],
        policy: &ContextSharingPolicy,
        omissions: &mut OmissionCounts,
    ) -> Self {
        let normalized = normalized_requested_fields(fields);
        let mut permitted = Vec::new();
        for field in &normalized {
            if !is_known_context_field(field) {
                omissions.unknown += 1;
            } else if !policy.permits_field(field) {
                omissions.not_permitted += 1;
            } else {
                permitted.push(field.clone());
            }
        }

        let mut read_paths = policy
            .baseline_fields()
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>();
        for field in &permitted {
            if !read_paths.contains(field) {
                read_paths.push(field.clone());
            }
        }

        Self {
            normalized,
            permitted,
            read_paths,
        }
    }

    pub(super) fn normalized(&self) -> Vec<String> {
        self.normalized.clone()
    }

    pub(super) fn permitted(&self) -> &[String] {
        &self.permitted
    }

    pub(super) fn read_paths(&self) -> Vec<String> {
        self.read_paths.clone()
    }
}

fn normalized_requested_fields(fields: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for field in fields {
        let field = normalize_context_field(field);
        if !field.is_empty() && !normalized.contains(&field) {
            normalized.push(field);
        }
    }
    normalized
}

/// Select the baseline first, then the permitted requests, keeping one copy of
/// each entry by version-scoped source identity and content.
pub(super) fn select_entries(
    entries: &[PersonalContextEntry],
    policy: &ContextSharingPolicy,
    permitted: &[String],
    omissions: &mut OmissionCounts,
) -> AppResult<Vec<PersonalContextEntry>> {
    let mut selected = Vec::new();
    let mut seen_identity = BTreeSet::new();
    let mut seen_content = BTreeSet::new();

    for field in policy.baseline_fields() {
        let matches = matching(entries, field);
        if matches.is_empty() {
            return Err(AppError::new(
                ErrorCode::Validation,
                "The required personal context baseline is incomplete.",
            ));
        }
        push_unique(
            &mut selected,
            &mut seen_identity,
            &mut seen_content,
            matches,
        );
    }

    for field in permitted {
        if policy.baseline_fields().contains(&field.as_str()) {
            continue;
        }
        let matches = matching(entries, field);
        if matches.is_empty() {
            omissions.no_match += 1;
            continue;
        }
        push_unique(
            &mut selected,
            &mut seen_identity,
            &mut seen_content,
            matches,
        );
    }

    Ok(selected)
}

fn matching(entries: &[PersonalContextEntry], field: &str) -> Vec<PersonalContextEntry> {
    entries
        .iter()
        .filter(|entry| entry.field_path == field)
        .cloned()
        .collect()
}

fn push_unique(
    selected: &mut Vec<PersonalContextEntry>,
    seen_identity: &mut BTreeSet<String>,
    seen_content: &mut BTreeSet<String>,
    entries: Vec<PersonalContextEntry>,
) {
    for entry in entries {
        if seen_identity.insert(entry.source_path.clone())
            && seen_content.insert(entry.content_hash.clone())
        {
            selected.push(entry);
        }
    }
}
