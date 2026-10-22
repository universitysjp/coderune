# Local code graph options

Source review: 22 October 2026. This is an architecture recommendation for the
later graph increment. The current increment collects and stores local context.

## Recommendation

Use **colbymchenry/codegraph** as the leading target for a later graph adapter.
Its TypeScript implementation, SQLite graph, FTS5 search, and TypeScript/Python
parsers fit this project's direction. This recommendation is an engineering
judgment based on the reviewed source, not a measured performance comparison.

Keep the application's developer profiles, project context, and collected evidence
in its own Bun-managed SQLite store. Treat a graph index as derived data that can
be rebuilt. Graph indexing does not require PostgreSQL, pgvector, or a separate
vector service.

## Source comparison

| Repository | Observed implementation | Implication |
| --- | --- | --- |
| [colbymchenry/codegraph](https://github.com/colbymchenry/codegraph/tree/e8ddc4e2be02578d735c350df55f1769b7d6978a) | TypeScript, SQLite nodes/edges/files and FTS5, Tree-sitter grammars for TypeScript and Python. | Closest match to the selected stack. Its current storage adapter targets Node's `node:sqlite`; direct SDK compatibility with Bun is unverified. |
| [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify/tree/847a93f3cab5e2c82ea9170014b21cd2c52a4417) | Python and a NetworkX/JSON graph pipeline. `extract --code-only` performs local AST extraction without an API key. | Useful code-only option, with an additional Python environment. Mixed-document extraction can invoke configured LLM providers. |
| [codegraph-ai/CodeGraph](https://github.com/codegraph-ai/CodeGraph/tree/590c2ca7dd7b37bd1d41caf45fac535b6522f969) | Rust, default RocksDB graph backend, optional ONNX embeddings. `--graph-only` skips embedding/model loading. | A different implementation from the SQLite-based Codegraph. Adds a native binary and a different graph store. |
| [Cirilcetra/codegraph](https://github.com/Cirilcetra/codegraph/tree/fbf43faf93d8b7c9b343d89bc26f44572ddba7ca) | TypeScript packages, Kuzu, and web-tree-sitter. Embeddings are optional; feature clustering can use an LLM router. | Supports a local graph approach, with additional native-engine compatibility to assess for Bun. |

## Integration implications

- Prefer a separate indexing process for the Colby adapter initially. The
  application's Bun runtime need not load the tool's Node SQLite driver.
- Keep exact source coordinates, content versions, and provenance when relating
  a graph result to captured evidence. Recheck stale or inferred relationships
  before later verification uses them.
- Graphify distinguishes extracted, inferred, and ambiguous edges. Deterministic
  parsing does not make every resolved relationship an explicit source fact.
- Colby's CLI includes telemetry hooks. Audit and configure telemetry and update
  behavior before integrating it under this project's explicit egress policy.
  No graph tool is installed or invoked by the current capture increment.

## Primary source evidence

- Colby: [SQLite/FTS5 schema](https://github.com/colbymchenry/codegraph/blob/e8ddc4e2be02578d735c350df55f1769b7d6978a/src/db/schema.sql), [SQLite adapter](https://github.com/colbymchenry/codegraph/blob/e8ddc4e2be02578d735c350df55f1769b7d6978a/src/db/sqlite-adapter.ts), [language grammars](https://github.com/colbymchenry/codegraph/blob/e8ddc4e2be02578d735c350df55f1769b7d6978a/src/extraction/grammars.ts), [CLI and telemetry hooks](https://github.com/colbymchenry/codegraph/blob/e8ddc4e2be02578d735c350df55f1769b7d6978a/src/bin/codegraph.ts).
- Graphify: [code-only CLI](https://github.com/Graphify-Labs/graphify/blob/847a93f3cab5e2c82ea9170014b21cd2c52a4417/graphify/__main__.py), [edge provenance](https://github.com/Graphify-Labs/graphify/blob/847a93f3cab5e2c82ea9170014b21cd2c52a4417/ARCHITECTURE.md).
- Rust CodeGraph: [backend configuration](https://github.com/codegraph-ai/CodeGraph/blob/590c2ca7dd7b37bd1d41caf45fac535b6522f969/crates/codegraph/Cargo.toml), [graph-only CLI](https://github.com/codegraph-ai/CodeGraph/blob/590c2ca7dd7b37bd1d41caf45fac535b6522f969/crates/codegraph-server/src/main.rs).
- Cirilcetra: [Kuzu dependency](https://github.com/Cirilcetra/codegraph/blob/fbf43faf93d8b7c9b343d89bc26f44572ddba7ca/packages/graph-db/package.json), [optional embeddings and clustering](https://github.com/Cirilcetra/codegraph/blob/fbf43faf93d8b7c9b343d89bc26f44572ddba7ca/packages/ingestion/src/orchestrator.ts).
