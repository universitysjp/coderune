# Design system

The desktop look and the web look share one palette, type ramp, and control
language. They are implemented twice because the runtimes differ.

## Sources of truth

- Desktop (Rust/Rune): `crates/ui/src/theme.rs` and `crates/ui/src/components.rs`.
- Web (React): `packages/ui/src/web/tokens.css`, `base.css`, and `ui/*`.
- Font assets: `packages/tokens/fonts` (Geist and Paper Mono, OFL).
- Layout, spacing, and type roles: the constants in `crates/ui/src/theme.rs`.

Do not introduce a second palette or type scale in either surface.

## Colour

| Role | Value | Use |
| --- | --- | --- |
| Canvas | `#FCFCFC` | Window and page background |
| Panel | `#FFFFFF` | Cards, fields, menus, sidebar |
| Ink | `#161616` | Primary text, icons, focus ring, primary fill |
| Muted | `#454545` | Secondary text |
| Faint | `#656565` | Tertiary text and placeholders |
| Line | `#DEDEDE` | Component frames |
| Row line | `#ECECEC` | Row rules and dividers |
| Control line | `#DADADA` | Input and secondary-button edges |
| Accent | `#4C94FC` | Non-text marks only |
| Accent wash | `#E3EEFA` | Selected rows, tabs, text selection |
| Danger | `#B72728` | Error and destructive text/edges |
| Danger wash | `#FFE6E3` | Destructive hover |
| Code well | `#F0F0F0` | Code, JSON, paths, read-only wells |

Rules:

- Fixed light appearance. No dark theme.
- Blue is never small text on a light surface.
- Primary fills are ink; destructive actions are paper + danger edge + danger word.
- Focus is a 2px ink ring with a 2px offset.
- Selection is an accent wash with ink content, never a side rail.

## Type

Geist for UI and prose. Paper Mono for code, paths, IDs, hashes, and JSON.

| Role | Size / line | Use |
| --- | --- | --- |
| Display | 40 / 44 | Reserved |
| Heading lg | 32 / 36 | Page title |
| Heading | 24 / 28 | Section titles |
| Heading sm | 20 / 24 | Subsection |
| Heading xs | 16 / 20 | Card title |
| Body lg | 16 / 22 | Lede |
| Body | 14 / 20 | Default prose, page description |
| Body sm | 13 / 18 | Card description, hints, alert body |
| Label | 14 / 16 | Controls, tabs, menu items |
| Label sm | 12 / 16 | Badges and small tags |
| Code | 13 / 20 | Code and JSON |
| Code sm | 12 / 16 | Paths, IDs, hashes |

Weight is 400 only. Hierarchy comes from size, colour, and family. No uppercase,
tracking, or synthetic bold in the desktop UI.

## Space, shape, layout

- Spacing: 0, 2, 4, 8, 12, 16, 20, 24, 32, 40.
- Radius: 4 (inner), 6 (control/card inner), 10 (frame).
- Hairline: 1px; focus: 2px.
- Sidebar 248; topbar 56; page gutter 24/32; content max 1120; field max height 176.
- Controls: 24/28/32/44 high; rows 40/56; icon 20.

## Desktop components

`crates/ui` owns page headers, cards, fields, alerts, badges, navigation rows,
and code wells. `apps/desktop/src/widgets.rs` owns app-specific builders
(buttons, text inputs, choice chips). Buttons, text fields, and menus come from
`rune-kit` so focus, IME, accessibility, and keyboard behavior stay intact.

## Web components

`packages/ui/src/web` is the shared React registry. It is scoped under `.ui`,
compiles Tailwind utilities for its own sources, and exposes badges, buttons,
charts, tables, panels, filters, inputs, and statuses. `apps/web` imports them
from `@coderune/ui/web`; do not duplicate the registry inside the app.

## Verification

- `bun run test`
- `bun run lint`
- `bun run check`
- `bun run verify:ui`
- `bun run --cwd apps/web build`
