# Native desktop port status

`apps/desktop` is the Rune desktop application. `crates/ui` is its Rust design
system. The website is a separate React app in `apps/site`.

## Implemented

- One onboarding dialog over the workspace that asks the four required baseline
  answers, with progress saved as a resumable draft. Later in Profile opens the
  same form in the Profile panel; a confirmed baseline is still required before
  project work.
- Profile inspection and versioned editing that preserves fields the form does
  not show, plus in-app completion of optional self-reported skills, education,
  employment, project experience, certifications, links, and learning
  preferences.
- Project creation from the system folder picker, which opens a quick-questions
  dialog for the name and goals and leaves scope and limits to the panel;
  selection, versioned editing, launch-directory matching, open-folder, and
  editor launch.
- Markdown, plain-text, PDF, and DOCX document extraction, review, save, and
  agent sharing previews. DOCX extraction reads `word/document.xml`; PDF text
  comes from `pdf-extract`. The original document is never copied.
- Coding-agent CLI discovery in Settings follows Gui's 13-agent catalog.
  Installed and missing tools are listed separately, with install links for
  missing tools. Check again re-reads the login-shell PATH.
- Reviewed repository capture with included/skipped lists, Git state,
  stale-preview rejection, versioned history, content inspection, and deletion.
- JSON export and confirmed delete-all.
- SQLite storage through SeaORM entities and schema-builder migrations; the
  legacy database path and data are preserved.
- A desktop workspace with a Rune Kit titlebar strip. Sidebar navigation
  switches pages inside its Workspace tab: Home, Documents, Subagents, and
  Capture sit together at the top, and the Projects list follows below its own
  add button. The strip is the place for future coding-session tabs. Drag the
  sidebar edge to resize, collapse, or reopen it, or use Shift–Command–B
  (Ctrl+Shift+B on Windows and Linux). Below 1080 points the sidebar starts
  collapsed. Wider windows restore the wide layout's sidebar state. Settings in
  the footer opens a dialog with General, Storage, Agents, and Shortcuts.
  Subagents in the workspace sidebar is reserved for Coderune's defined
  specialists.
- Headless scenes for onboarding, the new-project dialog, and every workspace
  page.
- A composed app icon from the supplied mark: grey-to-black gradient, the mark
  at three-quarter scale inside the rounded field, generated `icon.icns` and
  PNGs, with the editable `AppIcon.icon` source under `apps/desktop/assets`.
  macOS uses the bundle icon for packaged builds and the rounded `icon-dock.png`
  for standalone builds.

## Not yet wired

- Applying an agent draft to a profile or project.
- Draft execution from the UI for Codex, OpenCode, and Antigravity.
- Per-preview scope overrides beyond the project scope.
- Additional settings, packaging, signing, auto-update, tray behavior, and the installed
  `coderune <directory>` launcher.
- Installed-editor detection and a persisted editor choice.
- Skill library, connections, defined subagents, model assignments,
  verification, and learning evidence (later increments).

## Commands

```sh
bun run setup:rune
bun run dev:all
bun run dev:desktop
cargo run -p coderune-desktop -- .
bun run build:desktop
bun run check:desktop
bun run test
bun run test:desktop
bun run lint
bun run check
bun run verify:ui
bun run icon:build
bun run build:site
```

Turborepo discovers the Cargo workspace through the experimental
`experimentalCargoWorkspaces` flag. The vendored Rune patch crates under
`.rune/vendor` are workspace members so Turbo can hash them, but they are not
default members: `cargo build` and `cargo test` still target this app and its
crates. Use `bun run test`/`bun run lint` rather than `cargo test --workspace` or
`cargo clippy --workspace`, because `--workspace` includes the platform-specific
vendored crates.
