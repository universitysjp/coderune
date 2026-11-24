//! Desktop appearance tokens for Rune controls.

use std::{cell::Cell, sync::OnceLock};

use rune::{Color, Family, TextStyle};
use rune_kit::{kit::Type, Kit};

thread_local! {
    static DARK: Cell<bool> = const { Cell::new(false) };
}

/// Resolve semantic colors and controls for one synchronous view render.
pub fn with_dark<R>(dark: bool, render: impl FnOnce() -> R) -> R {
    struct Restore(bool);

    impl Drop for Restore {
        fn drop(&mut self) {
            DARK.set(self.0);
        }
    }

    let _restore = Restore(DARK.replace(dark));
    render()
}

pub fn kit() -> &'static Kit {
    kit_for(DARK.get())
}

pub fn kit_for(dark: bool) -> &'static Kit {
    static KITS: [[OnceLock<Kit>; 2]; 2] = [const { [const { OnceLock::new() }; 2] }; 2];
    KITS[usize::from(dark)][usize::from(rune::theme::blends_srgb())].get_or_init(|| {
        let mut k = rune_theme_opencode::kit(dark).clone();
        k.ty = Type {
            ui: BODY.family(Family::Sans),
            ui_medium: BODY.family(Family::Sans),
            ui_compact: LABEL.family(Family::Sans),
            small: LABEL_SM.family(Family::Sans),
            small_medium: LABEL_SM.family(Family::Sans),
            body: BODY.family(Family::Sans),
            body_medium: BODY.family(Family::Sans),
            text_small: BODY_SM.family(Family::Sans),
            text_small_medium: BODY_SM.family(Family::Sans),
            text_xsmall: LABEL_SM.family(Family::Sans),
            text_xsmall_medium: LABEL_SM.family(Family::Sans),
            mono: CODE,
        };
        if dark {
            let c = &mut k.c;
            // Use neutral grays for dark selections and focus.
            c.bg_accent = c.text_faint;
            c.text_accent = c.text_base;
            c.icon_accent = c.icon_base;
            c.border_focus = c.text_base;
            c.border_active = c.border_strong;
            c.info.bg = c.layer_02;
            c.info.fg = c.text_muted;
            c.info.border = c.border_base;
            c.info.icon = c.icon_muted;
        }
        k
    })
}

/// Switches share the workspace's blue accent in both appearances.
pub fn switch_kit(dark: bool) -> &'static Kit {
    if !dark {
        return kit_for(false);
    }
    static KITS: [OnceLock<Kit>; 2] = [const { OnceLock::new() }; 2];
    KITS[usize::from(rune::theme::blends_srgb())].get_or_init(|| {
        let mut kit = kit_for(true).clone();
        kit.c.bg_accent = kit_for(false).c.bg_accent;
        kit
    })
}

pub fn bg() -> Color {
    kit().c.bg_deep
}

pub fn surface() -> Color {
    let k = kit();
    if k.dark {
        k.c.layer_01
    } else {
        k.c.page
    }
}

pub fn panel() -> Color {
    kit().c.bg_base
}

pub fn ink() -> Color {
    kit().c.text_base
}

pub fn muted() -> Color {
    kit().c.text_muted
}

pub fn faint() -> Color {
    kit().c.text_faint
}

pub fn line() -> Color {
    kit().c.border_base
}

pub fn row_line() -> Color {
    kit().c.border_muted
}

pub fn control_line() -> Color {
    kit().c.border_base
}

pub fn control_hover() -> Color {
    kit().c.border_strong
}

pub fn accent() -> Color {
    kit().c.bg_accent
}

pub fn accent_soft() -> Color {
    kit().c.layer_02
}

pub fn danger() -> Color {
    kit().c.danger.fg
}

pub fn danger_soft() -> Color {
    kit().c.danger.bg
}

pub fn code() -> Color {
    kit().c.layer_02
}

pub fn hover() -> Color {
    kit().c.hover
}

pub fn pressed() -> Color {
    kit().c.pressed
}

pub fn inverse() -> Color {
    kit().c.text_inverse
}

pub const SIDEBAR: f32 = 236.0;
pub const TOPBAR: f32 = 48.0;
pub const FRAME: f32 = 8.0;
pub const GUTTER: f32 = 32.0;
pub const CONTAINER: f32 = 1120.0;
pub const PAGE_HEADER_MAX: f32 = 720.0;
pub const FIELD_MAX_HEIGHT: f32 = 176.0;
pub const ICON: f32 = 20.0;

pub const HEADING_LG: TextStyle = TextStyle::new(32.0, 400.0).lh(36.0);
pub const HEADING: TextStyle = TextStyle::new(24.0, 400.0).lh(28.0);
pub const HEADING_SM: TextStyle = TextStyle::new(20.0, 400.0).lh(24.0);
pub const HEADING_XS: TextStyle = TextStyle::new(16.0, 400.0).lh(20.0);
pub const BODY: TextStyle = TextStyle::new(14.0, 400.0).lh(20.0);
pub const BODY_SM: TextStyle = TextStyle::new(13.0, 400.0).lh(18.0);
pub const LABEL: TextStyle = TextStyle::new(14.0, 400.0).lh(16.0);
pub const LABEL_SM: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0);
pub const CODE: TextStyle = TextStyle::new(13.0, 400.0).lh(20.0).mono();
pub const CODE_SM: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0).mono();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_appearance_reaches_shared_controls_and_restores_the_theme() {
        let light_surface = surface();
        with_dark(true, || {
            assert!(kit().dark);
            assert_ne!(surface(), light_surface);
            assert_eq!(with_dark(false, surface), light_surface);
            assert!(kit().dark);
        });
        assert!(!kit().dark);
    }

    #[test]
    fn dark_chrome_uses_grays_for_selection_and_interaction() {
        let palette = rune_theme_opencode::kit(true);
        with_dark(true, || {
            let colors = &kit().c;
            assert_eq!(surface(), palette.c.layer_01);
            assert_eq!(panel(), palette.c.bg_base);
            for color in [accent(), accent_soft(), colors.border_focus, colors.info.fg] {
                assert_eq!(color.r, color.g);
                assert_eq!(color.g, color.b);
            }
        });
    }

    #[test]
    fn dark_switch_uses_the_existing_blue_accent() {
        let accent = switch_kit(true).c.bg_accent;
        assert_eq!(accent, kit_for(false).c.bg_accent);
        assert!(accent.b > accent.r);
        assert_ne!(accent, kit_for(true).c.bg_accent);
    }
}
