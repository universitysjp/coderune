//! Page frame, raised cards, settings rows, and empty states.

use rune::{col, icon, row, text, El, Icon, ScrollSpec};
use rune_kit::Kit;

use crate::theme::{
    self, BODY, BODY_SM, HEADING_LG, HEADING_SM, HEADING_XS, LABEL, LABEL_SM, SETTINGS_DESCRIPTION,
    SETTINGS_RAIL, SETTINGS_ROW_DESCRIPTION, SETTINGS_ROW_TITLE, SETTINGS_SECTION, SETTINGS_TITLE,
};

use super::controls::WhenExt;
use super::text::subtle;

/// A single continuous frame that keeps child backgrounds inside its border.
pub fn surface_frame(k: &Kit, fill: rune::Color, radius: f32) -> El {
    col()
        .radius(radius)
        .continuous_corners()
        .fill(fill)
        .border(1.0, k.c.border_weak)
        .padding(1.0)
        .clip_rounded(radius)
}

/// A row's inset inside its card, and the gap between sections.
pub const ROW_PAD: f32 = 14.0;
pub const ROW_H: f32 = 44.0;
pub const ROW_H_DESCRIBED: f32 = 56.0;
/// A Settings card's inset, and the vertical padding of each of its rows.
pub const SETTINGS_ROW_PAD: f32 = 20.0;
pub const SECTION_GAP: f32 = 20.0;
pub const SECTION_LABEL_GAP: f32 = 6.0;

/// The page column every workspace destination stacks its sections in.
pub fn page(key: &str) -> El {
    col().key(key).w_full().gap(SECTION_GAP)
}

/// A labelled group of rows on one raised card.
pub fn section(k: &Kit, key: &str, title: &str, rows: impl IntoIterator<Item = El>) -> El {
    col()
        .key(key)
        .w_full()
        .gap(SECTION_LABEL_GAP)
        .child(text(title).font(LABEL_SM).color(theme::muted()))
        .child(rows_card(k, &format!("{key}-card"), rows))
}

/// A [`section`] whose label line carries a control, such as an action.
pub fn section_with(
    k: &Kit,
    key: &str,
    title: &str,
    accessory: El,
    rows: impl IntoIterator<Item = El>,
) -> El {
    col()
        .key(key)
        .w_full()
        .gap(SECTION_LABEL_GAP)
        .child(
            row()
                .w_full()
                .items_center()
                .gap(12.0)
                .child(text(title).font(LABEL_SM).color(theme::muted()).grow(1.0))
                .child(accessory),
        )
        .child(rows_card(k, &format!("{key}-card"), rows))
}

/// Rows on one raised card, a hairline between each pair.
pub fn rows_card(k: &Kit, key: &str, rows: impl IntoIterator<Item = El>) -> El {
    let children = rows.into_iter().enumerate().flat_map(|(index, item)| {
        let separator = (index > 0).then(|| {
            rune::el()
                .h(1.0)
                .mx(ROW_PAD)
                .fill(k.c.border_muted)
                .a11y_hidden()
        });
        separator
            .into_iter()
            .chain(std::iter::once(item.px(ROW_PAD)))
    });
    let mut surface = col()
        .key(key)
        .w_full()
        .radius(k.radius.lg)
        .continuous_corners()
        .fill(theme::raised())
        .clip_rounded(k.radius.lg)
        .children(children);
    for shadow in &k.elevation.raised {
        surface = surface.box_shadow(*shadow);
    }
    surface
}

/// A title, an optional one-line description, and the control on the right.
pub fn setting_row(title: &str, detail: Option<String>, action: El) -> El {
    let described = detail.is_some();
    let mut copy = col()
        .grow(1.0)
        .min_w(0.0)
        .gap(2.0)
        .child(text(title).font(LABEL).color(theme::ink()).wrap());
    if let Some(detail) = detail {
        copy = copy.child(text(detail).font(BODY_SM).color(theme::muted()).wrap());
    }
    row()
        .w_full()
        .min_h(if described { ROW_H_DESCRIBED } else { ROW_H })
        .items_center()
        .gap(16.0)
        .child(copy)
        .child(action)
}

/// A Settings card: a raised panel whose rows are split by hairline rules and inset
/// `SETTINGS_ROW_PAD` from its sides.
pub fn settings_card(k: &Kit, key: &str, rows: impl IntoIterator<Item = El>) -> El {
    let rows: Vec<El> = rows.into_iter().collect();
    let count = rows.len();
    let children = rows.into_iter().enumerate().flat_map(|(index, row)| {
        let divider = (index + 1 < count).then(|| rune::rule(0.5, k.c.border_base));
        divider.into_iter().chain(std::iter::once(row))
    });
    col()
        .key(key)
        .w_full()
        .px(SETTINGS_ROW_PAD)
        .radius(k.radius.md)
        .continuous_corners()
        .fill(theme::raised())
        .border(1.0, theme::card_ring())
        .clip_rounded(k.radius.md)
        .children(children)
}

/// A Settings row: its title and description beside a control kept at the row's end.
pub fn settings_row(title: &str, description: Option<String>, control: El) -> El {
    let mut copy = col().grow(1.0).shrink().min_w(0.0).gap(8.0).child(
        text(title)
            .key("title")
            .font(SETTINGS_ROW_TITLE)
            .color(theme::ink())
            .wrap(),
    );
    if let Some(description) = description {
        copy = copy.child(
            text(description)
                .key("about")
                .font(SETTINGS_ROW_DESCRIPTION)
                .color(theme::muted())
                .wrap(),
        );
    }
    row()
        .w_full()
        .gap(16.0)
        .py(SETTINGS_ROW_PAD)
        .center_y()
        .child(copy)
        .child(row().key("control").shrink().justify_end().child(control))
}

/// A Settings section: an optional title over its card, 16 points apart.
pub fn settings_section(_k: &Kit, key: &str, title: Option<&str>, card: El) -> El {
    col()
        .key(key)
        .w_full()
        .gap(16.0)
        .children(title.map(|title| {
            text(title)
                .font(SETTINGS_SECTION)
                .color(theme::ink())
                .key("title")
        }))
        .child(card)
}

/// A two-pane screen: a sticky rail of destinations beside the open page, both
/// inside one scroll view at the Settings screen's width and insets.
pub fn two_pane(key: &str, rail: El, content: El) -> El {
    col()
        .key(key.to_owned())
        .w_full()
        .h_full()
        .grow(1.0)
        .shrink()
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .scroll_edge_fade(16.0)
        .child(
            row().w_full().px(theme::PANE_INSET).justify_center().child(
                row()
                    .w_full()
                    .max_w(theme::PANE_CONTAINER)
                    .items_start()
                    .child(rail)
                    .child(content),
            ),
        )
}

/// The fixed column a two-pane screen's destinations sit in.
pub fn pane_rail(key: &str) -> El {
    col()
        .key(key.to_owned())
        .w(theme::PANE_RAIL)
        .shrink()
        .pt(32.0)
        .pb(48.0)
        .pr(24.0)
        .gap(16.0)
        .sticky_top(0.0)
}

/// A rail's destinations, as one keyboard group.
pub fn pane_nav(key: &str) -> El {
    col()
        .key(key.to_owned())
        .role("tablist")
        .focus_group()
        .w_full()
        .gap(4.0)
}

/// A 28 pt rail destination: glyph, label, a 6 pt radius, and the shared
/// 120 ms hover wash.
pub fn pane_nav_item<S: 'static>(
    key: &str,
    label: &str,
    glyph: &'static Icon,
    selected: bool,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    pane_nav_item_with(key, label, glyph, selected, None, on_press)
}

/// [`pane_nav_item`] with a slot at its end, such as a count or a state dot.
pub fn pane_nav_item_with<S: 'static>(
    key: &str,
    label: &str,
    glyph: &'static Icon,
    selected: bool,
    trailing: Option<El>,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let ink = if selected {
        theme::ink()
    } else {
        theme::muted()
    };
    let mut row = row()
        .key(key.to_owned())
        .role("tab")
        .label(label)
        .focusable(true)
        .focus_selected(selected)
        .w_full()
        .h(theme::PANE_ROW)
        .px(6.0)
        .gap(8.0)
        .center_y()
        .radius(theme::PANE_RADIUS)
        .fill(if selected {
            theme::rail_selected()
        } else {
            rune::Color::TRANSPARENT
        })
        .hover(if selected {
            theme::rail_selected()
        } else {
            theme::rail_hover()
        })
        .hover_motion(theme::HOVER_FADE, theme::HOVER_FADE)
        .on_press(on_press)
        .child(icon(glyph, 16.0).color(ink).a11y_hidden())
        .child(
            text(label)
                .font(SETTINGS_RAIL)
                .color(ink)
                .grow(1.0)
                .min_w(0.0)
                .truncate()
                .a11y_hidden(),
        );
    if let Some(trailing) = trailing {
        row = row.child(trailing);
    }
    row
}

/// The open page beside a rail, capped at the readable line length.
pub fn pane_page(key: &str) -> El {
    col()
        .key(key.to_owned())
        .grow(1.0)
        .shrink()
        .min_w(0.0)
        .max_w(theme::PANE_MAX)
        .pl(theme::PANE_GAP)
}

/// A two-pane page's title band: sticky under the window chrome, with the
/// Settings screen's 15/640 title and 13/440 description.
pub fn pane_heading(key: &str, title: &str, description: &str) -> El {
    pane_heading_row(key, title, description, Vec::new())
}

/// [`pane_heading`] with a trailing action row.
pub fn pane_heading_row(key: &str, title: &str, description: &str, actions: Vec<El>) -> El {
    let mut band = row()
        .key(key.to_owned())
        .w_full()
        .items_start()
        .gap(16.0)
        .pt(32.0)
        .fill(theme::canvas())
        .sticky_top(0.0)
        .child(settings_header(title, description).grow(1.0).min_w(0.0));
    if !actions.is_empty() {
        band = band.child(row().shrink().items_center().gap(8.0).children(actions));
    }
    band
}

/// The scrolling body under a pane heading.
pub fn pane_body(key: &str, label: &str) -> El {
    col()
        .key(key.to_owned())
        .role("group")
        .label(label)
        .w_full()
        .gap(32.0)
        .pt(24.0)
        .pb(80.0)
}

/// A Settings page's header: its title and description, 16 points apart from the body.
pub fn settings_header(title: &str, description: &str) -> El {
    col()
        .key("settings-tab-header")
        .w_full()
        .gap(4.0)
        .child(text(title).font(SETTINGS_TITLE).color(theme::ink()))
        .children((!description.is_empty()).then(|| {
            text(description)
                .font(SETTINGS_DESCRIPTION)
                .color(theme::muted())
                .w_full()
                .wrap()
        }))
}

pub fn page_header(title: &str, description: &str, actions: Vec<El>) -> El {
    let heading = col()
        .grow(1.0)
        .min_w(280.0)
        .shrink()
        .gap(6.0)
        .max_w(theme::PAGE_HEADER_MAX)
        .child(text(title).font(HEADING_LG).color(theme::ink()))
        .child(text(description).font(BODY).color(theme::muted()).wrap());
    row()
        .w_full()
        .flex_wrap()
        .gap(12.0)
        .items_start()
        .justify_between()
        .child(heading)
        .when(!actions.is_empty(), |this| {
            this.child(row().flex_wrap().gap(8.0).children(actions))
        })
}

pub fn card(k: &Kit, title: Option<&str>, description: Option<&str>, children: Vec<El>) -> El {
    card_leading(k, None, title, description, children)
}

/// A card whose header leads with a visual, such as an agent's avatar.
pub fn card_leading(
    k: &Kit,
    leading: Option<El>,
    title: Option<&str>,
    description: Option<&str>,
    children: Vec<El>,
) -> El {
    let mut root = col()
        .w_full()
        .fill(theme::raised())
        .radius(k.radius.lg)
        .continuous_corners()
        .padding(20.0)
        .gap(16.0)
        .shrink()
        .clip_rounded(k.radius.lg);
    for shadow in &k.elevation.raised {
        root = root.box_shadow(*shadow);
    }
    if let Some(title) = title {
        let mut heading = col()
            .grow(1.0)
            .min_w(0.0)
            .gap(4.0)
            .child(text(title).font(HEADING_XS).color(theme::ink()).wrap());
        if let Some(description) = description {
            heading = heading.child(text(description).font(BODY_SM).color(theme::muted()).wrap());
        }
        let mut header = row().w_full().gap(12.0).items_center();
        if let Some(leading) = leading {
            header = header.child(leading);
        }
        root = root.child(header.child(heading));
    }
    root.child(col().w_full().gap(16.0).items_start().children(children))
}

/// A centered placeholder for a page or card with nothing in it yet.
///
/// The optional glyph sits in a muted tile above the title; the optional
/// action closes the block, so an empty state can start the first item.
pub fn empty_state(
    glyph: Option<&'static Icon>,
    title: &str,
    description: &str,
    action: Option<El>,
) -> El {
    let mut body = col().w_full().items_center().py(32.0).gap(12.0);
    if let Some(glyph) = glyph {
        body = body.child(
            row()
                .size(36.0, 36.0)
                .center()
                .radius(theme::kit().radius.md)
                .fill(theme::code())
                .a11y_hidden()
                .child(icon(glyph, theme::ICON).color(theme::muted())),
        );
    }
    body = body
        .child(text(title).font(HEADING_SM).color(theme::ink()))
        .child(subtle(description));
    if let Some(action) = action {
        body = body.child(action);
    }
    body
}
