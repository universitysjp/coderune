# Desktop and web UI

The desktop is a native Rust application in `apps/desktop`, built with
[Rune](https://github.com/anomalyco/rune). The web app is React in `apps/web`.
They share the same visual language through two implementations.

## Boundaries

Desktop:

- `crates/ui` — Rune theme, icons, and reusable primitives.
- `apps/desktop/src/screens/` — one module per screen.
- `apps/desktop/src/app.rs` — state, actions, and the workspace shell.
- `apps/desktop/src/widgets.rs` — app-specific builders shared by screens.
- `crates/domain`, `crates/storage`, `crates/collectors`, `crates/agents`, and
  `crates/app` — framework-independent services.

Web:

- `packages/ui/src/web` — the shared React registry.
- `apps/web/src/components` — page-specific components and layout.
- `apps/web/src/routes` — TanStack routes.

Screens never import storage or SQL. Service crates never import the UI. The web
app imports registry components from `@coderune/ui/web`; it does not keep a
second copy of the registry.

## Visual rules

Use `crates/ui/src/theme.rs` on desktop and `packages/ui/src/web/tokens.css` on
web. The theme is fixed light: neutral paper/ink surfaces, ink focus rings, a
blue wash for selection, Geist for UI text, and Paper Mono for code, paths, IDs,
hashes, and JSON. Do not add a dark theme, purple, green success colors, shadows
on flat surfaces, or a second icon set. `rune-kit` supplies accessible desktop
buttons, fields, and menus; the web registry supplies the React equivalents.

## Verification

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bun run check
bun run verify:ui
bun run --cwd apps/web build
```

`verify:ui` renders onboarding and every workspace scene to
`target/ui-snapshots` through Rune's headless renderer. Debug builds also take
Rune's `--tree`, `--script`, `--frames`, and `--trace` arguments for element-tree
and interaction checks.

## Storage

`crates/storage` uses SeaORM entities and schema-builder migrations. Do not add
hand-written table DDL or raw SQL for repository operations. The legacy
`Context Bridge` database path and data are preserved; migrations are
`if_not_exists` and SeaORM tracks them in `seaql_migrations`.
