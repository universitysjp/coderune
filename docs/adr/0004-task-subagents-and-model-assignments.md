# Defined subagents with inherited model defaults

The developer selects the main coding model, while each defined specialist subagent has a configured default agent connection and model across the developer's connected subscriptions. Coderune routes specific tasks to those defined subagents, and every new subagent run inherits its subagent's default model. Model configuration belongs to the reusable defined subagent; the proposal for individual per-task model overrides was rejected.

External coding tools retain sign-in, credentials, provider configuration, and their own tool execution loops, while Coderune schedules tasks and requests supported model selections. Independent verification runs check the same fixed code snapshot in parallel and report evidence without modifying code. Coderune combines their findings using code evidence and the agreed checks, preserving each finding and its source; unresolved disagreements remain visible as **Needs review**. Teaching consumes the verification results.

Coderune runs the project's configured canonical check commands against the reviewed snapshot and captures the command, exit status, and output as shared execution evidence. Verification subagents interpret these captured results and can suggest additional checks.

Active subagent runs retain their initial connection/model selection, skill contents and versions, code snapshot, and brief. Configuration updates apply to new runs; retries create linked attempts using the current saved assignment while preserving earlier evidence and provenance.

Model availability, task routing, concurrency limits, and other failure handling remain open in the [shaping record](../skills-verification-design.md). The initial six-subagent roster is accepted for now and remains revisable. The existing document-draft adapters do not yet implement this orchestration or explicit model selection.
