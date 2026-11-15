# 01 — Design system for the Rust/Rune rewrite

Status: implementation specification. Visual language is frozen; layout/UX structure is intentionally out of scope.

This document is the single source of truth for colour, type, spacing, dimensions, component appearance, interaction states, icons, and the names to use in Rust. It does **not** prescribe the new layout. Do not copy the old `app-before.tsx` layout.

## 0. Sources and precedence

Read in this order when values disagree:

1. `packages/ui/src/tokens.ts` — canonical resolved native roles (hex, px, weights). This is the authority for the Rust constants.
2. `crates/ui/src/components.rs` and `packages/ui/src/native/internal/styles.ts` — canonical component anatomy, state composition, and accessible names.
3. `packages/tokens/src/tokens.css` + `packages/tokens/README.md` — font registration and the **web/marketing** palette. Only the fonts transfer to the desktop.
4. `packages/ui/src/web/ui/*.tsx`, `packages/ui/src/web/tokens.css`, `packages/ui/src/web/styles.css`, `.agents/skills/ui/SKILL.md` — used only to clarify state intent and hard rules. Web-only values (cream, kind hues, shadows, underlines, 20px button padding) are **not** native.
5. `docs/desktop-ui.md`, `docs/desktop-ui-spec.md`, `.design-system/*.md`.

Guiding sentence from `docs/desktop-ui.md`: *“Native resolved sRGB colors project the UI roles into neutral black/white surfaces; blue marks selection and focusable accents. Primary actions use ink, and destructive actions use a fail edge and word. Cream web roles stay in tokens.css. Geist is for UI and prose; Paper Mono is for code, paths, IDs, hashes, and JSON.”*

Hard constraints already enforced by lint:

- Native TSX may only use UI tokens for dimensions and colours (`ui-native/tokens-only`).
- Native TSX may only use `View`/`Text`/`Input`/`Textarea`, never raw host elements (`ui-native/components-only`).
- Every input/textarea needs a name from one `Field` or an explicit `aria-label` (`ui-native/controls-named`).
- Web app rules additionally ban raw colours, arbitrary values, hue fills, red-filled destructive buttons, pills, shadows on flat things, glass, gradients, mono for numbers, and a second icon set.

---

## 1. Colour

### 1.1 Canonical native colour tokens

All values are **opaque sRGB hex**. There is no dark theme. `nativeTheme.appearance` is fixed to `"light"`.

| Token | Hex | Derivation / role | Semantic usage |
| --- | --- | --- | --- |
| `bg` | `#FCFCFC` | ui canvas at 99.1% OKLCH, neutral | App canvas behind everything; onboarding page background; loading screen |
| `panel` | `#FFFFFF` | paper at 100% | Cards, fields, menus, tooltips, alerts, sidebar; selected tab fill; menu surface |
| `ink` | `#161616` | ink at 20% OKLCH, chroma removed | All primary text, icons, focus ring, ink rules; primary button fill; caret |
| `muted` | `#454545` | ink at 80% over paper | Secondary text: descriptions, hints, alert bodies, meta, loading text |
| `faint` | `#656565` | ink at 66% over paper | Tertiary text and placeholders; the readable floor for small text |
| `line` | `#DEDEDE` | ink at 14% over white | Component frame lines: card, alert, sidebar, menu, tooltip, section rules |
| `rowLine` | `#ECECEC` | ink at 8% over white | Row rules, card header divider, top-bar bottom border, disabled field edge |
| `controlLine` | `#DADADA` | ink at 16% over white | Control edges: input, textarea, secondary button, tabs/menu boundary where used |
| `controlHover` | `#B4B4B4` | ink at 32% over white | Hovered control edge |
| `accent` | `#4C94FC` | existing `--palette-blue-700` (OKLCH 0.671 0.171 257.83) | Blue accent for **non-text** marks only: explicitly overridden selected-chip edges and selection rules. Focus rings are ink (§1.3), never blue. Never small text on light |
| `accentSoft` | `#E3EEFA` | ui `--hue-3-wash` (blue wash) | Selected / pressed / toggled wash; text selection background; icon-rail selected fill |
| `danger` | `#B72728` | ui `--fail-text` (OKLCH 51% 0.18 26) | Destructive text and destructive button edge; error title; invalid edge |
| `dangerSoft` | `#FFE6E3` | ui `--fail-wash` family | Destructive button hover fill; destructive confirmation wash |
| `success` | `#161616` | ink — deliberately no green | Success/complete text uses ordinary ink. There is no green success hue; do not introduce one |
| `successSoft` | `#F0F0F0` | grey field | Neutral “saved / complete / confirmed” fill. Never green |
| `code` | `#F0F0F0` | ui grey field at 95.6% | Code blocks, JSON, read-only wells, badge fill, neutral chrome (shipped shell top bar and rail background) |
| `hover` | `#F3F3F3` | ink at 5% over white | Hover wash: secondary/ghost buttons, tabs, menu items, rows |
| `pressed` | `#ECECEC` | ink at 8% over white | Mouse-down wash for non-toggled secondary/ghost/destructive buttons and rows |
| `disabled` | `#A2A2A2` | ink at 40% over white | Disabled **text/icon** colour. Do not use for required text. Buttons instead use opacity |
| `primaryHover` | `#373737` | ink at 86% over white | Primary button hover / active fill |
| `inverse` | `#FFFFFF` | paper | Text and icons on an ink fill (primary button) |
| `transparent` | `transparent` | — | Ghost button border/fill, unselected tab fill |

Do not add tints, alpha variants, gradients, or extra greys. If a state needs a value not listed, reuse the nearest token above rather than inventing a colour.

### 1.2 Contrast (computed, WCAG 2.x)

| Pair | Ratio | Consequence |
| --- | --- | --- |
| ink on bg | **17.64:1** | Body text is safe |
| ink on panel | **18.10:1** | Body text is safe |
| muted on panel | **9.59:1** | Secondary text is safe |
| muted on bg | **9.35:1** | Secondary text is safe |
| faint on panel | **5.83:1** | Minimum for small text |
| faint on bg | **5.68:1** | Minimum for small text |
| danger on panel | **6.30:1** | Error/destructive text is safe |
| danger on bg | **6.14:1** | Error/destructive text is safe |
| inverse on ink | **18.10:1** | Primary button label is safe |
| ink on accentSoft | **15.40:1** | Selected wash text is safe |
| danger on dangerSoft | **5.30:1** | Destructive hover text is safe |
| accent on bg | **2.95:1** | Non-text only (meets 3:1 for UI components approximately; not body text) |
| accent on panel | **3.02:1** | Non-text only |
| accent on ink | **5.99:1** | Only usable on dark ink |
| inverse/white on accent | **3.02:1** | **Forbidden** for text |
| ink on accentSoft | **15.40:1** | Selection text is safe |
| disabled on panel | **2.55:1** | Disabled only; exempt, never required text |

Rules:

- All prose uses `ink`; secondary prose uses `muted`; metadata uses `faint` only at ≥12px and short.
- Blue is never text on a light surface. Use ink for words.
- Never put white text on `accent`; primary fill is ink, not blue.
- `danger` is text/edge. Never fill a destructive button with `danger` or `dangerSoft` at rest.

### 1.3 Focus ring

Native focus is **ink**, not blue:

| Property | Value |
| --- | --- |
| Width | `2px` (`ui.line.focus`) |
| Style | solid |
| Colour | `#161616` (`colors.ink`) |
| Offset | `2px` (`ui.line.focusOffset`) |
| Trigger | keyboard focus (`focusVisible`); fields also turn their border to `ink` |

- Applies to every focusable element: buttons, inputs, textareas, tabs, menu items, icon buttons.
- Never remove or recolour the ring. No focus halo, no shadow ring, no blue ring.
- In a clipped container (inside an overflow scroller or a shared edge), draw the same 2px ink ring inset with `offset = -2px` (web `focus-ring-inset` intent).
- On an ink-filled surface where an ink ring would disappear, use a 2px `inverse` ring inset 4px (`offset = -4px`) — web `focus-ring-ink`. This is the only permitted inverse ring.
- The web field’s soft `focus-halo` shadow (`0 0 0 3px ink/8%`) is **web-only**; do not port it.

### 1.4 Selection and caret

| Item | Value |
| --- | --- |
| Selected row / tab / toggle wash | `accentSoft` `#E3EEFA` |
| Text on selection wash | `ink` `#161616` |
| Hover (unselected) | `hover` `#F3F3F3` |
| Pressed (not toggled) | `pressed` `#ECECEC` |
| Text selection background | `accentSoft` `#E3EEFA` (native projection of ui `--hue-3-wash`) |
| Text selection foreground | `ink` `#161616` |
| Caret | `ink` `#161616` |
| Selected tab fill | `panel` `#FFFFFF` on a `code`/`bg` bar |
| Selected chip border (only when the app explicitly overrides) | `accent` `#4C94FC` |

Distinguish three states:

1. **Hover** — `hover` grey, no blue.
2. **Selected/toggled** — `accentSoft` blue wash, ink content. Hovering a selected item keeps the blue wash (or intensifies it), never replaces it with grey.
3. **Keyboard focus** — ink ring from §1.3, independent of selection.

The web `::selection` uses the run-blue field `#82ACE6`; the native projection deliberately uses the lighter `accentSoft`. Rust must use `accentSoft` for text selection.

### 1.5 Global state fills

| State | Fill | Edge |
| --- | --- | --- |
| Hover (neutral control/row) | `hover` `#F3F3F3` | `controlHover` `#B4B4B4` for bordered controls; unchanged for borderless |
| Mouse-down (neutral) | `pressed` `#ECECEC` | unchanged |
| Selected / toggled | `accentSoft` `#E3EEFA` | unchanged; app may add `accent` edge |
| Focus-visible | unchanged | 2px ink ring, 2px offset |
| Disabled | unchanged or `code` for read-only | Disabled text `#A2A2A2`; buttons use whole-control `opacity: 0.4` |

Disabled opacity: `0.4` (`ui.opacity.disabled`). Applies to the whole button, including border and label. Do not additionally recolour the label.

Read-only field (registry intent, not coded in the native adapter): fill `code` `#F0F0F0`, edge transparent, text `ink`; focus still shows the ink ring. Disabled field: text `#A2A2A2`, edge `rowLine` `#ECECEC`, no hover change, no lift.

### 1.6 No dark theme

The native design has no dark tokens. `nativeTheme.appearance` is `"light"`. Do not add `dark` variants, `prefers-color-scheme`, or inverted palettes in the Rust port. If a future dark theme is requested it must be designed as a new token set, not derived ad hoc.

---

## 2. Typography

### 2.1 Families and files

| Role | Family name in renderer | File |
| --- | --- | --- |
| UI / prose | `Geist` (web CSS also calls it `Geist Variable`) | `/Users/pruthivithejan/Projects/coderune/packages/tokens/fonts/Geist-Variable.ttf` |
| Code / mono | `Paper Mono` | `/Users/pruthivithejan/Projects/coderune/packages/tokens/fonts/PaperMono-Variable.ttf` |

Repo-relative paths:

- `packages/tokens/fonts/Geist-Variable.ttf` — variable, weights 100–900, OFL at `packages/tokens/fonts/Geist-OFL.txt`.
- `packages/tokens/fonts/PaperMono-Variable.ttf` — Paper Mono v1.000, variable, weights 100–800, OFL at `packages/tokens/fonts/PaperMono-OFL.txt`. Includes keyboard, box-drawing, geometric glyphs, and `₪` (U+20AA).
- Web WOFF2 copies live beside them; the desktop uses the TTF files.

Registration contract (from `crates/ui/src/theme.rs`):

- Register **both** TTFs with the renderer before the first layout (`renderer.registerFont(bytes)`).
- Do not mount application UI until both registrations resolve. The existing `UIFonts` component gates children and throws on failure.
- Fail fast if registration is unavailable. Do not silently fall back to a system sans.
- Exported asset keys used by the build: `@coderune/tokens/fonts/geist-variable.ttf`, `@coderune/tokens/fonts/paper-mono-variable.ttf`.

In Rust/Rune, load the TTF bytes and register under exactly `Geist` and `Paper Mono`, then reference those family names. Weights are variable; the design uses only **400**.

### 2.2 Type roles (canonical native values)

Every native role is weight **400**. There is no bold, medium, or semibold role. Hierarchy is made by size, colour, and family only. Line heights are exact px, not unitless ratios.

| Role | Size / line-height | Family | Weight | Usage |
| --- | --- | --- | --- | --- |
| `display` | 40 / 44 | Geist | 400 | Defined but unused by native chrome; reserve for a future hero/empty screen |
| `headingLg` | 32 / 36 | Geist | 400 | **PageHeader H1** (the one page title) |
| `heading` | 24 / 28 | Geist | 400 | Section / dialog titles; not used by a native component today |
| `headingSm` | 20 / 24 | Geist | 400 | Subsection titles; not used by a native component today |
| `headingXs` | 16 / 20 | Geist | 400 | **Card title** |
| `bodyLg` | 16 / 22 | Geist | 400 | Onboarding/prose lede; not the PageHeader description |
| `body` | 14 / 20 | Geist | 400 | **Default Text**, PageHeader description, Textarea, prose, descriptions |
| `bodySm` | 13 / 18 | Geist | 400 | Card description, field hint, Alert body, dense meta |
| `label` | 14 / 16 | Geist | 400 | **Controls**: Button xs/sm/default, Field label, Input, Tabs, menu item, tooltip |
| `labelSm` | 12 / 16 | Geist | 400 | **Badge**, small tags, column heads |
| `button` | 16 / 20 | Geist | 400 | **Large Button only** (size `lg`) |
| `code` | 13 / 20 | Paper Mono | 400 | Code blocks, JSON, read-only code |
| `codeSm` | 12 / 16 | Paper Mono | 400 | Inline paths, IDs, hashes, keys; may sit inside a Button |

### 2.3 Usage rules

- **Geist** for every word that is not copied: headings, labels, buttons, prose, hints, badges, navigation, tab labels, menu items, tooltips, numbers, dates, durations.
- **Paper Mono** only for code, commands, file paths, IDs, hashes, keys, JSON, and read-only source text. Nothing else.
- Numbers, durations, counts, versions and sizes are **not** mono. Use Geist body/label.
- Do not synthesize bold or semibold. Do not use `fontWeight: 500/600/650/700`. If emphasis is needed, switch colour (`ink` → `muted`) or role size.
- Do not add letter-spacing or uppercase to desktop text. Web marketing tracking (`-0.025em`, `-0.015em`, `0.01em`, `0.04em`) and uppercase are marketing-only. Desktop labels are sentence case.
- Do not use `text-wrap: pretty`, text shadows, or gradient text.
- The Card title is `headingXs`, **not** `headingSm` or `heading`.
- The PageHeader title is `headingLg`, 32/36. The PageHeader description is `body` 14/20 muted.
- The large button is the only `button` role; default, sm, and xs buttons use `label` 14/16.
- Mono inside a Button keeps `codeSm` 12/16 deliberately; do not inherit the button label role over a path/hash.
- Placeholder text uses `faint` `#656565` (web registry maps placeholder to `text-3`; native theme exposes `textFaint`). Disabled placeholder uses `disabled` `#A2A2A2`.

### 2.4 Forbidden type drift

From the pre-rewrite app: `fontSize: 18`, `fontSize: 12`, `fontWeight: 650`, raw `fontFamily: "monospace"`, and mixed arbitrary sizes are forbidden. Replace with a role from the table. Web-only weights (340 body, 380 body-sm, 500 strong) do not exist in the native projection.

---

## 3. Space, shape, lines, dimensions

### 3.1 Spacing scale

Native `ui.space` keys and values (px). Only these steps exist.

| Token | px | Rust name (recommended) | Typical use |
| --- | --- | --- | --- |
| `space[0]` | 0 | `space::S0` | Zero, min-width guards, transparent padding |
| `space[0.5]` | 2 | `space::S0_5` | Optical nudge |
| `space[1]` | 4 | `space::S1` | Button vertical padding, card header gap, sidebar gap, menu padding |
| `space[2]` | 8 | `space::S2` | Field gap, alert gap, tooltip padding, action gap, badge/input horizontal at xs |
| `space[3]` | 12 | `space::S3` | Control horizontal padding, alert padding, menu/row horizontal padding |
| `space[4]` | 16 | `space::S4` | Card padding, sidebar padding, page gutter (desktop), default card gap |
| `space[5]` | 20 | `space::S5` | Icon size (20), medium gaps |
| `space[6]` | 24 | `space::S6` | Section gaps, topbar-adjacent gaps, gutter |
| `space[8]` | 32 | `space::S8` | Between app sections; wide page padding |
| `space[10]` | 40 | `space::S10` | Marketing/roomy block gap; icon-button size |
| `space[15]` | 60 | `space::S15` | Reserved (marketing) |
| `space[20]` | 80 | `space::S20` | Reserved; macOS traffic-light inset in shipped shell |
| `space[30]` | 120 | `space::S30` | Reserved (marketing hero) |

There are no 6, 10, 14, 18, 28, 36, 48, 56 px spacing steps. Do not invent them.

### 3.2 Radii

| Token | px | Rust name | Use |
| --- | --- | --- | --- |
| `none` | 0 | `Radius::None` | Square/shared edges, joins |
| `inner` | 4 | `Radius::Inner` | Badge, menu item, a box inside a control/row |
| `soft` | 6 | `Radius::Soft` | Button, input, textarea, tab, alert, menu, tooltip |
| `frame` | 10 | `Radius::Frame` | Card and component outer frame |

Full/pill (`999px`) is forbidden for boxes/tags in desktop. It is reserved for avatars, radio, and switches, none of which exist in the native set. Never use 7px, 8px, or arbitrary radii.

### 3.3 Border widths and focus

| Token | px | Rust name | Use |
| --- | --- | --- | --- |
| `line.hair` | 1 | `line::HAIR` | Every component frame, divider, control edge |
| `line.focus` | 2 | `line::FOCUS` | Focus ring width |
| `line.focusOffset` | 2 | `line::FOCUS_OFFSET` | Focus ring offset |
| `opacity.disabled` | 0.4 | `DISABLED_OPACITY` | Whole-control disabled fade |

No 2px accent underline, no 3px border, no 0.5px hairlines. Native has no `--line-accent` usage.

### 3.4 Control heights, rows, tags, icons

| Token | px | Rust name | Use |
| --- | --- | --- | --- |
| `layout.controlXs` | 24 | `ControlHeight::Xs` | Button xs, tab close |
| `layout.controlSm` | 28 | `ControlHeight::Sm` | Button sm |
| `layout.control` | 32 | `ControlHeight::Md` | Button default, input, textarea min, tabs |
| `layout.controlLg` | 44 | `ControlHeight::Lg` | Button lg, touch target |
| `layout.row` | 40 | `RowHeight::Md` | Icon button, menu item |
| `layout.rowLg` | 56 | `RowHeight::Lg` | Two-line row (reserved) |
| `layout.tag` | 20 | `TAG_HEIGHT` | Badge |
| icon | 20 (`space[5]`) | `icon::SIZE` | Every native icon |

All control heights are **minimums**, not fixed. A wrapping label grows the control. Native buttons use `minHeight`; tabs use `minHeight`; inputs use `minHeight`. Do not clamp text or force single-line truncation unless the new layout explicitly specifies it.

### 3.5 Layout dimensions

| Token | px | Rust name | Meaning |
| --- | --- | --- | --- |
| `layout.sidebar` | 248 | `layout::SIDEBAR` | Full sidebar width; also minimum menu width |
| `layout.sidebarIcon` | 48 | `layout::SIDEBAR_ICON` | Collapsed icon-rail width |
| `layout.topbar` | 56 | `layout::TOPBAR` | Top bar height; macOS top inset in onboarding |
| `layout.gutter` | 24 | `layout::GUTTER` | Desktop page gutter |
| `layout.gutterSm` | 16 | `layout::GUTTER_SM` | Narrow page gutter |
| `layout.container` | 1120 | `layout::CONTAINER` | Maximum content width |
| `layout.pageHeaderMax` | 720 | `layout::PAGE_HEADER_MAX` | Maximum PageHeader copy width |
| `layout.overlay` | 480 | `layout::OVERLAY` | Default dialog/sheet width |
| `layout.overlayLg` | 640 | `layout::OVERLAY_LG` | Wide dialog/panel width |
| `layout.fieldMaxHeight` | 176 | `layout::FIELD_MAX_HEIGHT` | Textarea max visible height before scrolling |

These are the only native layout constants. The new layout may choose different arrangements, but when it uses a sidebar/topbar/gutter/container/overlay/field limit, use these values.

Web-only reference values (present in `web/tokens.css`, **not** in the native projection; do not port without design approval):

| Web token | px | Note |
| --- | --- | --- |
| `--overlay-sm` | 400 | Small confirm dialog |
| `--popover` | 288 | Popover panel |
| `--toast-width` | 384 | Toast |
| `--menu-min` | 192 | Web menu minimum; native menu uses 248 |
| `--tooltip-max` | 256 | Tooltip line length |
| `--tab-max` | 220 | Web tab truncation; native tabs scroll instead |
| `--row-sm` | 32 | Dense web row |
| `--measure` | 68ch | Web prose measure |

### 3.6 Motion (web canonical, native has no tokens)

The native projection has no durations or easing. If the Rust UI animates, use the web system’s values; otherwise render state changes instantly.

| Motion | Value |
| --- | --- |
| Hover colour/edge | 120 ms, `cubic-bezier(0.2, 0, 0, 1)` |
| Press | 60 ms, same curve |
| Overlay enter | 160 ms (opacity only) |
| Overlay exit | 120 ms |
| Dialog/sheet | 180 ms |
| Reduced motion | 0 ms |

Do not add looping, pulsing, shimmering, bouncing, or decorative motion. A running indicator may be a single small arc; no status dots.

---

### 3.7 Separation and framing (global visual rules)

These are system-wide and must be applied by every Rust component/layout:

- **One frame per component.** A component draws one 1px `line` outer edge. Never nest a bordered box inside a bordered box; split the inside with 1px `rowLine` rules or a `code` well.
- **The grey field is a well, not a container.** `code` `#F0F0F0` is only for code, logs, output, JSON, read-only values, badges, and neutral chrome. Never use it as a general card/section background.
- **Fields before lines.** Prefer a grey field or a row rule over drawing new borders. Hairlines go where a field cannot.
- **Selection is a wash, never a side rail.** No left/right stripe, inset bar, or edge marker on a selected row or tab. Use `accentSoft` and keep the content ink.
- **Fills are flat.** No ring around a filled mark/badge, no gradient, no sheen, no inner/outer shadow on flat controls.
- **Warm colour is only danger.** `danger`/`dangerSoft` appear only for fail/destructive states, never decoration.
- **Status is a word.** No coloured dots, squares, pills, or traffic lights. Write “Ready”, “Failed”, “Waiting”.
- **No terminal cosplay.** No prompt signs, block cursors, or bracketed labels outside an actual code/terminal block.

---

## 4. Component specifications

All components are built from `View` (flex container with `minWidth: 0`) and `Text`. Sizes below are exact native values. “Min height” means the box may grow.

### 4.0 Shared primitives

#### View

- A plain flex container.
- Default: `minWidth: 0`.
- No background, border, radius, padding, or gap of its own. All appearance comes from the caller.
- Rust: `View` / `Stack` with `min_w(px(0.0))` default; never map it to a DOM-like block with margins.
- Accessibility: none by default; callers may set `role`/`aria-label` (e.g. Card uses `group`, Sidebar uses `navigation`).

#### Text

- Defaults: `body` role 14/20, Geist 400, colour `ink`, `whiteSpace: normal`.
- Props: `tone` = `ink | muted | faint | danger`; `mono` boolean; `size` = any type role; `style` override applied last.
- Tone map: `ink`→`#161616`, `muted`→`#454545`, `faint`→`#656565`, `danger`→`#B72728`.
- `mono` forces `Paper Mono`; a `size` role may also carry mono (`code`, `codeSm`).
- Style precedence: default body → inherited button/control text style → explicit `size` → `tone` → `mono` → caller style. Preserve this order.
- Text may receive `role` and `aria-level`; headings must use them (Card title level 2, PageHeader title level 1).
- No truncation, max lines, or ellipsis by default.
- Accessibility: plain text; no implicit role.

#### Icon

- 20 × 20 (`space[5]`), `flexShrink: 0`, colour `ink`.
- Source is an SVG string; every `currentColor` is replaced with `#161616` before render.
- Always `aria-hidden`. Icons never carry a name.
- No size prop, no colour prop, no hover state of its own.
- Rust: see §5 for source set and `icon::SIZE = 20.0`.

---

### 4.1 Button

Anatomy: a flex row, centred, `flexShrink: 0`, `maxWidth: 100%`, `paddingY: 4px`, `borderWidth: 1px`, `borderRadius: 6px`. Content may wrap (`whiteSpace: normal`); height is a minimum.

Sizes:

| Size | Min height | Padding X | Type role |
| --- | --- | --- | --- |
| `xs` | 24 | 8 (`space[2]`) | `label` 14/16 |
| `sm` | 28 | 12 (`space[3]`) | `label` 14/16 |
| `default` | 32 | 12 (`space[3]`) | `label` 14/16 |
| `lg` | 44 | 12 (`space[3]`) | `button` 16/20 |

Note: native `lg` uses **12px** horizontal padding, not the web’s 20px.

These are `minHeight` values. The rendered box is `max(minHeight, line-height + 8px vertical padding + 2px border)`. With the `label` role (16px line-height), an `xs` button’s content floor is 26px (16 + 8 + 2), so the 24px token is a floor, not a guaranteed height. Do not shrink the type or padding to force 24px. Default/sm/lg land on 32/28/44 because their content floors are lower.

Variants and all states:

| Variant | Rest (bg / border / text) | Hover | Mouse-down | Toggle `pressed=true` | Disabled |
| --- | --- | --- | --- | --- | --- |
| `primary` | `ink` / `controlLine` / `inverse` | `primaryHover` / `controlHover` / `inverse` | `primaryHover` / `controlLine` / `inverse` | No visual change (only `aria-pressed=true`) | Whole control `opacity: 0.4`, no hover/active |
| `secondary` (default) | `panel` / `controlLine` / `ink` | `hover` / `controlHover` / `ink` | `pressed` / `controlLine` / `ink` | `accentSoft` / `controlLine` / `ink` (hover keeps `accentSoft`, edge `controlHover`; active keeps `accentSoft`) | `opacity: 0.4` |
| `ghost` | `transparent` / `transparent` / `ink` | `hover` / `transparent` / `ink` | `pressed` / `transparent` / `ink` | `accentSoft` / `transparent` / `ink` | `opacity: 0.4` |
| `link` | `transparent` / `transparent` / `ink` (identical to `ghost`; no underline in native) | `hover` / `transparent` / `ink` | `pressed` / `transparent` / `ink` | `accentSoft` / `transparent` / `ink` | `opacity: 0.4` |
| `destructive` | `panel` / `danger` / `danger` | `dangerSoft` / `danger` / `danger` | `pressed` / `danger` / `danger` | `accentSoft` / `danger` / `danger` (hover/active keep `accentSoft`) | `opacity: 0.4` |

State precedence in the source: root → size → variant → `pressed` wash → disabled/enabled → hover → active → focus ring. Consequences to preserve:

- `pressed` is a **toggle** state, separate from mouse-down. It sets `aria-pressed` and a persistent `accentSoft` wash for every non-primary variant.
- Mouse-down on secondary/ghost/destructive without `pressed` uses `pressed` grey `#ECECEC`.
- Mouse-down on a destructive button clears `dangerSoft` to `#ECECEC`; that is the shipped native behaviour.
- Focus ring is drawn last and never suppressed by hover/active.
- Disabled removes hover and active entirely.

Focus: 2px `ink` outline, 2px offset (§1.3).

Text: colour as table; type `label` for xs/sm/default, `button` for lg. `whiteSpace: normal`. Icon + label pairs have **no** gap in the native Button; compose a child row if a gap is needed (do not assume the web `gap-2`).

Pending:

- `pending=true` disables the button (`disabled || pending`) and must not move focus when the state begins. Verify focus retention in GPUI; the native adapter sets `disabled`.
- If `pendingLabel` is supplied, the original content is hidden with `opacity: 0` and the pending label is centred absolutely over the same box; the box does not resize.
- Accessible name stays the original label. `aria-description` is set to the pending label. Do not change the name to “Refunding”.
- If no `pendingLabel`, the label remains visible and the button is simply disabled.
- Cursor: `pointer` when enabled, `not-allowed` when disabled.

Accessible name:

- Computed recursively from text/number children, concatenated and trimmed (`controlName`).
- Caller-supplied `aria-label` overrides that computed name (props spread after).
- Icon-only buttons **must** supply an explicit name; use `IconButton`.
- Do not remove focus, and do not use `aria-label` that differs from the visible verb unless the visible text is an icon.

Rules:

- One `primary` button per view. Default is `secondary`.
- Destructive is written, not painted: paper + danger edge + danger word.
- Never fill a header, card, tab, or icon tile with ink; ink fill is the primary action and checked/toggled controls only.

---

### 4.2 Card

Anatomy: vertical stack; `border: 1px line`; `radius: 10px`; background `panel`.

| Part | Spec |
| --- | --- |
| Root | Column; 1px `line` border; 10px radius; `panel`; no shadow; no hover |
| Header (only when `title` is set) | Column; `gap: 4px`; `padding: 16px`; `borderBottom: 1px rowLine` |
| Title | `Text` `headingXs` 16/20, `ink`, `role=heading`, `aria-level=2` |
| Description | `Text` `bodySm` 13/18, `muted` |
| Body | Column; `gap: 16px`; `padding: 16px` |

- If no title, no header and no divider; the body still has 16px padding and 16px gap.
- No box-in-box: don’t nest another bordered/grey box inside a card; use row rules (1px `rowLine`) or a `code` well for code/output.
- Accessibility: `role=group`, `aria-label=title` when a title exists. No landmark role.
- Cards are static; no hover/pressed/focus.

---

### 4.3 Field

Anatomy: vertical stack, `gap: 8px`.

| Part | Spec |
| --- | --- |
| Label | `Text` `label` 14/16, `ink` |
| Control | Caller-provided `Input`/`Textarea` |
| Hint | `Text` `bodySm` 13/18, `muted`, below the control |

- Provides `{label, hint}` to descendants through context.
- Accessibility: `role=group`, `aria-label=label`; the label text is visible and is the control’s name. Hint becomes `aria-description`.
- There is no `error` prop and no required marker in the native Field. Errors are shown with an `Alert`; do not invent a red field until the design system adds one.
- One control per Field. If a group has multiple controls (e.g. a row of choices), each control needs its own `aria-label`.

---

### 4.4 Input

| Property | Value |
| --- | --- |
| Width | `100%`, `minWidth: 0` |
| Min height | 32 (`control`) |
| Padding X | 12 (`space[3]`) |
| Border | 1px `controlLine` |
| Radius | 6 (`soft`) |
| Background | `panel` |
| Text | `ink`, `label` 14/16, Geist |
| Caret | `ink` |
| Placeholder | `faint` `#656565` |
| Hover | border `controlHover` `#B4B4B4` |
| Focus-visible | 2px ink ring at 2px offset **and** border `ink` |
| Disabled | text `disabled` `#A2A2A2`, border `rowLine` `#ECECEC`, no hover, no lift. Do not apply the button opacity; use these mapped colours |
| Read-only | `code` `#F0F0F0` fill, transparent edge, `ink` text; focus still rings |

- Theme object: `appearance: light`, `bg: panel`, `border: controlLine`, `text: ink`, `textMuted: muted`, `textFaint: faint`, `accent: accent`, `caret: ink`, `fontSans: Geist`, `fontMono: Paper Mono`.
- Single line; no explicit vertical padding (centre in 32px).
- The placeholder, disabled, and read-only rows above are the registry's canonical field states; the native adapter leaves them to the renderer. The Rust port should implement them explicitly.
- Accessibility: `aria-label` from Field label or explicit; `aria-description` from hint. Every input must have a name.

---

### 4.5 Textarea

Everything from Input, plus:

| Property | Value |
| --- | --- |
| Type role | `body` 14/20, Geist, `ink` |
| Padding Y | 8 (`space[2]`) top and bottom |
| Padding X | 12 |
| Min rows | 3 |
| Max rows | 12 (clamped to at least `minRows`) |
| Max visible height | 176 (`fieldMaxHeight`), then scroll |
| Width | 100% |
| Resize | none in native |

- Same border, radius, background, hover, focus, disabled, and read-only states as Input.
- Accessibility: same naming contract as Input.
- Use Textarea for multiple lines and JSON; `code` role only when the content is genuinely code/JSON.

---

### 4.6 Badge

| Property | Value |
| --- | --- |
| Min height | 20 (`tag`) |
| Padding X | 4 (`space[1]`) |
| Radius | 4 (`inner`) |
| Border | none |
| Background | `code` `#F0F0F0` |
| Text | `labelSm` 12/16, `ink`, Geist |
| Layout | flex row, centre/centre |

- One neutral variant only. The native Badge has no kind, status, outline, or tinted variant. Do not port the web kind/tint/outline badges.
- Empty/null/undefined/`""` children render **nothing** (return `null`).
- Static label; no hover/pressed/focus.
- Accessibility: no explicit role; the text is read as static content. Keep badges short and sentence case. Use words, never a coloured dot.

---

### 4.7 Alert

Anatomy: vertical stack, `gap: 4px`, `padding: 12px`, `border: 1px line`, `radius: 6px`, background `panel`.

| Content | Spec |
| --- | --- |
| Error title (only when `error` is set) | `Text` `label` 14/16, `danger`; literal string `Action failed` |
| Body | `Text` `bodySm` 13/18, `muted`; text = `error || message || children` |

- No tinted fill, no icon, no coloured stripe. Trouble is the danger title plus the words.
- If `error`, `message`, and `children` are all empty, render nothing.
- Accessibility:
  - `announce=true` (default): `role=alert` for an error, `role=status` otherwise.
  - `announce=false`: `role=note`.
- The error body is `muted`, not `danger`; only the title is danger. Preserve this.

---

### 4.8 PageHeader

| Part | Spec |
| --- | --- |
| Root | Row, wrap, `alignItems: center`, `gap: 16px` |
| Copy column | Column, `gap: 8px`, `flexGrow: 1`, `maxWidth: 720px` |
| Title | `Text` `headingLg` 32/36, `ink`, `role=heading`, `aria-level=1` |
| Description | `Text` `body` 14/20, `muted` |
| Children | Rendered under the description inside the copy column |
| Actions | Row, wrap, `gap: 8px`, after the copy column |

- One H1 per page.
- No border, background, shadow, or sticky behaviour in the component itself.
- Actions are right-aligned only by the surrounding layout; the component itself uses a wrapping row.
- Accessibility: the title is the page’s level-1 heading; description is plain text.

---

### 4.9 Sidebar

| Property | Value |
| --- | --- |
| Width | 248 (`layout.sidebar`), `flexShrink: 0` |
| Height | 100% |
| Padding | 16 (`space[4]`) |
| Gap | 4 (`space[1]`) |
| Border right | 1px `line` |
| Background | `panel` |
| Layout | Column |

- Accessibility: `role=navigation`, `aria-label="Workspace"` by default; callers may override (shipped shell uses `"Workspace navigation"`).
- Children: 40×40 icon buttons (§4.10) or labelled rows.
- Shipped workspace rail override (reference, not layout to copy): width 48 (`sidebarIcon`), padding 4, gap 4, background `code`, no border; selected icon uses ghost + `pressed` → `accentSoft`.
- No collapse animation, no shadow, no hue.

---

### 4.10 IconButton

- A `Button` with `variant="ghost"`, `width: 40`, `minHeight: 40`, `paddingX: 0`.
- Contains one `Icon` (20×20, ink, hidden from a11y).
- States: identical to ghost Button (§4.1); `pressed=true` → `accentSoft` fill, ink icon.
- Tooltip:
  - Surface `panel`; border 1px `line`; radius 6; padding 8 (`space[2]`); text `label` 14/16, `ink`.
  - Placement: right of the button, `sideOffset: 8` (`space[2]`).
  - Appears on hover **and** keyboard focus.
- Accessibility: `label` is mandatory and becomes the button’s `aria-label` **and** the tooltip text. Never rely on the tooltip as the only name.
- No filled/ink icon button; no icon tiles; no second icon set.

---

### 4.11 Tabs

Anatomy for a horizontal document tab strip.

| Part | Spec |
| --- | --- |
| Root | Row, `flexGrow: 1`, `minWidth: 0`, `overflowX: scroll`, `alignItems: center` |
| Tab wrapper | Row, `flexShrink: 0`, `alignItems: center`, `radius: 6`, transparent; selected wrapper `panel` |
| Tab | Row, `alignItems: center`, `minHeight: 32`, `paddingX: 12`, `radius: 6`, transparent |
| Tab label | `Text` `label` 14/16, `ink` |
| Tab icon (optional) | `Icon` 20×20; when present the label is not rendered, but `aria-label` still names the tab |
| Close button | `Button` ghost, `xs` (min 24, paddingX 8), containing `Icon` 20×20 |

States:

| State | Fill | Border | Other |
| --- | --- | --- | --- |
| Rest | transparent | none | pointer cursor |
| Hover | `hover` `#F3F3F3` | none | |
| Selected | wrapper `panel` `#FFFFFF` | none | no underline, no blue |
| Focus-visible | unchanged | none | 2px ink ring, 2px offset |
| Close hover | `hover` | none | ghost button states |

- Native selected tabs are a paper fill on a grey bar (`code`/`bg`), not an accent underline. Do not add a blue underline.
- No max-width/truncation in the native component; the strip scrolls horizontally instead.
- No gap is set between the tab and its close button; the wrapper holds both.
- The close control is a ghost `xs` Button around a 20px icon. Its rendered box is about 38×30px (20 icon + 8px × 2 horizontal padding + 1px × 2 border; 20 icon + 4px × 2 vertical padding + 1px × 2 border), not a 24×24 square. Preserve the padding and icon size if the new layout keeps this control.

Keyboard and accessibility:

- Root: `role=tablist`, `aria-label="Open workspace tabs"`.
- Each tab: `role=tab`, `aria-label=label`, `aria-selected=true|false`, roving `tabIndex` (selected 0, others -1).
- Left/Right wrap around; Home/End jump to first/last. Modifier keys (Cmd/Ctrl/Alt/Shift) suppress the shortcut. Arrow keys **select** the next tab, then move focus.
- After closing a tab, focus the nearest remaining tab: previous tab if the closed one was selected, otherwise the still-selected tab.
- Close button: `aria-label="Close {label} tab"`.
- The caller owns tab content and lifetime.

---

### 4.12 DropdownMenu

Trigger:

- A default `secondary` Button whose visible label is `` `${label} ▾` `` (the character is U+25BE “▾”, preceded by a space).
- `aria-haspopup="menu"`, `aria-expanded=true|false`.
- Clicking toggles. Click-away closes without restoring focus; Escape closes. Pressing the trigger while open closes it.

Menu surface:

| Property | Value |
| --- | --- |
| Min width | 248 (`layout.sidebar`) |
| Padding | 4 (`space[1]`) |
| Background | `panel` |
| Border | 1px `line` |
| Radius | 6 (`soft`) |
| Shadow | none in native (do not invent one) |
| Placement | bottom, align end, `sideOffset: 4` (`space[1]`) |

Items:

| Property | Value |
| --- | --- |
| Min height | 40 (`layout.row`) |
| Padding X | 12 (`space[3]`) |
| Radius | 4 (`inner`) |
| Background | transparent |
| Hover | `hover` `#F3F3F3` |
| Focus-visible | 2px ink ring, 2px offset |
| Text | `label` 14/16, `ink` |
| Cursor | pointer |

Empty state:

- `Text` `bodySm` 13/18, `muted`, padding 8 (`space[2]`), with the caller’s `emptyLabel`.

Keyboard and accessibility:

- Menu: `role=menu`, `aria-label=label`, `tabIndex=-1`.
- Items: `role=menuitem`, `aria-label=item.label`.
- Up/Down wrap; Home/End first/last. Tab leaves the menu: Shift+Tab moves to the previous focusable, Tab to the next. Escape closes.
- Initial focus goes to the first item; on close, restore focus to the trigger unless the close came from a click outside.
- One menu at a time; no nested menus in the native set.
- Do not add icons to every item; icons are only for actions recognised by shape.

---

## 5. Icons

### 5.1 Source

Native desktop icons are **Phosphor Icons, Regular weight**, imported as SVG text from `@phosphor-icons/core` in `apps/desktop/src/icons.ts`:

| Name | File |
| --- | --- |
| House | `@phosphor-icons/core/regular/house.svg` |
| FolderSimple | `@phosphor-icons/core/regular/folder-simple.svg` |
| Files | `@phosphor-icons/core/regular/files.svg` |
| Robot | `@phosphor-icons/core/regular/robot.svg` |
| Scan | `@phosphor-icons/core/regular/scan.svg` |
| Gear | `@phosphor-icons/core/regular/gear.svg` |
| UserCircle | `@phosphor-icons/core/regular/user-circle.svg` |
| SidebarSimple | `@phosphor-icons/core/regular/sidebar-simple.svg` |
| X | `@phosphor-icons/core/regular/x.svg` |

These are the only shipped native icon assets. The web registry uses Font Awesome Free Solid; that is a **web-only** set and must not be used in the Rust desktop. Do not mix in a third set.

### 5.2 Sizing and colour

| Rule | Value |
| --- | --- |
| Size | 20 × 20 (`space[5]`), fixed |
| Flex | `flexShrink: 0` |
| Colour | `ink` `#161616`; replace every `currentColor` with ink before render |
| Background | none; no icon tiles, no circular icon chips |
| Accessibility | always `aria-hidden`; the parent control carries the name |
| Stroke/fill | Phosphor Regular assets as supplied; do not restyle per-instance |
| Hover | inherited from the containing button/tab only |

- No size variants. If a different optical size is needed, it is a layout decision requiring a new token, not an ad-hoc number.
- Icons are for actions found by shape, not decoration on every label or heading.
- Never use an icon as the sole accessible name without an `aria-label` on the control.

---

## 6. Rust naming map

Put all values in one `theme` module (e.g. `crates/ui/src/theme.rs`) and import it everywhere. No hex or raw dimension may appear outside that module.

### 6.1 Colour constants

```rust
pub mod color {
    pub const BG: Rgba = rgb(0xFCFCFC);
    pub const PANEL: Rgba = rgb(0xFFFFFF);
    pub const INK: Rgba = rgb(0x161616);
    pub const MUTED: Rgba = rgb(0x454545);
    pub const FAINT: Rgba = rgb(0x656565);
    pub const LINE: Rgba = rgb(0xDEDEDE);
    pub const ROW_LINE: Rgba = rgb(0xECECEC);
    pub const CONTROL_LINE: Rgba = rgb(0xDADADA);
    pub const CONTROL_HOVER: Rgba = rgb(0xB4B4B4);
    pub const ACCENT: Rgba = rgb(0x4C94FC);
    pub const ACCENT_SOFT: Rgba = rgb(0xE3EEFA);
    pub const DANGER: Rgba = rgb(0xB72728);
    pub const DANGER_SOFT: Rgba = rgb(0xFFE6E3);
    pub const SUCCESS: Rgba = rgb(0x161616);
    pub const SUCCESS_SOFT: Rgba = rgb(0xF0F0F0);
    pub const CODE: Rgba = rgb(0xF0F0F0);
    pub const HOVER: Rgba = rgb(0xF3F3F3);
    pub const PRESSED: Rgba = rgb(0xECECEC);
    pub const DISABLED: Rgba = rgb(0xA2A2A2);
    pub const PRIMARY_HOVER: Rgba = rgb(0x373737);
    pub const INVERSE: Rgba = rgb(0xFFFFFF);
    pub const TRANSPARENT: Rgba = rgba(0, 0, 0, 0);
}
```

Token-to-constant map:

| UI token | Rust constant |
| --- | --- |
| `colors.bg` | `color::BG` |
| `colors.panel` | `color::PANEL` |
| `colors.ink` | `color::INK` |
| `colors.muted` | `color::MUTED` |
| `colors.faint` | `color::FAINT` |
| `colors.line` | `color::LINE` |
| `colors.rowLine` | `color::ROW_LINE` |
| `colors.controlLine` | `color::CONTROL_LINE` |
| `colors.controlHover` | `color::CONTROL_HOVER` |
| `colors.accent` | `color::ACCENT` |
| `colors.accentSoft` | `color::ACCENT_SOFT` |
| `colors.danger` | `color::DANGER` |
| `colors.dangerSoft` | `color::DANGER_SOFT` |
| `colors.success` | `color::SUCCESS` |
| `colors.successSoft` | `color::SUCCESS_SOFT` |
| `colors.code` | `color::CODE` |
| `colors.hover` | `color::HOVER` |
| `colors.pressed` | `color::PRESSED` |
| `colors.disabled` | `color::DISABLED` |
| `colors.primaryHover` | `color::PRIMARY_HOVER` |
| `colors.inverse` | `color::INVERSE` |
| `colors.transparent` | `color::TRANSPARENT` |

### 6.2 Type

```rust
pub enum FontFamily { Sans, Mono }       // "Geist" | "Paper Mono"
pub enum TextTone { Ink, Muted, Faint, Danger }
pub enum TextRole {
    Display, HeadingLg, Heading, HeadingSm, HeadingXs,
    BodyLg, Body, BodySm, Label, LabelSm, Button,
    Code, CodeSm,
}
pub const FONT_WEIGHT: FontWeight = FontWeight::NORMAL; // 400, every role
```

`TextRole` should expose `size()`, `line_height()`, and `family()` exactly as §2.2. Add no tracking/letter-spacing field.

Role-to-variant map:

| UI key | Rust variant |
| --- | --- |
| `type.display` | `TextRole::Display` |
| `type.headingLg` | `TextRole::HeadingLg` |
| `type.heading` | `TextRole::Heading` |
| `type.headingSm` | `TextRole::HeadingSm` |
| `type.headingXs` | `TextRole::HeadingXs` |
| `type.bodyLg` | `TextRole::BodyLg` |
| `type.body` | `TextRole::Body` |
| `type.bodySm` | `TextRole::BodySm` |
| `type.label` | `TextRole::Label` |
| `type.labelSm` | `TextRole::LabelSm` |
| `type.button` | `TextRole::Button` |
| `type.code` | `TextRole::Code` |
| `type.codeSm` | `TextRole::CodeSm` |

### 6.3 Space / shape / layout

```rust
pub mod space {
    pub const S0: f32 = 0.0;
    pub const S0_5: f32 = 2.0;
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 8.0;
    pub const S3: f32 = 12.0;
    pub const S4: f32 = 16.0;
    pub const S5: f32 = 20.0;
    pub const S6: f32 = 24.0;
    pub const S8: f32 = 32.0;
    pub const S10: f32 = 40.0;
    pub const S15: f32 = 60.0;
    pub const S20: f32 = 80.0;
    pub const S30: f32 = 120.0;
}

pub enum Radius { None, Inner, Soft, Frame } // 0 / 4 / 6 / 10
pub mod line { pub const HAIR: f32 = 1.0; pub const FOCUS: f32 = 2.0; pub const FOCUS_OFFSET: f32 = 2.0; }
pub const DISABLED_OPACITY: f32 = 0.4;

pub enum ControlHeight { Xs, Sm, Md, Lg } // 24 / 28 / 32 / 44
pub enum RowHeight { Md, Lg }             // 40 / 56
pub const TAG_HEIGHT: f32 = 20.0;

pub mod layout {
    pub const SIDEBAR: f32 = 248.0;
    pub const SIDEBAR_ICON: f32 = 48.0;
    pub const TOPBAR: f32 = 56.0;
    pub const GUTTER: f32 = 24.0;
    pub const GUTTER_SM: f32 = 16.0;
    pub const CONTAINER: f32 = 1120.0;
    pub const PAGE_HEADER_MAX: f32 = 720.0;
    pub const OVERLAY: f32 = 480.0;
    pub const OVERLAY_LG: f32 = 640.0;
    pub const FIELD_MAX_HEIGHT: f32 = 176.0;
}
```

### 6.4 Focus / selection / motion

```rust
pub mod focus {
    pub const WIDTH: f32 = 2.0;
    pub const OFFSET: f32 = 2.0;
    pub const COLOR: Rgba = color::INK;
    pub const INSET_OFFSET: f32 = -2.0;   // clipped contexts
    pub const ON_INK_COLOR: Rgba = color::INVERSE; // inset -4 on ink fills
}
pub mod selection {
    pub const WASH: Rgba = color::ACCENT_SOFT;
    pub const TEXT: Rgba = color::INK;
    pub const CARET: Rgba = color::INK;
}
pub mod motion {
    pub const HOVER_MS: u64 = 120;
    pub const PRESS_MS: u64 = 60;
    pub const EASE_STEP: (f32, f32, f32, f32) = (0.2, 0.0, 0.0, 1.0);
    // overlay enter 160, exit 120, dialog/sheet 180; 0 under reduced motion
}
```

### 6.5 Component mapping

| UI token / TS component | Rust name | Notes |
| --- | --- | --- |
| `colors.*` | `theme::color::*` | Opaque sRGB constants |
| `fonts.sans` / `fonts.mono` | `FontFamily::Sans` / `FontFamily::Mono` | Register `Geist` / `Paper Mono` |
| `ui.type.*` | `TextRole::*` | Enum carries size/line-height/family |
| `tone` prop of Text | `TextTone` | `Ink`, `Muted`, `Faint`, `Danger` |
| `ui.space[*]` | `theme::space::S*` | Only listed steps |
| `ui.radius.*` | `Radius` | `None`, `Inner`, `Soft`, `Frame` |
| `ui.line.*` | `theme::line::*` | `HAIR`, `FOCUS`, `FOCUS_OFFSET` |
| `ui.layout.*` | `theme::layout::*` | Uppercase constants |
| `ui.opacity.disabled` | `theme::DISABLED_OPACITY` | 0.4 |
| `Button` | `ui::Button` | Struct + `ButtonVariant`, `ButtonSize`, `ButtonState` |
| `ButtonVariant` | `ButtonVariant` | `Primary`, `Secondary`, `Ghost`, `Link`, `Destructive` |
| `ButtonSize` | `ButtonSize` | `Xs`, `Sm`, `Default`, `Lg` |
| Button state composition | `ButtonState` | `Rest`, `Hover`, `Active`, `Pressed`, `FocusVisible`, `Disabled` |
| `Button` pending | `ButtonPending { label: Option<SharedString> }` | Keeps original name; disables |
| `Card` | `ui::Card` | `Card::new(title).description(..)`; heading level 2 |
| `Field` | `ui::Field` | `Field { label, hint }`; supplies input name/description |
| `Input` | `ui::TextInput` | `InputState { Rest, Hover, Focus, Disabled, ReadOnly }` |
| `Textarea` | `ui::TextArea` | min rows 3, max rows 12, max height 176 |
| `Badge` | `ui::Badge` | Single neutral variant; empty → no element |
| `Alert` | `ui::Alert` | `AlertRole { Alert, Status, Note }`; `AlertTone` not needed |
| `PageHeader` | `ui::PageHeader` | H1 role; actions/description builders |
| `Sidebar` | `ui::Sidebar` | `role=navigation`, default label `"Workspace"` |
| `IconButton` | `ui::IconButton` | 40×40; label + tooltip; ghost + pressed |
| `Tabs` / `TabItem` | `ui::Tabs` / `ui::TabItem` | `TabItem { value, label, closable, icon }` |
| `DropdownMenu` | `ui::DropdownMenu` | Trigger label + `" ▾"`; menu min width 248 |
| `Icon` | `ui::Icon` / `IconName` | `IconName::{House, FolderSimple, Files, Robot, Scan, Gear, UserCircle, SidebarSimple, Close}` |
| icon size 20 | `theme::icon::SIZE` | `pub const SIZE: f32 = 20.0;` |
| `View` | `ui::View` | Flex container; `min_w(0)` |
| `Text` | `ui::Text` | Defaults to `Body`/`Ink`/Geist |

Recommended public enums:

```rust
pub enum ButtonVariant { Primary, Secondary, Ghost, Link, Destructive }
pub enum ButtonSize { Xs, Sm, Default, Lg }
pub enum ButtonState { Rest, Hover, Active, Pressed, FocusVisible, Disabled }
pub enum InputState { Rest, Hover, Focus, Disabled, ReadOnly }
pub enum AlertRole { Alert, Status, Note }
pub enum IconName { House, FolderSimple, Files, Robot, Scan, Gear, UserCircle, SidebarSimple, Close }
```

If the new layout needs a badge with kinds, that is a **new design decision**, not a port of the web badge. Until then, `Badge` has one variant.

---

## 7. Explicitly forbidden

### 7.1 Colours that must not appear

| Forbidden | Why |
| --- | --- |
| `#F6F6F3` (cream canvas), `#F9F9F9` (card), `#F4F4F2`, `#F3F3F0`, `#EDEDE9`, `#EAEAE6`, `#E2E2E2` (web paper/palette) | Cream marketing palette; desktop canvas is `#FCFCFC` and panel `#FFFFFF` |
| `#CFDFF5` (web blue-100 accent) and all `--palette-blue-*` except `#4C94FC` | Web accent wash; desktop blue is `#4C94FC` for non-text marks and `#E3EEFA` for selection |
| `#5B3DF5`, `#EDE9FF`, `#F4F2FA`, `#1D1A24`, `#6E6878`, `#DDD8E6` | Old pre-rewrite purple identity; explicitly replaced |
| `#17663A`, `#E7F6EC` (old green success) | Success is neutral ink/grey; no green in the desktop |
| `#B42318` (old red) | Use `#B72728` |
| Any amber/orange/yellow | Warm hues mean warn/fail only; the native component set has no warn hue |
| Violet/orchid/cyan/mint kind hues | Desktop is neutral + one blue |
| Gradients, glass/blur, coloured shadows, tinted alerts | Flat surfaces only; alerts are paper with a line |

### 7.2 Shapes, depth, decoration

- No shadows on cards, buttons, fields, tabs, headers, or bars. Native tokens define **no shadow at all**; the menu/tooltip use a border only. (The web registry allows a shadow on overlays only.)
- No gradients, sheen, glass, backdrop blur, translucent surfaces.
- No pills or full-round boxes/tags. Radii are 0/4/6/10 only.
- No underline on buttons or tabs in the native system (the web link/active-tab treatments are web-only).
- No side rails/stripes on selected rows; selection is a wash.
- No icon tiles, icon chips, or coloured icon backgrounds.
- No decorative motion, looping, pulsing, shimmer, skeleton sweep, or status dots.

### 7.3 Browser-only primitives

- No DOM elements (`div`, `text`, `input`, `textarea`), no CSS custom properties, no Tailwind classes, no Base UI, no React DOM, no portals, no `document`/`window` styling.
- No web `--palette-*`, `--surface-*`, `--text-*`, `--border-*`, `--hue-*`, `--accent-*`, `--button-*`, `--key-*` tokens. Use the §6 Rust constants.
- No Font Awesome icons in the desktop; Phosphor Regular only.
- No relying on browser focus behaviour; implement the ink ring and keyboard model explicitly.

### 7.4 Typography and content

- No raw font sizes, weights, families, letter-spacing, uppercase, or arbitrary line heights.
- No bold/semibold headings. Hierarchy is size + colour.
- No mono for numbers, durations, counts, versions, or prose. Mono only for code/paths/IDs/hashes/JSON.
- No em dash as an empty value; write “None”.
- No middle-dot separators between facts; use separate elements with a gap.
- No “Learn more”/arrow links; name the destination.
- No small category label (eyebrow) above a title.
- No second primary button in one view.

### 7.5 Layout-copying prohibition

`app-before.tsx.txt` is evidence of the old ad-hoc styling, not a layout reference. Do not copy its 7px button radius, 9px input padding, 18px section titles, `fontWeight: 650`, purple tokens, or arbitrary spacing. The new layout is being designed separately; this document freezes appearance only.

---

## 8. Implementation acceptance checklist

- [ ] Both TTFs registered as `Geist` and `Paper Mono` before the first frame; app UI gated on success.
- [ ] All colours come from `theme::color`; no literal hex outside `theme`.
- [ ] All dimensions come from `theme::space`, `theme::layout`, `ControlHeight`, `RowHeight`, `Radius`, `Line`; no magic px.
- [ ] Every text node uses a `TextRole`; all weights are 400; no uppercase/tracking.
- [ ] Focus is a 2px ink ring with 2px offset on every focusable, including menu items and tabs.
- [ ] Selection is `accentSoft` with ink content; hover is `#F3F3F3`; pressed is `#ECECEC`.
- [ ] Primary is ink fill with `inverse` text; destructive is paper + danger edge/text; no red fill.
- [ ] Button min heights 24/28/32/44 at xs/sm/default/lg, 4px vertical padding, 6px radius, correct variant state matrix.
- [ ] Inputs/Textareas: 32px min height, 12px X padding, 1px `controlLine` edge, 6px radius, hover `controlHover`, focus ink ring + ink edge.
- [ ] Cards: 1px `line`, 10px radius, `panel`, 16px padding/gap, header divider `rowLine`, title `headingXs` level 2.
- [ ] Alerts: paper + 1px `line`, 6px radius, 12px padding, 4px gap, danger title, muted body, correct live-region role.
- [ ] Badges: 20px min height, 4px padding X, 4px radius, `code` fill, `labelSm`, empty renders nothing.
- [ ] PageHeader: H1 `headingLg` 32/36, description `body` muted, copy max 720, actions gap 8.
- [ ] Sidebar: 248px / 48px rail, 16/4px padding+gap, `line` divider, `panel`/`code` fill, navigation name.
- [ ] Tabs: 32px min height, 12px padding, 6px radius, selected `panel`, hover `hover`, ink ring, roving focus, named close buttons.
- [ ] Dropdown: trigger label + `▾`, 248px menu, 4px padding, 6px radius, 1px `line`, 40px items, keyboard model, named menu/items.
- [ ] Icons: Phosphor Regular, 20×20, ink, `aria-hidden`, 40×40 icon-button target.
- [ ] No cream, no purple, no green success, no hue fills, no pills, no shadows on flat surfaces, no browser primitives, no second icon set.
