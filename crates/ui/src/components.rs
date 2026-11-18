//! Small layout helpers shared by every screen.
//!
//! Components that need editing, focus, accessibility, or animation come from `rune-kit`.
//! These helpers add Coderune's frame, type, and spacing roles on top.

use rune::{col, icon, row, text, El, Icon, ScrollSpec};
use rune_kit::{Badge, Kit};

/// Rune has no conditional builder; this keeps the call sites declarative.
pub trait WhenExt: Sized {
    fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self;
}

impl WhenExt for El {
    fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            f(self)
        } else {
            self
        }
    }
}

use crate::theme::{
    self, BODY, BODY_SM, CODE, HEADING_LG, HEADING_SM, HEADING_XS, LABEL, LABEL_SM,
};

pub fn page_header(title: &str, description: &str, actions: Vec<El>) -> El {
    col()
        .w_full()
        .gap(8.0)
        .max_w(theme::PAGE_HEADER_MAX)
        .child(text(title).font(HEADING_LG).color(theme::ink()))
        .child(text(description).font(BODY).color(theme::muted()).wrap())
        .when(!actions.is_empty(), |this| {
            this.child(row().gap(8.0).children(actions))
        })
}

pub fn card(title: Option<&str>, description: Option<&str>, children: Vec<El>) -> El {
    card_leading(None, title, description, children)
}

/// A card whose header leads with a visual, such as an agent's avatar.
pub fn card_leading(
    leading: Option<El>,
    title: Option<&str>,
    description: Option<&str>,
    children: Vec<El>,
) -> El {
    let mut root = col()
        .w_full()
        .fill(theme::panel())
        .border(1.0, theme::line())
        .radius(10.0)
        .shrink()
        .clip_rounded(10.0);
    if let Some(title) = title {
        let mut heading = col()
            .w_full()
            .gap(4.0)
            .child(text(title).font(HEADING_XS).color(theme::ink()));
        if let Some(description) = description {
            heading = heading.child(text(description).font(BODY_SM).color(theme::muted()).wrap());
        }
        let mut header = row()
            .w_full()
            .gap(12.0)
            .items_center()
            .padding(16.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], theme::row_line());
        if let Some(leading) = leading {
            header = header.child(leading);
        }
        root = root.child(header.child(heading));
    }
    root.child(
        col()
            .w_full()
            .padding(16.0)
            .gap(16.0)
            .items_start()
            .children(children),
    )
}

pub fn badge(label: &str, k: &Kit) -> El {
    Badge::new(label).build(k)
}

pub fn alert(error: Option<&str>, message: &str) -> El {
    let mut root = col()
        .w_full()
        .gap(4.0)
        .padding(12.0)
        .fill(theme::panel())
        .border(1.0, theme::line())
        .radius(6.0);
    if error.is_some() {
        root = root.child(text("Action failed").font(LABEL).color(theme::danger()));
    }
    root.child(text(message).font(BODY_SM).color(theme::muted()).wrap())
}

pub fn section_title(title: &str, description: &str) -> El {
    col()
        .gap(4.0)
        .child(text(title).font(HEADING_SM).color(theme::ink()))
        .child(text(description).font(BODY_SM).color(theme::muted()).wrap())
}

pub fn field(label: &str, hint: Option<&str>, control: El) -> El {
    let mut root = col()
        .w_full()
        .gap(8.0)
        .items_start()
        .child(text(label).font(LABEL).color(theme::ink()))
        .child(control);
    if let Some(hint) = hint {
        root = root.child(text(hint).font(BODY_SM).color(theme::muted()).wrap());
    }
    root
}

pub fn nav_item<S: 'static>(
    key: &'static str,
    label: &str,
    glyph: &'static Icon,
    selected: bool,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let base = row()
        .key(key)
        .hit()
        .focusable(true)
        .role("button")
        .label(label)
        .h(32.0)
        .w_full()
        .items_center()
        .gap(10.0)
        .px(8.0)
        .radius(6.0)
        .when(selected, |this| this.fill(theme::accent_soft()))
        .when(!selected, |this| this.hover(theme::hover()));
    base.child(icon(glyph, theme::ICON).color(theme::ink()).a11y_hidden())
        .child(text(label).font(LABEL).color(theme::ink()).a11y_hidden())
        .on_press(on_press)
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
