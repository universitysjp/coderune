//! Small layout helpers shared by every screen.
//!
//! Components that need editing, focus, accessibility, or animation come from `rune-kit`.
//! These helpers add Coderune's frame, type, and spacing roles on top.

use rune::{col, icon, row, text, El, Icon, ScrollSpec, Shadow, TextStyle};
use rune_kit::{Badge, Button, ControlSize, Kit};

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

/// Standard and compact desktop push buttons.
pub trait PushButtonExt {
    fn build_push(self, k: &Kit) -> El;
    fn build_small_push(self, k: &Kit) -> El;
}

impl PushButtonExt for Button {
    fn build_push(self, k: &Kit) -> El {
        self.control_size(ControlSize::Large)
            .build(k)
            .px(11.0)
            .radius(k.radius.md)
    }

    fn build_small_push(self, k: &Kit) -> El {
        self.control_size(ControlSize::Regular)
            .build(k)
            .px(9.0)
            .radius(k.radius.sm)
    }
}

use crate::theme::{
    self, BODY, BODY_SM, CODE, HEADING_LG, HEADING_SM, HEADING_XS, LABEL, LABEL_SM,
};

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

/// Related rows in one raised card with inset dividers.
pub fn settings_card(k: &Kit, key: &str, rows: impl IntoIterator<Item = El>) -> El {
    rows_card(k, key, rows)
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
        .focus_ring(2.0, -2.0, theme::ink())
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

/// Renders a shortcut as low-profile keycaps in a recessed tray.
/// Supply the accessible shortcut on the control associated with this display.
pub fn keycaps(key: &str, parts: &[&str], k: &Kit) -> El {
    let c = &k.c;
    let (tray_top, tray_bottom, rim, inset) = if k.dark {
        (
            c.bg_deep,
            c.bg_base.mix(c.layer_01, 0.7),
            k.light(0.07),
            0.6,
        )
    } else {
        (
            c.layer_04.mix(c.layer_03, 0.35),
            c.layer_03.mix(c.layer_02, 0.6),
            k.dark_a(0.07),
            0.14,
        )
    };
    let (cap_top, cap_bottom, face, ink, contact, cast) = if k.dark {
        (
            c.layer_03.mix(c.layer_04, 0.35),
            c.bg_deep,
            c.bg_base.mix(c.layer_01, 0.6),
            c.text_base.mix(c.text_faint, 0.15),
            0.8,
            0.5,
        )
    } else {
        (
            c.white,
            c.layer_04.mix(c.text_faint, 0.28).mix(c.warning.fg, 0.05),
            c.layer_02.mix(c.warning.bg, 0.35),
            c.text_base.mix(c.text_muted, 0.15),
            0.2,
            0.12,
        )
    };
    let face_radius = 4.0;
    let cap_rim = 0.5;
    let tray_padding = 3.0;
    let tray_radius = face_radius + cap_rim + tray_padding;
    row()
        .key(key)
        .padding(tray_padding)
        .gap(3.0)
        .center_y()
        .radius(tray_radius)
        .continuous_corners()
        .clip_rounded(tray_radius)
        .a11y_hidden()
        .fill(tray_bottom)
        .gradient(tray_top, tray_bottom)
        .border(0.5, rim)
        .box_shadow(Shadow {
            y: 1.0,
            blur: 2.5,
            color: k.dark_a(inset),
            inset: true,
            ..Shadow::default()
        })
        .children(parts.iter().enumerate().map(|(index, part)| {
            col()
                .key(format!("keycap-{index}"))
                .px(cap_rim)
                .pt(cap_rim)
                .pb(1.5)
                .radius(face_radius + cap_rim)
                .continuous_corners()
                .fill(cap_bottom)
                .gradient(cap_top, cap_bottom)
                .shadow(0.5, 0.5, k.dark_a(contact))
                .shadow(1.5, 3.0, k.dark_a(cast))
                .child(
                    row()
                        .key("face")
                        .min_w(21.0)
                        .h(20.0)
                        .px(6.0)
                        .center()
                        .radius(face_radius)
                        .continuous_corners()
                        .fill(face)
                        .gradient(k.light(0.03), k.dark_a(0.02))
                        .child(
                            text(*part)
                                .font(
                                    TextStyle::new(11.0, 400.0)
                                        .lh(14.0)
                                        .family(rune::Family::Sans),
                                )
                                .color(ink),
                        ),
                )
        }))
}
