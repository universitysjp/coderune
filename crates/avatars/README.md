# coderune-avatars

Animated bot avatars for agents, drawn with Rune's vector canvas and pixel layers.

Eighteen bodies with living faces — round eyes that blink and glance, and an
optional mouth — and three states an agent can be in: idle, working and
sleeping. Each one turns its head, looks around, hops, flips and breathes, and a
state change eases from the old pose to the new one instead of snapping.
The face uses pure white ink on light and dark backgrounds. Use `.ink(...)`
to set an explicit face colour.

```rust
use coderune_avatars::{BotAvatar, BotAvatarShading, BotAvatarState, BotAvatarType};

// A once-a-frame tick, next to the rest of the app's poll.
stage.set("Claude Code", BotAvatarType::Clover, BotAvatarState::Working);
let moved = stage.tick(now_seconds, 1.0);

// In the view.
let avatar = BotAvatar::new(BotAvatarType::Clover).shading(BotAvatarShading::Fabric);
let portrait = stage.view("Claude Code", avatar, BotAvatarState::Working, 44.0);
```

`BotAvatarStage` owns the sims, a shared clock, and one reusable pixel surface
per portrait ID. The Agents screen uses `Fabric`. The development gallery
(`--scene avatars`) compares the five shading modes.

## Ported from bot-avatars

The rig, the eighteen outlines, the preset palette and the face are a straight
port of [`bot-avatars`](https://github.com/Jakubantalik/Libraries.dev/tree/main/packages/bot-avatars)
by Jakub Antalik (MIT). `crates/avatars/upstream/` keeps the two upstream
sources the generators read; `bun run avatars:shapes` regenerates
`src/shapes.rs` and `src/presets.rs` from them.

What the port draws differently, and why:

| Upstream | Here |
| --- | --- |
| `fabric` (plush fur) and `plastic` materials | `Fabric` and `Plastic` use a native software rasterizer with shared body geometry, pose-dependent lighting, and procedural pile. `Crisp`, `Smooth` and `Flat` use the vector path. |
| Hats, glasses, headphones, bow ties | Not ported yet; the rig and the depth stack leave room for them. |
| A custom `path` prop | Not ported; the preset outlines are flattened once and cached. |

All materials use the same rig and vector face, including the hop, flip and
optional whirl around a spin. `BotAvatar::shape` exposes a vector approximation
for callers that draw a `Shape` directly.

## Layout

- `src/engine.rs` — the rig: sim, pose, wanderers, events, jumps.
- `src/render.rs` — body projection, vector shading, the face and the whirl.
- `src/material.rs` — cached body forms and software shading into Rune's `Pixels`.
- `src/svg.rs` — SVG path data to flattened polygons, plus the affines.
- `src/color.rs` — sRGB colour, HSL shading and OKLab vividness.
- `src/shapes.rs`, `src/presets.rs` — generated from `upstream/`.
- `src/lib.rs` — the widget, the look builder and `BotAvatarStage`.
