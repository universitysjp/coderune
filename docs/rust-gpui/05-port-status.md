# Native Rust/GPUI port status

Status after the first porting pass. The GPUIX/TypeScript desktop surface is
removed; `apps/desktop` is the native app and builds the `coderune` binary.

## Implemented

**Workspace and service crates**

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
- `crates/app` — the UI-facing operations: onboarding, profiles, projects,
  document extraction/review, captures with stale-preview checks, agent
  previews, export, deletion, directory opening, and editor launch.
- `crates/ui` — fixed light theme, native type roles, buttons, cards, fields,
  alerts, badges, choice chips, navigation rows, and an IME-aware text field
  with selection, clipboard, and undo/redo.

**Native app**

- Window, font registration, embedded Phosphor icons, text-input keybindings,
  and Tab/Shift-Tab focus traversal.
- Developer onboarding with resumable draft and confirmed baseline.
- Sidebar workspace shell with Home, Profile, Projects, Documents, Agents,
  Capture, and Data.
- Profile inspection and versioned editing.
- Project creation, selection, versioned editing, confirmed cascading delete,
  open-folder, and launch-in-VS-Code.
- Markdown/text document extraction, review, save, and agent sharing preview.
- Agent detection screen with honest readiness and restrictions.
- Repository capture preview, included/skipped lists, Git state, explicit
  review, stale-preview rejection, versioned history, content inspection, and
  deletion.
- JSON export and confirmed delete-all returning to onboarding.
- A directory argument opens a matching project or prefills a new project root.

## Not yet ported

- PDF and DOCX text extraction (manual entry remains available).
- Multiline text areas and word-wise text navigation; long values are
  horizontally clipped in the single-line field.
- Draft execution for Codex, OpenCode, and Antigravity; only Claude Code's
  headless mode runs today.
- Applying a reviewed agent draft into the profile or project.
- Per-preview scope/limit overrides and a scope editor in the UI; captures use
  the saved project scope.
- Self-reported skills, education, employment, certifications, links, and
  learning-preference editors. They are stored and exported but can only be
  changed through the data model today.
- Settings/preferences, packaging, signing, auto-update, and a tray icon.
- Later-increment capabilities: skill library, connections, defined subagents,
  model assignments, verification, and learning evidence.

## Commands

```sh
cargo test --workspace
cargo run -p coderune-desktop
cargo run -p coderune-desktop -- .
bun run --cwd apps/web check-types
```

`gpui 0.2.2` is built with `runtime_shaders` because the development machine has
Command Line Tools rather than full Xcode's Metal compiler. The app opens an
ordinary window; no custom titlebar or platform material is required.

## Validation notes

- `cargo test --workspace` passes across domain, storage, collectors, agents,
  and application operations.
- The web app type-checks after removing the native adapter package entry.
- The native window was launched against isolated storage and used to complete
  onboarding and navigate the workspace during the port.
- Visual checks against the token specification remain manual; no screenshot
  regression harness is configured for GPUI windows.
