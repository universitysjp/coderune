# WBS progress

Completion tracker for work selected in the current session's first increment.
Selection and ownership live in [the WBS](work-breakdown-structure.md); a tick
means selected, not complete. Unticked CI/test and later-phase tasks remain in
the full WBS and are omitted here.

Status as of 2026-10-22. The interview round is complete. Application features
below remain not started unless concrete evidence is listed; upstream tool
documentation and existing web/design scaffolds do not establish Rune or agent
adapter implementation.

Legend: `Not started`, `In progress`, `Blocked`, `Done`.

| ID | Work package | Owner | Status | Evidence or note |
| --- | --- | --- | --- | --- |
| 1.3 | Confirm increment requirements and data-sharing rules | Pruthivi Thejan | Done | Confirmed requirements, scope, and sharing rules in [the design](../data-collection-design.md). |
| 1.4 | Confirm increment architecture and module interfaces | Pruthivi Thejan | Done | Rune/Bun/SQLite, package responsibilities, and agent integration scope confirmed in the design. |
| 2.1 | Configure Bun and Turborepo workspace | Pruthivi Thejan | Done | Root package.json, turbo.json, and Bun workspaces exist; existing foundation status retained. |
| 2.2 | Configure TypeScript, lint, format, and test commands | Pruthivi Thejan | In progress | Root/web scripts and TypeScript configuration exist. Planned Rune/shared-package tooling is not implemented or demonstrated yet. |
| 2.4 | Establish domain glossary and ADR process | Pruthivi Thejan | Done | GLOSSARY.md and docs/agents/domain.md define vocabulary and the ADR process; create ADRs only for qualifying decisions. |
| 2.5 | Define core, collectors, storage, agents, and Rune app responsibilities | Pruthivi Thejan | Done | Planned app/core/collectors/storage/agents responsibilities and dependency direction are documented. This does not mean the packages are built. |
| 3.1 | Define developer profile and initial baseline domain models | Pruthivi Thejan | Not started |  |
| 3.2 | Implement developer profile entry and validation | Pruthivi Thejan | Not started |  |
| 3.3 | Define self-reported skill scale | Pruthivi Thejan | Done | Beginner/intermediate/advanced or unrated; self-report remains distinct from demonstrated mastery. |
| 3.4 | Define project context and source-provenance domain models | Pruthivi Thejan | Not started |  |
| 3.5 | Implement manual project goals, business knowledge, and constraints entry | Pruthivi Thejan | Not started |  |
| 3.6 | Confirm required onboarding fields and new-developer path | Pruthivi Thejan | Done | Q24 confirmed required name, experience, initial skills, learning goal, optional CV, and explicit new-developer path. |
| 3.7 | Implement collection permissions and exclusions | Pruthivi Thejan | Not started |  |
| 3.8 | Add review before save | Pruthivi Thejan | Not started |  |
| 3.9 | Require completed developer onboarding before project work; resume incomplete entry | Pruthivi Thejan | Not started |  |
| 3.10 | Extract selected PDF, DOCX, Markdown, and TXT document text locally | Pruthivi Thejan | Not started |  |
| 3.11 | Map and review CV drafts; collect required fields missing from the CV | Pruthivi Thejan | Not started |  |
| 3.12 | Import and review optional SRS drafts with document/developer source distinctions | Pruthivi Thejan | Not started |  |
| 3.13 | Handle import limits, extraction errors, source references, and manual fallback | Pruthivi Thejan | Not started |  |
| 3.14 | Reuse and edit saved profile/project configuration | Pruthivi Thejan | Not started |  |
| 4.1 | Implement repository selection and scope | Pruthivi Thejan | Not started |  |
| 4.2 | Capture Git state | Pruthivi Thejan | Not started |  |
| 4.3 | Capture dependency manifests | Pruthivi Thejan | Not started |  |
| 4.4 | Capture project configuration | Pruthivi Thejan | Not started |  |
| 4.5 | Capture `AGENTS.md`, `README`, and selected documentation | Pruthivi Thejan | Not started |  |
| 4.6 | Capture permitted TypeScript and Python source and test files | Pruthivi Thejan | Not started |  |
| 4.7 | Capture file metadata and hashes | Pruthivi Thejan | Not started |  |
| 4.8 | Record capture manifest and provenance | Pruthivi Thejan | Not started |  |
| 4.9 | Implement initial capture operation | Pruthivi Thejan | Not started |  |
| 4.10 | Implement manual refresh and duplicate detection | Pruthivi Thejan | Not started |  |
| 4.11 | Implement capture summary and inspection | Pruthivi Thejan | Not started |  |
| 4.12 | Enforce capture limits, skip reasons, and fresh preview after source changes | Pruthivi Thejan | Not started |  |
| 5.1 | Design SQLite schema for profiles, projects, captures, and imports | Nuwan Konara | Not started |  |
| 5.2 | Implement migrations and schema versioning | Nuwan Konara | Not started |  |
| 5.3 | Implement repositories, DAOs, and transactions | Nuwan Konara | Not started |  |
| 5.4 | Implement snapshot and history model | Nuwan Konara | Not started |  |
| 5.5 | Define update and duplicate policy | Pruthivi Thejan | Done | Version changed captures; reuse content; unchanged refresh creates no duplicate; preserve historical profile/project context. |
| 5.6 | Implement deletion and retention | Nuwan Konara | Not started |  |
| 5.7 | Define sensitive-content selection and retention policy | Pruthivi Thejan | Done | Approved selection/exclusions and document-text retention are documented; automated content redaction is not claimed. |
| 5.8 | Implement JSON export operation | Pruthivi Thejan | Not started |  |
| 5.10 | Persist approved document text, hashes, and field/source references | Nuwan Konara | Not started |  |
| 5.11 | Persist onboarding drafts, completion, and initial/profile versions | Nuwan Konara | Not started |  |
| 5.12 | Initialize the per-user database outside repositories and expose consistent reads | Nuwan Konara | Not started |  |
| 6.1 | Build Rune React/Bun application shell | Pruthivi Thejan | Not started |  |
| 6.2 | Implement navigation and screen flow | Pruthivi Thejan | Not started |  |
| 6.3 | Implement forms and validation feedback | Pruthivi Thejan | Not started |  |
| 6.4 | Add progress, cancellation, and timeouts | Pruthivi Thejan | Not started |  |
| 6.5 | Implement failure recovery and retry | Pruthivi Thejan | Not started |  |
| 6.6 | Implement inspect, save, edit, and history flows | Pruthivi Thejan | Not started |  |
| 6.8 | Add document/repository selection with typed-path fallback | Pruthivi Thejan | Not started |  |
| 6.9 | Configure desktop and supported terminal launch modes and platform feedback | Pruthivi Thejan | Not started |  |
| 6.10 | Add JSON export and confirmed project/capture/all-data deletion controls | Pruthivi Thejan | Not started |  |
| 11.1 | Define document-draft interface, capabilities, validation, and errors | Pruthivi Thejan | Not started |  |
| 11.2 | Select/detect existing coding tools and report setup or sign-in needs | Pruthivi Thejan | Not started |  |
| 11.3 | Add Codex App Server adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.4 | Add installed Claude Code CLI adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.5 | Add OpenCode ACP adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.6 | Add Antigravity CLI adapter for document draft jobs | Pruthivi Thejan | Not started |  |
| 11.7 | Enforce per-adapter import tool restrictions, progress, timeouts, and cancellation | Pruthivi Thejan | Not started |  |
| 11.8 | Preview shared document text and handle invalid/unsupported output without data loss | Pruthivi Thejan | Not started |  |

## Next implementation handoff

- Pruthivi: establish the core models/interfaces and Rune manual onboarding flow;
  require the confirmed developer baseline before starting project work.
- Nuwan: implement the per-user SeaORM SQLite store, migrations, onboarding/profile
  versions, approved imports, transactional captures, and history/deletion reads.
- Then connect the local document extractors, agent draft adapters, repository
  capture, and data-management flows against those interfaces.
