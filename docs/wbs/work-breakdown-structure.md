# Work breakdown structure

Status: revised after the confirmed interview round on 2026-10-22. The scope covers
the whole project; only ticked work is selected for the current implementation
increment. Selection does not mean completion.

Sources: [presentation.pdf](../presentation.pdf),
[research proposal](../proposal_group_23.pdf), and the confirmed
[data collection design](../data-collection-design.md). The interview's decisions
define this increment where they refine the original proposal.

## Team

| Member | Primary responsibility |
| --- | --- |
| Pruthivi Thejan | GPUIX application, collection, and agent integration |
| Nuwan Konara | Backend, local storage, and database |
| Raviru Rathnaweera | Project management, research, verification, and evaluation |

## Assignment rule

For tasks selected in this session, Pruthivi owns application, domain-design,
collection, document import, and agent integration work. Nuwan owns database-layer
implementation. Raviru retains project management, research, verification,
learning, and evaluation work across the full project.

All three members remain represented across the full project; each phase does not
need an artificial assignment to every member. Existing work-package IDs are
preserved. Diagram initials are PT = Pruthivi Thejan, NK = Nuwan Konara,
RR = Raviru Rathnaweera; the tables give the full names.

## Diagram

Pele renders the WBS from [work-breakdown-structure.mmd](work-breakdown-structure.mmd)
into SVG, then `@resvg/resvg-js` converts it to PNG.

- [work-breakdown-structure.svg](work-breakdown-structure.svg)
- [work-breakdown-structure.png](work-breakdown-structure.png)

![Coderune work breakdown structure](work-breakdown-structure.png)

Run `bun run wbs:render` after editing the Mermaid source.

Legend: `✓` marks a work package selected for the current session's first increment.
Unticked packages remain in the wider project plan. Phase ticks mean that the
phase contains selected tasks; leaf ticks identify the exact selection.
Task-level selection and completion are tracked separately in
[wbs-progress.md](wbs-progress.md).

## Current increment

The first increment collects and stores context locally.

In scope:

- Mandatory developer onboarding before project work: display name, experience
  stage, initial skills statement, and a learning goal. An explicit new-to-coding
  baseline is valid; a CV and prior employment are optional.
- Reusable confirmed developer profile, initial baseline version, and resumable
  incomplete onboarding.
- Manual entry or reviewed local CV import. Text-based PDF, DOCX, Markdown, and
  TXT are supported document targets; scanned-document OCR is deferred.
- Project context from optional SRS imports and developer-held business logic,
  purpose, rules, constraints, decisions, and acceptance criteria. Reuse existing
  project configuration and allow editing.
- Repository context selected and captured on explicit request.
- Git state, dependency manifests, project configuration, `AGENTS.md`, `README`,
  selected documentation, and permitted TypeScript and Python source and test files.
- One GPUIX React app with Bun, native desktop and supported terminal launch modes.
  Initial desktop targets are macOS Apple Silicon and Windows x64. Terminal mode
  requires supported macOS/Linux terminals; Windows/tmux terminal support is not
  claimed.
- Local Bun SQLite outside repositories, with transactions, versions, approved
  imported text, source references, and history.
- Inspect, save, reopen, edit, manually refresh, export JSON, and confirm deletion.
- `packages/agents` adapters targeting installed Codex, Claude Code, OpenCode, and
  Antigravity for user-requested CV/SRS drafts. Each tool retains its own sign-in,
  providers, and configuration; capabilities must be demonstrated per adapter.
- Preview shared text, review and correct generated drafts, validate their shape,
  and retain manual entry when a tool is unavailable.

Out of scope for this increment:

- AST and code-graph extraction.
- Assumption verification.
- Developer learning and gamification.
- Inferred skill assessment; a self-reported baseline is not verified mastery.
- Interactive project coding sessions, passive activity tracking, and OCR.
- Application-managed agent accounts, credentials, or model-provider sign-in.
- Browser/WASM hosting and a separate noninteractive application CLI.
- Remote or optional external learning services.
- The optional Eve/isolated learning subagent, unless separately requested.

Existing CI and automated-test packages remain visible in the wider plan and are
not selected for this session. The existing web scaffold and design assets are
preserved; the GPUIX app is planned in `apps/app`.

## Planned implementation order

1. Finalize domain requirements and module interfaces (1.3–1.4, 2.4–2.5, 3.3, 3.6).
2. Implement storage and the manual onboarding/project flow (3, 5, 6).
3. Add local CV/SRS text extraction and selected-agent draft adapters (3, 11).
4. Complete repository capture, review, refresh, history, and data management (4–6).

Storage and application work can proceed independently against the core's agreed
interfaces. Agent-assisted imports do not block manual onboarding.

## Work packages

### 1. Research and design

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 1.1 | Confirm problem statement, research questions, and objectives | Raviru Rathnaweera | |
| 1.2 | Complete literature review and gap matrix | Raviru Rathnaweera | |
| 1.3 | Confirm increment requirements and data-sharing rules | Pruthivi Thejan | ✓ |
| 1.4 | Confirm increment architecture and module interfaces | Pruthivi Thejan | ✓ |
| 1.5 | Maintain project plan, schedule, and risk register | Raviru Rathnaweera | |
| 1.6 | Run progress reporting and review cadence | Raviru Rathnaweera | |

### 2. Project foundations

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 2.1 | Configure Bun and Turborepo workspace | Pruthivi Thejan | ✓ |
| 2.2 | Configure TypeScript, lint, format, and test commands | Pruthivi Thejan | ✓ |
| 2.3 | Add CI and pre-commit checks | Nuwan Konara | |
| 2.4 | Establish domain glossary and ADR process | Pruthivi Thejan | ✓ |
| 2.5 | Define core, collectors, storage, agents, and GPUIX app responsibilities | Pruthivi Thejan | ✓ |

### 3. Developer and project context

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 3.1 | Define developer profile and initial baseline domain models | Pruthivi Thejan | ✓ |
| 3.2 | Implement developer profile entry and validation | Pruthivi Thejan | ✓ |
| 3.3 | Define self-reported skill scale | Pruthivi Thejan | ✓ |
| 3.4 | Define project context and source-provenance domain models | Pruthivi Thejan | ✓ |
| 3.5 | Implement manual project goals, business knowledge, and constraints entry | Pruthivi Thejan | ✓ |
| 3.6 | Confirm required onboarding fields and new-developer path | Pruthivi Thejan | ✓ |
| 3.7 | Implement collection permissions and exclusions | Pruthivi Thejan | ✓ |
| 3.8 | Add review before save | Pruthivi Thejan | ✓ |
| 3.9 | Require completed developer onboarding before project work; resume incomplete entry | Pruthivi Thejan | ✓ |
| 3.10 | Extract selected PDF, DOCX, Markdown, and TXT document text locally | Pruthivi Thejan | ✓ |
| 3.11 | Map and review CV drafts; collect required fields missing from the CV | Pruthivi Thejan | ✓ |
| 3.12 | Import and review optional SRS drafts with document/developer source distinctions | Pruthivi Thejan | ✓ |
| 3.13 | Handle import limits, extraction errors, source references, and manual fallback | Pruthivi Thejan | ✓ |
| 3.14 | Reuse and edit saved profile/project configuration | Pruthivi Thejan | ✓ |

### 4. Repository capture

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 4.1 | Implement repository selection and scope | Pruthivi Thejan | ✓ |
| 4.2 | Capture Git state | Pruthivi Thejan | ✓ |
| 4.3 | Capture dependency manifests | Pruthivi Thejan | ✓ |
| 4.4 | Capture project configuration | Pruthivi Thejan | ✓ |
| 4.5 | Capture `AGENTS.md`, `README`, and selected documentation | Pruthivi Thejan | ✓ |
| 4.6 | Capture permitted TypeScript and Python source and test files | Pruthivi Thejan | ✓ |
| 4.7 | Capture file metadata and hashes | Pruthivi Thejan | ✓ |
| 4.8 | Record capture manifest and provenance | Pruthivi Thejan | ✓ |
| 4.9 | Implement initial capture operation | Pruthivi Thejan | ✓ |
| 4.10 | Implement manual refresh and duplicate detection | Pruthivi Thejan | ✓ |
| 4.11 | Implement capture summary and inspection | Pruthivi Thejan | ✓ |
| 4.12 | Enforce capture limits, skip reasons, and fresh preview after source changes | Pruthivi Thejan | ✓ |
| 4.13 | Add separately consented passive activity/context collection | Nuwan Konara | |

### 5. Local storage

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 5.1 | Design SQLite schema for profiles, projects, captures, and imports | Nuwan Konara | ✓ |
| 5.2 | Implement migrations and schema versioning | Nuwan Konara | ✓ |
| 5.3 | Implement repositories, DAOs, and transactions | Nuwan Konara | ✓ |
| 5.4 | Implement snapshot and history model | Nuwan Konara | ✓ |
| 5.5 | Define update and duplicate policy | Pruthivi Thejan | ✓ |
| 5.6 | Implement deletion and retention | Nuwan Konara | ✓ |
| 5.7 | Define sensitive-content selection and retention policy | Pruthivi Thejan | ✓ |
| 5.8 | Implement JSON export operation | Pruthivi Thejan | ✓ |
| 5.9 | Add database tests and fixtures | Nuwan Konara | |
| 5.10 | Persist approved document text, hashes, and field/source references | Nuwan Konara | ✓ |
| 5.11 | Persist onboarding drafts, completion, and initial/profile versions | Nuwan Konara | ✓ |
| 5.12 | Initialize the per-user database outside repositories and expose consistent reads | Nuwan Konara | ✓ |

### 6. GPUIX application and user experience

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 6.1 | Build GPUIX React/Bun application shell | Pruthivi Thejan | ✓ |
| 6.2 | Implement navigation and screen flow | Pruthivi Thejan | ✓ |
| 6.3 | Implement forms and validation feedback | Pruthivi Thejan | ✓ |
| 6.4 | Add progress, cancellation, and timeouts | Pruthivi Thejan | ✓ |
| 6.5 | Implement failure recovery and retry | Pruthivi Thejan | ✓ |
| 6.6 | Implement inspect, save, edit, and history flows | Pruthivi Thejan | ✓ |
| 6.7 | Add end-to-end acceptance tests | Raviru Rathnaweera | |
| 6.8 | Add document/repository selection with typed-path fallback | Pruthivi Thejan | ✓ |
| 6.9 | Configure desktop and supported terminal launch modes and platform feedback | Pruthivi Thejan | ✓ |
| 6.10 | Add JSON export and confirmed project/capture/all-data deletion controls | Pruthivi Thejan | ✓ |

### 7. AST and code graph

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 7.1 | Integrate codegraph Tree-sitter parsing for TypeScript and Python | Raviru Rathnaweera | |
| 7.2 | Extract symbols, imports, and references | Raviru Rathnaweera | |
| 7.3 | Map derived local graph queries and provenance to stored captures | Nuwan Konara | |
| 7.4 | Implement incremental index refresh | Nuwan Konara | |
| 7.5 | Implement evidence retrieval API | Nuwan Konara | |
| 7.6 | Add graph quality and performance tests | Raviru Rathnaweera | |
| 7.7 | Build graph inspection UI | Pruthivi Thejan | |

### 8. Assumption verification

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 8.1 | Define assumption and claim schema | Raviru Rathnaweera | |
| 8.2 | Extract assumptions from AI-generated diffs | Raviru Rathnaweera | |
| 8.3 | Match claims to workspace evidence | Raviru Rathnaweera | |
| 8.4 | Implement verdict engine | Nuwan Konara | |
| 8.5 | Add constraint, build, and test checks | Nuwan Konara | |
| 8.6 | Record confidence and provenance | Raviru Rathnaweera | |
| 8.7 | Build verification review UI | Pruthivi Thejan | |
| 8.8 | Add verification acceptance tests | Raviru Rathnaweera | |

### 9. Human-AI alignment and learning

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 9.1 | Define skill profile and knowledge-gap model | Raviru Rathnaweera | |
| 9.2 | Extract concepts from generated diffs | Raviru Rathnaweera | |
| 9.3 | Define learning trigger policy | Raviru Rathnaweera | |
| 9.4 | Implement gamified quiz generation | Raviru Rathnaweera | |
| 9.5 | Implement skill evidence and mastery update | Nuwan Konara | |
| 9.6 | Build learning dashboard and session UI | Pruthivi Thejan | |
| 9.7 | Optional isolated learning subagent; excluded unless separately requested | Raviru Rathnaweera | |
| 9.8 | Add learning evaluation hooks | Nuwan Konara | |
| 9.9 | Review learning content quality | Raviru Rathnaweera | |

### 10. Evaluation and dissemination

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 10.1 | Write study protocol, consent, and ethics materials | Raviru Rathnaweera | |
| 10.2 | Define metrics and instruments | Raviru Rathnaweera | |
| 10.3 | Run pilot with 2-3 developers | Nuwan Konara | |
| 10.4 | Run counterbalanced three-condition study with 15-20 participants | All | |
| 10.5 | Analyse quantitative and qualitative data | Raviru Rathnaweera | |
| 10.6 | Write thesis and report | All | |
| 10.7 | Prepare demo and final presentation | Pruthivi Thejan | |
| 10.8 | Package artifacts for reproducibility | Nuwan Konara | |

### 11. External coding-agent integration

| ID | Work package | Owner | Session |
| --- | --- | --- | --- |
| 11.1 | Define document-draft interface, capabilities, validation, and errors | Pruthivi Thejan | ✓ |
| 11.2 | Select/detect existing coding tools and report setup or sign-in needs | Pruthivi Thejan | ✓ |
| 11.3 | Add Codex App Server adapter for document draft jobs | Pruthivi Thejan | ✓ |
| 11.4 | Add installed Claude Code CLI adapter for document draft jobs | Pruthivi Thejan | ✓ |
| 11.5 | Add OpenCode ACP adapter for document draft jobs | Pruthivi Thejan | ✓ |
| 11.6 | Add Antigravity CLI adapter for document draft jobs | Pruthivi Thejan | ✓ |
| 11.7 | Enforce per-adapter import tool restrictions, progress, timeouts, and cancellation | Pruthivi Thejan | ✓ |
| 11.8 | Preview shared document text and handle invalid/unsupported output without data loss | Pruthivi Thejan | ✓ |
| 11.9 | Host interactive project coding sessions and tool-approval flows | Pruthivi Thejan | |
| 11.10 | Evaluate interoperability and agent-specific behavior for the study | Raviru Rathnaweera | |

## Session acceptance criteria

Confirmed scope; completion evidence belongs in the progress tracker.

- Project work is blocked until required developer onboarding is confirmed. A
  new-to-coding developer can complete it manually without a CV or connected agent.
- A developer can import/review CV text, complete missing baseline fields, resume
  incomplete onboarding, and reuse the confirmed profile across projects.
- A project can be configured from optional SRS documents and developer-held
  business knowledge. Only its name, readable folder, and one goal are mandatory.
- PDF/DOCX/Markdown/TXT extraction is local. Approved text and source references
  are retained; original documents are not copied into storage by default.
- Available agent adapters can produce reviewed document drafts. Missing tools,
  required agent-owned sign-in, unsupported capabilities, and invalid output are
  reported accurately; manual entry remains usable. The application stores no
  agent/provider credentials.
- A developer can select a repository and exclude files or directories.
- Capture includes Git state, manifests, configuration, selected documents, and
  permitted TypeScript and Python source and test files.
- The developer can inspect a capture summary before saving.
- The developer can save and reload a local snapshot.
- A manual refresh detects unchanged captures and creates no duplicate capture;
  changed captures and the context associated with older captures remain available.
- Edits, history inspection, JSON export, and confirmed deletion work without
  deleting unrelated projects or the shared profile when one project is removed.
- Cancellation and capture failures leave saved data usable.
- Configurable repository limits default to 1 MiB/file and 20 MiB/capture; imports
  default to 20 MiB/document. Skips and errors show reasons.
- Repository scanning and local text extraction execute no project scripts and
  initiate no external calls. A separate agent import shares only previewed input
  after explicit confirmation; its selected agent may use a hosted provider.
- Native macOS Apple Silicon/Windows x64 launch targets and supported terminal
  capabilities are demonstrated before being reported as working. Windows/tmux
  terminal support, OCR, graph indexing, verification, and gamification are deferred.

## Milestones

| Milestone | Work packages | Exit condition |
| --- | --- | --- |
| M1 Onboarding and local context capture | Selected tasks in 1-6 and 11 | Required onboarding, optional document drafts, project/repository capture, local persistence, history, refresh, export, and deletion work together. |
| M2 Code structure | 7 | TypeScript and Python symbols and references are indexed and queryable. |
| M3 Assumption verification | 8 and 11.9 | Interactive agent work and assumptions connect to evidence-backed verdicts. |
| M4 Learning loop | 9 | Knowledge gaps produce optional learning interventions. |
| M5 Evaluation | 10 | Study results and thesis are complete. |

## Later decisions and implementation evidence

The current interview round is complete. These items belong to later design or
implementation evidence, rather than reopening agreed onboarding/storage choices:

- Demonstrate Bun/document-parser, GPUIX platform, and individual agent adapter
  compatibility during their work packages. No implementation support is implied
  merely by documenting an upstream capability.
- Add the selected `colbymchenry/codegraph` integration later, assessing its Node
  process boundary, egress controls, and derived-graph freshness. Keep the app's
  Bun SQLite store authoritative; no PostgreSQL/pgvector is planned. See
  [the graph comparison](../graph-tool-comparison.md).
- Specify full coding-session permissions, passive collection, verification,
  learning, and evaluation before implementing those later packages.
