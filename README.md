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

## Prerequisites

You need Git, [Bun](https://bun.sh), and Rust. Bun runs the monorepo scripts and
the web app. Rust builds the native desktop app; `rust-toolchain.toml` pins the
toolchain, so `rustup` installs it automatically.

Install Bun:

```sh
# macOS and Linux
curl -fsSL https://bun.sh/install | bash

# Windows PowerShell
powershell -c "irm bun.sh/install.ps1 | iex"
```

Install Rust with [rustup](https://rustup.rs):

```sh
# macOS and Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows: install Rustup from https://win.rustup.rs, then use the MSVC toolchain.
```

Native build prerequisites:

- **macOS:** `xcode-select --install`
- **Windows:** Visual Studio Build Tools with the "Desktop development with C++"
  workload, including the Windows SDK.
- **Linux (Debian/Ubuntu):**
  `sudo apt install build-essential pkg-config libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libwayland-dev libx11-dev`
  (Fedora: the equivalent `gcc`, `pkgconf-pkg-config`, `fontconfig-devel`,
  `freetype-devel`, `libxkbcommon-devel`, `wayland-devel`, and `libX11-devel`
  packages.)

Open a new terminal after installing Bun or Rust so `bun` and `cargo` are on
`PATH`.

## Setup

The web app uses Bun. The desktop app is native Rust with [Rune](https://github.com/anomalyco/rune), in
`apps/desktop`.

```sh
git clone https://github.com/universitysjp/coderune.git
cd coderune
bun install
bun run setup:rune
bun run build:desktop
```

`bun run setup:rune` clones the pinned Rune checkout into `.rune`. Run it before
any `cargo` command: the workspace patches Rune’s vendored crates, so Cargo
cannot resolve without that checkout.

Turbo drives the native tasks too: `bun run dev:desktop`, `bun run check:desktop`,
`bun run test`, and `bun run lint`. `bun run dev` stays web-only; use
`bun run dev:all` for web and desktop together. Run the desktop app with
`cargo run -p coderune-desktop`, or pass a directory:
`cargo run -p coderune-desktop -- .`. The binary is
`target/debug/coderune`; `cargo install --path apps/desktop` puts
`coderune` on your PATH. The app opens a project folder in Finder and can launch
`code`, `cursor`, `antigravity`, or `zed` when installed.

The native desktop notes live in `docs/rune/`. The desktop is Rust/Rune; the shared web UI is in `packages/ui`.

Pele is installed for diagrams. Matt Pocock's skills are installed in `.agents/skills`
and recorded in `skills-lock.json`.

Issues and specs will use [GitHub Issues](https://github.com/universitysjp/coderune/issues).
Skills conventions are documented in [docs/agents/](docs/agents/).
