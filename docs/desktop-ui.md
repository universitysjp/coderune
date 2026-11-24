# Desktop and website UI

The desktop is a native Rust application in `apps/desktop`, built with
[Rune](https://github.com/anomalyco/rune). The website is a React app in
`apps/site`, built with Vite and deployed to GitHub Pages. They share the token
package and visual language; the components do not.

## Boundaries

Desktop:

- `crates/ui` — Rune theme, icons, and reusable primitives.
- `apps/desktop/src/screens/` — one module per screen.
- `apps/desktop/src/app.rs` — state, actions, the RuneKit shell, and navigation.
- `apps/desktop/src/widgets.rs` — app-specific builders shared by screens.
- `crates/domain`, `crates/storage`, `crates/collectors`, `crates/agents`, and
  `crates/app` — framework-independent services.

Website:

- `apps/site/src/routes` — the five pages and the not-found page.
- `apps/site/src/components` — the site's own header, sections, panels and links.
- `packages/tokens` — the palette and font files the site imports.

Screens never import storage or SQL. Service crates never import the UI. The site
keeps its components in `apps/site`; it does not import the desktop screens.

## Visual rules

Use `crates/ui/src/theme.rs` on desktop and `packages/tokens/src/tokens.css` on
the website. The theme is fixed light: white and near-white neutral surfaces,
near-black ink and primary fills, Geist for UI text, and Paper Mono for code,
paths, IDs, hashes, and JSON. Do not add a dark theme, blue or cream accents,
purple, green success colors, shadows on flat surfaces, or a second icon set.
`rune-kit` supplies accessible desktop controls and the shell; the site uses the
token package and ui-sdk behaviors.

## Shell

The desktop follows Hex's Rune layout without copying its theme: on macOS the
content fills the window under the traffic lights, `rune_kit::shell::SplitView`
draws the sidebar and the framed main pane, and `rune_kit::shell::Page` draws
full-window onboarding. The sidebar is 236 points and carries navigation,
profile, and Data; the toolbar shows the current section and the selected
project. Do not hand-build a second sidebar, titlebar, or tab strip in a screen.

The app icon is composed, not painted into the source PNG. `icon-source.png` is
the mark alone; `scripts/make-app-icon.py` draws the grey-to-black gradient,
scales the mark, and writes `icon.icns` plus PNGs. `AppIcon.icon` is the
editable Icon Composer source.

## Verification

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bun run check
bun run verify:ui
bun run build:site
```

`verify:ui` renders onboarding, its later steps, profile completion, and every
workspace scene to `target/ui-snapshots` through Rune's headless renderer,
including narrow window sizes. Debug builds also take Rune's `--tree`,
`--script`, `--frames`, and `--trace` arguments for element-tree and
interaction checks.

## Storage

`crates/storage` uses SeaORM entities and schema-builder migrations. Do not add
hand-written table DDL or raw SQL for repository operations. The legacy
`Context Bridge` database path and data are preserved; migrations are
`if_not_exists` and SeaORM tracks them in `seaql_migrations`.
