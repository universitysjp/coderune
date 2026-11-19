# coderune-avatars

Animated bot avatars for agents, drawn with Rune's vector canvas.

Eighteen bodies with living faces — round eyes that blink and glance, and an
optional mouth — and three states an agent can be in: idle, working and
sleeping. Each one turns its head, looks around, hops, flips and breathes, and a
state change eases from the old pose to the new one instead of snapping.
The face uses pure white ink on light and dark backgrounds. Use `.ink(...)`
to set an explicit face colour.

```rust
use coderune_avatars::{BotAvatar, BotAvatarState, BotAvatarType};

// A once-a-frame tick, next to the rest of the app's poll.
stage.set("Claude Code", BotAvatarType::Clover, BotAvatarState::Working);
let moved = stage.tick(now_seconds, 1.0);

// In the view.
let portrait = match stage.sim("Claude Code") {
    Some(sim) => BotAvatar::new(BotAvatarType::Clover).view(sim, 44.0),
    None => BotAvatar::new(BotAvatarType::Clover).still(BotAvatarState::Working, 44.0),
};
```

`BotAvatarStage` owns the sims and one clock; the view only reads poses, which
keeps screens thin over app state as the rest of the desktop app is.

## Ported from bot-avatars

The rig, the eighteen outlines, the preset palette and the face are a straight
port of [`bot-avatars`](https://github.com/Jakubantalik/Libraries.dev/tree/main/packages/bot-avatars)
by Jakub Antalik (MIT). `crates/avatars/upstream/` keeps the two upstream
sources the generators read; `bun run avatars:shapes` regenerates
`src/shapes.rs` and `src/presets.rs` from them.

What the port draws differently, and why:

| Upstream | Here |
| --- | --- |
| `fabric` (plush fur) and `plastic` materials, shaded per pixel | `crisp`, `smooth` and `flat`, all vector: Rune paints flats, so the light is banded across the body's front. The per-pixel materials need a software rasteriser, not a shape list. |
| Hats, glasses, headphones, bow ties | Not ported yet; the rig and the depth stack leave room for them. |
| A custom `path` prop | Not ported; the preset outlines are flattened once and cached. |

Everything else matches: the same pose values, the same easing curves, the same
face geometry, the same hop and flip timings, and the cartoon whirl round a
spin (off by default, as upstream ships it).

## Layout

- `src/engine.rs` — the rig: sim, pose, wanderers, events, jumps.
- `src/render.rs` — pose to `rune::Shape`: the slice stack, the face, the whirl.
- `src/svg.rs` — SVG path data to flattened polygons, plus the affines.
- `src/color.rs` — sRGB colour, HSL shading and OKLab vividness.
- `src/shapes.rs`, `src/presets.rs` — generated from `upstream/`.
- `src/lib.rs` — the widget, the look builder and `BotAvatarStage`.
