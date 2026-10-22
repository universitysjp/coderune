# WBS progress

Completion tracker for the work packages selected in the first implementation
increment. Selection lives in
[work-breakdown-structure.md](work-breakdown-structure.md); this file tracks
completion separately.

Status as of 2026-10-22. Update the status and evidence as work lands.

Legend: `Not started`, `In progress`, `Blocked`, `Done`.

| ID | Work package | Owner | Status | Evidence or note |
| --- | --- | --- | --- | --- |
| 2.1 | Configure Bun and Turborepo workspace | Pruthivi Thejan | Done | Root `package.json`, `turbo.json`, and workspaces exist. |
| 2.2 | Configure TypeScript, lint, format, and test commands | Pruthivi Thejan | In progress | `typecheck` exists; lint, format, and test commands are not configured yet. |
| 2.3 | Add CI and pre-commit checks | Nuwan Konara | Not started | |
| 2.4 | Establish domain glossary and ADR process | Raviru Rathnaweera | In progress | `GLOSSARY.md` exists; `docs/adr/` does not. |
| 2.5 | Define package structure and module boundaries | Nuwan Konara | Not started | `apps/` and `packages/` contain only `.gitkeep`. |
| 3.1 | Design developer profile schema | Pruthivi Thejan | Not started | |
| 3.2 | Implement developer profile entry and validation | Pruthivi Thejan | Not started | |
| 3.3 | Define self-reported skill scale | Raviru Rathnaweera | Not started | |
| 3.4 | Design project context schema | Nuwan Konara | Not started | |
| 3.5 | Implement project goals, rules, and acceptance criteria entry | Pruthivi Thejan | Not started | |
| 3.6 | Decide required and optional onboarding fields | Raviru Rathnaweera | Not started | |
| 3.7 | Implement collection permissions and exclusions | Nuwan Konara | Not started | |
| 3.8 | Add review before save | Pruthivi Thejan | Not started | |
| 4.1 | Implement repository selection and scope | Pruthivi Thejan | Not started | |
| 4.2 | Capture Git state | Nuwan Konara | Not started | |
| 4.3 | Capture dependency manifests | Nuwan Konara | Not started | |
| 4.4 | Capture project configuration | Nuwan Konara | Not started | |
| 4.5 | Capture `AGENTS.md`, `README`, and selected documentation | Nuwan Konara | Not started | |
| 4.6 | Capture permitted TypeScript and Python source and test files | Nuwan Konara | Not started | |
| 4.7 | Capture file metadata and hashes | Nuwan Konara | Not started | |
| 4.8 | Record capture manifest and provenance | Raviru Rathnaweera | Not started | |
| 4.9 | Implement initial capture operation | Nuwan Konara | Not started | |
| 4.10 | Implement manual refresh and duplicate detection | Nuwan Konara | Not started | |
| 4.11 | Implement capture summary and inspection | Pruthivi Thejan | Not started | |
| 5.1 | Design SQLite schema for profiles, projects, and captures | Nuwan Konara | Not started | |
| 5.2 | Implement migrations and schema versioning | Nuwan Konara | Not started | |
| 5.3 | Implement repositories, DAOs, and transactions | Nuwan Konara | Not started | |
| 5.4 | Implement snapshot and history model | Nuwan Konara | Not started | |
| 5.5 | Define update and duplicate policy | Raviru Rathnaweera | Not started | |
| 5.6 | Implement deletion and retention | Nuwan Konara | Not started | |
| 5.7 | Define sensitive content and redaction policy | Raviru Rathnaweera | Not started | |
| 5.8 | Implement backup and export interface | Pruthivi Thejan | Not started | |
| 5.9 | Add database tests and fixtures | Nuwan Konara | Not started | |
| 6.1 | Build OpenTUI application shell | Pruthivi Thejan | Not started | |
| 6.2 | Implement navigation and screen flow | Pruthivi Thejan | Not started | |
| 6.3 | Implement forms and validation feedback | Pruthivi Thejan | Not started | |
| 6.4 | Add progress, cancellation, and timeouts | Pruthivi Thejan | Not started | |
| 6.5 | Implement failure recovery and retry | Nuwan Konara | Not started | |
| 6.6 | Implement inspect and save flow | Pruthivi Thejan | Not started | |
| 6.7 | Add end-to-end acceptance tests | Raviru Rathnaweera | Not started | |
