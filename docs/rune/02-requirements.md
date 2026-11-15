# Requirements

Coderune is a desktop application that keeps local, reviewable context about a
developer and their projects. The developer decides when context is collected,
reviewed, shared, exported, or deleted.

## Scope

In scope:

- Developer onboarding and a reusable developer baseline.
- One developer profile shared across projects.
- Project context: name, root folder, goals, purpose, business logic, rules,
  constraints, decisions, acceptance criteria, knowledge, scope, and limits.
- Repository context collection on explicit request, with review, versioned
  captures, history, refresh, export, and deletion.
- CV and SRS import with local text extraction, review, and optional agent
  sharing previews.
- Detection of installed coding agents without owning their sign-in, providers,
  models, or credentials.
- Local SQLite storage outside project repositories.
- A directory argument that opens a matching project or prefills a new one.

Out of scope for this increment:

- AST/code graph extraction, verification verdicts, learning evidence, and
  gamification.
- Interactive coding sessions, agent credential management, and model accounts.
- OCR, passive collection, browser hosting, and an in-app MCP server.

## Domain

- **Developer profile** — display name, experience stage, new-to-coding flag,
  skills statement, learning goals, self-reported skills, education,
  employment, projects, certifications, links, and learning preferences.
- **Onboarding draft** — a resumable partial profile.
- **Developer baseline** — the first confirmed profile version.
- **Project** — the project context and collection policy above.
- **Imported document** — reviewed CV or SRS text with kind, project link,
  source name/path, hash, and size.
- **Collection scope** — include paths, exclude paths, and root `.gitignore`.
- **Capture limits** — per-file and per-capture byte limits.
- **Context capture** — versioned file list, hashes, skipped reasons, Git state,
  bytes, profile version, and project version.
- **Coding agent** — Codex, Claude Code, OpenCode, or Antigravity, with
  readiness, capabilities, restrictions, and setup documentation.

## Invariants

1. One developer profile and one onboarding draft per database.
2. Project work requires a confirmed profile.
3. A profile or project save creates an immutable version; `createdAt` is kept.
4. Required profile fields: display name, experience stage, skills statement
   (unless explicitly new to coding), and at least one learning goal.
5. Required project fields: name, readable root directory, and at least one goal.
6. Captures are saved only after explicit review and only if the repository still
   matches the reviewed fingerprint.
7. An unchanged refresh creates no duplicate; a changed refresh creates the next
   version when saved.
8. File content is stored once per SHA-256 hash.
9. Deleting a project removes its captures, capture rows, project versions, and
   project-linked documents; the profile and other projects remain.
10. Agent drafts are never applied without review; failures preserve saved data.

## Screens and actions

- **Onboarding** — three short steps. Step one asks for a display name/pseudonym
  and experience stage. Step two asks for an initial skills statement (or an
  explicit new-to-coding answer) and at least one learning goal. Step three
  reviews the baseline and confirms it. Progress is saved as a draft between
  steps; optional profile detail is not part of onboarding.
- **Home** — baseline, project count, capture count, active project, open folder,
  launch editor, and a link into profile completion.
- **Profile** — inspect baseline, edit required fields, and complete optional
  self-reported skills, education, employment, project experience,
  certifications, links, and learning preferences.
- **Projects** — list/select/create/edit/delete, scope and limits, open folder.
- **Documents** — import path, extract PDF/DOCX/Markdown/text, review, save,
  select, prepare agent sharing preview.
- **Agents** — detection, readiness, capabilities, restrictions, setup links.
- **Capture** — preview, review files and skips, save, refresh, history, inspect,
  delete.
- **Data** — database location, JSON export, confirmed delete-all.

## Storage

The database is a per-user SQLite file under the legacy `Context Bridge`
application directory. SeaORM entities define the tables and schema-builder
migrations create them. JSON `data` columns remain the export shape; normalized
columns support ordering and filtering. Schema versioning is tracked by SeaORM;
older databases open because every table and index is `if_not_exists`.

## Collection

Collect only files inside the selected root. Never follow symlinks. Exclude
environment files, private keys, dependency directories, generated output,
binaries, and paths outside the root. Honour the root `.gitignore`. Enforce
per-file and per-capture limits. Store SHA-256 hashes, Git state, skipped reasons,
and deterministic ordering. Repository scanning never runs project tooling or
contacts a network service.

## Agents

Detection is per executable on `PATH`. The agent keeps its own sign-in, provider,
model, permissions, and credentials. Coderune never reads or exports credential
files. A sharing preview shows the exact text, byte count, agent limit, and fit
before any request. There is no automatic fallback to another agent or model.
Failures, cancellation, timeouts, invalid output, and unavailable tools leave
saved data unchanged and manual entry available.
