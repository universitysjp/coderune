# Bot avatars are a Rust port drawn as Rune shapes

Agents need a face in the desktop app: something that reads as "this tool is
here and this is what it is doing" at 28–48 points, in a list of four rows. We
port the [bot-avatars](https://github.com/Jakubantalik/Libraries.dev/tree/main/packages/bot-avatars)
package (MIT, © Jakub Antalik) into `crates/avatars` rather than embedding a web
view or shipping sprite sheets, because the app is native, offline and has to
animate the avatars itself.

The animation survives whole: `engine.ts` is arithmetic over a small pose, so
the sim, the wanderers, the events and the state cross-fade port line for line.
The renderer does not. Upstream composites canvas 2D paths and, in its default
`fabric` shading, a per-pixel plush material with a baked height field, fur
locks and a matcap. Rune paints rounded rects, gradients and polygon fills; it
has no group clipping to a path and no per-pixel shading hook for app code. So
the port draws the same slice stack — the body is seventeen copies of its
outline spaced along a depth axis and projected with the head's pose — and
replaces the gradient fills with bands of flat colour along the light's axis,
clipped to each polygon. Adjacent bands overlap so their hard interior edges do
not read as seams.

`crisp` is the default shading: a lit rim and a lit front. `smooth` keeps the
depth ramp without the rim, and `flat` keeps only the form. The `fabric` and
`plastic` materials are deliberately out of scope: a faithful port needs a
software rasteriser producing an RGBA buffer per frame and a per-pixel upload,
which is a different module and a different performance profile. Hats, glasses
and headphones are also out of scope for now; the rig and the depth stack leave
room for them.

The upstream outlines are SVG path strings. `crates/avatars/upstream/` keeps
`shapes.ts` and `presets.ts`, and `scripts/gen-avatar-shapes.ts` generates the
Rust constants from them, so refreshing upstream is one command
(`bun run avatars:shapes`) and the port stays honest about where its geometry
comes from.

The desktop app owns one `BotAvatarStage`; `poll` advances the sims only on the
screens that draw avatars and returns false everywhere else, so an idle window
still draws no frames.
