# Coderune

Shared vocabulary for Coderune's developer and project context, agent skills, verification, and learning.

## Language

**Developer profile**: A developer's professional background, education, work and project experience, skills, and learning preferences, shared across that developer's projects. _Avoid_: Project profile, user account

**Developer onboarding**: The collection, review, and confirmation of a developer's background, self-reported skills, and learning goals. _Avoid_: Agent sign-in, project onboarding

**Developer baseline**: A developer's confirmed starting description of their experience, skills, and learning goals, used as a reference for later progress. _Avoid_: Verified mastery, current skill assessment

**Coding agent**: An external coding assistant such as Claude Code, Codex, or OpenCode that uses its own configured models and tools. The Settings section for installed coding agents is labeled Agents. _Avoid_: Subagent, model provider

**Agent connection**: A developer-configured connection to a coding agent and the models available through it. _Avoid_: Subscription, model provider

**Bot avatar**: The animated character that represents a subagent in the desktop app, showing whether it is idle, working, or asleep. _Avoid_: Coding agent logo, profile picture

**Subagent** (defined subagent): A reusable Coderune specialist with a defined purpose and a configured default model assignment. The workspace destination for these specialists is labeled Subagents. _Avoid_: Coding agent, individual task, agent skill

**Subagent run**: One execution of a task assigned to a defined subagent, inheriting that subagent's configured default model. _Avoid_: Subagent definition, individual model choice

**Model assignment**: The agent connection and model configured for a defined subagent, independently of the main coding model selected by the developer. _Avoid_: Per-task override, agent skill selection

**Agent skill**: Reusable instructions and supporting resources that guide a coding agent through a particular kind of work. _Avoid_: Developer skill, self-reported skill

**Skill library**: The collection of agent skills available through Coderune, including Coderune-authored, third-party, and developer-added skills. _Avoid_: Developer profile, skill assessment

**Skill collection**: A group of agent skills from the same source within Coderune's skill library. _Avoid_: Developer skill set

**Project context**: A project's requirements, business logic, goals, constraints, acceptance criteria, and knowledge supplied by the developer. _Avoid_: Developer profile

**Repository context**: Information obtained from a selected project repository that describes its contents and state. _Avoid_: Developer profile

**Self-reported skill**: A developer's own statement of their proficiency in a topic. _Avoid_: Verified skill, demonstrated mastery

**Collection scope**: The repository paths and information types that a developer permits the system to collect for a project. _Avoid_: Entire workspace

**Context capture**: A saved, versioned record of repository context observed during a collection, together with its provenance. _Avoid_: Live workspace, graph index

**Verification brief**: The agreed scope, requirements, candidate claims, and checks for verifying one exact code snapshot and diff. _Avoid_: Verification result

**Canonical check**: A project-configured verification check whose observed execution result is shared evidence for the reviewed code snapshot. _Avoid_: Agent assertion

**Verification result**: The recorded findings and verdicts from verification, including observed evidence, check outcomes, unresolved gaps, and the source of the findings. _Avoid_: Verification brief

**Verification finding**: An observation reported by a verification subagent about a code change, together with its evidence and source. _Avoid_: Verified fact

**Verification verdict**: Coderune's assessment of a claim or code change based on code evidence and the agreed checks. _Avoid_: Agent consensus

**Verified**: A claim verdict indicating that evidence supports the claim and its required checks passed. _Avoid_: Plausible, agent agreement

**Contradicted**: A claim verdict indicating that observed evidence disproves the claim. _Avoid_: Check could not run

**Insufficient evidence**: A claim verdict indicating that required evidence or check results are missing. _Avoid_: Verified, contradicted

**Needs review**: A verification outcome indicating a disagreement that the available evidence and agreed checks have not resolved. _Avoid_: Verified, resolved

**Stale**: A freshness marker indicating that a verification result applies to earlier reviewed code or requirements rather than the current state. _Avoid_: Contradicted, insufficient evidence

**Learning evidence**: An evaluated developer attempt used to assess understanding of a concept and progress over time, linked to the relevant code snapshot. _Avoid_: Lesson view, self-reported skill

**Personal context entry**: A source-addressable item derived from one developer profile version for selective retrieval. _Avoid_: Independent profile record, verified skill

**Personal context bundle**: An immutable, purpose-filtered and size-limited selection of personal context entries, with its profile version, policy version, source references, and content hash. _Avoid_: Complete developer profile, verification brief

**Personal context purpose**: The kind of operation for which personal context is prepared, determining the profile entries and personal data that may be selected. _Avoid_: Retrieval score, sharing permission by text match

**Context sharing policy**: The versioned rules that determine which personal context entries a purpose may select and bundle for a consumer. _Avoid_: Sensitivity ordering, agent-owned privacy rule
