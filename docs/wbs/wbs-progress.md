# WBS progress

Completion tracker for the M1 work. Selection and ownership live in [the
WBS](work-breakdown-structure.md); a tick means selected, not complete.
Unticked CI/test and later-phase tasks remain in the full WBS and are omitted
here.

Status as of 2026-11-15. Evidence is the code and checks in this repository,
not the presence of an upstream library. See
[the port status](../rune/05-port-status.md) for the desktop summary.

Legend: `Not started`, `In progress`, `Blocked`, `Done`.

| ID | Work package | Owner | Status | Evidence or note |
| --- | --- | --- | --- | --- |
| 1.3 | Confirm increment requirements and data-sharing rules | Pruthivi Thejan | Done | Confirmed requirements, scope, and sharing rules in [the design](../data-collection-design.md). |
| 1.4 | Confirm increment architecture and module interfaces | Pruthivi Thejan | Done | Rune/Bun/SQLite, package responsibilities, and agent integration scope confirmed in the design. |
| 2.1 | Configure Bun and Turborepo workspace | Pruthivi Thejan | Done | Root package.json, turbo.json, and Bun workspaces exist. |
| 2.2 | Configure TypeScript, lint, format, and test commands | Pruthivi Thejan | Done | `bun run check`, `check-types`, `test`, `lint`, `format`, and desktop-specific scripts exist and run. The full lint task currently stops in vendored Rune crates' MSRV lint; scoped clippy passes. |
| 2.4 | Establish domain glossary and ADR process | Pruthivi Thejan | Done | `GLOSSARY.md` and `docs/agents/domain.md` define vocabulary and the ADR process. |
| 2.5 | Define core, collectors, storage, agents, and Rune app responsibilities | Pruthivi Thejan | Done | Responsibilities and dependency direction are documented and implemented. |
| 3.1 | Define developer profile and initial baseline domain models | Pruthivi Thejan | Done | `ProfileDraft`, `DeveloperProfile`, versions, and required-field rules in `crates/domain`. |
| 3.2 | Implement developer profile entry and validation | Pruthivi Thejan | Done | Three-step onboarding with resumable drafts and required-field checks. |
| 3.3 | Define self-reported skill scale | Pruthivi Thejan | Done | Beginner/intermediate/advanced or unrated; self-report remains distinct from mastery. |
| 3.4 | Define project context and source-provenance domain models | Pruthivi Thejan | Done | `Project`, `ProjectDraft`, `FieldSource`, and document provenance in `crates/domain`. |
| 3.5 | Implement manual project goals, business knowledge, and constraints entry | Pruthivi Thejan | In progress | Goals, scope, and limits are editable. Purpose, rules, constraints, decisions, and knowledge round-trip but have no dedicated editors yet. |
| 3.6 | Confirm required onboarding fields and new-developer path | Pruthivi Thejan | Done | Name, experience, skills or new-to-coding, and at least one goal are required; the new-developer path is explicit. |
| 3.7 | Implement collection permissions and exclusions | Pruthivi Thejan | Done | Include/exclude paths, `.gitignore`, protected files, and limits in `crates/collectors`. |
| 3.8 | Add review before save | Pruthivi Thejan | Done | Capture save requires the reviewed session and rejects a changed fingerprint. |
| 3.9 | Require completed developer onboarding before project work; resume incomplete entry | Pruthivi Thejan | Done | Workspace operations require a confirmed profile; drafts resume at the first incomplete step. |
| 3.10 | Extract selected PDF, DOCX, Markdown, and TXT document text locally | Pruthivi Thejan | Done | `extract_document_text` in `crates/collectors`; DOCX and PDF tests cover the new paths. |
| 3.11 | Map and review CV drafts; collect required fields missing from the CV | Pruthivi Thejan | Not started | Agent draft application is still pending. |
| 3.12 | Import and review optional SRS drafts with document/developer source distinctions | Pruthivi Thejan | Not started | Document storage exists; draft application is pending. |
| 3.13 | Handle import limits, extraction errors, source references, and manual fallback | Pruthivi Thejan | Done | Unsupported, unreadable, image-only, and oversized documents map to actionable errors; manual entry remains. |
| 3.14 | Reuse and edit saved profile/project configuration | Pruthivi Thejan | Done | Profile and project forms load saved values and create new versions. |
| 4.1 | Implement repository selection and scope | Pruthivi Thejan | Done | Project root, include/exclude, and validation in `crates/collectors`. |
| 4.2 | Capture Git state | Pruthivi Thejan | Done | Branch, head, and changed paths recorded when Git is available. |
| 4.3 | Capture dependency manifests | Pruthivi Thejan | Done | Manifests are classified as project files and captured within scope. |
| 4.4 | Capture project configuration | Pruthivi Thejan | Done | Supported configuration files are classified and captured. |
| 4.5 | Capture `AGENTS.md`, `README`, and selected documentation | Pruthivi Thejan | Done | Documentation classification and capture. |
| 4.6 | Capture permitted TypeScript and Python source and test files | Pruthivi Thejan | Done | Source and test classification; no project code runs. |
| 4.7 | Capture file metadata and hashes | Pruthivi Thejan | Done | Per-file SHA-256, bytes, and category. |
| 4.8 | Record capture manifest and provenance | Pruthivi Thejan | Done | Versioned capture rows and content hashes. |
| 4.9 | Implement initial capture operation | Pruthivi Thejan | Done | Preview, review, and save through `Workspace`. |
| 4.10 | Implement manual refresh and duplicate detection | Pruthivi Thejan | Done | Unchanged refresh reports the existing capture; changed content versions. |
| 4.11 | Implement capture summary and inspection | Pruthivi Thejan | Done | Included/skipped lists, file content inspection, and history. |
| 4.12 | Enforce capture limits, skip reasons, and fresh preview after source changes | Pruthivi Thejan | Done | Stale preview rejection and explicit skip reasons. |
| 5.1 | Design SQLite schema for profiles, projects, captures, and imports | Nuwan Konara | Done | SeaORM entities and schema-builder migrations in `crates/storage`. |
| 5.2 | Implement migrations and schema versioning | Nuwan Konara | Done | SeaORM migration tracking with `if_not_exists` schema. |
| 5.3 | Implement repositories, DAOs, and transactions | Nuwan Konara | Done | Storage operations used by `coderune-app`. |
| 5.4 | Implement snapshot and history model | Nuwan Konara | Done | Profile/project versions and capture history. |
| 5.5 | Define update and duplicate policy | Pruthivi Thejan | Done | Version changed captures; reuse content; unchanged refresh creates no duplicate. |
| 5.6 | Implement deletion and retention | Nuwan Konara | Done | Scoped project/capture deletion and confirmed delete-all. |
| 5.7 | Define sensitive-content selection and retention policy | Pruthivi Thejan | Done | Approved selection/exclusions and document-text retention are documented; automated redaction is not claimed. |
| 5.8 | Implement JSON export operation | Pruthivi Thejan | Done | Workspace export writes the local record as JSON. |
| 5.10 | Persist approved document text, hashes, and field/source references | Nuwan Konara | Done | Imported documents and profile field sources are persisted. |
| 5.11 | Persist onboarding drafts, completion, and initial/profile versions | Nuwan Konara | Done | Draft and confirmed profile versions in SQLite. |
| 5.12 | Initialize the per-user database outside repositories and expose consistent reads | Nuwan Konara | Done | Per-user data folder, outside project roots. |
| 6.1 | Build Rune desktop application shell | Pruthivi Thejan | Done | `rune-kit` `SplitView` workspace and `Page` onboarding. |
| 6.2 | Implement navigation and screen flow | Pruthivi Thejan | Done | Sidebar navigation, profile tabs, capture and document flows. |
| 6.3 | Implement forms and validation feedback | Pruthivi Thejan | Done | Required fields, inline alerts, and versioned saves. |
| 6.4 | Add progress, cancellation, and timeouts | Pruthivi Thejan | In progress | Local operations are synchronous; agent jobs are not executed yet. |
| 6.5 | Implement failure recovery and retry | Pruthivi Thejan | In progress | Local save failures preserve input; agent failure paths are pending. |
| 6.6 | Implement inspect, save, edit, and history flows | Pruthivi Thejan | Done | Profile, projects, documents, and captures. |
| 6.8 | Add document/repository selection with typed-path fallback | Pruthivi Thejan | Done | Typed document and root paths with validation. Native pickers are later work. |
| 6.9 | Configure desktop and supported terminal launch modes and platform feedback | Pruthivi Thejan | In progress | Launch-directory prefill exists; installed launcher and running-instance handoff remain (#8). |
| 6.10 | Add JSON export and confirmed project/capture/all-data deletion controls | Pruthivi Thejan | Done | Data screen. |
| 11.1 | Define document-draft interface, capabilities, validation, and errors | Pruthivi Thejan | In progress | Capability detection and sharing previews exist; draft application and execution are pending. |
| 11.2 | Select/detect existing coding tools and report setup or sign-in needs | Pruthivi Thejan | Done | Codex, Claude Code, OpenCode, and Antigravity detection with restrictions. |
| 11.3 | Add Codex App Server adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.4 | Add installed Claude Code CLI adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.5 | Add OpenCode ACP adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.6 | Add Antigravity CLI adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.7 | Enforce per-adapter import tool restrictions, progress, timeouts, and cancellation | Pruthivi Thejan | Not started |  |
| 11.8 | Preview shared document text and handle invalid/unsupported output without data loss | Pruthivi Thejan | In progress | Preview exists; invalid output and draft application are pending. |

## Remaining M1 work

1. Apply a reviewed agent draft to the developer profile or project.
2. Execute draft jobs through Codex, OpenCode, Claude Code, and Antigravity with
   cancellation, timeouts, and no silent agent switch.
3. Override capture scope for one preview without changing the project.
4. Finish installed editor selection (#9), the `coderune <directory>` launcher
   (#8), and desktop packaging.
5. Add dedicated editors for project purpose, business logic, rules,
   constraints, decisions, and knowledge.
