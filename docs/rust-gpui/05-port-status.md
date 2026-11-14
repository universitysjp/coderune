# Native Rust/Rune port status

Status after the second porting pass. The GPUIX/TypeScript surface and the GPUI
view layer are gone. `apps/desktop` is a native Rune app that builds the
`coderune` binary; `crates/ui` is its Rune design system.

Rune replaced GPUI after the first working GPUI port because Rune can render
deterministic PNGs, dump the element tree, and run scripted pointer/keyboard
input without a window. See [ADR 0005](../adr/0005-desktop-ui-framework.md).

## Implemented

**Service crates (unchanged by the UI switch)**

- `crates/domain` — glossary-aligned profile, onboarding, project, document,
  capture, agent, export, and error types with validation.
- `crates/storage` — legacy-compatible SQLite location and schema (versions
  1–4), versioned profile/project writes, documents, deduplicated capture
  content, export, and deletion cascades.
- `crates/collectors` — explicit, read-only repository walk; protected paths;
  root `.gitignore`; include/exclude matching; size limits; SHA-256; Git state;
  deterministic fingerprints.
- `crates/agents` — Codex, Claude Code, OpenCode, and Antigravity definitions,
  PATH detection, capabilities/restrictions, exact sharing previews, strict
  draft JSON validation, and a Claude Code headless request.
- `crates/app` — onboarding, profiles, projects, document review, captures with
  stale-preview checks, agent previews, export, deletion, directory opening, and
  editor launch.

**Native UI**

- `crates/ui` — fixed light theme, Geist/Paper Mono roles, Phosphor icons, and
  Rune primitives (page header, card, field, alert, badge, nav row, code well).
- `apps/desktop/src/screens/*` — one module per screen: onboarding, home,
  profile, projects, documents, agents, capture, data.
- `apps/desktop/src/app.rs` — state, actions, and the shell only.
- `apps/desktop/src/widgets.rs` — app-specific builders shared by screens.
- Rune `AppSpec` boot with embedded fonts, minimum window size, Cmd-Q, and named
  scenes for headless snapshots.
- Onboarding, profile versioning, project CRUD and deletion, Markdown/text
  document review, agent detection, reviewed captures with history and content
  inspection, JSON export, and confirmed delete-all.
- A directory argument selects a matching project or prefills a new root.

## Verification

Debug builds take Rune's tooling arguments:

```sh
cargo test --workspace
cargo run -p coderune-desktop
cargo run -p coderune-desktop -- .

./target/debug/coderune --snap onboarding.png --size 1280x860
./target/debug/coderune --snap home.png --scene home
./target/debug/coderune --tree tree.json --scene capture-preview
./target/debug/coderune --script script.json --frames 30 --out frames/
```

`scripts/fetch-rune.sh` (or `bun run setup:rune`) clones the pinned Rune
revision under `vendor/rune`. Cargo needs that checkout because the app patches
`wgpu-hal`, `winit`, `parley_engine`, and the accesskit crates to Rune's vendored
copies. The toolchain is pinned to 1.99.1.

The seven workspace scenes (`home`, `profile`, `projects`, `documents`,
`agents`, `capture`, `data`) were rendered headlessly at 1280×860 during the
port, plus `capture-preview` and empty-storage onboarding. The live window was
also launched and used to complete onboarding and navigate the workspace.

## Not yet ported

- PDF and DOCX text extraction (manual entry remains available).
- Multi-line editors for long prose; Rune fields are available, but the current
  forms use single-line inputs.
- Draft execution for Codex, OpenCode, and Antigravity; only Claude Code's
  headless mode runs today.
- Applying a reviewed agent draft into the profile or project.
- Per-preview scope/limit overrides and a scope editor in the UI.
- Self-reported skills, education, employment, certifications, links, and
  learning-preference editors. They are stored and exported but not editable.
- Settings/preferences, packaging, signing, auto-update, and a tray icon.
- Later-increment capabilities: skill library, connections, defined subagents,
  model assignments, verification, and learning evidence.
