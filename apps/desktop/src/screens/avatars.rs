//! Development gallery for bot bodies, states and materials.

use crate::app::CoderuneApp;
use coderune_avatars::{
    BotAvatar, BotAvatarShading, BotAvatarStage, BotAvatarState, BotAvatarType,
};
use coderune_ui::components as ui;
use coderune_ui::theme;
use rune::{col, row, text, El};
use rune_kit::Kit;

/// The sim id for one avatar in the gallery.
pub(crate) fn gallery_id(kind: BotAvatarType, state: BotAvatarState) -> String {
    format!("{kind:?}-{}", state.label())
}

const SAMPLE_BODIES: [BotAvatarType; 4] = [
    BotAvatarType::Clover,
    BotAvatarType::Droid,
    BotAvatarType::Blob,
    BotAvatarType::Star,
];
const SAMPLE_SHADING: [(BotAvatarShading, &str); 5] = [
    (BotAvatarShading::Fabric, "Fabric"),
    (BotAvatarShading::Plastic, "Plastic"),
    (BotAvatarShading::Crisp, "Crisp"),
    (BotAvatarShading::Smooth, "Smooth"),
    (BotAvatarShading::Flat, "Flat"),
];
const SAMPLE_STATE: BotAvatarState = BotAvatarState::Working;

fn sample_id(kind: BotAvatarType, label: &str) -> String {
    format!("{kind:?}-{label}")
}

pub(crate) fn register_gallery(stage: &mut BotAvatarStage) {
    for kind in BotAvatarType::ALL {
        for state in BotAvatarState::ALL {
            stage.set(&gallery_id(kind, state), kind, state);
        }
    }
    for kind in SAMPLE_BODIES {
        for (_, label) in SAMPLE_SHADING {
            stage.set(&sample_id(kind, label), kind, SAMPLE_STATE);
        }
    }
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

pub(crate) fn avatar_gallery(s: &CoderuneApp, k: &Kit) -> El {
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

    let mut shading_rows = Vec::new();
    for kind in SAMPLE_BODIES {
        let mut cells = vec![text(kind.label())
            .font(theme::BODY)
            .color(theme::ink())
            .w(90.0)];
        for (shading, label) in SAMPLE_SHADING {
            let avatar = BotAvatar::new(kind).shading(shading);
            let view = s
                .avatars
                .view(&sample_id(kind, label), avatar, SAMPLE_STATE, 64.0);
            cells.push(
                col()
                    .gap(4.0)
                    .items_center()
                    .child(view)
                    .child(caption(label)),
            );
        }
        shading_rows.push(row().gap(16.0).items_center().children(cells));
    }

    col()
        .w_full()
        .gap(16.0)
        .child(ui::page_header(
            "Avatars",
            "Compare bot avatar materials and animation states.",
            vec![],
        ))
        .child(ui::card(k, Some("Shading"), None, shading_rows))
        .child(ui::card(
            k,
            Some("Bodies and states"),
            Some("Idle, working and sleeping, at 64 and at 28 points."),
            rows,
        ))
}
