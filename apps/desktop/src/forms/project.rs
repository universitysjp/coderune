//! The project details form and its folder defaults.

use std::path::{Path, PathBuf};

use coderune_domain::{AppError, ProjectDraft};
use rune::TextEdit;

use super::{edit, split_list};

pub struct ProjectForm {
    pub name: TextEdit,
    pub root: TextEdit,
    pub goals: TextEdit,
    pub include_paths: TextEdit,
    pub exclude_paths: TextEdit,
    pub respect_gitignore: bool,
    pub per_file_kib: TextEdit,
    pub per_capture_mib: TextEdit,
    pub editing: Option<String>,
}

impl ProjectForm {
    /// The folder row the panel shows: the picked root, or a prompt when none is
    /// picked yet.
    pub(crate) fn folder(&self) -> Option<PathBuf> {
        let path = self.root.text.trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    }

    pub(crate) fn empty(initial_root: Option<PathBuf>) -> Self {
        Self {
            name: edit(""),
            root: edit(
                initial_root
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
            ),
            goals: edit(""),
            include_paths: edit(""),
            exclude_paths: edit(""),
            respect_gitignore: true,
            per_file_kib: edit("1024"),
            per_capture_mib: edit("20"),
            editing: None,
        }
    }

    /// A new project's form with the picked folder and its name already filled
    /// in, so the first dialog only asks what still matters.
    pub(crate) fn from_folder(root: &Path) -> Self {
        let mut form = Self::empty(Some(root.to_path_buf()));
        form.name = edit(project_name(root));
        form
    }

    pub(crate) fn from_project(project: &coderune_domain::Project) -> Self {
        Self {
            name: edit(project.name.clone()),
            root: edit(project.root_path.clone()),
            goals: edit(
                project
                    .goals
                    .iter()
                    .map(|goal| goal.text.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            include_paths: edit(project.scope.include_paths.join(", ")),
            exclude_paths: edit(project.scope.exclude_paths.join(", ")),
            respect_gitignore: project.scope.respect_gitignore,
            per_file_kib: edit((project.limits.per_file_bytes / 1024).to_string()),
            per_capture_mib: edit((project.limits.per_capture_bytes / (1024 * 1024)).to_string()),
            editing: Some(project.id.clone()),
        }
    }

    /// Preserve project fields the form does not edit (purpose, business logic, rules,
    /// constraints, decisions, acceptance criteria, knowledge) and overwrite visible ones.
    pub(crate) fn to_draft(
        &self,
        previous: Option<&coderune_domain::Project>,
    ) -> Result<ProjectDraft, AppError> {
        let parse = |value: &str, multiplier: u64, field: &str| {
            value
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .and_then(|value| value.checked_mul(multiplier))
                .ok_or_else(|| AppError::validation(vec![field.to_string()]))
        };
        let mut draft = previous.map(ProjectDraft::from_project).unwrap_or_default();
        draft.name = self.name.text.trim().to_string();
        draft.root_path = self.root.text.trim().to_string();
        draft.goals = split_list(&self.goals.text);
        draft.scope = coderune_domain::CollectionScope {
            include_paths: split_list(&self.include_paths.text),
            exclude_paths: split_list(&self.exclude_paths.text),
            respect_gitignore: self.respect_gitignore,
        };
        draft.limits = coderune_domain::CaptureLimits {
            per_file_bytes: parse(&self.per_file_kib.text, 1024, "per-file limit")?,
            per_capture_bytes: parse(&self.per_capture_mib.text, 1024 * 1024, "per-capture limit")?,
        };
        Ok(draft)
    }
}

/// A folder's own name, used as a new project's first guess at a name.
pub(crate) fn project_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "New project".to_string())
}
