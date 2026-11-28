//! Desktop appearance tokens for Rune controls.

use std::{cell::Cell, sync::OnceLock};

use rune::{Anim, Color, Curve, Easing, Family, TextStyle};
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

/// The page behind grouped content, one step under a raised card.
pub fn canvas() -> Color {
    kit().c.page
}

/// A raised card: the surface grouped rows and panels share.
pub fn raised() -> Color {
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

/// The fill of the open row in a navigation rail.
pub fn rail_selected() -> Color {
    kit().c.layer_03
}

/// The wash under a navigation rail row's pointer.
pub fn rail_hover() -> Color {
    kit().c.layer_01
}

/// A raised card's inner hairline, which also splits its rows.
pub fn card_ring() -> Color {
    kit().c.border_muted
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

pub const SIDEBAR: f32 = 264.0;
pub const TOPBAR: f32 = 44.0;
pub const FRAME: f32 = 8.0;
pub const GUTTER: f32 = 28.0;
pub const CONTAINER: f32 = 1080.0;
pub const PAGE_HEADER_MAX: f32 = 640.0;
pub const FIELD_MAX_HEIGHT: f32 = 176.0;
pub const ICON: f32 = 20.0;

/// A two-pane page: a destinations rail beside the open page.
pub const PANE_RAIL: f32 = 240.0;
pub const PANE_MAX: f32 = 728.0;
pub const PANE_CONTAINER: f32 = 1008.0;
pub const PANE_INSET: f32 = 24.0;
pub const PANE_GAP: f32 = 28.0;
pub const PANE_ROW: f32 = 28.0;
pub const PANE_RADIUS: f32 = 6.0;

/// The 120 ms ease every navigation row uses for its hover wash.
pub const HOVER_FADE: Anim = Anim::new(Curve::tween(0.12, Easing::EASE_IN_OUT));
/// A selection wash that settles just after the pointer leaves.
pub const RAIL_FADE: Anim = Anim::new(Curve::tween(0.16, Easing::EASE_OUT));

/// A compact scale. Titles sit close to body size, as desktop shells do, and
/// hierarchy comes from weight and colour more than from size.
pub const HEADING_LG: TextStyle = TextStyle::new(20.0, 500.0).lh(26.0).tracking(-0.3);
pub const HEADING: TextStyle = TextStyle::new(17.0, 500.0).lh(22.0).tracking(-0.2);
pub const HEADING_SM: TextStyle = TextStyle::new(15.0, 500.0).lh(20.0).tracking(-0.1);
pub const HEADING_XS: TextStyle = TextStyle::new(13.5, 500.0).lh(18.0);
pub const BODY: TextStyle = TextStyle::new(13.0, 400.0).lh(19.0);
pub const BODY_SM: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0);
pub const LABEL: TextStyle = TextStyle::new(13.0, 450.0).lh(16.0);
pub const LABEL_SM: TextStyle = TextStyle::new(11.0, 450.0).lh(14.0);
pub const CODE: TextStyle = TextStyle::new(12.0, 400.0).lh(18.0).mono();
pub const CODE_SM: TextStyle = TextStyle::new(11.0, 400.0).lh(15.0).mono();

/// A Settings page's title and its description line.
pub const SETTINGS_TITLE: TextStyle = TextStyle::new(15.0, 640.0).lh(20.0);
pub const SETTINGS_DESCRIPTION: TextStyle = TextStyle::new(13.0, 440.0).lh(24.0);
/// A Settings rail destination's label.
pub const SETTINGS_RAIL: TextStyle = TextStyle::new(13.0, 440.0).lh(16.0).tracking(-0.04);
/// A Settings section's title, over its card.
pub const SETTINGS_SECTION: TextStyle = TextStyle::new(13.0, 530.0).lh(16.0);
/// A Settings row's title, and the description under it.
pub const SETTINGS_ROW_TITLE: TextStyle = TextStyle::new(13.0, 530.0).lh(16.0);
pub const SETTINGS_ROW_DESCRIPTION: TextStyle = TextStyle::new(13.0, 440.0).lh(20.0);
/// The mode cards' names in Preferences.
pub const SETTINGS_MODE_NAME: TextStyle = TextStyle::new(12.0, 530.0).lh(16.0);

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
