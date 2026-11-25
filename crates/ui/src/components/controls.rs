//! Interactive controls: conditional building, push buttons, badges, alerts,
//! fields, navigation rows, and shortcut keycaps.

use rune::{col, icon, row, text, El, Icon, Shadow, TextStyle};
use rune_kit::{Badge, Button, ControlSize, Kit};

use crate::theme::{self, BODY_SM, LABEL};

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
