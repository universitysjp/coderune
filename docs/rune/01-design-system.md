# Design system

The native app has light and dark appearances. The website uses the neutral
light palette in `packages/tokens`.

## Sources of truth

- Desktop (Rust/Rune): `crates/ui/src/theme.rs` and `crates/ui/src/components/`
  (`layout`, `text`, and `controls`).
- Website (React): `packages/tokens/src/tokens.css` and `apps/site/src/index.css`.
- Fonts: Geist and Paper Mono in `packages/tokens/fonts`.
- Layout, spacing, and type roles: the constants in `crates/ui/src/theme.rs`.

Do not introduce a second palette or type scale in either surface.

## Desktop colour

General in Settings offers a persistent Dark mode switch. Window chrome, the
Settings header, and the Settings sidebar use the same gray surface in each
appearance. Dark selection and focus colors use the neutral ramp, and enabled
switches use the app's blue accent in both appearances.

## Web colour

| Role | Value | Use |
| --- | --- | --- |
| Canvas | `#FCFCFC` | Window and page background |
| Panel | `#FFFFFF` | Workspace, fields and menus |
| Surface | `#F7F7F7` | Desktop chrome and grouped page sections |
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

- Fixed light appearance.
- Blue is never small text on a light surface.
- Primary fills are ink; destructive actions are paper + danger edge + danger word.
- Focus is a 2px ink ring with a 2px offset.
- Selection is an accent wash with ink content, never a side rail.

## Type

Geist for UI and prose. Paper Mono for code, paths, IDs, hashes, and JSON.

| Constant | Size / line | Use |
| --- | --- | --- |
| `HEADING_LG` | 20 / 26 | Page title |
| `HEADING` | 17 / 22 | Dialog or welcome title |
| `HEADING_SM` | 15 / 20 | Section title |
| `HEADING_XS` | 13.5 / 18 | Card title |
| `BODY` | 13 / 19 | Default prose, page description |
| `BODY_SM` | 12 / 16 | Card description, hints, alert body |
| `LABEL` | 13 / 16 | Controls, tabs, menu items |
| `LABEL_SM` | 11 / 14 | Badges and small tags |
| `CODE` | 12 / 18 | Code and JSON |
| `CODE_SM` | 11 / 15 | Paths, IDs, hashes |

Hierarchy comes from weight and colour more than from size. Desktop text uses
weight 400. Buttons and shortcut keycaps use Geist.

## Space, shape, layout

- Spacing: 0, 2, 4, 8, 12, 16, 20, 24, 32, 40.
- Radius: 4 (inner), 6 (control), 10 (frame and card).
- Hairline: 1px; focus: 2px.
- The sidebar starts at 264 points and collapses below 1080 points of window
  width. The titlebar is 44 points, the frame inset is 8, and the page gutter
  is 28. Content is capped at 1080 points and tall code wells at 176.
- Every workspace destination builds its body from `ui::page` and `ui::section`:
  a muted label, then one raised card of rows with 14-point insets and hairline
  dividers. Settings, Profile, Projects, Documents, Subagents, and Capture share
  that rhythm; a page never invents its own card padding.
- Cards sit on the page canvas in the raised surface with the kit's raised
  elevation, so a section reads as one object in both appearances.
- Controls: 24/28/32/44 high; rows 44 plain and 56 described; icon 20.

## Desktop components

`crates/ui` owns page headers, cards, fields, alerts, badges, navigation rows,
empty states, and code wells, split into `layout`, `text`, and `controls`
modules. `apps/desktop/src/widgets.rs` owns app-specific builders (button
tones, text inputs, choice chips, status alerts). Buttons, text fields, menus,
the workspace's `Sidebar`, `WindowTitlebar` and `TabStrip`, and the onboarding
`Page` come from `rune-kit`.

Reach for the smallest layer that fits:

1. `rune-kit` when the control needs editing, focus, accessibility, or motion.
2. `coderune_ui` when the piece is part of the desktop design language.
3. `widgets` when the piece knows about `CoderuneApp` state.

Text buttons are 28 points high at standard size and 24 points high at compact
size. Shortcut keycaps use a recessed tray and separate key faces in
Settings → Shortcuts. The Settings and workspace panes share the same continuous
frame and scrolling treatment.

Application updates run in the app: macOS swaps the verified bundle beside the
installed one, and Windows runs the per-user installer quietly. Both reopen
Coderune from the **Restart to update** row in Settings → General.

## Website

`apps/site` is the public React site. `src/components` holds the frame,
navigation, sections, panels and links; `src/routes` holds the pages and copy.
The site keeps its own components; it does not import a shared React registry.

## Verification

- `bun run test`
- `bun run lint`
- `bun run check`
- `bun run verify:ui`
- `bun run build:site`
