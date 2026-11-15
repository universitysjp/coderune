# Coderune

Research into local shared context and assumption verification for AI coding assistants.

## Current stage

Project setup and requirements discussion. The proposal and architecture are in
[docs/presentation.pdf](docs/presentation.pdf).

The first planned milestone is collecting developer and project context and storing
it locally. Architecture and the work breakdown structure will be developed through
`/grill-with-docs` before implementation.

## Desktop architecture

- `apps/desktop/src/app.rs` — application state, actions, and the workspace shell.
- `apps/desktop/src/screens/` — one Rune view module per screen.
- `apps/desktop/src/widgets.rs` — app-specific controls shared by screens.
- `crates/ui` — the Rune design system: tokens, icons, cards, fields, alerts.
- `crates/domain`, `crates/storage`, `crates/collectors`, `crates/agents`, and
  `crates/app` — framework-independent domain and application operations.

Screens never own data and the service crates never import the UI. Rune's
headless snapshots make the views reviewable without a window.

## Setup

The web app uses Bun. The desktop app is native Rust with [Rune](https://github.com/anomalyco/rune), in
`apps/desktop`.

```sh
bun install
bun run setup:rune
cargo build -p coderune-desktop
```

Run the desktop app with `cargo run -p coderune-desktop`, or pass a directory:
`cargo run -p coderune-desktop -- .`. The binary is
`target/debug/coderune`; `cargo install --path apps/desktop` puts
`coderune` on your PATH. The app opens a project folder in Finder and can launch
`code`, `cursor`, `antigravity`, or `zed` when installed.

The native desktop notes live in `docs/rune/`. The desktop is Rust/Rune; the shared web UI is in `packages/ui`.

Pele is installed for diagrams. Matt Pocock's skills are installed in `.agents/skills`
and recorded in `skills-lock.json`.

Issues and specs will use [GitHub Issues](https://github.com/universitysjp/coderune/issues).
Skills conventions are documented in [docs/agents/](docs/agents/).
