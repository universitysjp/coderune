> Status: accepted product rules. The implementation is the native Rust desktop in `apps/desktop` with Rune and SeaORM.

# Data collection and storage interview

Status: interview round complete; scope confirmed by the developer on 2026-10-22, including Q24–Q25. This document defines the first collection and storage increment. Implementation progress is tracked separately in [the WBS](wbs/work-breakdown-structure.md) and [its progress tracker](wbs/wbs-progress.md).

## Accepted decisions

- The first increment collects the developer profile, project context, and selected repository context.
- The first milestone allows the developer to collect, inspect, and save that context locally.
- Use the native Rust desktop in `apps/desktop` with a Rune interface.
- Target native desktop on macOS Apple Silicon and Windows x64.
- One developer profile is reused across projects. Each project has its own goals, rules, and collection permissions.
- Developer onboarding is mandatory before starting project work. Completing and confirming the profile establishes the starting information needed by later learning and gamification features; those features are not implemented in this increment.
- Completing onboarding requires a display name or pseudonym, self-reported experience stage, an initial skills statement, and at least one learning goal. New developers can explicitly state that they are new to coding. A CV and prior employment are not required.
- Connect the developer's existing coding agents, targeting Codex, Claude Code, OpenCode, and Antigravity. The selected agent handles its own model providers and authentication. Coderune owns the developer/project context and integration, rather than managing model accounts or agent sign-in.
- Implement the first increment in TypeScript with Bun and SQLite.
- Do not use PostgreSQL/pgvector. Prefer local AST and graph tooling with embedded storage for later code structure retrieval.
- Collect repository context on explicit request, with an initial capture and a manual refresh operation.
- The initial repository capture covers Git state, dependency manifests, project configuration, AGENTS.md, README, selected documentation, and permitted source and test files.
- Support TypeScript and Python repositories through file-based capture initially. AST and code graph extraction follow in a later increment.
- Developer onboarding collects CV-style details and supports both manual entry and import from a developer-selected local CV in the first increment.
- When a project is not already configured, collect its SRS if available, its business logic, and the project knowledge held by the developer.
- Reuse saved project configuration on subsequent use, with editing and refresh available.
- Explicitly selected CV and SRS documents are separate imports from automatic repository traversal. Accept text-based PDF, DOCX, Markdown, and plain text initially; scanned-document OCR follows later.
- Extract document text locally, then request structured CV/SRS drafts through a selected coding-agent integration. The developer reviews, corrects, and confirms the draft before saving. Manual onboarding remains available without a connected agent.
- Preview the collection file list before saving and let the developer narrow the scope for each project.
- Respect ignore rules and exclude environment files, private keys, dependency directories, generated output, binaries, and paths outside the selected repository.
- Store data in one per-user SQLite database in the user's application data directory, outside project repositories.
- Keep versioned context captures containing approved text, paths, hashes, timestamps, and Git state. Preserve older captures and reuse unchanged content.
- Use `crates/domain` for domain models, `crates/app` for application operations, `crates/collectors` for filesystem and Git collection, and `crates/storage` for SeaORM entities, migrations, and persistence.
- Use `crates/agents` for connections to Codex, Claude Code, OpenCode, and Antigravity. Use their documented ACP or native interfaces and expose each adapter's actual capabilities. This increment uses agents only for reviewed CV/SRS draft jobs; full interactive coding sessions follow later.
- Inferred skill assessment follows in a later increment. CV import is now included in this increment.
- Assumption verification and developer learning belong to later increments.
- The optional Eve/learning subagent is outside the implementation scope.

## Native desktop decision

The desktop is a native Rust application in `apps/desktop`. Rune draws the
interface; `rune-kit` supplies accessible controls. The application operations,
local SQLite database, onboarding gate, capture rules, and agent boundaries in
this document remain the requirements. The service crates are `crates/domain`,
`crates/storage`, `crates/collectors`, `crates/agents`, and `crates/app`.

## Graph tooling recommendation

The selected target for the future graph adapter is `colbymchenry/codegraph`, using its local SQLite graph and FTS5 search. Its current driver targets Node, so direct Bun SDK compatibility is not assumed. Keep graph integration in the later increment and review its process boundary and egress controls then. The application's SQLite store remains responsible for developer profiles, project context, and collected evidence. See [the source comparison](graph-tool-comparison.md) for the alternatives and the distinction between extracted and inferred graph relationships.

## Interview outcome

The developer confirmed mandatory onboarding and the external-agent module in Q24–Q25. The current increment is collection, review, storage, and agent-assisted document import. The existing WBS is updated to reflect this shared understanding.

Retention, collection/recovery, and data-management defaults below are accepted. Adapter compatibility and platform behavior remain implementation work to demonstrate, rather than claims of completed support. The future graph adapter's exact integration contract, full coding sessions, passive activity collection, verification, and gamification are later-increment decisions.

## Developer onboarding

The onboarding step is required before starting any project, including an existing repository. A completed reusable profile satisfies the gate for subsequent projects. Manual entry and reviewed CV import are both supported, so an agent connection is not required to complete onboarding.

### Confirmed completion criteria

- Require a display name or pseudonym, self-reported experience stage, an initial skills statement, and at least one learning goal. An explicit new-to-coding/no-skills-yet response is valid; do not require invented experience.
- Provide CV-style sections for education, employment and project experience, certifications, links, and learning preferences. These supporting fields and the CV document itself remain optional.
- Imported CV fields count only after review and confirmation. Ask for required information that is missing from the CV, including learning goals.
- Persist onboarding completion only with the confirmed profile, preserve its starting version, and require completion again if the profile is deleted. Incomplete onboarding can be resumed after reopening.

### Other accepted onboarding defaults

- Preserve missing or ambiguous imported information as unresolved rather than inventing values.
- On first opening an unconfigured project, offer an SRS import if one is available and guided entry of business logic, project purpose, constraints, decisions, and other knowledge held by the developer.
- A project without an SRS can be configured through developer-provided information.
- Require a project name, readable root folder, and at least one goal. Permit business rules, constraints, decisions, and supporting documents to be completed later.
- Keep document-derived information distinguishable from developer statements.
- Rate a self-reported skill as beginner, intermediate, or advanced. An omitted rating remains unrated.

## External coding-agent decision

- Replace the proposed direct OpenAI-compatible endpoint with connections to the developer's existing coding tools, following the external-agent approach used by Zed.
- Leave sign-in, tokens, model selection, and provider configuration with the coding agent. Coderune should report a missing installation or required sign-in and direct the developer to the selected tool's own setup flow. Do not read or copy its credential files into application records or exports.
- Show the selected agent and the document text/context to be shared before an import run. Local extraction and local storage do not imply that the selected agent performs inference locally.
- Validate agent-generated fields against the import draft schema, preserve source references, and require review before saving them as confirmed profile/project information. Schema-valid output can still contain incorrect claims.
- Do not silently change to another agent, provider, or model when a connection fails. Keep manual entry available.

The compatibility target includes Codex, Claude Code, OpenCode, and Antigravity. This is an extensible integration target; listing a tool does not yet claim an implemented or verified adapter. Broad provider coverage comes through the providers supported by each developer-selected agent.

Official [Codex App Server documentation](https://learn.chatgpt.com/docs/app-server) describes a stdio integration with thread/turn lifecycle, streamed events, cancellation, permission requests, and per-turn structured-output schemas. Codex can own its authentication lifecycle. This is a usable integration surface to assess alongside Zed's agent protocol, rather than treating a coding agent as a generic model HTTP endpoint.

### Verified integration surfaces

Checked against primary documentation on 2026-10-22. These are documented capabilities, not completed compatibility checks for this application.

| Coding tool | Documented connection | Implication for this project |
| --- | --- | --- |
| Codex | [App Server](https://learn.chatgpt.com/docs/app-server), including stdio, streamed turns, cancellation, approvals, and per-turn output schemas. | Can support a native adapter while Codex retains its own configuration and sign-in. |
| Claude Code | Installed CLI [print/headless mode](https://code.claude.com/docs/en/headless), with JSON/streaming output and `--json-schema`. | A native CLI adapter can request import drafts using the user's tool configuration. Do not assume an SDK-based bridge has the same authentication behavior. |
| OpenCode | Native [`opencode acp`](https://docs.opencode.ai/docs/acp/) over stdio. | An ACP client can connect to OpenCode while its rules, providers, MCP configuration, and permissions remain with the agent. |
| Antigravity | Official [`agy` headless CLI](https://antigravity.google/docs/cli/headless/), with schema output, JSON/NDJSON, and multi-turn streams. | Use its documented CLI surface; no official ACP mode was located. A CLI connection does not establish attachment to an already-running IDE conversation. |

[Zed's external-agent boundary](https://zed.dev/docs/ai/external-agents#configuration-boundaries) is the integration reference. ACP currently documents [v1 as the stable protocol](https://agentclientprotocol.com/protocol/v1/initialization) and provides a [TypeScript SDK](https://agentclientprotocol.com/libraries/typescript). Rich interactive hosting needs streamed session updates, cancellation, and visible permission requests. ACP itself does not sandbox an agent or guarantee schema-enforced output; capabilities and execution restrictions need checking per adapter.

Claude's native CLI and SDK paths must remain distinct. Its [CLI reference](https://code.claude.com/docs/en/cli-reference) documents that `--bare` excludes OAuth/keychain/subscription login; ordinary CLI configuration is the relevant candidate for a user-owned installation. The [Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview#get-started) separately constrains third-party sign-in offerings. Coderune will not offer its own agent login or assume a third-party bridge inherits Zed's authentication arrangements.

### Confirmed integration structure and increment boundary

- Add `crates/agents` as the boundary for talking to external coding tools. Keep agent-specific commands, wire messages, capability checks, timeouts, cancellation, and errors inside it. The core requests a document draft and receives a validated draft or an actionable error.
- Use ACP for compatible agent sessions and native adapters where a tool has a different documented protocol. Avoid implementing a new agent loop or provider authentication system.
- Keep onboarding, document extraction, repository collection, and SQLite persistence usable without an agent. A connected agent is needed only when the developer requests agent-assisted interpretation.
- Limit this increment's agent work to explicit CV/SRS draft jobs: supplied text, progress/cancel, schema validation, and human review. Interactive project coding sessions follow later, along with gamification and verification.
- Import jobs must not receive permission to modify project files merely because a coding tool supports it. Verify each adapter's tool restrictions and working-directory isolation. Restricting client callbacks alone is insufficient for an agent with its own tools. Unsupported capabilities must be visible, with manual entry available.

## Implementation structure

These are planned responsibilities, not a statement that the packages are implemented.

| Location | Responsibility |
| --- | --- |
| `apps/desktop` | Rune application: onboarding, workspace screens, forms, navigation, and application composition. |
| `crates/domain` | Profile/project models, onboarding rules, import draft schemas, capture types, and errors. |
| `crates/app` | Application operations, capture orchestration, document review, export, and OS integration. |
| `crates/collectors` | Local repository/Git capture, selected-document extraction, scope enforcement, limits, hashes, and source metadata. |
| `crates/storage` | SeaORM entities and migrations, transactional persistence, versions, history, retention, and export reads. |
| `crates/agents` | Existing-tool detection, protocol adapters, capabilities, draft jobs, progress/cancellation, and actionable errors. |
| `apps/web`, `packages/ui`, `packages/tokens` | React site, shared web registry, and font assets. |

The core defines the interfaces used by its operations. Collectors, storage, and agents implement them and can depend on core types; the core does not import those implementations. The desktop app composes the implementations. Agent wire formats and database rows do not become UI form contracts.

The TypeScript/Bun implementation covers this increment without a Python backend. Supporting Python repositories means reading their permitted files; it does not require executing Python or installing an embedding service.

## Confirmed operating defaults

### Imported document retention

- Retain confirmed structured fields, the developer-approved extracted text, and source metadata such as document name, path, hash, and import time. Keep page or section references when the extractor provides them.
- Do not copy the original PDF/DOCX into application storage by default. Retained approved text keeps the source evidence available if the original moves or is removed.
- Use a configurable 20 MiB limit per imported document. Explain unsupported, unreadable, oversized, or image-only documents and offer manual entry.
- If the agent cannot accept the selected input size, request a smaller selection rather than silently dropping text.

### Capture and recovery

- Guide the developer through profile, project, scope, scan, preview, and save. Allow back navigation and cancellation.
- Save forms only when explicitly confirmed; save each context capture in one database transaction.
- Retain previously saved records when a capture is cancelled or fails. Show errors with a retry action.
- Capture text with configurable defaults of 1 MiB per file and 20 MiB per capture. List skipped files and their reasons.
- Record Git metadata when available. A readable project folder can still be captured without Git metadata.
- Never execute project scripts or contact external services during repository scanning or local document text extraction. Agent-assisted interpretation is a separate explicitly requested operation with previewed input.
- Bind saving to the reviewed capture. If source content changes during review, require a fresh preview before saving.
- The separate imported-document size limit is defined above; repository capture limits do not silently apply to a selected CV/SRS document.

### Data management and acceptance

- Support inspection and editing of developer/project records, capture history, JSON export, and confirmed deletion of saved project/capture data.
- Deleting a project removes its saved context and captures, while preserving the reusable developer profile and other projects. Provide a separate confirmed action for deleting all application data.
- Profile/project edits affect future captures; preserve the context associated with earlier captures.
- Report an unchanged refresh without duplicating an identical capture; retain changed captures as new versions.
- The increment is complete when onboarding, capture, save, reopening, inspection, changed/unchanged refresh, export, and deletion work together for TypeScript and Python projects.
- Project work remains gated while onboarding is incomplete, including after reopening or deleting the profile. An explicit new-to-coding baseline can complete onboarding without a CV or agent connection.
- Selected PDF/DOCX/Markdown/TXT imports produce reviewable text and structured drafts where the connected adapter supports them. Missing sign-in, unavailable tools, invalid output, cancellation, and import errors preserve saved data and permit manual entry.
- Native launch targets and supported terminal launch modes are reported accurately; an unverified or unsupported adapter/platform capability is not presented as working.

### WBS ownership when the round closes

Preserve the existing draft's task coverage and all three team members. Apply the developer's explicit instruction to this session's implementation tasks: Pruthivi owns UI/collection/integration and Nuwan owns the database layer. Keep Raviru's research, evaluation, and later verification work visible. Broader assignments outside this session need not change solely to match the session's implementation ownership.

## Work breakdown structure instructions

Use the developer's starting [WBS](wbs/work-breakdown-structure.md) and edit it as the design develops after this interview round. Keep the WBS in `docs/wbs` and render its diagram with Pele. Preserve existing work while updating the agreed scope. Use these three team members, as named on the final slide of [the proposal](presentation.pdf):

- Pruthivi Thejan
- Nuwan Konara
- Raviru Rathnaweera

Mark tasks selected for this session with a tick and track completion separately. Assign this session's implementation tasks to Pruthivi Thejan, with the database layer tasks assigned to Nuwan Konara. Include all three team members in the project WBS.
