# Desktop workflow decisions

Meeting record, 11 November 2026. This captures product direction; it does not imply that the deferred capabilities are implemented.

## Direction

- Coderune is a desktop-only application.
- Put an agent inside the application to obtain user context. Its relationship to external coding agents and their execution loops remains to be designed; see [ADR 0004](adr/0004-task-subagents-and-model-assignments.md) for the earlier external-agent arrangement.
- Provide a terminal command that opens a directory in Coderune, comparable to `code .`.
- Let a developer open the current project in a preferred installed IDE from the application.
- Enforce standards for agent skills while allowing users to customize them. The exact standards and customization boundaries remain open; see [ADR 0003](adr/0003-application-owned-agent-skills.md).
- Consider an MCP interface for other agents to call the application. This is optional, particularly if the agent is inside the app; no decision to build it has been made.
- Guide architecture decisions for a new project through agent-led questions using an agent skill. The questions and resulting decision workflow remain to be designed.

## Current implementation scope

Implement the directory-opening terminal command and the installed-editor launch action. Adapt the desktop shell to the current Hex/RuneKit layout: a `SplitView` sidebar with profile and Data at the bottom, a framed main pane, and the selected project in the toolbar. (The earlier OneGit icon-only rail and top tab strip are superseded; Coderune does not add a second navigation model.) Use Coderune's UI design guidelines for typography, color and controls. Existing application capabilities should remain accessible.

The in-app agent, skill standards and customization, MCP interface, and architecture Q&A are recorded for future work and are outside this implementation scope.
