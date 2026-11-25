//! Small app-specific builders shared by screens.

use crate::app::CoderuneApp;
use coderune_domain::ExperienceStage;
use coderune_ui::components::PushButtonExt;
use coderune_ui::theme::{self, LABEL};
use coderune_ui::WhenExt;
use rune::{row, text, El, TextEdit};
use rune_kit::{Button, ButtonVariant, Kit, TextArea, TextInput};

pub(crate) fn primary(label: &str, press: impl Fn(&mut CoderuneApp) + 'static, k: &Kit) -> El {
    Button::new(label)
        .variant(ButtonVariant::Contrast)
        .on_press(press)
        .build_push(k)
}

/// A primary button whose action needs the frame's transaction, such as one
/// that opens a system dialog.
pub(crate) fn primary_tx(
    label: &str,
    press: impl Fn(&mut CoderuneApp, &mut rune::Tx) + 'static,
    k: &Kit,
) -> El {
    Button::new(label)
        .variant(ButtonVariant::Contrast)
        .on_press_tx(press)
        .build_push(k)
}

pub(crate) fn secondary(label: &str, press: impl Fn(&mut CoderuneApp) + 'static, k: &Kit) -> El {
    Button::new(label)
        .variant(ButtonVariant::Neutral)
        .on_press(press)
        .build_push(k)
}

/// A secondary button whose action needs the frame's transaction.
pub(crate) fn secondary_tx(
    label: &str,
    press: impl Fn(&mut CoderuneApp, &mut rune::Tx) + 'static,
    k: &Kit,
) -> El {
    Button::new(label)
        .variant(ButtonVariant::Neutral)
        .on_press_tx(press)
        .build_push(k)
}

pub(crate) fn ghost(label: &str, press: impl Fn(&mut CoderuneApp) + 'static, k: &Kit) -> El {
    Button::new(label)
        .variant(ButtonVariant::Ghost)
        .on_press(press)
        .build_push(k)
}

pub(crate) fn destructive(label: &str, press: impl Fn(&mut CoderuneApp) + 'static, k: &Kit) -> El {
    Button::new(label)
        .variant(ButtonVariant::Danger)
        .on_press(press)
        .build_push(k)
}

pub(crate) fn text_input<'a>(
    value: &TextEdit,
    placeholder: &str,
    label: &str,
    edit: impl for<'b> Fn(&'b mut CoderuneApp) -> &'b mut TextEdit + 'static,
    k: &Kit,
) -> El {
    TextInput::new(value)
        .placeholder(placeholder)
        .label(label)
        .width(None)
        .height(32.0)
        .on_edit(edit)
        .build(k)
}

pub(crate) fn text_area<'a>(
    value: &TextEdit,
    placeholder: &str,
    label: &str,
    rows: usize,
    edit: impl for<'b> Fn(&'b mut CoderuneApp) -> &'b mut TextEdit + 'static,
    k: &Kit,
) -> El {
    TextArea::new(value)
        .placeholder(placeholder)
        .label(label)
        .width(None)
        .rows(rows)
        .on_edit(edit)
        .build(k)
}

pub(crate) fn stage_chips(s: &CoderuneApp) -> El {
    let mut chips = Vec::new();
    for stage in ExperienceStage::ALL {
        let selected = s.profile_form.stage == Some(stage);
        let mut chip = row()
            .key(format!("stage-{stage:?}"))
            .hit()
            .focusable(true)
            .role("button")
            .label(stage.label())
            .h(32.0)
            .px(12.0)
            .center()
            .radius(6.0)
            .border(
                1.0,
                if selected {
                    theme::accent()
                } else {
                    theme::control_line()
                },
            )
            .fill(if selected {
                theme::accent_soft()
            } else {
                theme::panel()
            })
            .when(!selected, |this| this.hover(theme::hover()))
            .child(
                text(stage.label())
                    .font(LABEL)
                    .color(theme::ink())
                    .a11y_hidden(),
            )
            .on_press(move |s: &mut CoderuneApp| {
                s.profile_form.stage = Some(stage);
                s.error = None;
            });
        chip = chip.key(format!("stage-chip-{stage:?}"));
        chips.push(chip);
    }
    row().w_full().gap(8.0).items_center().children(chips)
}

pub(crate) fn option_chip<S: 'static>(
    key: String,
    label: &str,
    selected: bool,
    press: impl Fn(&mut S) + 'static,
) -> El {
    row()
        .key(key)
        .hit()
        .focusable(true)
        .role("button")
        .label(label)
        .h(32.0)
        .px(12.0)
        .center()
        .radius(6.0)
        .border(
            1.0,
            if selected {
                theme::accent()
            } else {
                theme::control_line()
            },
        )
        .fill(if selected {
            theme::accent_soft()
        } else {
            theme::panel()
        })
        .when(!selected, |this| this.hover(theme::hover()))
        .child(text(label).font(LABEL).color(theme::ink()).a11y_hidden())
        .on_press(press)
}

// ---------------------------------------------------------------------------
// Root and shell
// ---------------------------------------------------------------------------
