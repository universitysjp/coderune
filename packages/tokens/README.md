# @coderune/tokens

Geist, Paper Mono, and neutral color tokens shared by Coderune apps.

## Usage

`apps/site` is the primary consumer: the Rune website includes `src/tokens.css`
at compile time and reads its palette from the custom properties, and the build
copies `fonts/*.ttf` beside the static page. Editing a colour here changes both
the canvas and the HTML first frame after a rebuild.

A TypeScript app can still import the package as before:

```json
{
  "dependencies": {
    "@coderune/tokens": "workspace:*"
  }
}
```

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

Neutral tokens from OpenCode's v2 light theme, the same palette the Rune desktop
draws with. There is no blue accent and no cream paper: the canvas is white,
raised surfaces are near-white grey, and the primary action is near-black.
Semantic tokens follow the shadcn/ui naming convention; `--palette-*` tokens are
the source values behind them.

| Token | Value |
| --- | --- |
| `--palette-white` | `#FFFFFF` |
| `--palette-paper-raised` | `#FAFAFA` |
| `--palette-paper` | `#FFFFFF` |
| `--palette-paper-soft` | `#FAFAFA` |
| `--palette-paper-muted` | `#F2F2F2` |
| `--palette-neutral-100` | `#EDEDED` |
| `--palette-surface` | `#F2F2F2` |
| `--palette-surface-sunken` | `#EEEEEE` |
| `--palette-border` | `oklch(0 0 0 / 0.1)` |
| `--palette-ink-muted` | `#5C5C5C` |
| `--palette-ink` | `#161616` |
| `--palette-black` | `#000000` |

`--primary` is the contrast fill (`#242424`) with white text
(`--primary-foreground`). Use `--foreground` for text and `--primary` for fills,
rules, focus rings, and other non-text accents.

## Fonts for UI

The website and the Rune desktop share the Geist and Paper Mono files. This package
is the source of the bundled variable TTFs and their OFL licences: `apps/site`
copies them beside the static page, and the Rune desktop embeds them before
layout. See [UI in Coderune](../../docs/desktop-ui.md) for the desktop component
and verification commands and [Website typography and layout](../../docs/web-typography-layout.md)
for the site's type scale.
