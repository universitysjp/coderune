//! The avatar gallery: every body, every state, and the three shadings Rune
//! paints. A development scene for checking the port; it is not in the nav.

use crate::app::CoderuneApp;
use coderune_avatars::{BotAvatar, BotAvatarShading, BotAvatarState, BotAvatarType};
use coderune_ui::components as ui;
use coderune_ui::theme;
use rune::{col, row, text, El};

/// The sim id for one avatar in the gallery.
pub(crate) fn gallery_id(kind: BotAvatarType, state: BotAvatarState) -> String {
    format!("{kind:?}-{}", state.label())
}

fn avatar(s: &CoderuneApp, kind: BotAvatarType, state: BotAvatarState, size: f32) -> El {
    let avatar = BotAvatar::new(kind);
    match s.avatars.sim(&gallery_id(kind, state)) {
        Some(sim) => avatar.view(sim, size),
        None => avatar.still(state, size),
    }
}

fn caption(label: &str) -> El {
    text(label).font(theme::LABEL_SM).color(theme::faint())
}

pub(crate) fn avatar_gallery(s: &CoderuneApp) -> El {
    let mut rows = Vec::new();
    for kind in BotAvatarType::ALL {
        rows.push(
            row()
                .w_full()
                .gap(16.0)
                .items_center()
                .padding(8.0)
                .child(
                    col()
                        .w(110.0)
                        .shrink()
                        .child(text(kind.label()).font(theme::BODY).color(theme::ink()))
                        .child(caption(kind.preset().color)),
                )
                .child(
                    row().gap(12.0).items_center().children(
                        BotAvatarState::ALL
                            .iter()
                            .map(|state| avatar(s, kind, *state, 64.0)),
                    ),
                )
                .child(
                    row().gap(8.0).items_end().children(
                        BotAvatarState::ALL
                            .iter()
                            .map(|state| avatar(s, kind, *state, 28.0)),
                    ),
                ),
        );
    }

    let shadings = [
        BotAvatarShading::Crisp,
        BotAvatarShading::Smooth,
        BotAvatarShading::Flat,
    ];
    let mut shading_rows = Vec::new();
    for kind in [
        BotAvatarType::Clover,
        BotAvatarType::Droid,
        BotAvatarType::Blob,
        BotAvatarType::Star,
    ] {
        let mut cells = vec![text(kind.label())
            .font(theme::BODY)
            .color(theme::ink())
            .w(90.0)];
        for shading in shadings {
            let state = BotAvatarState::Working;
            let avatar = BotAvatar::new(kind).shading(shading);
            let view = match s.avatars.sim(&gallery_id(kind, state)) {
                Some(sim) => avatar.view(sim, 72.0),
                None => avatar.still(state, 72.0),
            };
            cells.push(
                row()
                    .gap(6.0)
                    .items_center()
                    .child(view)
                    .child(caption(shading_label(shading))),
            );
        }
        shading_rows.push(row().gap(16.0).items_center().children(cells));
    }

    col()
        .w_full()
        .gap(16.0)
        .child(ui::page_header(
            "Avatars",
            "The bot avatars ported to Rune: eighteen bodies, three states, three shadings.",
            vec![],
        ))
        .child(ui::card(
            Some("Bodies and states"),
            Some("Idle, working and sleeping, at 64 and at 28 points."),
            rows,
        ))
        .child(ui::card(
            Some("Shading"),
            Some("Crisp lights a rim round the front; smooth ramps the depth; flat keeps the form alone."),
            shading_rows,
        ))
}

fn shading_label(shading: BotAvatarShading) -> &'static str {
    match shading {
        BotAvatarShading::Crisp => "crisp",
        BotAvatarShading::Smooth => "smooth",
        BotAvatarShading::Flat => "flat",
    }
}
