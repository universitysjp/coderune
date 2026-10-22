# Data collection and storage interview

Status: interview in progress. Record accepted answers here as the design develops.

## Accepted decisions

- The first increment collects the developer profile, project context, and selected repository context.
- The first milestone allows the developer to collect, inspect, and save that context locally.
- The interface is a CLI built with OpenTUI from the OpenCode team.
- One developer profile is reused across projects. Each project has its own goals, rules, and collection permissions.
- Implement the first increment in TypeScript with Bun and SQLite.
- Do not use PostgreSQL/pgvector. Prefer local AST and graph tooling with embedded storage for later code structure retrieval.
- Collect repository context on explicit request, with an initial capture and a manual refresh operation.
- The initial repository capture covers Git state, dependency manifests, project configuration, AGENTS.md, README, selected documentation, and permitted source and test files.
- Support TypeScript and Python repositories through file-based capture initially. AST and code graph extraction follow in a later increment.
- Enter developer profiles and project context manually initially. Capture experience, self-reported skills and levels, learning preferences, project goals, rules, and acceptance criteria.
- CV import and inferred skill assessment follow in later increments.
- Assumption verification and developer learning belong to later increments.
- The optional Eve/learning subagent is outside the implementation scope.

## Graph tooling recommendation

The leading future graph adapter is `colbymchenry/codegraph`, using its local
SQLite graph and FTS5 search. Its current driver targets Node, so direct Bun SDK
compatibility is not assumed. Keep graph integration in the later increment and
review its process boundary and egress controls then. The application's SQLite
store remains responsible for developer profiles, project context, and collected
evidence. See [the source comparison](graph-tool-comparison.md) for the alternatives
and the distinction between extracted and inferred graph relationships.

## Decisions still to resolve

- Module boundaries and the future graph adapter's exact integration contract.
- Required versus optional onboarding fields and the self-reported skill scale.
- Collection permissions, exclusions, and handling of sensitive content.
- Storage location, updates, history, deletion, and duplicate captures.
- OpenTUI renderer, navigation, cancellation, and failure recovery.
- Acceptance criteria for the first increment.

## Work breakdown structure instructions

Create the WBS in `docs` using Pele after the interview is complete and the design is confirmed.
Draft created early per request: [work-breakdown-structure.md](work-breakdown-structure.md).
Use these three team members, as named on the final slide of [the proposal](presentation.pdf):

- Pruthivi Thejan
- Nuwan Konara
- Raviru Rathnaweera

Mark tasks selected for this session with a tick and track completion separately.
Assign this session's implementation tasks to Pruthivi Thejan, with the database
layer tasks assigned to Nuwan Konara. Include all three team members in the project WBS.
