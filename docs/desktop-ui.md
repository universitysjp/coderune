# Desktop UI in Coderune

The desktop is a native Rust application in `apps/desktop`, built with
[Rune](https://github.com/anomalyco/rune). There is no browser, web view, GPUIX
React tree, or DOM adapter. See [ADR 0005](./adr/0005-desktop-ui-framework.md).

## Boundaries

- `crates/ui` — Rune design system: `theme.rs` (tokens and `Kit`), `icons.rs`
  (Phosphor bodies), `components.rs` (page header, card, field, alert, badge,
  nav row, code well).
- `apps/desktop/src/screens/` — one module per screen.
- `apps/desktop/src/app.rs` — state, actions, and the window shell.
- `apps/desktop/src/widgets.rs` — app-specific builders shared by screens.
- `crates/domain`, `crates/storage`, `crates/collectors`, `crates/agents`, and
  `crates/app` — framework-independent services. Screens never import storage
  or SQL; the service crates never import the UI.

## Visual rules

Use `crates/ui/src/theme.rs` for every aesthetic value. The theme is fixed
light: neutral paper/ink surfaces, ink focus rings, a blue wash for selection,
Geist for UI text, and Paper Mono for code, paths, IDs, hashes, and JSON. Do not
add a dark theme, purple, green success colors, shadows on flat surfaces, or a
second icon set. `rune-kit` supplies accessible buttons, fields, and menus.

## Verification

```sh
cargo test --workspace
bun run verify:ui
```

`verify:ui` renders onboarding and every workspace scene to
`target/ui-snapshots` through Rune's headless renderer. Debug builds also take
Rune's `--tree`, `--script`, `--frames`, and `--trace` tooling arguments for
element-tree and interaction checks.

## Storage

`crates/storage` uses SeaORM entities and schema-builder migrations. Do not add
hand-written table DDL or raw SQL for repository operations. The legacy
`Context Bridge` database path and data are preserved; migrations are
`if_not_exists` and SeaORM tracks them in `seaql_migrations`.
