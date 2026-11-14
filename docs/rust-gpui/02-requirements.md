# 02 — Functional requirements and domain model

Status: implementation requirements for the native Rust/GPUI rewrite.

Scope: the current Coderune increment (M1, “Onboarding and local context capture”), plus explicitly labelled later domain commitments. This document describes behaviour, data, and rules. It intentionally does not prescribe layout, visual style, or component structure; those belong to [`01-design-system.md`](./01-design-system.md).

Sources of truth:

1. `GLOSSARY.md` for domain names.
2. `docs/data-collection-design.md`, `docs/desktop-workflow-decisions.md`, `docs/skills-verification-design.md`, and `docs/adr/*.md` for product decisions.
3. `docs/wbs/*` for increment scope and acceptance.
4. `packages/core`, `packages/collectors`, `packages/storage`, and `packages/agents` for the implemented operation, collection, persistence, and adapter contracts.
5. `apps/desktop/src` and `apps/desktop/test` for the operations exposed by the current desktop app and the behaviour its tests protect.

Where documents and code disagree, the documents define intended product behaviour and the code/tests define what the current increment actually does. Both are called out below where the distinction matters.

---

## 1. Product scope and non-goals

### 1.1 Product statement

Coderune is a desktop application that builds and maintains local, reviewable context about a developer and their projects, so that coding agents can receive the context behind the code without Coderune taking ownership of the agent’s models, providers, sign-in, or credentials.

Coderune is desktop-only. A terminal command opens a directory in Coderune, comparable to `code .`. The developer can launch the current project directory in a preferred installed IDE from the application.

### 1.2 Current increment: M1 “Onboarding and local context capture”

The increment is complete when onboarding, project configuration, document import, repository capture, local persistence, history, refresh, export, deletion, and agent-assisted CV/SRS drafts work together for TypeScript and Python projects.

In scope:

- Mandatory **Developer onboarding** before any project work.
- A reusable confirmed **Developer profile** and **Developer baseline**, with the initial version preserved.
- Resumable incomplete onboarding.
- Manual profile entry or reviewed local CV import.
- Optional local **SRS** import for **Project context**.
- Developer-authored goals, purpose, business logic, rules, constraints, decisions, acceptance criteria, and knowledge.
- Explicit-request **Repository context** collection with an initial capture and a manual refresh.
- Git state, dependency manifests, project configuration, `AGENTS.md`, `README`, selected documentation, and permitted TypeScript/Python source and test files.
- Local SQLite persistence outside project repositories, with versions, history, approved text, hashes, timestamps, and source references.
- Inspection, editing, reopening, refresh, JSON export, and confirmed deletion.
- Connections to installed Codex, Claude Code, OpenCode, and Antigravity for user-requested CV/SRS draft jobs.
- Previewing shared text, reviewing/correcting drafts, schema validation, and manual entry when an agent is unavailable.
- Native macOS Apple Silicon and Windows x64 desktop targets; supported terminal launch modes where the runtime provides them.

### 1.3 Non-goals for the current increment

- AST or code-graph extraction.
- Assumption verification and the verdict engine.
- Developer learning, gamification, and learning evidence.
- Inferred skill assessment. A **Self-reported skill** is not verified mastery.
- Interactive project coding sessions and tool-approval flows.
- Passive activity/context collection.
- OCR for scanned documents.
- Application-managed agent accounts, credentials, or model-provider sign-in.
- Browser/WASM hosting and a separate non-interactive application CLI.
- Remote or optional external learning services.
- The optional Eve/isolated learning subagent.
- An in-app agent, MCP interface, architecture Q&A skill, or user-customisable skill standards. These are recorded product direction only.

### 1.4 Later increments and product direction

The following are accepted design commitments but are not implemented in M1. They must not be presented as working:

- **Skill library**, **Skill collection**, **Agent skill** management: full pinned Matt and pstack libraries plus Coderune-authored skills, application-owned storage, source folders, enable/disable, add-your-own, on-demand loading, update/rollback, and source precedence.
- **Agent connection**, **Defined subagent**, **Subagent run**, and **Model assignment**: stored connections and model inventories, six initial defined subagents, inherited model defaults, unavailable-model recovery, and linked retry attempts.
- **Verification brief**, **Canonical check**, **Verification result**, **Verification finding**, **Verification verdict**, and result freshness.
- AST/code graph integration and graph inspection.
- Passive activity collection, OCR, interactive coding sessions, and the optional learning subagent.

Product direction from the 11 November 2026 workflow record — an agent inside the application, skill standards/customisation, an MCP interface, and agent-led architecture Q&A — remains future work and is outside the current increment.

### 1.5 Platform and launch behaviour

- Native desktop is the primary target. macOS Apple Silicon and Windows x64 are the claimed native targets once demonstrated.
- Terminal mode uses the same application operations and persistence boundary. Windows and tmux terminal combinations must be reported as unsupported rather than shown as working.
- The application stores data per user, outside project repositories, in one SQLite database.
- Starting with a directory argument must select an existing project whose root matches that directory; otherwise the directory becomes the initial directory for a new project.

---

## 2. Glossary-aligned domain model

### 2.1 Glossary binding

Use these names exactly in Rust types, messages, and documentation.

| Glossary term | Current code / storage | Meaning in this rewrite |
| --- | --- | --- |
| **Developer profile** | `DeveloperProfile`, `developer_profiles` | The developer’s professional background, education, work/project experience, skills, and learning preferences, shared across all projects. One profile per database. |
| **Developer onboarding** | `OnboardingDraft`, `OnboardingState`, `onboarding_draft` | Collection, review, and confirmation of background, self-reported skills, and learning goals. Required before project work. |
| **Developer baseline** | Confirmed `DeveloperProfile` starting at version 1 | The confirmed starting description used as a reference for later progress. It is not verified mastery or a current assessment. |
| **Coding agent** | `CodingAgentId`, `AGENT_DEFINITIONS` | An external assistant selected by the developer, with its own configured models, tools, and authentication. |
| **Agent connection** | Not persisted in M1; per-job `CodingAgentId` selection only | A developer-configured connection to a coding agent and its models. Later increment. |
| **Defined subagent** | Later | Reusable specialist with a purpose and a configured default model assignment. |
| **Subagent run** | Later | One execution of a task assigned to a defined subagent, inheriting its configured default model. |
| **Model assignment** | Later | The agent connection and model configured for a defined subagent. |
| **Agent skill** | Later | Reusable instructions and resources that guide a coding agent. |
| **Skill library** | Later | All skills available through Coderune. |
| **Skill collection** | Later | A source-grouped set of agent skills. |
| **Project context** | `Project` statements | Requirements, business logic, goals, constraints, acceptance criteria, and developer-held knowledge for one project. |
| **Repository context** | `RepositoryCollection`, `ContextCapture` | Information obtained from a selected repository describing its contents and state. |
| **Self-reported skill** | `SelfReportedSkill` | The developer’s own proficiency statement; never verified mastery. |
| **Collection scope** | `CollectionScope` | The repository paths and information types the developer permits for a project. |
| **Context capture** | `ContextCapture` (reviewed through `CapturePreview`) | A saved, versioned record of repository context observed during a collection, with provenance. |
| **Verification brief** | Later | Agreed scope, requirements, candidate claims, and checks for one exact code snapshot and diff. |
| **Canonical check** | Later | A project-configured check whose observed execution is shared evidence. |
| **Verification result** | Later | Recorded findings, verdicts, observed evidence, check outcomes, gaps, and sources. |
| **Verification finding** | Later | One verification subagent’s observation about a change, with evidence and source. |
| **Verification verdict** | Later | Coderune’s assessment of a claim or change from evidence and agreed checks. |
| **Verified** | Later claim verdict | Evidence supports the claim and required checks passed. |
| **Contradicted** | Later claim verdict | Observed evidence disproves the claim. |
| **Insufficient evidence** | Later claim verdict | Required evidence or check results are missing. |
| **Needs review** | Later claim/overall verdict | A disagreement the available evidence and checks did not resolve. |
| **Stale** | Later freshness flag | The result applies to earlier reviewed code or requirements, not the current state. |
| **Learning evidence** | Later | An evaluated developer attempt linked to the relevant code snapshot. |

### 2.2 Core entities and fields

All timestamps are ISO-8601 strings. IDs are UUIDs except the singleton profile (`"developer"`), the singleton onboarding draft (`"onboarding"`), and integer migration/version row IDs. JSON field names are camelCase.

#### Developer profile and onboarding

**Developer profile** (`DeveloperProfile`):

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | Always `"developer"`. |
| `version` | integer | Starts at 1; increments on every confirmed create/edit. |
| `displayName` | string | Trimmed; required. A pseudonym is valid. |
| `experienceStage` | enum | `new-to-coding`, `early-career`, `mid-career`, `experienced`. Required. |
| `newToCoding` | boolean | Explicit new-to-coding flag. |
| `skillsStatement` | string | Trimmed; required unless `newToCoding` is true. When `newToCoding` is true and blank, normalized to `"New to coding; no skills listed yet."` |
| `learningGoals` | string[] | At least one non-blank goal. Trimmed; blanks dropped. |
| `selfReportedSkills` | `{ topic: string, rating: "beginner" \| "intermediate" \| "advanced" \| null }[]` | Topic trimmed; blank topics dropped. An omitted rating stays explicitly `null` (“unrated”), never guessed. |
| `education` | `{ institution, qualification?, period?, details? }[]` | Optional. |
| `employment` | `{ employer, role, period?, details? }[]` | Optional. |
| `projects` | `{ name, role?, period?, details? }[]` | Optional project experience. |
| `certifications` | `{ name, issuer?, year? }[]` | Optional. |
| `links` | `{ label, url }[]` | Optional. |
| `learningPreferences` | `{ formats?: string[], notes?: string } \| null` | Optional. |
| `fieldSources` | map of top-level field name to `FieldSource` | Provenance retained from reviewed document drafts. |
| `createdAt`, `updatedAt` | ISO-8601 string | `createdAt` is preserved across edits. |

`FieldSource` is either `{ kind: "developer" }` or `{ kind: "document", documentId, documentName, reference? }`.

**Onboarding draft** (`OnboardingDraft`): a partial profile field set plus `updatedAt`; no version, no ID. Stored in the singleton `onboarding_draft` row. It is replaced on each save and cleared on confirmation.

**Onboarding state** (`OnboardingState`):

| Field | Type | Meaning |
| --- | --- | --- |
| `requiredComplete` | boolean | True iff a profile row exists. |
| `canStartProject` | boolean | Same as `requiredComplete`. |
| `missing` | string[] | Required-field labels missing from the current draft/profile, or empty when complete. |
| `profile` | `DeveloperProfile \| null` | The confirmed profile, if any. |
| `draft` | `OnboardingDraft \| null` | The resumable draft, if any. |

Required-field labels are exactly: `display name`, `experience stage`, `initial skills statement`, `learning goal`.

#### Project context

**Project** (`Project`):

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | UUID. |
| `version` | integer | Starts at 1; increments on each update. |
| `name` | string | Trimmed; required. |
| `rootPath` | string | Required; must resolve to a readable directory. Stored as entered/resolved by the caller. |
| `goals` | `ProjectStatement[]` | At least one non-blank goal. |
| `purpose` | `ProjectStatement \| null` | Optional. |
| `businessLogic` | `ProjectStatement \| null` | Optional. |
| `rules` | `ProjectStatement[]` | Optional. |
| `constraints` | `ProjectStatement[]` | Optional. |
| `decisions` | `ProjectStatement[]` | Optional. |
| `acceptanceCriteria` | `ProjectStatement[]` | Optional. |
| `knowledge` | `ProjectStatement[]` | Optional developer-held knowledge. |
| `scope` | `CollectionScope` | Default: no includes, no excludes, respect `.gitignore`. |
| `limits` | `CaptureLimits` | Default: 1 MiB per file, 20 MiB per capture. |
| `createdAt`, `updatedAt` | ISO-8601 string | `createdAt` preserved across versions. |

**ProjectStatement**:

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | UUID for each distinct statement. |
| `text` | string | Trimmed; blank statements are dropped. |
| `provenance` | `{ kind: "developer" } \| { kind: "document", documentId, documentName, reference? }` | Developer-authored by default. Document provenance is retained for reviewed SRS fields. |

**CollectionScope**:

| Field | Type | Rules |
| --- | --- | --- |
| `includePaths` | string[] | Empty means the whole repository. Trimmed, de-duplicated, relative to the root; paths outside the root are rejected/skipped. |
| `excludePaths` | string[] | Trimmed, de-duplicated. Supports path prefixes, directory patterns, bare names, and glob patterns. |
| `respectGitignore` | boolean | Default `true`. |

**CaptureLimits**:

| Field | Type | Rules |
| --- | --- | --- |
| `maxFileBytes` | integer | Positive, finite. Default `1_048_576` (1 MiB). |
| `maxCaptureBytes` | integer | Positive, finite. Default `20_971_520` (20 MiB). |

#### Imported documents

**ExtractedDocument**:

| Field | Type | Rules |
| --- | --- | --- |
| `name` | string | Basename of the selected file. |
| `path` | string | Absolute resolved path. |
| `extension` | `.pdf`, `.docx`, `.md`, `.markdown`, or `.txt` | Lower-cased. |
| `kind` | `"cv"` or `"srs"` | CV belongs to the developer; SRS belongs to a project when linked. |
| `hash` | string | SHA-256 of the file bytes, lower-case hex. |
| `size` | integer | Byte length. |
| `text` | string | Locally extracted text. |
| `extractedAt` | ISO-8601 string | Extraction time. |
| `pageCount` | integer \| null | PDF page count where available; otherwise `null`. |

**ImportedDocument** = `ExtractedDocument` plus:

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | UUID. |
| `projectId` | string \| null | `null` for CVs; the project ID for linked SRS imports. |
| `importedAt` | ISO-8601 string | Confirmation time. |

The original PDF/DOCX is not copied into application storage. The approved extracted text and source metadata are retained; the original path remains a reference only.

#### Repository context and captures

**CollectedFile**:

| Field | Type | Rules |
| --- | --- | --- |
| `path` | string | Repository-relative path using `/` separators. |
| `size` | integer | Bytes. |
| `hash` | string | SHA-256 of file bytes. |
| `category` | enum | `manifest`, `configuration`, `agent-instructions`, `readme`, `documentation`, `source`, `test`, `other`. |
| `content` | string | UTF-8 decoded text (invalid sequences replaced, not fatal). |

**SkippedFile**: `{ path, reason }`, where `reason` is one of `outside-scope`, `excluded`, `ignored`, `environment-file`, `private-key`, `dependency-directory`, `generated-output`, `binary`, `unsupported-type`, `file-too-large`, `capture-limit`, `escapes-repository`, `not-readable`, `symbolic-link`.

**GitState**:

| Field | Type | Rules |
| --- | --- | --- |
| `head` | string \| null | `git rev-parse HEAD`, or `null`. |
| `branch` | string \| null | Abbreviated branch, or `null` for detached/unavailable. |
| `dirty` | boolean | True when `git status --porcelain=v1` has any lines. |
| `status` | string[] | Porcelain status lines. |
| `capturedAt` | ISO-8601 string | Observation time. |

**RepositoryCollection**:

| Field | Type | Rules |
| --- | --- | --- |
| `files` | `CollectedFile[]` | Sorted by path. |
| `skipped` | `SkippedFile[]` | Sorted by path. |
| `git` | `GitState \| null` | `null` for a non-Git readable folder. |
| `totalBytes` | integer | Sum of included file sizes. |
| `fingerprint` | string | Repository fingerprint; see §6.5. |

**CapturePreview** (transient, in memory only):

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | UUID for this preview. |
| `projectId` | string | Project being previewed. |
| `projectVersion` | integer | Project version at preview time. |
| `profileVersion` | integer | Developer profile version at preview time. |
| `files` | `{ path, size, hash, category }[]` | Metadata only; content is not in the preview object. |
| `skipped` | `SkippedFile[]` | Includes reasons. |
| `git` | `GitState \| null` | Git state at preview. |
| `totalBytes` | integer | Included bytes. |
| `scope` | `CollectionScope` | Effective scope used for this preview, including overrides. |
| `limits` | `CaptureLimits` | Effective limits. |
| `fingerprint` | string | Full capture fingerprint; see §6.5. |
| `createdAt` | ISO-8601 string | Preview time. |

The in-memory preview record also holds the collected contents, the profile and project snapshots used, and any preview-only extra exclusions. These are not serialized into the preview object.

**ContextCapture**:

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | UUID. |
| `projectId` | string | Owning project. |
| `version` | integer | Per-project capture version, starting at 1. |
| `fingerprint` | string | Full capture fingerprint recorded at save. |
| `createdAt` | ISO-8601 string | Save time. |
| `files` | `CollectedFile[]` | Approved content, including text. |
| `skipped` | `SkippedFile[]` | Skip reasons. |
| `git` | `GitState \| null` | Recorded Git state. |
| `totalBytes` | integer | Included bytes. |
| `profileSnapshot` | `DeveloperProfile` | Exact profile used for this capture. |
| `projectSnapshot` | `Project` | Exact project used for this capture. |

**CaptureRefreshResult**: `{ status: "unchanged" | "changed", preview: CapturePreview, capture: ContextCapture | null }`. `unchanged` returns the existing latest capture; `changed` returns a fresh preview and `capture: null`.

#### Coding-agent draft types

**CodingAgentStatus**:

| Field | Type | Rules |
| --- | --- | --- |
| `id` | `codex`, `claude-code`, `opencode`, `antigravity` | Stable agent identifier. |
| `name` | string | `Codex`, `Claude Code`, `OpenCode`, `Antigravity`. |
| `availability` | `ready`, `not-installed`, `unavailable` | Current detection result. The local gateway only emits `ready` or `not-installed`; tests/stubs may produce `unavailable`. |
| `detail` | string | Actionable installation/sign-in wording. |
| `setupUrl` | string | The agent’s own setup documentation. |
| `capabilities` | `CodingAgentCapabilities` | See below. |
| `verified` | boolean | Always `false` until platform/agent compatibility is demonstrated. |

**CodingAgentCapabilities**:

| Field | Type | Rules |
| --- | --- | --- |
| `structuredOutput` | boolean | Whether the adapter requests native structured output. |
| `cancellation` | boolean | Whether a running draft can be cancelled. |
| `streaming` | boolean | Whether progress can stream. |
| `documentOnlyIsolation` | `enforced`, `unverified`, `unsupported` | Current definitions all say `unverified`. |
| `maxInputBytes` | integer \| null | Maximum accepted request text size, or no reported limit. |
| `restrictions` | string[] | Human-readable limits and isolation statements. |

**DraftSharingPreview**:

| Field | Type | Rules |
| --- | --- | --- |
| `agent` | `CodingAgentStatus` | Selected agent. |
| `kind` | `"cv"` or `"srs"` | Draft kind. |
| `document` | `ImportedDocument` | Source document. |
| `project` | `Project \| null` | Present for SRS; `null` for CV. |
| `text` | string | Exact text that will be sent. |
| `estimatedBytes` | integer | UTF-8 byte length of `text`. |
| `maxInputBytes` | integer \| null | Agent limit copied from capabilities. |
| `fits` | boolean | `maxInputBytes === null || estimatedBytes <= maxInputBytes`. |
| `createdAt` | ISO-8601 string | Preview time. |

**AgentDraftResult**: `{ agent: CodingAgentStatus, draft: CvDraft | SrsDraft }`.

**CvDraft** (strict schema; extra keys are invalid):

- `kind: "cv"` (required literal).
- Optional: `displayName`, `experienceStage`, `newToCoding`, `skillsStatement`, `learningGoals[]`, `selfReportedSkills[]`, `education[]`, `employment[]`, `projects[]`, `certifications[]`, `links[]`, `learningPreferences`.
- `sources: { field: string, reference?: string }[]` (defaults to empty).
- `warnings: string[]` (defaults to empty).

**SrsDraft** (strict schema; extra keys are invalid):

- `kind: "srs"` (required literal).
- Optional: `purpose`, `businessLogic`, `goals[]`, `rules[]`, `constraints[]`, `decisions[]`, `acceptanceCriteria[]`, `knowledge[]`.
- `sources: { field: string, reference?: string }[]` (defaults to empty).
- `warnings: string[]` (defaults to empty).

#### Export bundle

`ExportBundle`:

| Field | Type |
| --- | --- |
| `exportedAt` | ISO-8601 string |
| `application` | literal `"Coderune"` |
| `schemaVersion` | `1` |
| `profile` | `DeveloperProfile \| null` |
| `profileVersions` | `DeveloperProfile[]` |
| `projects` | `Project[]` |
| `projectVersions` | `Project[]` (all versions of all projects) |
| `documents` | `ImportedDocument[]` |
| `captures` | `ContextCapture[]` (all projects; includes file content) |

There is no application-data JSON import operation in the current increment.

### 2.3 Invariants

1. One **Developer profile** per database, with fixed ID `"developer"`.
2. One **Onboarding draft** per database, with fixed ID `"onboarding"`.
3. A profile exists if and only if onboarding is complete and project work is allowed. `requiredComplete` and `canStartProject` are both true exactly when a profile row exists.
4. Profile versions are monotonically increasing and immutable. `createdAt` is preserved; `updatedAt` changes. Every save writes a new `developer_profile_versions` row.
5. The initial confirmed profile version is the **Developer baseline**. Later edits do not rewrite the baseline or any capture snapshot.
6. Confirming onboarding requires all four required fields and clears the onboarding draft.
7. A `new-to-coding` developer completes onboarding without inventing skills; the normalized `skillsStatement` is the fixed new-to-coding statement.
8. An unrated **Self-reported skill** remains `null`, never inferred.
9. Every project has a name, a readable root directory, and at least one goal.
10. Project versions are monotonically increasing and immutable in history. `createdAt` is preserved; `updatedAt` changes. Every create/update writes a `project_versions` row.
11. Project statement IDs and provenance are preserved when the statement text is unchanged; a changed statement becomes a new statement with `{ kind: "developer" }` provenance.
12. Capture limits must be positive, finite byte counts.
13. The effective scope for a capture is the project scope merged with any per-preview overrides. Per-preview overrides do not update the saved project.
14. An **ImportedDocument** is created only after explicit review confirmation. Its kind must match the requested draft kind, and an SRS linked to a project must belong to that project.
15. Repository files are included only if they are within the selected root, not symlinks, not protected, not excluded, supported, readable, non-binary, and within both size limits.
16. The same repository path is recorded at most once per collection. Files and skipped paths are sorted deterministically.
17. File content is stored once per SHA-256 hash in `contents`; captures reference content by hash.
18. A capture is saved only from a preview that still matches the current repository/profile/project fingerprint and after explicit confirmation.
19. An unchanged refresh creates no duplicate capture. A changed refresh creates a new preview; saving it creates the next capture version.
20. Older captures and their profile/project snapshots remain unchanged when the profile or project is edited.
21. Deleting a project removes its captures, capture rows, project versions, and project-linked documents, but preserves the profile and all other projects.
22. Deleting all application data removes profile, draft, projects, versions, documents, captures, captured file rows, skipped rows, and content rows. It does not delete the SQLite file or schema migrations.
23. Agent drafts are never applied without review confirmation. Schema-valid output is not treated as fact.
24. A failed, cancelled, timed-out, unavailable, oversized, or invalid agent run changes no saved profile/project data.
25. No agent fallback occurs. Coderune never silently switches agent, provider, or model.
26. No adapter is marked verified merely because its upstream tool documents a capability.

### 2.4 State transitions

#### Developer onboarding and profile

| From | Event | To | Guards / side effects |
| --- | --- | --- | --- |
| No profile, no draft | App opens | Onboarding required | `missing` contains all required labels. |
| No profile | Save draft | Draft saved | Partial input accepted; `updatedAt` set; no validation of completeness. |
| Draft saved | Reopen | Draft resumable | Draft loaded from the singleton row. |
| Draft / no draft | Confirm with missing fields | Rejected | `ValidationError` lists missing labels; no profile written. |
| Draft / no draft | Confirm valid | Profile v1 confirmed | Draft cleared; profile version 1; baseline established; project work allowed. |
| Profile vN | Update valid | Profile vN+1 | `requireDeveloperBaseline`; validate; normalize; write new version; old versions retained. |
| Profile vN | Edit with missing required field | Rejected | Profile unchanged. |
| Profile vN | Delete all data | No profile | Onboarding required again; all other data removed. Current app exposes no profile-only delete. |

#### Project

| From | Event | To | Guards / side effects |
| --- | --- | --- | --- |
| No profile | Create/update/list/delete project | Rejected | `onboarding-required` before validation. |
| No project | Create valid | Project v1 | Name/root/goal validation; readable root; statements normalized; limits/scope defaulted. |
| Project vN | Update valid | Project vN+1 | Same validation; omitted optional statement fields become `null`/empty; omitted `scope`/`limits` fall back to defaults (the desktop editor always supplies current values); changed text gets developer provenance; unchanged text preserves ID/provenance. |
| Project vN | Update invalid | Unchanged | `validation` or `unreadable-path`. |
| Project exists | Delete, confirmed in UI | Gone | Cascades captures, project versions, and project-linked documents; profile and other projects remain. |
| Project exists, profile missing in DB | Read via `getProject` | Still readable | `getProject` has no onboarding gate; the UI must still block project work. |

#### Document import

| From | Event | To | Guards / side effects |
| --- | --- | --- | --- |
| No document selected | Extract path | Extracted candidate or error | Local only; supported extension; readable; within per-document limit; non-empty text. |
| Extraction error | Show error | No saved document | Error codes: `unsupported-document`, `unreadable-document`, `image-only-document`, `document-too-large`; `manualEntryAvailable: true`. |
| Extracted candidate | Confirm reviewed text | ImportedDocument | Requires `confirmed: true`; optional project must exist. Original file is not copied. |
| ImportedDocument | Select for draft | Draft preview | Kind must match CV/SRS. |
| ImportedDocument | No edit operation | Immutable | A correction requires a new import. |
| Project deleted | Project-linked document | Removed | Documents for other projects/null project remain. |

#### Context capture

| From | Event | To | Guards / side effects |
| --- | --- | --- | --- |
| No preview | Preview capture | Preview available | Requires profile + project; collect repository; store preview in memory; no DB write. |
| Preview available | Confirm + save | Capture v1, or existing latest | Requires `confirmed`; re-collect and compare fingerprint; stale/expired rejected; unchanged latest returned instead of duplicate. |
| Capture vN exists | Refresh, fingerprint same | Unchanged | Return latest capture; no new row. |
| Capture vN exists | Refresh, fingerprint differs | Changed preview | Fresh preview; no save yet. |
| Changed preview | Confirm + save | Capture vN+1 | Version = latest version + 1; new immutable snapshot. |
| Preview | Source/profile/project changed | Preview stale | Save rejected with `capture-stale`; fresh preview required. |
| Preview | App restarted | Preview expired | Save rejected with `preview-expired`; fresh preview required. |
| Capture vN | Delete, confirmed in UI | Gone | Content orphans cleaned; history renumbers only by existing versions, not re-sequenced. |
| Profile/project edited | Existing captures | Unchanged | Future previews/captures use the new versions. |

#### Agent connection and draft job

Current increment: no stored **Agent connection**. A job carries an agent ID, and status is detected at request time.

| From | Event | To | Guards / side effects |
| --- | --- | --- | --- |
| Agent not installed | Detect on PATH | `not-installed` | Cannot be selected for a job. |
| Agent installed | Detect on PATH | `ready` | Can be selected; `verified` remains false. |
| `ready` | Prepare preview | **DraftSharingPreview** | No agent call; exact request text, byte count, limit, fit. |
| Preview, does not fit | Request | Rejected | `agent-input-too-large`; no call; narrow the input. |
| Preview, fits, unconfirmed | Request | Rejected | `confirmation-required`; no call. |
| Preview, fits, confirmed | Request | Running | Progress messages; cancellable. |
| Running | Valid output | Schema-valid draft | Kind must match; no save yet. |
| Running | Cancel/timeout | Error | `agent-cancelled` / `agent-timeout`; saved data unchanged. |
| Running | Unavailable/auth failure | Error | `agent-unavailable`; no fallback. |
| Running | Invalid output | Error | `agent-invalid-output`; no save. |
| Schema-valid draft | Review + confirm apply | Profile/project version | `confirmation-required` otherwise; field sources retained; validation still applies. |
| Any failure | Agent not switched | Error | Manual entry remains available. |

#### Agent skills, subagents, and verification (later, not implemented)

These transitions are design commitments only. They must not be built into M1.

- **Agent skill**: installed → enabled/disabled → update available → updated (previous revision retained for rollback). Enabled skills are discoverable by name/description and loaded on demand; disabled skills are excluded from a job’s discovery catalog. Source precedence is Coderune built-ins → imported packs → project skills → user skills.
- **Skill library / Skill collection**: full pinned Matt and pstack libraries plus Coderune-authored skills, stored in application-owned folders grouped by source (`coderune/`, `mattpocock/`, `pstack/`), never written into the project repository. No skill fetching during an agent job.
- **Defined subagent / Model assignment / Subagent run**: the developer selects the main coding model; each defined subagent stores a default agent connection and model. A new task creates a run inheriting that default. Active runs retain connection/model, skill contents/versions, code snapshot, and brief. A retry creates a linked attempt using the current saved assignment and preserves earlier evidence.
- **Unavailable model recovery**: show **Unavailable** with **Retry** and **Select another model**; replacement is an explicit developer choice. Completed findings are preserved.
- **Verification brief**: exact snapshot/diff, permitted scope, requirements, acceptance criteria, candidate claims, required checks.
- **Canonical check**: Coderune runs the project’s configured commands against the reviewed snapshot and records command, exit status, and output as shared evidence. Agents may suggest extra checks; **Run once** contributes to the current review, **Save to project checks** adds them for future reviews.
- **Verification result**: extracted assumptions, per-claim verdicts, code references/evidence, check results (including checks that did not run), unresolved conflicts/missing evidence, and the subagent/model/skill versions used.
- **Claim verdicts**: `Verified`, `Contradicted`, `Insufficient evidence`, `Needs review`.
- **Overall verdict precedence**: any in-scope claim contradicted → `Contradicted`; otherwise unresolved conflict → `Needs review`; otherwise incomplete evidence / required checks not passed / no claims assessed → `Insufficient evidence`; otherwise every in-scope claim verified and required checks passed → `Verified`.
- **Stale**: if reviewed code or requirements change, preserve the historical verdict and mark it `stale` for the current state. Freshness is separate from the claim verdict.

### 2.5 Provenance rules

- Developer-entered profile and project fields are `{ kind: "developer" }` unless a reviewed document supplied the field.
- SRS-derived project statements use `{ kind: "document", documentId, documentName, reference? }`.
- CV-derived profile fields record `fieldSources[topLevelField]` from the draft’s `sources[].field` base name (text before the first `.`). Multiple array entries collapse to the same top-level key; the last reference wins in current code.
- Project editing preserves existing provenance when the statement text is unchanged. Changing the text creates a new developer-provenance statement.
- Current profile editing preserves the existing `fieldSources` map; it does not automatically convert manually edited fields to developer provenance. A reimplementation must either preserve this or make a deliberate, documented change.
- Document-derived information must remain distinguishable from developer statements in inspection and export.

### 2.6 Versioning and freshness rules

- Profile version = number of saves; initial baseline is 1.
- Project version = number of saves; initial version is 1.
- Capture version = per-project sequence, starting at 1; latest is the highest version. History is ordered descending by version in the store.
- A capture’s fingerprint covers repository fingerprint + profile version/updatedAt + project version/updatedAt.
- A preview is valid only for the exact content it reviewed. Any source change invalidates it.
- A verification result (later) is fresh only for the exact code snapshot and requirements it reviewed; otherwise it is **Stale**.

---

## 3. User-facing screens and views for the current increment

The native rewrite may arrange these views differently from the old shell. The inventory below is the required functional surface, not a layout prescription.

### 3.0 Startup, loading, and gates

- On launch, initialize storage and load onboarding state, projects, documents, and agents.
- Show a loading state until the initial read completes.
- If no confirmed profile exists, show only **Onboarding**. No project, capture, SRS import, or data-management workspace is reachable; onboarding still exposes manual entry and optional CV import/draft.
- If a profile exists, show the workspace.
- A terminal/platform warning must be shown when the launch mode is unsupported (Windows terminal, tmux).
- A directory argument selects an existing project by equivalent real path, or pre-fills the new-project root and Home directory.

### 3.1 Onboarding

Purpose: establish the **Developer baseline** before any project work.

Reads:

- **Onboarding state** (complete/incomplete, missing labels, draft, profile).
- Imported CV documents.
- Coding-agent statuses for optional assisted CV import.

Actions:

- Edit required fields: display name/pseudonym, experience stage, new-to-coding flag, initial skills statement, learning goals.
- Edit optional profile detail: education, employment, project experience, certifications, links, self-reported skills, learning preferences.
- Save the onboarding draft (resumable after close/reopen).
- Confirm the developer baseline.
- Optional CV import:
  - Select a local PDF/DOCX/Markdown/TXT file.
  - Extract text locally.
  - Review the extracted text.
  - Confirm saving the approved text as an ImportedDocument.
  - Select an imported CV and an installed coding agent.
  - Prepare the exact sharing preview.
  - Confirm the exact text to share.
  - Request a schema-valid CV draft, with progress and cancellation.
  - Review/correct the structured draft.
  - Confirm applying the reviewed fields.

Validation and rules:

- Required: display name, experience stage, at least one learning goal; initial skills statement unless `newToCoding`.
- An explicit new-to-coding baseline is valid; do not require a CV, prior employment, or invented skills.
- Imported CV fields count only after review/confirmation.
- A failed CV import or draft preserves manual entry and existing data.
- Confirmation creates profile version 1 and clears the draft.
- Applying a reviewed CV draft can establish the profile directly; the current code does not require a separate `confirmOnboarding` call. The reimplementation must preserve the baseline gate after the profile is written.

### 3.2 Home

Purpose: orient the developer in the current workspace.

Reads:

- Selected project (if any) and its root path.
- Initial directory, when launched with one.

Actions:

- Navigate to Projects.
- Open the selected project (navigate to project detail/editor).

Empty state: when no project or initial directory exists, prompt to select or create a project. When a project exists, show its name/root and an “open project” action.

### 3.3 Profile

Purpose: inspect and edit the reusable **Developer profile**.

Reads:

- Current profile: display name, version, experience stage, skills statement, learning goals.
- Whether document provenance exists.
- Profile versions are available for export, even if the screen does not list them.

Actions:

- Edit all profile fields (same required/optional fields as onboarding).
- Save a new profile version.
- No profile-only deletion action.

Validation and rules:

- Required-field validation applies on every edit.
- Edits affect future captures; existing captures keep their profile snapshots.
- Saving reports the new version.
- Self-reported skills remain self-reported; never presented as verified mastery.

### 3.4 Projects

Purpose: create, select, inspect, edit, and delete **Project context**.

Reads:

- All projects, ordered by creation time ascending.
- Selected project and its version.
- Initial directory for pre-filling a new project root.

Actions:

- Select an existing project.
- Start a new project.
- Create a project.
- Edit an existing project and save a new version.
- Delete the selected project with explicit confirmation.

Validation and rules:

- Required: project name, readable root folder, at least one goal.
- Root must be an existing readable directory; files and missing paths are rejected.
- Optional: purpose, business logic, rules, constraints, decisions, acceptance criteria, knowledge, include paths, exclude paths, respect-`.gitignore`, per-file limit, per-capture limit.
- Limits must be positive MiB values in the UI and positive byte counts in the domain.
- Project context combines developer statements with reviewed SRS-derived fields.
- Deleting a project removes its captures and project-linked documents, but preserves the profile and other projects.
- Creating/updating/listing/deleting is gated on a confirmed baseline.

### 3.5 Documents

Purpose: import CV/SRS source documents and request reviewed agent drafts.

Reads:

- Imported CV documents and imported SRS documents (filtered by kind).
- Selected project for SRS.
- Coding-agent statuses and capabilities.

Actions:

- Enter/select a local document path and per-document MiB limit.
- Extract text locally for PDF, DOCX, Markdown, or TXT.
- Review extracted text.
- Confirm/save approved text as an ImportedDocument.
- Select an imported document.
- Select an installed coding agent.
- Prepare the exact sharing preview (text, bytes, limit, fit).
- Confirm the exact text and request a draft; show progress; cancel a running draft.
- Review/correct the returned JSON draft.
- Confirm and apply the reviewed CV/SRS fields.

Validation and rules:

- CV panel: `projectId = null`; SRS panel requires a selected project.
- Unsupported, unreadable, image-only, or oversized documents explain the problem and keep manual entry available.
- The original document is not copied; approved text is retained.
- The selected agent and provider do not change after a failure.
- The request is blocked when the exact text exceeds the agent’s reported input limit.
- No draft is applied without explicit confirmation.
- SRS draft output must belong to the selected project; a document already linked to a different project is rejected.
- There is no document deletion/edit action in the current application surface.

### 3.6 Agents

Purpose: show **Coding agent** compatibility and capabilities.

Reads:

- Four agent statuses in stable order: Codex, Claude Code, OpenCode, Antigravity.
- Availability, detail, setup URL, capabilities, restrictions, verified flag.

Actions:

- No connect, sign-in, model selection, or credential action. The screen is informational; agent selection happens in the Documents/Onboarding draft panels.

Rules:

- Each agent owns sign-in, provider, and model configuration.
- No adapter is marked verified until compatibility is demonstrated.
- Show unsupported/unknown capabilities explicitly; do not imply working compatibility.
- Manual entry remains available when an agent is unavailable.

### 3.7 Capture

Purpose: preview, approve, save, inspect, refresh, and delete **Repository context** as versioned **Context captures**.

Reads:

- Selected project and its scope/limits.
- Capture history for the project.

Actions:

- Edit include paths, exclude paths, respect-`.gitignore`, per-file limit, per-capture limit for the next preview.
- Preview capture.
- Refresh context.
- Review the preview file list, skipped files with reasons, total bytes, and Git state.
- Confirm that the exact reviewed capture was reviewed.
- Save the reviewed capture.
- Inspect a historical capture: version, time, file count, bytes, skipped count, Git state, file list, per-file content by hash.
- Delete a capture with explicit confirmation.

Validation and rules:

- Requires a confirmed developer baseline and a selected project.
- Preview is explicit; no background scan.
- Save requires confirmation and a still-valid preview.
- Source changes during review require a fresh preview.
- Unchanged refresh adds no duplicate; changed refresh creates a new version when saved.
- Skipped files must show reasons, not disappear silently.
- A readable non-Git folder can still be captured; Git state is then unavailable.
- Capture panel scope edits are per-preview and do not persist to the project. Clicking Refresh in the current UI re-runs with the project’s saved scope, not the edited form values. A native rewrite must either preserve this distinction or deliberately persist scope first.
- Saved captures do not currently store the effective `scope`/`limits`/extra exclusions as top-level fields; they are implied by the file set and project. A rewrite that needs exact reproducibility should add this only as an explicit schema change.
- Cancellation/failure leaves previously saved captures usable.
- Errors need a retry path.

### 3.8 Data and Settings

Purpose: export local context and manage retained data. The current app labels the Data surface “Settings”; there is no separate preferences view.

M1 settings/data surface requirements: JSON export and confirmed delete-all data. The preferred editor is currently remembered implicitly after a successful launch and may remain implicit. Agent credentials, model selection, and skill settings are explicitly later work.

Reads:

- Export bundle.
- Data directory/database location for diagnostics if desired.

Actions:

- Prepare JSON export in memory.
- Save the JSON export to a user-selected file (the current UI writes to the process working directory; that is not a requirement to carry over).
- Delete all application data with explicit confirmation.

Validation and rules:

- Export includes profile, all profile versions, projects, all project versions, documents, and all captures with content.
- Delete-all removes the profile, projects, documents, captures, and onboarding draft, and returns the app to onboarding.
- Delete-all must not delete unrelated files or the SQLite schema. The current UI also removes the ad-hoc preferred-editor file.
- There is no general preferences screen in the current increment. Editor preference is remembered implicitly after a successful launch.

### 3.9 Global behaviours and notifications

- A success/informational message area and an error area are available across the workspace.
- Errors must name the failed operation and preserve data.
- Destructive actions require an explicit second confirmation before the domain delete call. The core delete operations themselves have no `confirmed` flag, so the native UI owns this guard.
- Navigation must expose Home, Profile, Projects, Documents, Agents, Capture, and Settings/Data. Layout and tab mechanics are not requirements.
- The editor launch control offers installed editors and reports launch failure.

---

## 4. Application operations inventory

All operations below are synchronous unless marked `Promise`. The application composes a `ContextStore`, a `DocumentExtractor`, a `RepositoryCollector`, a `CodingAgentGateway`, and a clock. The core does not import platform implementations.

### 4.1 Error taxonomy

`ApplicationError` codes:

`validation`, `onboarding-required`, `not-found`, `unreadable-path`, `unsupported-document`, `unreadable-document`, `image-only-document`, `document-too-large`, `capture-stale`, `capture-limit`, `preview-expired`, `confirmation-required`, `agent-unavailable`, `agent-input-too-large`, `agent-cancelled`, `agent-timeout`, `agent-invalid-output`, `agent-failed`.

`ValidationError` is a `validation` error with `details: string[]`. Document extraction returns structured errors instead of throwing for the four document failure codes. The Rust rewrite should preserve the code semantics even if the concrete error type differs.

### 4.2 Onboarding and profile operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `getOnboardingState()` | none | `OnboardingState` | store errors | Reads profile + draft; missing labels; no writes. |
| `saveOnboardingDraft(input)` | `Partial<DeveloperProfileFields>` | `OnboardingDraft` | store errors | Sets `updatedAt`; overwrites the singleton draft; no completeness validation. |
| `confirmOnboarding(input)` | `DeveloperProfileInput` | `DeveloperProfile` | `validation` with missing labels | Normalizes; writes the next profile version (1 on first confirmation); clears draft. |
| `getProfile()` | none | `DeveloperProfile \| null` | store errors | No gate. |
| `updateProfile(input)` | `DeveloperProfileInput` | `DeveloperProfile` | `onboarding-required`, `validation` | Requires existing baseline; increments version; preserves `createdAt`; writes version history. |
| `requireDeveloperBaseline()` | none | `DeveloperProfile` | `onboarding-required` | Internal gate used by project/capture/SRS operations. |

### 4.3 Project operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `createProject(input)` | `ProjectInput` | `Project` | `onboarding-required`, `validation`, `unreadable-path` | Requires baseline; validates name/root/goal; root readable directory; normalizes statements/scope/limits; version 1; writes current + version row. |
| `updateProject(id, input)` | id, `ProjectInput` | `Project` | `onboarding-required`, `not-found`, `validation`, `unreadable-path` | Requires baseline; version+1; omitted optional statements clear; omitted `scope`/`limits` reset to defaults; unchanged text preserves provenance; writes version history. |
| `listProjects()` | none | `Project[]` | `onboarding-required` | Ordered by `created_at` ascending. |
| `getProject(id)` | id | `Project \| null` | store errors | No baseline gate. |
| `deleteProject(id)` | id | `void` | `onboarding-required`, `not-found` | Cascades project captures, project versions, and project-linked documents; preserves profile and other projects; cleans orphan content. |

`ProjectInput` requires `name`, `rootPath`, `goals: string[]`; optional `purpose`, `businessLogic`, `rules[]`, `constraints[]`, `decisions[]`, `acceptanceCriteria[]`, `knowledge[]`, `scope`, `limits`.

### 4.4 Document operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `extractDocument(request)` | `{ path, kind, maxBytes? }` | `Promise<DocumentExtractionResult>` | structured document errors | Local extraction only; supported extension; readable; size limit (default 20 MiB); SHA-256; no save. |
| `confirmDocumentImport(document, options)` | `ExtractedDocument`, `{ confirmed, projectId? }` | `ImportedDocument` | `confirmation-required`, `not-found` | Requires review confirmation; optional project must exist; generates ID/import time; saves approved text and metadata; never copies the original. |
| `listDocuments()` | none | `ImportedDocument[]` | store errors | Ordered by `imported_at` ascending. |
| store-only `deleteDocument(id)` | id | `void` | store errors | Not exposed as an application operation in M1; project deletion removes project-linked documents. |

Document extraction failure codes and messages:

- `unsupported-document`: extension not in `.pdf`, `.docx`, `.md`, `.markdown`, `.txt`.
- `unreadable-document`: missing, unreadable, corrupt, encrypted, or otherwise failed extraction.
- `image-only-document`: no text extracted (scanned/image-only).
- `document-too-large`: file size exceeds `maxBytes`; ask for a narrower selection or manual entry.

### 4.5 Coding-agent draft operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `listCodingAgents()` | none | `CodingAgentStatus[]` | store/which errors | Stable order; detection only; no credential check. |
| `prepareCvDraft(documentId, agentId)` | document ID, agent ID | `Promise<DraftSharingPreview>` | `not-found`, `validation`, `agent-unavailable` | Document must exist and be kind `cv`; agent must be ready; builds exact text; no agent call. |
| `prepareSrsDraft(documentId, agentId, projectId)` | document ID, agent ID, project ID | `Promise<DraftSharingPreview>` | `onboarding-required`, `not-found`, `validation`, `agent-unavailable` | Requires baseline and project; document kind `srs`; no agent call. |
| `requestCvDraft(documentId, agentId, options)` | IDs, `{ confirmed, signal?, onProgress? }` | `Promise<AgentDraftResult>` | `confirmation-required`, `agent-input-too-large`, `agent-unavailable`, `agent-invalid-output`, `agent-cancelled`, `agent-timeout`, `agent-failed`, `not-found`, `validation` | Builds preview internally; no save; validates output kind/schema. |
| `requestSrsDraft(documentId, agentId, projectId, options)` | IDs, options | `Promise<AgentDraftResult>` | same as CV plus `onboarding-required`, project `not-found` | Same, with project context in request text. |
| `applyCvDraft(documentId, value, { confirmed })` | document ID, JSON value, confirmation | `DeveloperProfile` | `confirmation-required`, `not-found`, `validation`, `agent-invalid-output` | Document must be `cv`; parses/validates draft; merges only provided fields with existing profile; when creating the first profile, omitted `newToCoding` defaults to `false`; records field sources; validates required fields; saves a new profile version. Can create the first profile. Does not clear the onboarding draft in current code. |
| `applySrsDraft(projectId, documentId, value, { confirmed })` | project ID, document ID, JSON value, confirmation | `Project` | `confirmation-required`, `onboarding-required`, `not-found`, `validation`, `agent-invalid-output` | Requires baseline/project; document `srs` and not linked to another project; validates draft; replaces provided statements with document provenance, except goals are replaced only when the draft supplies a non-empty `goals` array; requires at least one goal; version+1. |

Draft request text must include:

- CV instructions: use only facts in the source; do not invent; return one JSON object only; include a quote/section label in `sources[].reference` for filled fields; omit absent fields; exact field list; developer will review.
- SRS instructions: same constraints plus the SRS field list.
- Source metadata: document name, kind, hash, and project name for SRS.
- The full source document text.

The adapters may request native structured output (Codex/Claude/Antigravity) or validate text JSON (OpenCode). All outputs go through the same strict CV/SRS schema and kind check.

### 4.6 Capture operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `previewCapture(projectId, overrides?)` | project ID; `{ scope?, limits?, excludePaths? }` | `Promise<CapturePreview>` | `onboarding-required`, `not-found`, `validation` | Requires baseline + project; merges scope/limits; collects repository; stores transient preview with contents and snapshots; no DB write. |
| `saveCapture(previewId, { confirmed })` | preview ID, confirmation | `Promise<ContextCapture>` | `confirmation-required`, `preview-expired`, `onboarding-required`, `not-found`, `capture-stale` | Re-collects and re-fingerprints; stale preview rejected; unchanged latest returned; otherwise writes capture version latest+1 and content rows transactionally. |
| `refreshProjectContext(projectId)` | project ID | `Promise<CaptureRefreshResult>` | `onboarding-required`, `not-found`, `validation` | Fresh preview; compares to latest capture fingerprint; `unchanged` returns latest, `changed` returns preview with null capture. |
| `listCaptures(projectId)` | project ID | `ContextCapture[]` | store errors | Ordered by version descending; hydrated content. |
| `getCapture(id)` | capture ID | `ContextCapture \| null` | store errors | Hydrates file content by hash; missing content becomes `""`. |
| `deleteCapture(id)` | capture ID | `void` | `not-found` | Deletes capture, file/skipped rows, and orphan content. No baseline gate. |
| `summarizePreview(preview)` | preview | `CaptureSummary` | none | Pure helper: file/skip counts, total bytes, counts by category, counts by skip reason, git availability/dirty. |

`CaptureRefreshResult` is the only refresh contract; there is no separate “changed files” diff.

### 4.7 Data operations

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| `exportData()` | none | `ExportBundle` | store errors | Complete JSON bundle; no baseline gate; no file write. |
| `deleteAllData()` | none | `void` | store errors | Deletes all application rows in one transaction; clears in-memory previews; leaves schema/migrations. |

### 4.8 Desktop and OS operations

These are current desktop-app behaviours, not core domain operations.

| Operation | Input | Output | Errors | Invariants / side effects |
| --- | --- | --- | --- | --- |
| Open directory / start | CLI args + persisted projects | launched app with `initialDirectory` | same usage/path errors | Resolve the directory, match an existing project by real path, otherwise pre-fill the new-project root; never create a project implicitly. |
| `directoryArgument(args)` | CLI args | `string \| null` | throws `Usage: coderune [directory]` or `Not a readable directory: …` | Zero args → `null`; exactly one non-option arg → resolved readable directory; multiple args or an option-like arg → usage error. |
| `sameDirectory(a, b)` | two paths | boolean | none | Compares real paths; false when either cannot resolve. |
| `installedEditors()` | none | editor list | none | Detects `code`, `cursor`, `antigravity`, `zed` on PATH; on macOS also standard `/Applications/<name>.app`; returns executable/app path. |
| `openEditor(editor, directory)` | editor + directory | `Promise<void>` | throws launch error/stderr | Opens the directory in the editor; non-zero exit is an error. |
| preferred editor load/save | data directory | name or null | ignored when absent | Current UI uses `preferred-editor.txt`; this ad-hoc file is not a domain requirement. |
| save export file | export text | path | file write error | Current UI writes `coderune-export-<timestamp>.json` to `process.cwd()`; a native save dialog is acceptable and preferred. |

### 4.9 Store and collector ports

The store must expose at least:

- Onboarding: `getProfile`, `getProfileVersions`, `saveProfile`, `getOnboardingDraft`, `saveOnboardingDraft`, `clearOnboardingDraft`.
- Projects: `createProject`, `updateProject`, `getProject`, `getProjectVersions`, `listProjects`, `deleteProject`.
- Documents: `saveDocument`, `getDocument`, `listDocuments(projectId?)`, `deleteDocument`.
- Captures: `saveCapture`, `getCapture`, `listCaptures(projectId)`, `getCaptureContent(hash)`, `deleteCapture`, `deleteProjectCaptures`.
- Data: `deleteAllData`, `initialize`, `close`.

The repository collector must expose `validateRoot(rootPath)` and `collect(request)`. The document extractor must expose `extract(request)`. The agent gateway must expose `listAgents()`, `getAgent(id)`, and `draft(request, options)`.

### 4.10 Cross-cutting gates and confirmations

- Baseline gate: `createProject`, `updateProject`, `listProjects`, `deleteProject`, `previewCapture`, `saveCapture`, `refreshProjectContext`, `prepareSrsDraft`, `requestSrsDraft`, `applySrsDraft`, and `requireDeveloperBaseline`.
- No baseline gate: `getOnboardingState`, `saveOnboardingDraft`, `confirmOnboarding`, `getProfile`, `updateProfile` (requires an existing profile), `extractDocument`, `confirmDocumentImport`, `listCodingAgents`, `prepareCvDraft`, `requestCvDraft`, `applyCvDraft`, `listDocuments`, `getProject`, `listCaptures`, `getCapture`, `deleteCapture`, `exportData`, `deleteAllData`.
- Confirmation flags in the domain: `confirmDocumentImport`, `requestCvDraft`, `requestSrsDraft`, `applyCvDraft`, `applySrsDraft`, `saveCapture`.
- Confirmation for destructive project/capture/all-data deletion is a UI responsibility in the current code; the native UI must not call delete without an explicit user confirmation.
- Any mutation must be transactional where it spans multiple rows.

---

## 5. SQLite schema, migrations, and retention

### 5.1 Database location and connection rules

Legacy-compatible location (must be preserved to reopen existing data):

- macOS: `~/Library/Application Support/Context Bridge/context-bridge.sqlite`
- Windows: `%APPDATA%\Context Bridge\context-bridge.sqlite` (fallback `~/AppData/Roaming/Context Bridge`)
- Linux: `$XDG_DATA_HOME/context-bridge/context-bridge.sqlite` or `~/.local/share/context-bridge/context-bridge.sqlite`

The application directory name is `Context Bridge`; the database filename is `context-bridge.sqlite`. These names are intentionally not “Coderune” because existing data must remain discoverable.

Connection settings:

- `PRAGMA journal_mode = WAL`
- `PRAGMA foreign_keys = ON` (note: the current schema declares no foreign-key clauses; see §5.7)
- `PRAGMA busy_timeout = 5000`
- Create the parent directory recursively.
- On first creation, best-effort `chmod 0600` on the database file (POSIX only).
- Open with create enabled.

### 5.2 Migration mechanism

- `schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)` is created with `IF NOT EXISTS`.
- Migrations are ordered by version, applied only if their version is absent, each in a transaction that runs the DDL and inserts the migration row.
- No down migrations.
- Current database version is **4**. After applying migrations, the maximum `schema_migrations.version` must equal 4; otherwise initialization throws `Unsupported Coderune database version`.
- Migration versions:
  1. `developer_profiles`, `developer_profile_versions`, `onboarding_draft`.
  2. `projects`, `project_versions`.
  3. `documents` plus indexes.
  4. `contents`, `captures`, `capture_files`, `capture_skipped` plus capture index.

### 5.3 Tables and columns

```sql
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE developer_profiles (
  id TEXT PRIMARY KEY,
  version INTEGER NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE developer_profile_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  profile_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(profile_id, version)
);

CREATE TABLE onboarding_draft (
  id TEXT PRIMARY KEY CHECK (id = 'onboarding'),
  data TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  version INTEGER NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE project_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(project_id, version)
);

CREATE TABLE documents (
  id TEXT PRIMARY KEY,
  project_id TEXT,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  hash TEXT NOT NULL,
  size INTEGER NOT NULL,
  text TEXT NOT NULL,
  data TEXT NOT NULL,
  imported_at TEXT NOT NULL
);

CREATE INDEX documents_project_id ON documents(project_id);
CREATE INDEX documents_hash ON documents(hash);

CREATE TABLE contents (
  hash TEXT PRIMARY KEY,
  text TEXT NOT NULL,
  size INTEGER NOT NULL
);

CREATE TABLE captures (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  fingerprint TEXT NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(project_id, version)
);

CREATE INDEX captures_project_id ON captures(project_id);

CREATE TABLE capture_files (
  capture_id TEXT NOT NULL,
  path TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  size INTEGER NOT NULL,
  category TEXT NOT NULL,
  PRIMARY KEY (capture_id, path)
);

CREATE TABLE capture_skipped (
  capture_id TEXT NOT NULL,
  path TEXT NOT NULL,
  reason TEXT NOT NULL,
  PRIMARY KEY (capture_id, path)
);
```

### 5.4 Keys and indexes

- Primary keys: profile `id`, project `id`, document `id`, capture `id`, content `hash`, and composite `(capture_id, path)` for capture files/skipped.
- Unique constraints: `(profile_id, version)`, `(project_id, version)`, and `(project_id, version)` on captures.
- Indexes: `documents_project_id`, `documents_hash`, `captures_project_id`.
- `developer_profiles.id` and `onboarding_draft.id` are effectively singleton checks; only the onboarding row has an explicit `CHECK`.
- `project_versions` and `developer_profile_versions` use surrogate integer IDs plus a unique business key.
- There is no explicit uniqueness constraint on document hash; duplicate imports are allowed as separate document rows.

### 5.5 JSON payloads and hydration

The normalized columns duplicate fields needed for ordering/filtering. The `data` columns hold the full serialized entity and remain the source of truth for most reads:

- `developer_profiles.data`: full `DeveloperProfile`.
- `developer_profile_versions.data`: full `DeveloperProfile`.
- `onboarding_draft.data`: `OnboardingDraft`.
- `projects.data`: full `Project`.
- `project_versions.data`: full `Project`.
- `documents.data`: full `ImportedDocument`; `text` is also a separate column.
- `captures.data`: full `ContextCapture` with each file’s `content` stripped.
- `contents.text`: file text keyed by SHA-256; `contents.size` matches the file size.

Read behaviours:

- `getProfile()` reads the row with `id = 'developer'`.
- `getProfileVersions()` reads `profile_id = 'developer'` ordered by version ascending.
- `listProjects()` reads all projects ordered by `created_at` ascending.
- `listCaptures(projectId)` reads captures ordered by `version` descending and hydrates file content by hash.
- `listDocuments()` without a project filter reads all documents ordered by `imported_at` ascending; with `null` it filters `project_id IS NULL`; with an ID it filters that project.
- Missing content for a capture file hydrates as an empty string.
- JSON field names and ISO-8601 timestamps must be preserved for export and database compatibility.

### 5.6 Retention and deletion matrix

| Data | Created / updated by | Deleted by | Notes |
| --- | --- | --- | --- |
| Current developer profile | `confirmOnboarding`, `updateProfile`, `applyCvDraft` | `deleteAllData` | Singleton `developer`; no profile-only delete. |
| Developer profile versions | Every profile save | `deleteAllData` | All versions retained indefinitely; no pruning. |
| Onboarding draft | `saveOnboardingDraft` | `confirmOnboarding`, `deleteAllData` | Overwritten on each save; survives app restart. |
| Current project | `createProject`, `updateProject`, `applySrsDraft` | `deleteProject`, `deleteAllData` | Version increments. |
| Project versions | Every project save | `deleteProject`, `deleteAllData` | All versions retained until project deletion. |
| Imported documents | `confirmDocumentImport` | `deleteProject` (project-linked only), `deleteAllData` | No automatic TTL; no UI delete in M1. |
| Captures | `saveCapture` | `deleteCapture`, `deleteProject`, `deleteAllData` | Per-project version sequence. |
| Capture files | `saveCapture` | Same as capture | Composite key; content is in `contents`. |
| Capture skipped rows | `saveCapture` | Same as capture | Reasons retained with the capture. |
| Content blobs | `saveCapture` | Orphan cleanup after capture/project deletion, `deleteAllData` | Deduplicated by hash; no TTL. |
| Schema migrations | `initialize` | Never | Not removed by `deleteAllData`. |
| SQLite file | First open | Not deleted by app data deletion | `deleteAllData` empties rows, not the file. |

Deletion semantics in detail:

- `deleteCapture(id)`: delete `capture_skipped`, `capture_files`, `captures` rows; then delete `contents` rows whose hash is not referenced by any `capture_files`.
- `deleteProjectCaptures(projectId)`: same for every capture belonging to the project.
- `deleteProject(id)`: delete all project captures and their file/skipped rows; delete `project_versions`; delete `documents` where `project_id = id`; delete the project row; delete orphan contents. Profile and other projects remain.
- `deleteAllData()`: delete `capture_skipped`, `capture_files`, `captures`, `contents`, `documents`, `project_versions`, `projects`, `developer_profile_versions`, `developer_profiles`, `onboarding_draft` in that order. `schema_migrations` remains.
- There is no automatic retention/TTL for profiles, projects, documents, captures, or content.

### 5.7 Schema constraints and compatibility notes

- The current schema declares no `FOREIGN KEY` clauses even though `PRAGMA foreign_keys = ON`. Referential integrity is enforced manually by operations and deletion order. A Rust rewrite may add real foreign keys only if cascade/delete semantics remain identical and existing databases can still be opened/migrated.
- `documents.project_id` has no foreign key; project deletion manually removes project-linked documents.
- `captures.project_id`, `capture_files.capture_id`, and `capture_skipped.capture_id` have no foreign keys; manual cascades are required.
- `ContextCapture` does not store `scope`, `limits`, or extra exclusions at top level. The effective scope/limits exist only in `CapturePreview` and are not persisted with the saved capture. Preserve current behaviour or introduce a versioned migration deliberately.
- `context capture` content is deduplicated across captures; deleting one capture must not delete content still referenced by another.
- The legacy database path/name and schema version matter. A rewrite that changes either needs an explicit migration/import path; silently creating `Coderune/coderune.sqlite` would orphan existing data.
- The database is the authoritative local store; no PostgreSQL/pgvector, remote database, or graph database is used in M1.

---

## 6. Repository collection rules

### 6.1 Root and scope resolution

- `validateRoot(rootPath)`: `statSync` must show a directory and `accessSync(R_OK)` must succeed. Files, missing paths, and unreadable paths are rejected.
- `collect(request)` resolves the real root path (`realpathSync`) before walking.
- If `includePaths` is empty, walk the whole root.
- If `includePaths` is non-empty, resolve each entry relative to the root:
  - A path outside the root is skipped as `escapes-repository`.
  - A symlink/real path that escapes the root is skipped.
  - A missing/unreadable include is skipped as `not-readable`.
  - A directory include is walked; a file include is added through the single-file path.
  - Protected ancestors (sensitive/generated/dependency) cannot be bypassed by explicitly including them.
- Paths are normalized to repository-relative `/` separators; `.` and leading `./`/`/` are removed for matching.

### 6.2 Included file types and categories

Allowed files:

- Dependency manifests: `package.json`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lock`, `bun.lockb`, `pyproject.toml`, `poetry.lock`, `Pipfile`, `Pipfile.lock`, `uv.lock`, `setup.py`, `setup.cfg`, `requirements.txt`, and any `requirements*.txt`.
- Configuration: `tsconfig*.json`, `jsconfig.json`, `turbo.json`, `nx.json`, `vite.config.*`, `vitest.config.*`, `jest.config.*`, `eslint.config.*`, `.eslintrc*`, `.prettierrc*`, `.editorconfig`, `.python-version`, `.nvmrc`, `.tool-versions`, `pytest.ini`, `tox.ini`, `mypy.ini`, `ruff.toml`, `pylintrc`.
- `AGENTS.md` (case-insensitive basename).
- `README` with any suffix (case-insensitive).
- Documentation: `.md`, `.markdown`, `.txt`.
- Source: `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.py`.
- Tests are a category, not a separate allowlist: test paths are detected by `__tests__/`, `tests?/`, `*.test.*`, `*.spec.*`, `test_*.py`, or `*_test.py`; they must also be an allowed source/document type to be included.

Category derivation order:

1. Basename `AGENTS.md` → `agent-instructions`.
2. Basename matching `README*` → `readme`.
3. Manifest → `manifest`.
4. Configuration → `configuration`.
5. Test path → `test`.
6. Allowed source extension → `source`.
7. Allowed documentation extension → `documentation`.
8. Otherwise → `other` (normally unreachable because unsupported files are skipped first).

### 6.3 Exclusions and skip reasons

Hard exclusions always apply, including when explicitly included:

- Sensitive names: `.env`, `.env.*`, `.npmrc`, `.pypirc`, `.netrc`, `id_rsa`, `id_ed25519`, `id_ecdsa`, `*.pem`, `*.key`, `*.p12`, `*.pfx` → `environment-file` or `private-key`.
- Dependency directories: `.git`, `.hg`, `.svn`, `.venv`, `venv`, `env`, `node_modules`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.turbo`, `.next`, `.nuxt`, `.cache`, `dist`, `build`, `out`, `coverage`, `vendor`, `target` → `dependency-directory`.
- Generated output directories: `dist`, `build`, `out`, `coverage`, `.next`, `.turbo` → `generated-output` (checked after sensitive, before dependency).
- Binary/archive/media/font/database extensions: `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.ico`, `.bmp`, `.tiff`, `.mp3`, `.mp4`, `.mov`, `.avi`, `.wav`, `.pdf`, `.docx`, `.doc`, `.xlsx`, `.zip`, `.gz`, `.tar`, `.rar`, `.7z`, `.exe`, `.dll`, `.so`, `.dylib`, `.bin`, `.woff`, `.woff2`, `.ttf`, `.otf`, `.sqlite`, `.db` → `binary`.
- Symlinks → `symbolic-link`; do not follow.
- Non-regular files → ignored (not read).
- Files outside the root, or whose relative path starts with `..` → `escapes-repository`.
- Unreadable file/directory → `not-readable`.
- Files larger than `maxFileBytes` → `file-too-large`.
- Files that would exceed `maxCaptureBytes` → `capture-limit`.
- Files containing a NUL byte in the first 8,000 bytes → `binary`, even if the extension is allowed.
- Unsupported types → `unsupported-type`.
- Scope exclusions and extra preview exclusions → `excluded`.
- `.gitignore` matches, when `respectGitignore` is true → `ignored`.

Scope/ignore matching:

- `excludePaths` and preview-only `extraExcludePaths` support: directory patterns ending in `/`, bare names matching any path segment, path prefixes, and glob patterns (`*`, `?`, `[]`, `{}` via `Bun.Glob`).
- Only the root `.gitignore` is read. Nested `.gitignore` files are not loaded. Comments (`#`) are skipped; `!` negations are supported in order.
- An excluded directory is skipped before descending, so its contents are not enumerated.
- `respectGitignore` can be disabled per project/preview.

### 6.4 Limits, ordering, and counting

- Defaults: 1 MiB per file and 20 MiB per capture (repository capture).
- The separate imported-document limit defaults to 20 MiB and is applied per selected CV/SRS document; repository limits do not silently apply to document imports.
- A file over the per-file limit is skipped with `file-too-large`.
- A file that would push the total over the per-capture limit is skipped with `capture-limit`; the total never exceeds the limit.
- `totalBytes` counts only included files.
- Entries are sorted by basename within a directory before recursion; final `files` and `skipped` are sorted by path. This makes collection deterministic.
- `maxFileBytes <= 0` is classified as `file-too-large`; the application-level validation prevents non-positive limits for normal project creation.

### 6.5 Hashing and fingerprints

- Every included file is hashed with SHA-256 over its raw bytes; hash is lower-case hex.
- Every imported document is hashed with SHA-256 over its raw bytes.
- Repository fingerprint:

```text
sha256(JSON.stringify({
  files: [[path, hash], ...],
  git: git ? [git.head, git.branch, git.status] : null,
  scope,
  limits
}))
```

- Full capture fingerprint:

```text
sha256(JSON.stringify({
  repository: repositoryFingerprint,
  profileVersion,
  profileUpdatedAt,
  projectVersion,
  projectUpdatedAt
}))
```

- Consequences:
  - File mtimes, skipped list, and preview timestamps do not affect the fingerprint.
  - A Git status change (including untracked/working-tree changes reported by porcelain) changes the repository fingerprint and therefore the capture fingerprint.
  - Changing scope, limits, profile version, or project version changes the capture fingerprint even when file contents are identical.
  - Changing only skipped paths (with identical included files/git/scope/limits) does not change the fingerprint.
  - Comparing fingerprints is the unchanged-refresh and duplicate-capture test.

### 6.6 Git metadata

- Git metadata is read with system `git` commands executed directly (no shell), using the repository root:
  - `git -C <root> rev-parse --is-inside-work-tree`
  - `git -C <root> rev-parse HEAD`
  - `git -C <root> rev-parse --abbrev-ref HEAD`
  - `git -C <root> status --porcelain=v1`
- If the folder is not inside a Git work tree, `git` is `null`.
- `head`/`branch` are `null` when the individual command fails or returns empty.
- `dirty` is true when the porcelain status has at least one line.
- `capturedAt` is the collection time.
- A readable non-Git folder must still collect files successfully.

### 6.7 Refresh and unchanged behaviour

- Initial capture: `previewCapture` → review → `saveCapture` creates capture version 1.
- Manual refresh: `refreshProjectContext` performs a new preview and compares its full fingerprint with the latest saved capture.
- `unchanged`: same fingerprint; return the latest capture and do not add a row.
- `changed`: different fingerprint; return the new preview; saving creates `latestVersion + 1`.
- `saveCapture` re-collects and re-fingerprints the current state. If it differs from the reviewed preview, the preview is discarded and `capture-stale` is returned.
- If the current fingerprint equals the latest saved capture fingerprint, `saveCapture` returns that existing capture instead of duplicating it.
- A preview lives only in memory and expires when the process restarts (`preview-expired`).
- Cancelled/failed captures must not remove previously saved captures.
- The UI must show the reviewed file list and skip reasons before saving.

### 6.8 Security and side-effect constraints

- Never execute project scripts, build tools, tests, or package managers during scanning or local document extraction.
- Never contact external services during scanning or local document extraction.
- Never follow symlinks out of the repository.
- Never read or collect environment files, private keys, credential stores, dependency directories, generated output, or binaries.
- Never collect paths outside the selected repository root.
- Agent-assisted interpretation is a separate, explicitly requested operation; only previewed text is shared, and the selected agent may use a hosted provider.
- Agent draft processes run in temporary working directories with restrictions intended to prevent project-file modification. Capability claims must be accurate per adapter; all current isolation states are `unverified`.
- Do not read or copy agent credential files into application records or exports. The current TS subprocess launcher passes `process.env` through; the Rust rewrite should minimize child environment exposure, never log secrets, and never persist credential material.
- Draft jobs must not receive permission to modify project files merely because the agent supports such tools. Enforce working-directory isolation and per-adapter tool restrictions; rejecting client callbacks alone is insufficient.
- Parse/validate all agent output; schema-valid output can still be factually wrong and must be reviewable.

---

## 7. Agent integration requirements

### 7.1 Adapters

| ID | Name | Executable | Connection | Structured output | Streaming | Cancellation | Max input | Isolation |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `codex` | Codex | `codex` | Codex App Server over stdio (`codex app-server`) | yes | yes | yes | 20 MiB | `unverified` |
| `claude-code` | Claude Code | `claude` | Installed CLI print/headless JSON mode | yes (`--json-schema`) | no | yes | 10 MiB | `unverified` |
| `opencode` | OpenCode | `opencode` | Native ACP over stdio (`opencode acp`), protocol v1 | no (text JSON validated by Coderune) | yes | yes | 20 MiB | `unverified` |
| `antigravity` | Antigravity | `agy` | Official headless CLI stream-json | yes (`--json-schema`) | yes | yes | 20 MiB | `unverified` |

All agent definitions currently report:

- `verified: false`.
- “Compatibility is not yet demonstrated on this platform.”
- “The coding agent keeps its own sign-in, provider, and model configuration.”
- Defined restriction text: “Draft runs use a temporary working directory and cannot change project files.” Isolation remains `unverified`.
- Adapter-specific restrictions (Codex read-only sandbox; Claude non-interactive with tools denied; OpenCode rejects ACP permissions; Antigravity sandbox mode/temporary workspace).

### 7.2 Detection and status

- Detection is per executable on `PATH` (`which`), with an injectable lookup for tests.
- `ready` when found; `not-installed` otherwise.
- `unavailable` exists in the domain for sign-in/auth/protocol failures or missing support; the local gateway does not prefetch sign-in state.
- The status detail must tell the developer where the tool was found and that sign-in/provider/model choice remain with the tool, or that the tool was not found and must be installed/signed in.
- The four agents are listed in stable order: `codex`, `claude-code`, `opencode`, `antigravity`.
- No model inventory, account, subscription, or credential is read or stored.
- No agent is selected automatically for a failed agent; no fallback.

### 7.3 Draft job lifecycle

1. Developer selects an imported CV or SRS document (kind must match).
2. Developer selects an installed agent.
3. `prepare*Draft` builds the exact request text and a `DraftSharingPreview`; no agent process starts.
4. UI shows the exact text, estimated bytes, agent limit, and fit.
5. Developer confirms the exact text.
6. `request*Draft` checks confirmation, fit, and agent readiness, then runs the adapter.
7. Progress messages are reported through `onProgress`; a running job can be cancelled through `AbortSignal`.
8. Adapter returns a JSON object.
9. Coderune parses/validates it against the strict CV/SRS schema and checks the `kind`.
10. UI shows the returned JSON for review/correction.
11. Developer confirms review; `apply*Draft` validates again and writes a new profile/project version with document provenance.
12. Failure at any point leaves saved data unchanged and manual entry available.

The request text must include the kind-specific instruction block, source metadata (name, kind, hash, project name for SRS), and full source text. Instructions must say: use only facts present, do not invent, one JSON object only, include source references, omit absent fields, and the developer will review everything.

### 7.4 Adapter-specific behaviour

**Codex**

- Spawn `codex app-server` in a fresh temporary directory.
- JSON-RPC: `initialize` with client info (`coderune`, `Coderune`), then `initialized`.
- `thread/start` with `cwd`, `approvalPolicy: "never"`, `sandbox: "readOnly"`, `serviceName: "coderune"`.
- `turn/start` with `threadId`, `cwd`, `approvalPolicy: "never"`, read-only sandbox policy, text input, and `outputSchema` for the draft kind.
- Capture `item/completed` agent messages; finish on `turn/completed`; `failed` → `agent-failed`; `interrupted` → `agent-cancelled`.
- Decline command/file-change approvals; reject interactive tool input; decline MCP elicitation.
- Default timeout 120 s; cleanup temp directory and process.

**Claude Code**

- Spawn installed `claude` with print mode, JSON output, `--json-schema`, `--permission-mode dontAsk`, no permission prompts, and disallowed tools (`Bash`, `Edit`, `Write`, `NotebookEdit`, `WebFetch`, `WebSearch`).
- Send request text on stdin; parse JSON envelope.
- `is_error: true` with auth/login wording → `agent-unavailable`; otherwise `agent-failed`.
- Prefer `structured_output`, then `result`; invalid/missing → `agent-invalid-output`.
- No streaming; cancellation kills the process.

**OpenCode**

- Spawn `opencode acp`; JSON-RPC/ACP v1.
- `initialize` with `protocolVersion: 1`, no filesystem/terminal client capabilities, client info.
- Reject any `session/request_permission` by selecting a reject option or cancelling.
- `session/new` with `cwd`, no MCP servers; require a session ID.
- `session/prompt` with text; collect `session/update` → `agent_message_chunk` text.
- `stopReason: "cancelled"` → `agent-cancelled`; anything other than `end_turn` → `agent-failed`.
- Parse the accumulated text as JSON.
- Default timeout 120 s.

**Antigravity**

- Spawn `agy` with stream-json input/output, `--json-schema`, `--sandbox`, `--print-timeout 2m`.
- Send a single `{ event: "user", message: { content: request.text } }` line.
- Read NDJSON lines; use the final `event: "result"` envelope (`result` or top-level).
- `status: "SUCCESS"` required; `CANCELED`/`INTERRUPTED` → `agent-cancelled`; auth wording → `agent-unavailable`; other non-success → `agent-failed`.
- Prefer `structured_output`, then `response`; invalid/missing → `agent-invalid-output`.

### 7.5 Validation, cancellation, timeouts, and errors

- Default process/request timeout is 120 s. Codex and OpenCode use 120 s; Claude uses the shared default; Antigravity additionally configures its own 2-minute print timeout.
- On timeout, the process is killed and `agent-timeout` is returned.
- On cancellation, the process/connection is closed and `agent-cancelled` is returned. If the adapter throws an `AbortError` or the signal is already aborted, map it to `agent-cancelled`.
- A missing executable/start failure → `agent-unavailable`.
- A non-zero exit with auth/login/sign-in/unauthorized wording → `agent-unavailable`; otherwise `agent-failed` with the tail of stderr.
- Non-JSON output or schema mismatch → `agent-invalid-output`.
- JSON parsing accepts a direct object, a JSON string, a fenced JSON block, or the substring from the first `{` to the last `}`.
- Schema validation is strict: extra keys are rejected; optional fields may be omitted; `kind` must match the requested draft.
- No automatic retry, no fallback agent, no provider/model switch.
- Cancellation and failure must preserve existing profile/project/capture data.

### 7.6 No credential copying

- Coderune must not read, copy, store, or export the coding agent’s credential files, tokens, keychain entries, or provider configuration.
- Sign-in, provider, model selection, and permissions remain owned by the external tool.
- If the tool requires sign-in, report it and direct the developer to the tool’s own setup flow.
- Manual entry must remain available without any agent connection.
- The current TS implementation passes the parent environment to child processes; the Rust rewrite should avoid persisting or logging that environment and should pass only what the adapter needs.

### 7.7 Later agent connection, subagent, and skill work

Not part of M1:

- Persisted **Agent connection** records and available model inventories per adapter.
- Developer selection of the main coding model.
- **Defined subagent** roster: Context reviewer, Claim extractor, Code verifier, Impact analyst, Teacher, Learning evaluator.
- Per-subagent **Model assignment** defaults; new **Subagent run** inherits them.
- **Unavailable** state with **Retry** and **Select another model**; explicit replacement; linked retry attempts preserving evidence.
- Active run snapshots of connection/model, skill contents/versions, code snapshot, and verification brief.
- **Skill library** / **Skill collection** storage, enable/disable, add-your-own, on-demand loading, provenance, update/rollback, and source precedence.
- Parallel verification subagent runs against one snapshot; no code edits; preserved findings and **Needs review** conflicts.

Open design branches that must not be treated as decided: unqualified skill names/aliases; adapter delivery from application-owned storage and its fallback behaviour; task routing, concurrency, and failure policy; snapshot creation/retention, resource limits, and cancellation for subagent runs. The six-subagent roster is accepted for now but remains revisable.

Any implementation of these later capabilities must not retroactively change M1 capture/profile semantics.

---

## 8. Minimal but complete MVP slice for the Rust rewrite

The MVP is the current increment, not the full product. It should stop after slice 7. Slices 8+ are explicitly later.

### Slice 0 — Compatibility and storage foundation

Build:

- Domain structs/enums for profile, onboarding, project, document, repository collection, capture preview/capture, agent status/draft, and export.
- SQLite store with migrations 1–4, legacy path/name, WAL/busy timeout, singleton rows, version history, content dedupe, and transactional deletes.
- Error taxonomy and a clock abstraction.

Acceptance criteria:

- A new database initializes to schema version 4; opening an existing version-4 database reads profile, projects, documents, and captures without loss.
- Reopening preserves the confirmed profile, project versions, documents, captures, and content.
- Deleting a project/capture/all data matches the retention matrix.
- Export produces the documented camelCase JSON bundle with all versions and capture content.
- Unit tests cover migration order, version increments, content hydration, and cascade deletion.

### Slice 1 — Onboarding and developer baseline

Build:

- Onboarding state, draft save/resume, confirmation, validation, new-to-coding normalization, profile edit/versioning.
- Gate all project/capture/SRS work on the baseline.

Acceptance criteria:

- With no profile, project work is blocked and onboarding shows the four missing required labels.
- A draft can be saved, the app reopened, and the draft resumed.
- Confirming a valid baseline creates profile version 1, clears the draft, and allows project work.
- A new-to-coding developer can complete onboarding without a CV, agent, or invented skills; normalized statement is exact.
- Invalid confirmation returns specific missing labels and writes nothing.
- Editing the profile creates version 2+ and preserves version 1 in export.

### Slice 2 — Projects and project context

Build:

- Project list/create/update/delete, statement normalization, developer/document provenance, scope/limits validation, root readability validation.

Acceptance criteria:

- Name, readable folder, and one goal are required; missing/file roots are rejected with the right error code.
- Created project is version 1; editing is version 2 and preserves `createdAt`.
- Unchanged statements keep IDs/provenance; changed statements become developer-provenance statements.
- Project deletion removes its captures and linked documents but preserves the profile and other projects.
- Duplicate/blank paths are normalized; limits must be positive.

### Slice 3 — Local document import

Build:

- PDF/DOCX/Markdown/TXT extraction, per-document limit, SHA-256, structured failures, review confirmation, retention.
- CV and SRS linking.

Acceptance criteria:

- All four formats extract text locally; PDF page count is captured when available.
- Missing/unsupported/image-only/oversized files return the documented codes and keep manual entry available.
- Unconfirmed import writes nothing; confirmed import stores approved text + metadata and not the original file.
- SRS import can be linked to an existing project; a nonexistent project is rejected.
- Imported documents are listed by `imported_at` ascending (oldest first).

### Slice 4 — Repository capture

Build:

- Repository walker with scope, protections, categories, skip reasons, limits, Git metadata, hashing, deterministic ordering, fingerprints.
- Preview/inspect, confirm/save, stale/expired checks, duplicate prevention, refresh, history, view, delete.

Acceptance criteria:

- A fixture repository includes manifests, config, `AGENTS.md`, `README`, docs, TypeScript/Python source/tests; excludes `.env`, `node_modules`, `dist`, private keys, binaries, and ignored files with correct reasons.
- Explicit includes cannot bypass protected directories or key exclusions.
- Per-file/per-capture limits produce skip reasons rather than silent drops.
- A non-Git readable folder captures with `git: null`.
- Save without confirmation fails; source change after preview yields `capture-stale`; app restart yields `preview-expired`.
- Unchanged refresh adds no capture; changed refresh creates version 2; profile edit affects future captures only; history shows versions descending with snapshots.
- Capture view hydrates content by hash; delete removes only that capture and unreferenced content.

### Slice 5 — Refresh, export, and deletion

Build:

- `refreshProjectContext`, export bundle, project deletion confirmation, delete-all confirmation.
- Data/settings surface.

Acceptance criteria:

- Refresh reports unchanged/changed correctly and never duplicates.
- Export includes profile, all profile versions, projects, all project versions, documents, and captures with content.
- Delete project preserves profile/other projects; delete all returns the app to onboarding and preserves schema.
- Every destructive action requires explicit UI confirmation.

### Slice 6 — Coding-agent draft jobs

Build:

- Agent detection/status for Codex, Claude Code, OpenCode, Antigravity with exact capabilities.
- CV/SRS request text, sharing preview, fit check, confirmation, progress, cancellation, timeout, output parsing/validation.
- Review/apply with provenance; manual fallback.

Acceptance criteria:

- All four agents are listed in stable order; unavailable tools are clearly reported; `verified` is false unless demonstrated.
- Preparing a preview starts no process and shows the exact text/bytes/limit/fit.
- Unconfirmed, oversized, or unavailable requests make no agent call.
- Cancelled/timed-out/failed/invalid runs preserve existing profile/project data.
- Schema-valid CV/SRS drafts can be reviewed, corrected, and applied; missing required fields are rejected; provenance is recorded.
- Invalid output and wrong kind return `agent-invalid-output`.
- No credential file is read/copied/stored; no fallback agent/provider/model occurs.

### Slice 7 — Desktop launch, directory, and editor

Build:

- Native window/app composition.
- CLI directory argument, existing-project match by real path, initial directory for new project.
- Installed editor detection and launch.
- Loading/error/success feedback and baseline gate.

Acceptance criteria:

- `coderune` opens without a directory; `coderune <readable-dir>` opens with that directory; multiple args/missing paths fail with actionable errors.
- If the directory matches an existing project root, that project is selected; otherwise the new-project form is pre-filled.
- Installed `code`, `cursor`, `antigravity`, and `zed` are offered; macOS `/Applications` fallback works; choosing one opens the current project/initial directory; launch failure is reported.
- The workspace remains gated until onboarding is complete; reopening a completed profile goes straight to the workspace.
- Unsupported terminal/platform combinations are reported, not faked.

### Slice 8+ — Later increments (exclude from MVP)

- Agent skills and skill library.
- Agent connections, defined subagents, model assignments, and subagent runs.
- Verification brief/checks/results/verdicts/freshness.
- AST/code graph and graph inspection.
- Passive collection, OCR, interactive coding sessions, learning/gamification.
- MCP interface, in-app agent, architecture Q&A, browser hosting.

MVP is done when slices 0–7 pass their acceptance criteria and the security/retention rules in §5–§7 are covered by tests.

---

## 9. Things that must NOT be carried over from the old TypeScript UI

The old desktop UI is an implementation reference for behaviour only. Do not port the following:

1. **The GPUIX/React renderer and `@coderune/ui/native` adapters.** Rust/GPUI types replace them.
2. **The old OneGit-style workspace shell**: icon rail, top tab strip, workspace surface, editor dropdown placement, and all layout/geometry.
3. **Workspace tab mechanics**: `tabs`, `visitedTabs`, closable tabs, tab-close focus restoration, and remount keys. These are UI state, not requirements.
4. **Section-in-one-page composition**: all sections mounted and hidden with `display: none`; `SectionHost`, `WorkspaceContext`, and the single giant workspace state object.
5. **The onboarding single-form with an “Optional profile detail (JSON)” textarea** and debounced 700 ms draft autosave. Use structured optional sections and an explicit draft save.
6. **Raw JSON textareas as the primary editing surface** for profile detail, draft review, and export. Use typed forms/structured review; JSON can remain a read-only export/diagnostic view.
7. **Typed-path-only document/repository selection.** Native file/folder pickers are expected; a typed-path fallback is acceptable, but the old text-input-only flow is not a requirement.
8. **Inline MiB text inputs and string-to-number clamping** (`Math.max(1, Number(value))`). Use validated numeric inputs and positive byte limits; keep the documented defaults.
9. **`process.cwd()` export writes and `coderune-export-<timestamp>.json` naming.** Use a native save dialog and a user-selected path.
10. **`preferred-editor.txt` in the data directory.** Persist the preferred editor through proper settings/storage, not an ad-hoc text file.
11. **“Armed” destructive buttons** where one click arms and a second inline button confirms. Use a native confirmation dialog; keep the explicit confirmation requirement.
12. **The `Data` tab labelled “Settings”** and the assumption that data management is the whole settings surface. A native app may have a real Settings view, but M1 still only needs data export/deletion and editor preference.
13. **GPUIX test IDs, automation hooks, `connectTest`/`connectStdio`, and native test-renderer scaffolding.** These are not product requirements.
14. **React state/race workarounds**: `refreshGeneration`, `selectionInitialized`, remount-by-version keys, effect-driven refresh, and “ignore draft save errors while typing.”
15. **The old CSS/DOM/Tailwind implementation and the old component tree as functional requirements.** Do not copy the old shell or component structure; visual tokens, fonts, and icons are specified separately in `01-design-system.md`.
16. **The `apps/web` marketing scaffold, browser workspace, and web-only code paths.**
17. **Bun-specific APIs as domain contracts**: `Bun.spawn`, `Bun.which`, `Bun.file`, `Bun.CryptoHasher`, `Bun.Glob`, `Bun.write`. The Rust rewrite uses native equivalents while preserving the documented behaviour.
18. **Agent wire messages, adapter internals, or database rows as UI form contracts.** The UI must depend on typed domain models, not raw adapter JSON or row blobs.
19. **The current UI’s agent default-selection heuristic** (prefer a ready non-Codex agent, then any ready agent). Agent selection is a developer choice; no default rule is a domain requirement.
20. **The current generic error strings and copy as literal requirements.** Preserve actionable meaning, error codes, and data-preservation semantics, not exact sentences.
21. **The current absence of document editing/deletion as a designed feature.** Documents are immutable in M1; if the native app adds deletion, it must be a deliberate extension with retention rules.
22. **The old terminal-mode fallback assumptions.** Do not claim Windows/tmux terminal support; report unsupported combinations.

---

## Appendix A — Quick source map

| Concern | Primary source |
| --- | --- |
| Domain types | `packages/core/src/types.ts` |
| Application operations | `packages/core/src/operations.ts` |
| Draft schemas/prompts | `packages/core/src/drafts.ts` |
| Agent contracts | `packages/core/src/agents.ts` |
| Ports | `packages/core/src/ports.ts` |
| Errors | `packages/core/src/errors.ts`, `validation-error.ts` |
| Repository walker | `packages/collectors/src/repository.ts` |
| Document extraction | `packages/collectors/src/documents.ts` |
| SQLite store/migrations | `packages/storage/src/sqlite-store.ts` |
| Data paths | `packages/storage/src/paths.ts` |
| Agent gateway/definitions | `packages/agents/src/gateway.ts`, `definitions.ts` |
| Adapters | `packages/agents/src/adapters/*.ts` |
| Desktop composition | `apps/desktop/src/application.ts` |
| CLI/editor actions | `apps/desktop/src/desktop-actions.ts` |
| Screen operations | `apps/desktop/src/shell/*`, `apps/desktop/src/features/*` |
| Protected behaviour | `apps/desktop/test/*.test.ts*` |
