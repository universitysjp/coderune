use gpui::{
    div, prelude::FluentBuilder, px, ClickEvent, CursorStyle, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, App,
};

use crate::theme::{self, radius, space, TextRole, TypeStyled};

pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Md,
    Lg,
}

impl ButtonSize {
    pub fn min_height(self) -> f32 {
        match self {
            Self::Xs => 24.0,
            Self::Sm => 28.0,
            Self::Md => 32.0,
            Self::Lg => 44.0,
        }
    }

    pub fn padding_x(self) -> f32 {
        match self {
            Self::Xs => space::S2,
            _ => space::S3,
        }
    }

    pub fn role(self) -> TextRole {
        match self {
            Self::Lg => TextRole::Button,
            _ => TextRole::Label,
        }
    }
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    full_width: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        variant: ButtonVariant,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant,
            size: ButtonSize::Md,
            disabled: false,
            full_width: false,
            on_click: None,
        }
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Self {
            id,
            label,
            variant,
            size,
            disabled,
            full_width,
            on_click,
        } = self;

        let (rest_bg, hover_bg, active_bg, border, text) = match variant {
            ButtonVariant::Primary => (
                Some(theme::ink()),
                Some(theme::primary_hover()),
                Some(theme::primary_hover()),
                theme::control_line(),
                theme::inverse(),
            ),
            ButtonVariant::Secondary => (
                Some(theme::panel()),
                Some(theme::hover()),
                Some(theme::pressed()),
                theme::control_line(),
                theme::ink(),
            ),
            ButtonVariant::Ghost => (
                None,
                Some(theme::hover()),
                Some(theme::pressed()),
                gpui::rgba(0x00000000),
                theme::ink(),
            ),
            ButtonVariant::Destructive => (
                Some(theme::panel()),
                Some(theme::danger_soft()),
                Some(theme::pressed()),
                theme::danger(),
                theme::danger(),
            ),
        };

        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .min_h(px(size.min_height()))
            .px(px(size.padding_x()))
            .py(px(space::S1))
            .rounded(px(radius::SOFT))
            .tab_index(0)
            .tab_stop(!disabled)
            .border_1()
            .border_color(border)
            .type_role(size.role())
            .text_color(text)
            .when(full_width, |this| this.w_full())
            .when_some(rest_bg, |this, color| this.bg(color))
            .when(disabled, |this| this.opacity(theme::DISABLED_OPACITY))
            .when(!disabled, |this| {
                this.cursor(CursorStyle::PointingHand)
                    .hover(move |style| {
                        let style = match hover_bg {
                            Some(color) => style.bg(color),
                            None => style,
                        };
                        style.border_color(theme::control_hover())
                    })
                    .active(move |style| {
                        let style = match active_bg {
                            Some(color) => style.bg(color),
                            None => style,
                        };
                        style.border_color(border)
                    })
            })
            .when(!disabled, |this| {
                this.when_some(on_click, |this, handler| this.on_click(handler))
            })
            .child(label)
    }
}
