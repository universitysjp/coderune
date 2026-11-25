//! Small text helpers shared by more than one form.

use rune::TextEdit;

/// A text field seeded with `value` and its caret at the end.
pub(crate) fn edit(value: impl Into<String>) -> TextEdit {
    let mut edit = TextEdit::default();
    edit.text = value.into();
    edit.anchor = edit.text.len();
    edit.focus = edit.text.len();
    edit
}

/// Split a comma, newline, or semicolon separated field into trimmed values.
pub(crate) fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n', ';'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}
