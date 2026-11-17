# Developer profile retrieval architecture review

Status: architecture review, 17 November 2026. This review compares the proposed personal-context design with primary sources.

**Update:** personalized explanations and quizzes were later confirmed as first-class retrieval consumers. ADR 0006 and [the implementation specification](personal-context-retrieval-spec.md) accept FTS5 with the privacy, exact-lookup, provenance, and bundle-hash controls identified here. The earlier recommendation to defer FTS no longer describes the accepted target.

## Verdict

The design has the right product rules:

- Keep one authoritative developer profile and immutable versions.
- Search the current profile by default.
- Share only fields needed for the task's purpose.
- Keep provenance and the exact profile version in the result.
- Put Coderune between the database and external agents.
- Start without embeddings or a remote retrieval service.

The proposed two-table FTS5 index is possible, but it is too much machinery for the first implementation. One developer profile has a bounded set of fields, and the current domain model has no measured size or query-latency target. Load the current profile once and select fields in memory first. Hide that choice behind the same `PersonalContextBundle` interface. Add persistent entries and FTS5 only after measurements show a need.

Do not copy a large-codebase retrieval design into personal-profile retrieval without evidence. The stronger ideas from other systems are scoped instructions, bounded context, stable provenance, explicit agent boundaries, and iterative access to more evidence.

## What the compared systems establish

| System | Primary-source observation | Relevance to Coderune |
| --- | --- | --- |
| Zed instructions | Zed separates always-on personal/project instructions from on-demand skills. Its agent documentation also says explicitly supplied context can improve response quality and reduce latency. [Instructions](https://zed.dev/docs/ai/instructions), [Agent Panel](https://zed.dev/docs/ai/agent-panel). | Keep a compact pinned baseline separate from task-dependent profile entries. Do not put every profile field in always-on context. |
| Zed profiles and external agents | Zed profiles control available tools. Tool permissions are a separate control. Zed profiles do not generally apply to external agents, which own their runtime, authentication, tools, and native configuration. [Agent Profiles](https://zed.dev/docs/ai/agent-profiles), [External Agents](https://zed.dev/docs/ai/external-agents). | Coderune must apply its field-sharing policy before invoking the agent. Do not assume the agent will preserve Coderune's native profile rules. |
| Zed MCP | Zed can connect agents to external tools and data through MCP. Configured MCP servers can also be forwarded to external agents over ACP. [MCP](https://zed.dev/docs/ai/mcp). | A future `search_personal_context` tool fits the proposed boundary. It must expose a restricted operation, not the SQLite file or arbitrary SQL. |
| Delta / DeltaDB | DeltaDB records fine-grained edits and conversations together. Every delta has a stable identity. References attach to a delta rather than only a line number. [DeltaDB](https://zed.dev/blog/introducing-deltadb), [Delta](https://zed.dev/blog/introducing-delta). | Keep exact links from task to context bundle, agent run, generated change, and profile version. This will matter more than an FTS ranking score during verification and explanation. The public material reviewed here does not establish DeltaDB's profile-search or ranking implementation. |
| Continue | Continue now recommends agent exploration tools, rules, and MCP for codebase awareness. Its deprecated `@Codebase` provider combined locally calculated embeddings with keyword search, then selected a larger candidate set before a smaller final set. [Current codebase guidance](https://docs.continue.dev/guides/codebase-documentation-awareness), [Legacy `@Codebase`](https://docs.continue.dev/reference/deprecated-codebase). | Do not assume embeddings are required. Add iterative retrieval later, and separate candidate retrieval from final budgeted selection if a search index becomes necessary. |
| Aider | Aider sends a compact map with key symbols, uses dependency-graph ranking for large maps, and applies a token budget. The model can request full files when it needs more detail. [Repository map](https://aider.chat/docs/repomap.html). | A compact summary plus on-demand detail is useful. Aider's graph ranking addresses code structure, not a developer profile. Do not copy that ranking model into personal data. |

## Review of the proposed design

### Strong parts

**One source of truth.** Keep `developer_profiles` and `developer_profile_versions` authoritative. Search data must be a generated projection that can be discarded and rebuilt.

**Current-version reads.** Pin one profile version for each bundle. Later saves must not alter a bundle that an agent already received.

**Purpose-based sharing.** Field permission must be independent of text similarity. A strong match for an employer's name does not make that employer useful to a coding task.

**Mediated agent access.** Coderune should provide the initial bundle itself. A future agent tool can request more fields through the same restricted operation. Never rely on the agent to issue the first retrieval call.

**Deterministic ordering.** Fixed tie-breaking and byte-limited output make tests and reviews repeatable.

**Provenance.** A result should identify its profile ID, profile version, entry kind, and exact source within that version.

### Problems to fix before implementation

#### 1. FTS5 is not justified for the first profile retriever

SQLite describes FTS5 as a way to select matching documents from large collections. That is not evidence that it helps one small profile. [SQLite FTS5 overview](https://sqlite.org/fts5.html).

The first implementation can read the current profile once, project its fields into entries in memory, apply policy and matching, and return the bundle. This avoids a migration, duplicate text storage, index update logic, and a rebuild path. It also preserves the same caller interface for a later SQLite index.

#### 2. Personal context is not proof of code correctness

The glossary calls these entries self-reported skills and a developer baseline. They describe the developer. They do not establish project requirements or observed code behavior. Verification can use them to choose an explanation level or investigate a stated preference. It must use exact code evidence, requirements, and check results for verdicts.

#### 3. Entry identity needs a precise scope

A JSON array pointer such as `selfReportedSkills[2]` identifies a field only inside one saved profile version. An edit can reorder the array. Do not present that pointer as a stable identity across versions.

Use `(profile_id, profile_version, source_path)` as the identity for the first in-memory projection. Add durable item IDs only when the product needs to track one item across edits. Delta's stable-delta model is a reason to preserve exact versioned references, not a reason to copy DeltaDB.

#### 4. A search index needs its own version and consistency rules

If FTS5 is added later, record the projection/version that produced each entry. Existing profile rows need a backfill. Confirmation, history, generated entries, search-index changes, and draft deletion must have clear transaction behavior. Retrieval must read the pinned profile version and candidates from one consistent database view.

SQLite makes the application responsible for consistency between an external-content FTS5 table and its content table. It documents triggers and the `rebuild` command. Inconsistent rows and index data can produce unpredictable results. [External content tables](https://sqlite.org/fts5.html#external_content_tables), [FTS5 pitfalls and rebuild](https://sqlite.org/fts5.html#external_content_table_pitfalls).

A normal FTS5 table with copied search text is simpler for a small derived index. External content needs an integer row ID and triggers. Do not choose external content just to avoid small text duplication.

#### 5. Raw task text is not a safe FTS5 expression

FTS5 `MATCH` has its own phrases, boolean operators, prefix operators, quoting, and tokenization. Binding the whole expression as a SQL parameter prevents SQL injection, but does not make arbitrary natural-language input a valid or intended FTS query. Tokenize task text in Coderune and quote each term under one defined AND/OR policy. [FTS5 query syntax](https://sqlite.org/fts5.html#full_text_query_syntax).

The default `unicode61` tokenizer separates punctuation. Exact skill matching therefore needs normalized topics and curated aliases rather than relying only on raw task text. Test names such as `C++`, `.NET`, `JavaScript`, and `TypeScript`. [FTS5 tokenizers](https://sqlite.org/fts5.html#tokenizers).

#### 6. Similarity cannot override privacy

Apply purpose rules before candidate ranking, logging, or prompt rendering. `requested_sections` must not expand access beyond the purpose unless the developer gives explicit permission. Replace the vague phrase “clearly relevant” with a written policy and exact input-preview behavior.

FTS5 ranks matching text. It cannot decide whether a field is necessary or true. Pass exact, typed entries through the adapter and show the developer the exact bundle before sharing.

#### 7. The budget needs a unit and coverage rule

Do not claim a model-independent exact token count. Start with entry-count and UTF-8 byte limits. Count the rendered labels, source references, and instructions as well as entry text. Later adapters can apply model-specific token accounting when such an interface exists.

Reserve space for the required compact baseline. If that baseline exceeds the budget, report an error rather than silently dropping required context or exceeding the limit. Put a cap on profile field sizes before indexing them.

#### 8. The bundle needs a lifetime beyond retrieval

A transient result is enough for the first unit test. Before an actual agent run, persist or otherwise retain the exact bundle, policy version, profile version, selection reasons, omissions, byte count, and sharing approval with that run. Otherwise Coderune cannot later establish what the agent saw.

This is the main lesson from Delta for the later verification engine: preserve the relationship between the task, context, conversation, and generated change.

#### 9. Performance and retrieval quality need measurements

The current design has no maximum profile size, query-latency target, relevance corpus, or threshold for adopting FTS5. Add fixtures with many skills, goals, education, employment, and project entries. Measure index-building cost, retrieval latency, database growth, and result quality before choosing an index.

A useful evaluation set includes exact matches, aliases, no matches, repeated entries, stale versions, restricted fields, and budget truncation. Test both a missing result and an incorrect result. Empty search results must not cause the whole profile to be returned.

## Recommended first implementation

Keep the public result proposed earlier but simplify its implementation:

```text
retrieve_personal_context(query) -> PersonalContextBundle

1. Read the current DeveloperProfile once.
2. Pin its ID and version.
3. Derive entries in memory with exact source paths.
4. Remove fields denied by the task purpose.
5. Add the compact required baseline.
6. Match normalized task terms and requested sections.
7. Use stable ordering and apply entry/byte limits.
8. Return selected entries, provenance, reasons, and omissions.
```

Place domain request/result types in `crates/domain`. Let `coderune-app` expose the operation to callers and keep UI and agent protocols out of it. Use `coderune-storage` to read the profile. Do not add a new `retrieval` crate or a trait with only one implementation yet.

The interface hides candidate selection, policy, ranking, budgeting, and provenance. A future SQLite/FTS5 adapter can replace the in-memory candidate search without changing agent adapters or UI callers.

## If measurements later justify FTS5

Add the two proposed tables with these corrections:

- Give `profile_context_entries` an integer row ID.
- Unique-scope entries by profile, version, and source path.
- Store a projection version and content hash.
- Index `(profile_id, profile_version, kind)`.
- Use a normal FTS5 table unless measured storage costs justify external content.
- If external content is used, install insert/update/delete triggers and a tested backfill/rebuild command.
- Write the profile, history, projection, and index changes in one transaction.
- Query only an explicitly pinned version and a purpose-approved entry set.
- Convert natural-language task text to a safe FTS expression in code.
- Keep BM25 only as one ranking input, with exact-topic and requested-section boosts. SQLite returns lower `bm25()` values for better matches. [BM25](https://sqlite.org/fts5.html#the_bm25_function).
- Export authoritative profile data, not disposable search-index contents. Delete search rows during scoped profile/all-data deletion and test index rebuilding after reopening.

## Required checks for the first implementation

- Current profile only; explicit historical lookup remains separate.
- Exact source paths point into the pinned JSON version.
- Experience and skills baseline survives ordinary budget truncation.
- Identity, links, education, and employment do not leak into coding bundles by default.
- Explicit section requests do not bypass the field policy.
- Exact topics and aliases produce stable results with no FTS query-syntax errors.
- No match returns only the allowed baseline, never the full profile.
- Duplicate entries appear once and ties have a fixed order.
- Byte limits include rendering overhead; oversized baseline reports an error.
- Agent adapters receive the bundle but no database connection.

## Decision

Approve the product rules and the `PersonalContextBundle` seam. Defer persistent entries and FTS5 until a measured profile size or retrieval-quality problem requires them. This keeps the architecture extensible without making a small first feature carry an unnecessary index and migration lifecycle.
