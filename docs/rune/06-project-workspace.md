# Project workspace

Home offers a new chat for the current project or a first-project action.
The sidebar lists projects and their context readiness. Select a project to
open its settings, or use its disclosure to show chats. Each open chat gets
a closable titlebar tab. The Workspace tab returns to the previous page
and project selection. The gear at the right end of the sidebar profile row opens Settings.

Project settings show readiness for the folder, goals, collection scope, and
first capture. Create a project with its name, folder, and goals. Set paths
and capture limits in the Capture scope disclosure. An empty path list uses
the default collection scope.

## Run a coding agent

Choose an installed, enabled coding agent in the composer. Native project chat
supports Codex, Claude Code, OpenCode, and Antigravity CLI. Choose a model and,
when the agent offers it, a reasoning effort. Click **Send** or press
**Command+Enter** on macOS or **Ctrl+Enter** elsewhere. The agent uses the
project folder and its own sign-in and permissions. When a supported action
requires approval, the composer shows the action and asks for a one-time decision.

The agent logo identifies each conversation in the sidebar. Prompts sit on the
right and replies on the left. Replies render Markdown, including lists, code
blocks, tables, and diagrams; links open in the browser and the text is
selectable. A turn can continue when its tab is closed. Use **Stop** in the
composer to cancel it. Coderune saves prompts, replies, and model choices locally
and reuses the provider session for later turns. Selecting a different agent starts
a session with that agent and keeps the existing transcript visible.

**Command+J** on macOS or **Ctrl+J** elsewhere opens a separate terminal in the
active project folder. Coderune uses the system launcher or the executable
specified by `TERMINAL`.

Open **Settings → Agents** to see installed coding agents, their installed
versions when detectable, and available updates where Coderune can verify them.

## Verify

Use `bun run test:desktop` for workspace navigation and chat persistence.
Use `cargo test -p coderune-agents --locked` for the provider protocol checks
and `bun run verify:ui` for dark, light, and narrow chat renders.
