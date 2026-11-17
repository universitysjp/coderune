# Versioned personal context retrieval with SQLite FTS5

Coderune will keep the complete developer profile JSON and its immutable versions as the source of truth. A generated projection of the current profile will support exact field lookup and SQLite FTS5 retrieval for coding, explanation, and quiz purposes. Each result will be an immutable, purpose-filtered, byte-limited personal context bundle with source references, a policy version, and a SHA-256 hash.

This design makes personal context searchable without sending the whole profile to agents. It supports personalized teaching while keeping personal data out of verification verdicts. A normal FTS5 table with copied derived text and triggers is preferred over external-content FTS. Embeddings, a new retrieval crate, and a custom in-memory cache are deferred.

## Considered Options

- **Read and filter the current JSON in memory.** This is simpler for a small profile and remains a valid fallback. It was rejected for the target design because explanations, quizzes, and richer personal records need one retrieval model that can grow without changing its caller contract.
- **Normalize profile data into the authoritative schema.** This was rejected because it creates a second profile write model and complicates the existing export/history contract. Only the search projection is normalized; it is disposable and rebuildable.
- **External-content FTS5.** This was rejected because it adds row-ID, trigger, and delete-order consistency risks while saving little text for one profile.
- **Embeddings or a remote retrieval service.** This was rejected for the first implementation because exact fields and local lexical retrieval are deterministic, inspectable, and do not send personal data to another provider.

## Consequences

Profile confirmation must replace the projection and FTS records in the same transaction as the profile version. Existing databases need a tested backfill and rebuild path. Raw task text must not become FTS query grammar. Sharing rules must filter SQL candidates before denied text is returned. Approved bundles must be retained separately from the disposable search projection so future agent runs and teaching results can reference their exact inputs.

Changing the projection, sharing policy, bundle schema, or canonical serialization requires explicit versioning. Search relevance does not grant sharing permission and does not establish verification truth. Performance claims about FTS or caching require measurements.
