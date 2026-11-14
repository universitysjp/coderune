//! Fixed light theme for Coderune's native desktop.
//!
//! Values are transcribed from `packages/ui/src/tokens.ts` and
//! `docs/rust-gpui/01-design-system.md`. There is no dark theme. Blue is a
//! non-text accent; focus rings are ink.

use gpui::{rgb, Rgba};

pub const GEIST_TTF: &[u8] =
    include_bytes!("../../../packages/tokens/fonts/Geist-Variable.ttf");
pub const PAPER_MONO_TTF: &[u8] =
    include_bytes!("../../../packages/tokens/fonts/PaperMono-Variable.ttf");

pub const FONT_SANS: &str = "Geist";
pub const FONT_MONO: &str = "Paper Mono";

pub fn bg() -> Rgba { rgb(0xfcfcfc) }
pub fn panel() -> Rgba { rgb(0xffffff) }
pub fn ink() -> Rgba { rgb(0x161616) }
pub fn muted() -> Rgba { rgb(0x454545) }
pub fn faint() -> Rgba { rgb(0x656565) }
pub fn line() -> Rgba { rgb(0xdedede) }
pub fn row_line() -> Rgba { rgb(0xececec) }
pub fn control_line() -> Rgba { rgb(0xdadada) }
pub fn control_hover() -> Rgba { rgb(0xb4b4b4) }
pub fn accent() -> Rgba { rgb(0x4c94fc) }
pub fn accent_soft() -> Rgba { rgb(0xe3eefa) }
pub fn danger() -> Rgba { rgb(0xb72728) }
pub fn danger_soft() -> Rgba { rgb(0xffe6e3) }
pub fn success_soft() -> Rgba { rgb(0xf0f0f0) }
pub fn code() -> Rgba { rgb(0xf0f0f0) }
pub fn hover() -> Rgba { rgb(0xf3f3f3) }
pub fn pressed() -> Rgba { rgb(0xececec) }
pub fn disabled() -> Rgba { rgb(0xa2a2a2) }
pub fn primary_hover() -> Rgba { rgb(0x373737) }
pub fn inverse() -> Rgba { rgb(0xffffff) }

pub const DISABLED_OPACITY: f32 = 0.4;

pub mod space {
    pub const S0: f32 = 0.0;
    pub const S0_5: f32 = 2.0;
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 8.0;
    pub const S3: f32 = 12.0;
    pub const S4: f32 = 16.0;
    pub const S5: f32 = 20.0;
    pub const S6: f32 = 24.0;
    pub const S8: f32 = 32.0;
    pub const S10: f32 = 40.0;
}

pub mod radius {
    pub const INNER: f32 = 4.0;
    pub const SOFT: f32 = 6.0;
    pub const FRAME: f32 = 10.0;
}

pub mod layout {
    pub const SIDEBAR: f32 = 248.0;
    pub const SIDEBAR_ICON: f32 = 48.0;
    pub const TOPBAR: f32 = 56.0;
    pub const GUTTER: f32 = 24.0;
    pub const GUTTER_SM: f32 = 16.0;
    pub const CONTAINER: f32 = 1120.0;
    pub const PAGE_HEADER_MAX: f32 = 720.0;
    pub const OVERLAY: f32 = 480.0;
    pub const OVERLAY_LG: f32 = 640.0;
    pub const FIELD_MAX_HEIGHT: f32 = 176.0;
}

pub const ICON_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Display,
    HeadingLg,
    Heading,
    HeadingSm,
    HeadingXs,
    BodyLg,
    Body,
    BodySm,
    Label,
    LabelSm,
    Button,
    Code,
    CodeSm,
}

impl TextRole {
    pub fn size(self) -> f32 {
        match self {
            Self::Display => 40.0,
            Self::HeadingLg => 32.0,
            Self::Heading => 24.0,
            Self::HeadingSm => 20.0,
            Self::HeadingXs => 16.0,
            Self::BodyLg => 16.0,
            Self::Body => 14.0,
            Self::BodySm => 13.0,
            Self::Label => 14.0,
            Self::LabelSm => 12.0,
            Self::Button => 16.0,
            Self::Code => 13.0,
            Self::CodeSm => 12.0,
        }
    }

    pub fn line_height(self) -> f32 {
        match self {
            Self::Display => 44.0,
            Self::HeadingLg => 36.0,
            Self::Heading => 28.0,
            Self::HeadingSm => 24.0,
            Self::HeadingXs => 20.0,
            Self::BodyLg => 22.0,
            Self::Body => 20.0,
            Self::BodySm => 18.0,
            Self::Label | Self::LabelSm => 16.0,
            Self::Button => 20.0,
            Self::Code => 20.0,
            Self::CodeSm => 16.0,
        }
    }

    pub fn family(self) -> &'static str {
        match self {
            Self::Code | Self::CodeSm => FONT_MONO,
            _ => FONT_SANS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Ink,
    Muted,
    Faint,
    Danger,
    Inverse,
    Disabled,
}

impl Tone {
    pub fn color(self) -> Rgba {
        match self {
            Self::Ink => ink(),
            Self::Muted => muted(),
            Self::Faint => faint(),
            Self::Danger => danger(),
            Self::Inverse => inverse(),
            Self::Disabled => disabled(),
        }
    }
}

/// Apply a native type role to any GPUI styled element.
pub trait TypeStyled: gpui::Styled + Sized {
    fn type_role(self, role: TextRole) -> Self {
        self.font_family(role.family())
            .text_size(gpui::px(role.size()))
            .line_height(gpui::px(role.line_height()))
    }

    fn tone(self, tone: Tone) -> Self {
        self.text_color(tone.color())
    }
}

impl<T: gpui::Styled + Sized> TypeStyled for T {}
