# Native desktop port status

`apps/desktop` is the Rune desktop application. `crates/ui` is its Rust design
system. The web app keeps its own React registry in `packages/ui`.

## Implemented

- Onboarding with resumable draft, required-field validation, and a confirmed
  developer baseline.
- Profile inspection and versioned editing that preserves fields the form does
  not show.
- Project creation, selection, versioned editing, scope and capture limits,
  launch-directory matching, open-folder, and editor launch.
- Markdown/text document extraction, review, save, and agent sharing previews.
- Coding-agent detection for Codex, Claude Code, OpenCode, and Antigravity with
  honest readiness and restrictions.
- Reviewed repository capture with included/skipped lists, Git state,
  stale-preview rejection, versioned history, content inspection, and deletion.
- JSON export and confirmed delete-all.
- SQLite storage through SeaORM entities and schema-builder migrations; the
  legacy database path and data are preserved.
- Rune headless scenes for onboarding and every workspace screen.

## Not yet wired

- PDF/DOCX extraction (manual entry remains available).
- Applying an agent draft to a profile or project.
- Draft execution from the UI for Codex, OpenCode, and Antigravity.
- Per-preview scope overrides beyond the project scope.
- Self-reported skills, education, employment, certifications, links, and
  learning-preference editors.
- Settings, packaging, signing, auto-update, and tray behavior.
- Skill library, connections, defined subagents, model assignments,
  verification, and learning evidence (later increments).

## Commands

```sh
bun run setup:rune
cargo run -p coderune-desktop
cargo run -p coderune-desktop -- .
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bun run check
bun run verify:ui
bun run --cwd apps/web build
```
