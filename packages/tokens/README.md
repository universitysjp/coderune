# @coderune/tokens

Geist, Paper Mono, and OKLCH color tokens shared by Coderune apps.

## Usage

Add the workspace package to an app:

```json
{
  "dependencies": {
    "@coderune/tokens": "workspace:*"
  }
}
```

Import the stylesheet once at the app entry point:

```ts
import "@coderune/tokens";
```

The import registers `Geist Variable` and `Paper Mono`, exposes the color variables on `:root`, and
makes `--font-family-base` and `--font-family-mono` available.

## Typefaces

`@fontsource-variable/geist` provides locally bundled variable WOFF2 fonts with
weights from 100 to 900. `fonts/PaperMono-Variable.woff2` provides the full Paper
Mono v1.000 variable font with weights from 100 to 800, including its keyboard,
box-drawing, and geometric glyphs. Both use `font-display: swap`; no external
font service is needed.

`--font-family-base` uses Geist for prose and interface text.
`--font-family-mono` uses Paper Mono for code, the index, and section side titles.
The font remains available through `@coderune/tokens/fonts/paper-mono-variable.woff2`.
The site's logo uses Paper Mono's `₪` glyph (U+20AA).

## Colors

All color values are authored in OKLCH. Semantic tokens follow the shadcn/ui
naming convention; `--palette-*` tokens are the source values behind them.

| Token | Source | OKLCH |
| --- | --- | --- |
| `--palette-white` | `#FFFFFF` | `oklch(1 0 0)` |
| `--palette-paper-raised` | `#F9F9F9` | `oklch(0.982 0 0)` |
| `--palette-paper` | `#F6F6F3` | `oklch(0.972 0.004 106.47)` |
| `--palette-paper-soft` | `#F4F4F2` | `oklch(0.967 0.003 106.45)` |
| `--palette-paper-muted` | `#F3F3F0` | `oklch(0.963 0.004 106.47)` |
| `--palette-neutral-100` | `#F2F2F2` | `oklch(0.961 0 0)` |
| `--palette-surface` | `#EDEDE9` | `oklch(0.945 0.005 106.5)` |
| `--palette-surface-sunken` | `#EAEAE6` | `oklch(0.936 0.005 106.5)` |
| `--palette-border` | `#E2E2E2` | `oklch(0.913 0 0)` |
| `--palette-ink-muted` | `#747473` | `oklch(0.559 0.002 106.45)` |
| `--palette-ink` | `#222222` | `oklch(0.252 0 0)` |
| `--palette-black` | `#000000` | `oklch(0 0 0)` |
| `--palette-blue-100` | `#CFDFF5` | `oklch(0.898 0.035 256.34)` |
| `--palette-blue-200` | `#9BBCE8` | `oklch(0.787 0.073 255.74)` |
| `--palette-blue-300` | `#8BADDC` | `oklch(0.74 0.078 256.57)` |
| `--palette-blue-400` | `#82ACE6` | `oklch(0.736 0.096 256.58)` |
| `--palette-blue-500` | `#81ACEC` | `oklch(0.739 0.104 258.14)` |
| `--palette-blue-600` | `#7AACF6` | `oklch(0.739 0.121 258.02)` |
| `--palette-blue-700` | `#4C94FC` | `oklch(0.671 0.171 257.83)` |

The blue steps are ordered by intensity. The supplied blues sit at similar
lightness, so the scale is not a lightness ramp.

`--primary` uses the strongest blue as a fill color. It has 2.79:1 contrast
against `--background`, so do not use it for small text on the cream canvas. Use
`--foreground` for text and `--primary` for fills, rules, focus rings, and other
non-text accents. Text placed on the primary fill uses `--primary-foreground`,
which has 5.26:1 contrast.

## Fonts for UI

`@coderune/ui` owns the UI design system, including its native token projection at
`crates/ui/src/theme.rs`. This package remains the source of the Geist and Paper Mono font files and is embedded by the Rune desktop.
Bundled variable TTF files and OFL licenses live beside the web WOFF2 font; the native entry registers
the TTF files before layout. See
[UI in Coderune](../../docs/desktop-ui.md) for the adapter interface and verification commands.
