# Context Bridge

Shared vocabulary for collecting developer and project context for AI coding assistants.

## Language

**Developer profile**: A developer's professional background, education, work and project experience, skills, and learning preferences, shared across that developer's projects. _Avoid_: Project profile, user account

**Developer onboarding**: The collection, review, and confirmation of a developer's background, self-reported skills, and learning goals. _Avoid_: Agent sign-in, project onboarding

**Developer baseline**: A developer's confirmed starting description of their experience, skills, and learning goals, used as a reference for later progress. _Avoid_: Verified mastery, current skill assessment

**Coding agent**: An external assistant selected by the developer to carry out coding-related work using its own configured models and tools. _Avoid_: Model provider, developer profile

**Project context**: A project's requirements, business logic, goals, constraints, acceptance criteria, and knowledge supplied by the developer. _Avoid_: Developer profile

**Repository context**: Information obtained from a selected project repository that describes its contents and state. _Avoid_: Developer profile

**Self-reported skill**: A developer's own statement of their proficiency in a topic. _Avoid_: Verified skill, demonstrated mastery

**Collection scope**: The repository paths and information types that a developer permits the system to collect for a project. _Avoid_: Entire workspace

**Context capture**: A saved, versioned record of repository context observed during a collection, together with its provenance. _Avoid_: Live workspace, graph index
