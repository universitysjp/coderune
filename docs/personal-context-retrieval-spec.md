# Personal context storage and retrieval

Status: implementation specification. The implementation tickets require review before publication. GitHub publication is pending because the `gh` CLI is unavailable.

## Problem Statement

Coderune saves one complete developer profile as JSON. Agent operations need only selected parts of that profile. Coding agents need relevant technical context. Explanations and quizzes also need the developer's experience, goals, and learning preferences. Sending the complete profile wastes context and can expose personal data that the task does not need.

The current storage has no selective retrieval operation, field-level sharing policy, searchable profile projection, or immutable record of the personal context used for an agent operation.

## Solution

Add a versioned personal-context projection inside the existing per-user SQLite database. Build it from the confirmed developer profile. Keep the complete profile JSON and its history as the source of truth.

Expose one application operation that prepares a personal context bundle. It reads only the current confirmed profile, performs exact field lookups and FTS5 text search, applies the task's sharing policy before reading candidate content into Rust, and returns a bounded bundle with provenance and a deterministic hash.

Profile confirmation replaces the current projection in the same database transaction. Opening an existing database creates or refreshes the projection. Preparing a bundle has no agent or network side effects. A separate operation records the exact bundle after the developer confirms sharing. Later agent runs can reference that hash.

Personal context can adapt explanations and quizzes. It does not establish code correctness. Verification verdicts continue to use code evidence, project requirements, and check results.

## User Stories

1. As a developer, I want my confirmed profile indexed locally so that Coderune can find relevant personal context.
2. As a developer, I want existing databases upgraded without losing my profile or onboarding state.
3. As a developer, I want profile edits to replace the searchable current projection so that old skills do not enter new tasks.
4. As a developer, I want older profile versions and approved bundles kept unchanged so that old work remains explainable.
5. As a developer, I want exact requests such as `learningPreferences` to read that field directly instead of relying on text matching.
6. As a developer, I want requests for a normalized skill such as `skill:rust` to find every matching entry in the current profile.
7. As a developer, I want a free-text task about Rust and databases to find matching skills and project experience.
8. As a developer, I want Coderune to use a fixed ranking and order so that repeated queries return the same result.
9. As a developer, I want a coding operation to include experience and my skills statement without sending unrelated personal history.
10. As a developer, I want explanation and quiz operations to use relevant goals and learning preferences.
11. As a developer, I want no matching search results to produce only the permitted baseline, not the complete profile.
12. As a developer, I want restricted or purpose-denied fields excluded before candidate text is read into memory.
13. As a developer, I want a requested field to remain denied when the task's purpose does not allow it.
14. As a developer, I want the context size limit to include labels, source references, and rendering overhead.
15. As a developer, I want an error when the required baseline exceeds the limit instead of silent truncation or an oversized prompt.
16. As a developer, I want every selected entry linked to its exact source in one profile version.
17. As a developer, I want an unchanged query against an unchanged profile and policy to produce the same bundle hash.
18. As a developer, I want Coderune to retain the exact approved bundle so that later reviews can show what the agent received.
19. As a developer, I want preparing a bundle to leave the agent disconnected and send no data.
20. As a developer, I want delete-all to remove searchable personal data and approved bundle history.
21. As a developer, I want exports to include authoritative profiles and approved bundles without disposable search-index data.
22. As an agent adapter, I want one typed bundle so that I do not receive SQLite access or build profile queries myself.
23. As a future teacher, I want personal context tied to verification findings without using that context as proof of code correctness.

## Implementation Decisions

### Module boundaries

- The domain model owns personal-context purposes, sensitivity classes, queries, entries, bundle contracts, and the versioned sharing-policy definition.
- Storage owns projection generation, migrations, FTS5 consistency, exact and text candidate queries, index rebuilding, and approved-bundle persistence.
- The application owns the single retrieval orchestration operation, baseline and candidate ordering, budget enforcement, bundle rendering, hashing, and the sharing-confirmation operation.
- Existing desktop views and agent adapters do not own profile queries. They do not change in this increment.
- Do not add a retrieval crate, a storage trait with one implementation, or another profile cache. Revisit those choices only when real consumers or measurements require them.

The highest test seam is the application retrieval operation. Storage tests cover migration, persistence, and transaction invariants that the application contract cannot expose directly.

### Retrieval purposes

Use `coding`, `verification`, `explanation`, and `quiz` as versioned purposes.

- `coding` permits the experience and skills baseline and relevant technical skills or project experience.
- `verification` selects no personal entries by default. It never uses a profile to prove code correctness.
- `explanation` can also select learning goals and learning preferences.
- `quiz` can also select relevant learning goals and learning preferences. Later evaluated learning evidence can extend this policy through a new policy version.

Each entry has one sensitivity class:

- `technical`: skills statements, self-reported skills, and project experience.
- `personal`: display name, experience stage, new-to-coding state, learning goals, education, employment, certifications, links, and learning preferences.
- `restricted`: reserved for future data that needs explicit per-field consent. No current profile field receives this class by default.

Sensitivity describes data. The sharing policy decides which sensitivity and entry-kind combinations each purpose can select. Do not compare sensitivity classes with numeric or string ordering. Query explicit allowed values.

Requested fields never expand the purpose's permissions. Preparing an explanation for an external agent still shares personal data externally, so the exact sharing preview and confirmation rules apply.

### Domain contracts

`PersonalContextQuery` contains:

- Free-text task text, which can be empty for exact-only retrieval.
- One purpose.
- Zero or more requested logical field paths.
- Maximum entry count and maximum UTF-8 rendered bytes.

The initial operation uses the current confirmed profile. It does not accept an arbitrary profile version. Historical profile reads already exist, and approved bundles retain their own exact source version.

`PersonalContextEntry` contains:

- Entry kind, logical field path, and exact JSON source path.
- Sensitivity and searchable text.
- Structured payload and optional field-source provenance.
- Content hash.

`PersonalContextBundle` contains:

- Bundle schema version, profile ID, and profile version.
- Purpose and sharing-policy version.
- The canonical query and size limits.
- Ordered selected entries and stable selection reasons.
- Bounded omission records that contain counts and reason codes, not denied personal text.
- Exact rendered text and its UTF-8 byte count.
- SHA-256 bundle hash.

The bundle schema and policy versions start at 1 despite the database migration number. Change them only through an explicit compatibility decision.

### Search-entry identity

Use two different paths:

- `field_path` is a logical lookup key, such as `experienceStage`, `learningPreferences`, or `skill:rust`. Several entries can share a requested field path.
- `source_path` is a JSON Pointer into the pinned profile version, such as `/selfReportedSkills/2`. Array indexes identify a position only within that version.

`(profile_id, profile_version, source_path)` identifies an indexed entry. Do not treat a source-path index as a durable item identity across profile edits.

Projection uses the current profile's normalized field values, lower-case normalized technology topics for logical skill lookup, and canonical JSON for payload hashes. Empty entries are omitted. Entry output order is deterministic.

### SQLite schema

Add `profile_context_entries` with these columns:

| Column | Rule |
| --- | --- |
| `id` | Integer primary key and FTS row ID. |
| `profile_id` | Current developer profile ID; required. |
| `profile_version` | Confirmed current version; required. |
| `entry_kind` | Controlled profile-entry kind; required. |
| `field_path` | Logical lookup path; required. |
| `source_path` | Version-scoped JSON Pointer; required. |
| `sensitivity` | `technical`, `personal`, or `restricted`; required. |
| `search_text` | Plain text generated for search; required. |
| `payload` | Canonical structured entry JSON; required. |
| `field_source` | Nullable JSON containing existing field-source provenance. |
| `content_hash` | SHA-256 of the canonical entry payload; required. |
| `projection_version` | Projection algorithm version; required. |
| `created_at` | Projection timestamp; required, excluded from bundle hashes. |

Add a unique key on `(profile_id, profile_version, source_path)`. Add an index on `(profile_id, profile_version, field_path)` and another on `(profile_id, profile_version, sensitivity, entry_kind)`.

Only the current profile appears in this table. Profile edits replace its entries. Old entries are unnecessary because profile JSON history and approved bundles preserve earlier evidence.

Add `profile_context_fts` as a normal FTS5 virtual table with `search_text`. Its row ID equals the entry ID. It stores a derived copy of search text rather than using external content. This avoids external-content delete/update pitfalls for a small index.

Add insert and delete triggers on `profile_context_entries`. New entries insert matching FTS rows. Removed entries delete their exact FTS rows using the special FTS delete operation and the old text. Entries are immutable; a profile edit deletes and rebuilds the current projection rather than updating rows.

Add `profile_context_index_state` with:

- `profile_id` as primary key.
- Indexed `profile_version`.
- `projection_version`.
- `updated_at`.

The state identifies stale or missing projections. A current-version or projection-version mismatch triggers a synchronous rebuild when the database opens. Retrieval reports an infrastructure error rather than using stale entries if the state is inconsistent. No fallback returns the whole profile.

Add `personal_context_bundles` with:

| Column | Rule |
| --- | --- |
| `bundle_hash` | SHA-256 text primary key. |
| `schema_version` | Bundle serialization version; required. |
| `profile_id` | Source profile ID; required. |
| `profile_version` | Exact source version; required. |
| `purpose` | Purpose used for selection; required. |
| `policy_version` | Sharing-policy version; required. |
| `data` | Exact canonical bundle, including its hash; required. |
| `byte_count` | Rendered UTF-8 byte count; required. |
| `confirmed_at` | Sharing-confirmation timestamp; required. |

Store only bundles whose sharing was explicitly confirmed. `confirmed_at` records approval; it does not claim that an agent run occurred. A future agent run references `bundle_hash`. Inserting the same approved bundle twice returns the existing record without changing its first confirmation time.

Profile and profile-version foreign keys use existing unique identities. Delete bundles before profile history during delete-all. Derived entries and index state are disposable. Project deletion does not affect either table.

### Transactions and migrations

The next SeaORM migration creates the entry, FTS, state, and bundle tables, indexes, triggers, and foreign keys. Migration tests confirm that the linked SQLite build supports FTS5 and fail at startup with a database error if it does not.

Profile confirmation performs these writes in one transaction:

1. Write the current profile JSON and immutable version.
2. Delete the old current projection and its triggered FTS rows.
3. Insert the new projection and FTS rows.
4. Update the index state.
5. Delete the onboarding draft.

Opening an existing or new database runs migrations, then checks current profile/index state. A missing or stale projection is deleted and rebuilt in one transaction. An explicit rebuild operation uses the same implementation.

Bundle preparation reads the profile, index state, exact entries, and text candidates through one consistent storage operation. Exact and text reads agree on one pinned profile version.

Approved-bundle persistence validates that the supplied hash matches its canonical payload before writing it.

### Exact and text retrieval

Exact lookup uses indexed `(profile_id, profile_version, field_path)` predicates. Normalize known aliases before constructing the query. Unknown requested paths return no field entries and remain visible as omission reason codes.

FTS5 is only for free-text task descriptions. Convert task text into safe tokens in Rust. Quote terms under a fixed OR expression. Bound the expression length and token count. Never concatenate raw task text as FTS grammar.

Join FTS rows to normal entries and apply the pinned version plus exact sensitivity and entry-kind permissions in SQL. Use a bounded candidate count. Do not read denied rows into Rust for later filtering.

Baseline fields come from exact policy paths, not FTS. No text match returns the permitted baseline only. Exact requests and text candidates are combined and deduplicated by identity, then by identical selected content.

### Ranking and budgeting

Required baseline entries keep their fixed order. Requested exact fields follow. Remaining matches use exact-topic boost, then FTS5 `bm25()` ascending, then field path and source path ascending. Lower BM25 values rank first. Internal floating-point scores do not enter canonical bundle serialization or hashes.

Use fixed candidate and query limits in policy version 1. Do not expose an unbounded `LIMIT` parameter.

Apply entry-count and UTF-8 byte limits to the complete rendered bundle, including instructions, labels, provenance, and punctuation. Reserve baseline space first. Remaining entries are added only when complete. `over-budget` omissions retain stable reason codes and counts.

A zero or otherwise invalid budget is a validation error. If the baseline exceeds the limit, return a validation error. Do not truncate profile facts mid-entry, silently drop baseline fields, or send an oversized result.

### Canonical bundle and hash

Canonicalize a bundle core containing the query, metadata, selected entries, reasons, omissions, and exact rendered text. Use struct-defined field order, explicitly sorted arrays, normalized newlines, and UTF-8. Exclude timestamps, database row IDs, and raw ranking floats.

Hash the canonical core bytes with SHA-256. Store the hash with the core. Use the existing project SHA-256 dependency rather than adding BLAKE3 now.

The hash identifies the exact personal context and policy used. It does not encrypt the data or prove that an agent used it correctly. A later verification result can link this hash to its generated-code snapshot. Profile changes mark future uses as belonging to another snapshot; they never rewrite an old bundle or finding.

### Export and deletion

Bump the export schema to version 2 and include approved personal-context bundles. Do not export derived entries, FTS data, or index state. They can be rebuilt from the authoritative profile.

Delete-all removes approved bundles, derived search state, and profile records in foreign-key-safe order. Search triggers remove FTS rows. Reopening the empty database returns to onboarding with no personal entries or approved bundle history.

### Caching

Do not add an in-memory profile cache in this increment. The application already retains its current profile for display. Retrieval correctness takes priority over speculative optimization.

Measure database reads, index rebuilds, result quality, and database growth first. A later cache must hold immutable versioned projections and handle writes from every supported process. Any future `Arc`, `RwLock`, or `arc-swap` choice needs actual concurrent readers.

## Testing Decisions

Test external behavior through the application retrieval and bundle-confirmation operations. Use storage integration tests for migrations, foreign keys, triggers, index state, and transaction recovery. Follow the existing in-memory and temporary-database tests in storage and application, plus domain validation tests.

Required test groups:

1. Fresh migration creates every table, index, trigger, and FTS capability.
2. A temporary database containing an existing profile opens with a complete backfilled projection and no profile data changes.
3. Confirmation writes profile history, projection, FTS rows, index state, and draft removal atomically.
4. Profile version 2 replaces current version 1 entries. Earlier JSON versions and approved version 1 bundles remain unchanged.
5. Exact scalar and normalized skill lookups return stable source paths and all matching entries.
6. Empty and unknown lookups return no whole-profile fallback.
7. Free-text Rust/database queries match expected entries and rank equal results deterministically.
8. Quotes, boolean words, operators, wildcards, long input, and punctuation cannot change or break the intended FTS query policy.
9. Purpose rules exclude denied personal data before candidate results reach application ordering.
10. Coding, verification, explanation, and quiz purposes expose only their accepted baseline and allowed candidates.
11. No match returns only the permitted baseline. An empty baseline is valid only where policy permits it.
12. Requested fields do not override purpose permissions or leak denied values in omissions.
13. Duplicate entries and content are returned once with fixed source-priority rules.
14. Entry and rendered-byte limits are enforced, including overhead. Oversized baselines return errors without truncation.
15. Repeated retrieval over the same snapshot produces byte-identical canonical bundles and hashes. Profile edits produce a new version-linked result.
16. Bundle tampering is rejected before persistence. Reopening returns the exact approved bundle and original confirmation timestamp.
17. Preparing a bundle leaves approved-bundle history empty and creates no agent calls.
18. Delete-all removes personal entries, FTS matches, index state, approved bundles, profiles, and drafts.
19. Export version 2 contains profiles and approved bundles but excludes derived search records.
20. The full storage and application suites pass, followed by the repository's relevant checks.

Use fixtures for no matches, exact matches, aliases such as JavaScript/TypeScript where explicitly defined, punctuation-heavy technologies such as C++ and .NET, long education and employment entries, duplicate skills, and reordered arrays. Alias handling is curated and test-driven; no semantic equivalence is inferred.

## Out of Scope

- Coding-agent invocation, MCP tools, ACP changes, and adapter-specific tokenizers.
- Desktop or web UI changes.
- Project-context chunking, code parsing, embeddings, vector search, and repository-graph retrieval.
- Verification briefs, claims, verdicts, checks, generated-code diff capture, or verification tables.
- Summary, explanation, teaching, or quiz generation.
- Evaluated learning evidence and mastery tracking.
- Cross-version stable identity for profile array items.
- A new profile cache or concurrent agent-retrieval runtime.
- Multiple developer profiles in one database.
- Automatic synonym discovery or profile claims inferred from code.

## Further Notes

This specification follows the accepted decisions in ADR 0006. The earlier comparative research remains useful for its cautions about FTS consistency, privacy, provenance, and unmeasured optimization. Its initial recommendation to defer FTS was revised after personalized explanations and quizzes became explicit first-class retrieval consumers.

No implementation is authorized for work outside this specification. GitHub issue publication and implementation-ticket publication remain pending.

## Delivery status

Ticket 1 is implemented: profile confirmation and database opening maintain the current `profile_context_entries` projection and index state, and `coderune-app` exposes exact-field retrieval through `PersonalContextBundle`. FTS5 search, approved-bundle history, export version 2, agent adapters, and UI remain for later tickets.
