//! Bundle rendering and budget enforcement.

use coderune_domain::{
    ErrorCode, PersonalContextEntry, PersonalContextOmission, PersonalContextOmissionReason,
    PersonalContextQuery,
};

use crate::{AppError, AppResult};

pub(super) fn render_header(purpose: &str, profile_version: u32, policy_version: u32) -> String {
    format!("Personal context | {purpose} | profile v{profile_version} | policy v{policy_version}")
}

fn render_entry(entry: &PersonalContextEntry) -> String {
    let text = entry.search_text.replace(['\r', '\n'], " ");
    format!(
        "{}: {} (source: {})",
        entry.kind.label(),
        text,
        entry.source_path
    )
}

/// Add entries until the budget is full. Entries that do not fit are counted,
/// never truncated, and the required baseline must fit whole.
pub(super) fn fit_budget(
    header: String,
    selected: Vec<PersonalContextEntry>,
    query: &PersonalContextQuery,
    required_baseline: usize,
    omissions: &mut OmissionCounts,
) -> AppResult<(Vec<PersonalContextEntry>, Vec<String>)> {
    let mut lines = vec![header];
    let mut bytes = lines[0].len();
    let mut accepted = Vec::new();

    for entry in selected {
        let line = render_entry(&entry);
        let cost = 1 + line.len();
        if accepted.len() + 1 > query.max_entries || bytes + cost > query.max_bytes {
            omissions.over_budget += 1;
            continue;
        }
        bytes += cost;
        lines.push(line);
        accepted.push(entry);
    }

    if accepted.len() < required_baseline {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The required personal context baseline exceeds the size limit.",
        ));
    }

    Ok((accepted, lines))
}

#[derive(Debug, Default)]
pub(super) struct OmissionCounts {
    pub(super) unknown: usize,
    pub(super) not_permitted: usize,
    pub(super) no_match: usize,
    pub(super) over_budget: usize,
}

impl OmissionCounts {
    pub(super) fn into_vec(self) -> Vec<PersonalContextOmission> {
        [
            (PersonalContextOmissionReason::UnknownField, self.unknown),
            (
                PersonalContextOmissionReason::NotPermitted,
                self.not_permitted,
            ),
            (PersonalContextOmissionReason::NoMatch, self.no_match),
            (PersonalContextOmissionReason::OverBudget, self.over_budget),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(reason, count)| PersonalContextOmission { reason, count })
        .collect()
    }
}
