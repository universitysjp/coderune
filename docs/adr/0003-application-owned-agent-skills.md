# Application-owned agent skills

Coderune will ship full, pinned Matt and pstack skill libraries alongside its own context, verification, and teaching skills, with Settings controls to enable or disable skills and add developer-defined skills. Store and load installed skill folders from application-owned storage rather than populating project repositories or code worktrees; adapters may use prompt injection when they cannot load those folders. This gives developers the full libraries while preserving repository ownership and reproducible job inputs: third-party copies carry source, commit, license, and hash provenance, and agent jobs do not fetch skills at runtime.

Group skills from the same source in a collection folder within application storage, such as `mattpocock/`, `pstack/`, or `coderune/`, and retain that grouping in Settings. Collection-qualified identities keep both versions of same-name skills available without one source overwriting another.

Coderune-authored skills update with the app, and developers update their own skills. Settings offers **Check for updates** and **Update** per external source collection, showing installed and available revisions and a change summary, with the previous revision retained for rollback. Updates apply to new runs while active runs retain their captured skill contents and versions.

Resolve sources in ascending precedence from Coderune built-ins through imported packs and project skills to user skills, and show the active source. Unqualified aliases and native adapter loading remain open in the [shaping record](../skills-verification-design.md).
