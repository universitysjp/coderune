//! The fonts installed on this computer, for the appearance choices.

use std::collections::BTreeSet;
use std::sync::OnceLock;

/// Family names of every installed face, sorted and deduplicated. Loading the
/// system database costs a few hundred milliseconds, so it happens once, when
/// a settings page first asks.
pub(crate) fn installed() -> &'static [String] {
    static FAMILIES: OnceLock<Vec<String>> = OnceLock::new();
    FAMILIES
        .get_or_init(|| {
            let mut database = fontdb::Database::new();
            database.load_system_fonts();
            let mut seen = BTreeSet::new();
            let mut names: Vec<String> = database
                .faces()
                .filter_map(|face| face.families.first())
                .map(|(name, _)| name.trim())
                // Dot-prefixed families are private faces the system hides.
                .filter(|name| !name.is_empty() && !name.starts_with('.'))
                .filter(|name| seen.insert(name.to_lowercase()))
                .map(str::to_owned)
                .collect();
            names.sort_by_key(|name| name.to_lowercase());
            names
        })
        .as_slice()
}

/// A family is offered when the system lists it, and the app's own families
/// are always offered so a choice can be undone on a machine that lacks them.
pub(crate) fn choices(current: Option<&str>) -> Vec<String> {
    let mut choices: Vec<String> = installed().to_vec();
    for fallback in [crate::DEFAULT_UI_FONT, crate::DEFAULT_MONO_FONT] {
        if !choices.iter().any(|name| name == fallback) {
            choices.push(fallback.to_string());
        }
    }
    if let Some(current) = current {
        if !choices.iter().any(|name| name == current) {
            choices.push(current.to_string());
        }
    }
    choices
}
