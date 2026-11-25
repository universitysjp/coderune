//! Small type roles: section headings, code wells, and supporting copy.

use rune::{col, text, El, ScrollSpec};

use crate::theme::{self, BODY_SM, CODE, HEADING_SM, LABEL_SM};

pub fn section_title(title: &str, description: &str) -> El {
    col()
        .gap(4.0)
        .child(text(title).font(HEADING_SM).color(theme::ink()))
        .child(text(description).font(BODY_SM).color(theme::muted()).wrap())
}

pub fn code_well(text_value: &str) -> El {
    col()
        .w_full()
        .fill(theme::code())
        .radius(6.0)
        .padding(12.0)
        .child(text(text_value).font(CODE).color(theme::ink()).wrap())
}

pub fn code_well_tall(text_value: &str) -> El {
    col()
        .w_full()
        .h(theme::FIELD_MAX_HEIGHT)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .fill(theme::code())
        .radius(6.0)
        .padding(12.0)
        .child(text(text_value).font(CODE).color(theme::ink()).wrap())
}

pub fn mono_line(value: &str, color: rune::Color) -> El {
    text(value).font(CODE).color(color).wrap()
}

pub fn subtle(value: &str) -> El {
    text(value).font(BODY_SM).color(theme::muted()).wrap()
}

pub fn tiny(value: &str) -> El {
    text(value).font(LABEL_SM).color(theme::faint()).wrap()
}
