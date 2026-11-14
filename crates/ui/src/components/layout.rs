use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, CursorStyle, Div, ElementId, IntoElement,
    InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, Window,
};

use crate::theme::{self, layout, radius, space, TextRole, Tone, TypeStyled};

pub fn card(title: Option<&str>, description: Option<&str>, body: impl IntoElement) -> Div {
    let mut card = div()
        .flex()
        .flex_col()
        .bg(theme::panel())
        .border_1()
        .border_color(theme::line())
        .rounded(px(radius::FRAME))
        .overflow_hidden();
    if let Some(title) = title {
        let mut header = div()
            .flex()
            .flex_col()
            .gap(px(space::S1))
            .p(px(space::S4))
            .border_b_1()
            .border_color(theme::row_line())
            .child(crate::text(title, TextRole::HeadingXs, Tone::Ink));
        if let Some(description) = description {
            header = header.child(crate::text(description, TextRole::BodySm, Tone::Muted));
        }
        card = card.child(header);
    }
    card.child(
        div()
            .flex()
            .flex_col()
            .gap(px(space::S4))
            .p(px(space::S4))
            .child(body),
    )
}

pub fn page_header(title: &str, description: &str, actions: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(space::S4))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(space::S2))
                .flex_1()
                .max_w(px(layout::PAGE_HEADER_MAX))
                .child(crate::text(title, TextRole::HeadingLg, Tone::Ink))
                .child(crate::text(description, TextRole::Body, Tone::Muted)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(space::S2))
                .child(actions),
        )
}

pub fn field(label: &str, hint: Option<&str>, control: impl IntoElement) -> Div {
    let mut root = div()
        .flex()
        .flex_col()
        .gap(px(space::S2))
        .child(crate::text(label, TextRole::Label, Tone::Ink))
        .child(control);
    if let Some(hint) = hint {
        root = root.child(crate::text(hint, TextRole::BodySm, Tone::Muted));
    }
    root
}

pub fn alert(error: Option<&str>, message: &str) -> Div {
    let mut root = div()
        .flex()
        .flex_col()
        .gap(px(space::S1))
        .p(px(space::S3))
        .bg(theme::panel())
        .border_1()
        .border_color(theme::line())
        .rounded(px(radius::SOFT));
    if error.is_some() {
        root = root.child(crate::text("Action failed", TextRole::Label, Tone::Danger));
        root = root.child(crate::text(message, TextRole::BodySm, Tone::Muted));
    } else {
        root = root.child(crate::text(message, TextRole::BodySm, Tone::Muted));
    }
    root
}

pub fn badge(label: &str) -> Div {
    div()
        .flex()
        .items_center()
        .min_h(px(20.0))
        .px(px(space::S1))
        .rounded(px(radius::INNER))
        .bg(theme::code())
        .type_role(TextRole::LabelSm)
        .text_color(theme::ink())
        .child(label.to_string())
}

pub fn section_title(title: &str, description: &str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(space::S1))
        .child(crate::text(title, TextRole::HeadingSm, Tone::Ink))
        .child(crate::text(description, TextRole::BodySm, Tone::Muted))
}

pub fn nav_item(
    id: impl Into<ElementId>,
    label: &str,
    icon: &str,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(space::S3))
        .min_h(px(40.0))
        .px(px(space::S3))
        .rounded(px(radius::SOFT))
        .cursor(CursorStyle::PointingHand)
        .when(selected, |this| this.bg(theme::accent_soft()))
        .when(!selected, |this| {
            this.hover(|style| style.bg(theme::hover()))
        })
        .text_color(theme::ink())
        .on_click(on_click)
        .child(
            gpui::svg()
                .path(icon.to_string())
                .size(px(theme::ICON_SIZE))
                .flex_shrink_0()
                .text_color(theme::ink()),
        )
        .child(
            div()
                .type_role(TextRole::Label)
                .text_color(theme::ink())
                .child(label.to_string()),
        )
}

pub fn icon_button(
    id: impl Into<ElementId>,
    icon: &str,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(40.0))
        .rounded(px(radius::SOFT))
        .cursor(CursorStyle::PointingHand)
        .when(selected, |this| this.bg(theme::accent_soft()))
        .when(!selected, |this| this.hover(|style| style.bg(theme::hover())))
        .on_click(on_click)
        .child(
            gpui::svg()
                .path(icon.to_string())
                .size(px(theme::ICON_SIZE))
                .text_color(theme::ink()),
        )
}

/// A stateless row used for list surfaces.
pub fn list_row(id: impl Into<ElementId>, body: impl IntoElement, on_click: Option<crate::components::button::ClickHandler>) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .flex_col()
        .gap(px(space::S1))
        .w_full()
        .px(px(space::S3))
        .py(px(space::S2))
        .rounded(px(radius::INNER))
        .hover(|style| style.bg(theme::hover()))
        .when_some(on_click, |this, handler| this.on_click(handler))
        .child(body)
}
