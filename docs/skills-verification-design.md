# Agent skills and verification shaping

Status: interview in progress; accepted decisions below are design commitments, not implemented behavior or a completed specification.

This continues `/grill-with-docs` for Coderune's context, verification, and learning capabilities. The latest answers replace the earlier six-skill installation, curated default packs, and worktree-local skill delivery proposals.

## Accepted decisions

### Full libraries and developer control

- Install the full pstack library through the standard skills CLI rather than selecting only the six verification skills. Ship the full Matt and pstack libraries by default.
- Settings lets the developer toggle agent skills on or off and add their own skills. Curated role packs are no longer the default shipping model.
- Keep same-name skills from different sources, including `tdd` and `teach`, in their respective source collections. The developer chose source folders rather than a flat application library; no default for an unqualified skill name has been selected.
- Repository development tooling under `.agents/skills` and `skills-lock.json` is separate from the product's application-managed skill library. This shaping record does not install or replace development tooling.

### Application-owned storage and delivery

- Keep installed agent skill folders in application-owned storage and load them through Coderune. Do not install them into the developer's project repository or an isolated code worktree.
- Use real skill folders and adapter-specific loading from application-owned locations where an adapter supports it. Use prompt injection only when an adapter cannot load those skills directly. The exact per-adapter mechanism still needs investigation.
- Coderune owns orchestration in application code; do not create a fourth orchestrator skill.

### Source collections

Group installed skills into folders by source in the application layer. Multiple skills from the same source belong together; Coderune-authored skills have their own collection. An illustrative excerpt under the application-owned skill directory is:

```text
skills/
  coderune/
    coderune-context/SKILL.md
    coderune-verify/SKILL.md
    coderune-teach/SKILL.md
  mattpocock/
    tdd/SKILL.md
    teach/SKILL.md
  pstack/
    tdd/SKILL.md
    teach/SKILL.md
```

The full libraries still ship; the tree shows only examples. At the application layer, the collection and skill name together distinguish entries such as `mattpocock/tdd` and `pstack/tdd`. Settings should preserve this source grouping. Native adapter loading and any unqualified aliases remain separate decisions; source folders alone do not prove an adapter supports nested discovery.

### On-demand loading

Enabled skills are discoverable by name and description, with their source collection visible. Load full skill instructions only when a skill is selected for a job, and read supporting resources as needed. Shipping the full libraries does not mean including every skill's instructions in every job's context. Disabled skills are excluded from the job's discovery catalog.

### Defined subagents and model assignments

- The developer selects the main coding model. Coderune coordinates the primary coding workflow and assigns specific tasks to defined specialist subagents.
- Each defined subagent has its own configured default agent connection and model. Every new task assigned to that subagent starts a subagent run that inherits its configured default; the model is configured on the defined subagent, not chosen separately for every task.
- Multiple defined subagents can use different available connections and models across the developer's connected subscriptions. Selecting the main coding model does not replace the defaults configured for those subagents.
- The Q10 proposal for task-type presets with individual per-task model overrides was rejected. The accepted controls are selection of the main coding model and configuration of model defaults on defined subagents.
- This extends the earlier draft-only design's agent-owned model selection for the later orchestration workflow: Coderune records and requests the selected connection/model, while sign-in, credentials, and provider configuration remain with the external coding tool. Available models and selectable profiles must be established per adapter; a connected subscription alone is not proof that an arbitrary model is available.
- After a coding change is ready, independent verification tasks run in parallel against the same fixed code snapshot. Verification subagent runs report evidence without editing the code. Coderune combines their findings, and teaching follows from the verification results.
- A defined subagent's model assignment is distinct from its agent skills. Each subagent run discovers enabled skills and loads the instructions it needs on demand.

Task breakdown and routing, supported adapter controls, and scheduling/failure policies remain to be shaped.

### Initial defined subagents

The developer accepted this initial roster for now; it remains open to revision:

| Defined subagent | Responsibility |
| --- | --- |
| Context reviewer | Review relevant, approved project context. |
| Claim extractor | Identify assumptions and anchor claims to the diff. |
| Code verifier | Check claims against code, requirements, and check evidence. |
| Impact analyst | Examine affected behavior and regression risks. |
| Teacher | Explain findings at the developer's baseline. |
| Learning evaluator | Evaluate optional developer attempts. |

Each defined subagent has its own configured connection/model default, which new tasks inherit, and loads relevant skills on demand.

### Unavailable model recovery

When a subagent's configured connection/model is unavailable, show an explicit **Unavailable** state with **Retry** and **Select another model** actions. Let the developer select an available replacement through the defined subagent's saved connection/model configuration, consistent with the accepted default-inheritance model. Retry the affected task using that configured assignment and preserve completed findings. Model replacement is an explicit developer choice.

### Run inputs and retry attempts

Active subagent runs retain the connection/model selection, skill contents and versions, code snapshot, and verification brief captured when they started. Changes to configured defaults or installed skills apply to new runs. A retry creates a new linked attempt using the current saved assignment, preserving the original attempt's evidence and provenance.

### Evidence and disagreement policy

Determine verification verdicts using code evidence and the agreed checks. Preserve every subagent's finding with its source so the developer can inspect conflicting results. When the evidence and checks do not resolve a disagreement, mark the unresolved conflict as **Needs review** and keep it visible in the combined result.

### Canonical checks

Coderune runs the project's configured check commands against the reviewed code snapshot and captures the command, exit status, and output. Share this execution evidence with all verification subagents, which interpret the captured results. Agents can also suggest additional checks.

Offer **Run once** and **Save to project checks** for agent-suggested checks. **Run once** contributes execution evidence to the current review. **Save to project checks** adds the check to the project's configured set for future reviews.

### Verification brief and result contract

Use a versioned, structured contract for `coderune-verify`.

The verification brief contains:

- The exact code snapshot and diff to verify.
- The permitted scope, requirements, and acceptance criteria.
- Candidate claims to investigate.
- The required checks.

The verification result contains:

- Extracted assumptions and per-claim verdicts.
- Code references and observed evidence.
- Check results, explicitly identifying checks that did not run.
- Unresolved conflicts and missing evidence.
- The subagent, model, and skill versions used for the run.

The contract's fields and versioning are accepted.

### Claim verdicts

Each claim uses one of four verdicts:

| Verdict | Meaning |
| --- | --- |
| Verified | Evidence supports the claim and required checks passed. |
| Contradicted | Observed evidence disproves the claim. |
| Insufficient evidence | Required evidence or check results are missing. |
| Needs review | Conflicting findings remain unresolved. |

A check that cannot execute leaves incomplete evidence. An observed behavioral failure can contradict a claim. Record the actual check outcome and evidence used for the claim's verdict.

### Overall verdict

Apply these rules in order to the in-scope claims and required checks:

| Condition | Overall verdict |
| --- | --- |
| Any in-scope claim is contradicted. | Contradicted |
| Otherwise, conflicting findings remain unresolved. | Needs review |
| Otherwise, evidence is incomplete, required checks have not passed, or no claims were assessed. | Insufficient evidence |
| Every in-scope claim is verified and required checks passed. | Verified |

Keep claim counts, individual findings, and the separate freshness flag visible alongside the overall verdict.

### Result freshness

Verification verdicts remain attached to the exact code snapshot and requirements that were reviewed. If the reviewed code or requirements change, preserve the historical verdict and mark the result as **stale** for the current state until verification runs again. Freshness is separate from the claim verdict: an earlier **Verified** result does not establish that the changed code is verified.

### Teaching from mixed verification results

`coderune-teach` can explain current **Verified** claims even when the overall review has problems. Explain verified behavior for the reviewed snapshot, contradicted claims as mistakes, and unresolved claims as open questions. Discuss other claims with their explicit verdicts and evidence, and keep stale results clearly tied to their original snapshot.

### Learning evidence and participation

Demonstrated progress comes from optional, evaluated developer attempts, such as explaining a change, predicting behavior, answering a quiz, or completing a small exercise. Save the concept, prompt, developer response, evaluation, and linked code snapshot as learning evidence. Record lesson views as participation, and keep demonstrated progress separate from the self-reported developer baseline.

### First Coderune-authored skills

- `coderune-context`: capture, scope, preview, and review project context.
- `coderune-verify`: extract assumptions, anchor claims, request canonical checks, and produce a verdict.
- `coderune-teach`: explain verified code at the developer's baseline and emit learning evidence.

### Provenance and updates

- Vendor pinned third-party copies with attribution and a sync manifest recording the source repository, commit, license, and local hash. Updates are deliberate, reviewed changes.
- Do not fetch skills during agent jobs. Showing available external-skill updates in Settings is a separate management operation.
- Coderune-authored skills update with the application. Settings can show when external skill updates are available. Developer-defined skills are updated by the developer.
- Settings offers **Check for updates** and **Update** per external source collection, showing the installed and available revisions and a change summary.
- Retain the previous revision for rollback. Apply installed updates to new runs; active subagent runs retain their captured skill contents and versions.

### Resolution precedence

Resolve overlapping skills in ascending precedence: Coderune built-ins, imported packs, project skills, then user skills. Show the resolved source in the UI. Reading project-provided skills does not authorize writing skill folders into the project.

## Current implementation boundary

The existing app supports CV/SRS document drafts through one selected coding agent, using temporary working directories. It has no application skill registry, skill settings, update flow, job-specific skill loading, parallel verification scheduler, verification brief, or verdict engine. Repository-installed development skills do not constitute product support for these capabilities.

Current draft requests select only a coding-tool identifier. They have no agent connection/profile identity, defined-subagent or run identity, model assignment, or model inventory, and adapters inherit the connected tool's configuration. These are later-workflow capabilities to add, not existing support for multiple subscriptions.

The accepted collection increment in [data-collection-design.md](./data-collection-design.md) remains distinct from this later work. The [work breakdown structure](./wbs/work-breakdown-structure.md) places assumption verification and learning in later milestones.

## Open design branches

- Deferred at the developer's request: unqualified skill names and ambiguous short-name resolution. Revisit later; Q24 was not accepted or rejected. Source folders and distinct collection-qualified entries remain accepted, and no short-name default has been selected.
- Adapter delivery from application storage, including unsupported capabilities and fallback behavior.
- Task routing and dependencies, model availability, and connection/profile capabilities. The six-subagent initial roster is accepted for now and remains revisable; configured default changes apply to new runs, and retries are linked attempts.
- Snapshot creation and retention, resource limits, cancellation, and partial failure for subagent runs. Unavailable model feedback, retry, and explicit replacement selection are accepted, alongside per-subagent configured defaults, parallel verification against one snapshot, separate result freshness, and teaching after verification.

Continue the interview before producing the final specification and implementation tickets. No product implementation is authorized by an unanswered design proposal.

Related decision: [Application-owned agent skills](./adr/0003-application-owned-agent-skills.md).

Related decision: [Defined subagents with inherited model defaults](./adr/0004-task-subagents-and-model-assignments.md).
