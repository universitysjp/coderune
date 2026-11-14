//! Coderune's fixed light theme for Rune.
//!
//! Values come from `packages/ui/src/tokens.ts` and the native design notes. There is no
//! dark theme. Blue is a non-text accent; focus rings are ink.

use rune::{color::srgb_decode, Color, Family, Shadow, TextStyle};
use rune_kit::{kit::Type, Kit};

fn hex(value: u32) -> Color {
    let channel = |shift: u32| srgb_decode(((value >> shift) & 0xff) as f32 / 255.0);
    Color {
        r: channel(16),
        g: channel(8),
        b: channel(0),
        a: 1.0,
    }
}

pub fn bg() -> Color {
    hex(0xfcfcfc)
}
pub fn panel() -> Color {
    hex(0xffffff)
}
pub fn ink() -> Color {
    hex(0x161616)
}
pub fn muted() -> Color {
    hex(0x454545)
}
pub fn faint() -> Color {
    hex(0x656565)
}
pub fn line() -> Color {
    hex(0xdedede)
}
pub fn row_line() -> Color {
    hex(0xececec)
}
pub fn control_line() -> Color {
    hex(0xdadada)
}
pub fn control_hover() -> Color {
    hex(0xb4b4b4)
}
pub fn accent() -> Color {
    hex(0x4c94fc)
}
pub fn accent_soft() -> Color {
    hex(0xe3eefa)
}
pub fn danger() -> Color {
    hex(0xb72728)
}
pub fn danger_soft() -> Color {
    hex(0xffe6e3)
}
pub fn code() -> Color {
    hex(0xf0f0f0)
}
pub fn hover() -> Color {
    hex(0xf3f3f3)
}
pub fn pressed() -> Color {
    hex(0xececec)
}
pub fn inverse() -> Color {
    hex(0xffffff)
}

pub const SIDEBAR: f32 = 248.0;
pub const TOPBAR: f32 = 56.0;
pub const GUTTER: f32 = 32.0;
pub const CONTAINER: f32 = 1120.0;
pub const PAGE_HEADER_MAX: f32 = 720.0;
pub const FIELD_MAX_HEIGHT: f32 = 176.0;
pub const ICON: f32 = 20.0;

#[allow(dead_code)]
pub const DISPLAY: TextStyle = TextStyle::new(40.0, 400.0).lh(44.0);
pub const HEADING_LG: TextStyle = TextStyle::new(32.0, 400.0).lh(36.0);
pub const HEADING: TextStyle = TextStyle::new(24.0, 400.0).lh(28.0);
pub const HEADING_SM: TextStyle = TextStyle::new(20.0, 400.0).lh(24.0);
pub const HEADING_XS: TextStyle = TextStyle::new(16.0, 400.0).lh(20.0);
#[allow(dead_code)]
pub const BODY_LG: TextStyle = TextStyle::new(16.0, 400.0).lh(22.0);
pub const BODY: TextStyle = TextStyle::new(14.0, 400.0).lh(20.0);
pub const BODY_SM: TextStyle = TextStyle::new(13.0, 400.0).lh(18.0);
pub const LABEL: TextStyle = TextStyle::new(14.0, 400.0).lh(16.0);
pub const LABEL_SM: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0);
pub const CODE: TextStyle = TextStyle::new(13.0, 400.0).lh(20.0).mono();
pub const CODE_SM: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0).mono();

fn ts(size: f32, weight: f32, line: f32) -> TextStyle {
    TextStyle::new(size, weight).lh(line).family(Family::Sans)
}

/// The single light kit every screen reads.
pub fn kit() -> Kit {
    let mut k = Kit::neutral(false);
    k.dark = false;
    k.invalid_text = false;
    k.ring = None;
    k.elevation.raised.clear();
    k.elevation.floating.clear();
    k.elevation.overlay.clear();
    k.elevation.button_neutral = vec![Shadow {
        x: 0.0,
        y: 0.0,
        blur: 0.0,
        spread: 0.5,
        color: control_line(),
        inset: false,
    }];
    k.elevation.button_contrast.clear();
    k.elevation.elements.clear();
    k.elevation.switch_off.clear();
    k.elevation.switch_on.clear();
    k.elevation.md.clear();
    k.elevation.knob.clear();

    k.c.page = bg();
    k.c.bg_base = panel();
    k.c.bg_deep = bg();
    k.c.layer_01 = bg();
    k.c.layer_02 = code();
    k.c.layer_03 = pressed();
    k.c.layer_04 = accent_soft();
    k.c.bg_inverse = ink();
    k.c.bg_contrast = ink();
    k.c.bg_button_neutral = panel();
    k.c.bg_icon_button_contrast = ink();
    k.c.bg_accent = accent();
    k.c.submit_highlight = inverse().alpha(0.0);
    k.c.submit_hover = ink().alpha(0.9);
    k.c.submit_fg = inverse();
    k.c.submit_icon = inverse();
    k.c.text_base = ink();
    k.c.text_muted = muted();
    k.c.text_faint = faint();
    k.c.text_inverse = inverse();
    k.c.text_contrast = inverse();
    k.c.text_accent = ink();
    k.c.icon_base = ink();
    k.c.icon_muted = muted();
    k.c.icon_faint = faint();
    k.c.icon_inverse = inverse();
    k.c.icon_contrast = inverse();
    k.c.icon_accent = ink();
    k.c.border_muted = row_line();
    k.c.border_base = line();
    k.c.border_strong = control_line();
    k.c.border_focus = ink();
    k.c.hover = hover();
    k.c.pressed = pressed();
    k.c.contrast_hover = hex(0x373737);
    k.c.contrast_pressed = hex(0x373737);
    k.c.scrim = ink().alpha(0.25);
    k.c.depth_top = inverse().alpha(0.0);
    k.c.depth_bottom = inverse().alpha(0.0);
    k.c.white = inverse();
    k.c.black = ink();
    k.c.knob = inverse();
    k.c.knob_on = inverse();
    k.c.danger.fg = danger();
    k.c.danger.bg = danger_soft();
    k.c.danger.border = danger();
    k.c.danger.icon = danger();
    k.c.success.fg = ink();
    k.c.success.bg = code();
    k.c.success.border = line();
    k.c.success.icon = ink();
    k.c.warning.fg = ink();
    k.c.warning.bg = code();
    k.c.warning.border = line();
    k.c.warning.icon = ink();
    k.c.info.fg = ink();
    k.c.info.bg = code();
    k.c.info.border = line();
    k.c.info.icon = ink();
    k.c.surface_base = code();
    k.c.surface_weak = code();
    k.c.surface_hover = hover();
    k.c.surface_raised = panel();
    k.c.surface_raised_base = panel();
    k.c.surface_raised_hover = hover();
    k.c.surface_float = panel();
    k.c.surface_disabled = code();
    k.c.border_weak = row_line();
    k.c.border_hover = control_hover();
    k.c.border_active = accent();
    k.c.border_disabled = row_line();
    k.c.text_strong = ink();
    k.c.text_weak = muted();
    k.c.text_invert_strong = inverse();
    k.c.icon_weaker = faint();
    k.c.std_border = control_line();
    k.c.std_icon = ink();
    k.c.std_icon_invert = inverse();
    k.c.std_icon_strong = ink();
    k.c.std_text = ink();
    k.blend_over = Some(bg());

    let ui = ts(14.0, 400.0, 20.0);
    k.ty = Type {
        ui,
        ui_medium: ui,
        ui_compact: ts(14.0, 400.0, 16.0),
        small: ts(12.0, 400.0, 16.0),
        small_medium: ts(12.0, 400.0, 16.0),
        body: ts(14.0, 400.0, 20.0),
        body_medium: ts(14.0, 400.0, 20.0),
        text_small: ts(13.0, 400.0, 18.0),
        text_small_medium: ts(13.0, 400.0, 18.0),
        text_xsmall: ts(12.0, 400.0, 16.0),
        text_xsmall_medium: ts(12.0, 400.0, 16.0),
        mono: TextStyle::new(13.0, 400.0).lh(20.0).mono(),
    };
    k.radius.xs = 2.0;
    k.radius.sm = 4.0;
    k.radius.md = 6.0;
    k.radius.lg = 10.0;
    k.radius.xl = 10.0;
    k
}
