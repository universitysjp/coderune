# Project workspace

Home offers a new chat for the current project or a first-project action.
The sidebar lists projects and their context readiness. Select a project to
open its settings, or use its disclosure to show chats. Each open chat gets
a closable titlebar tab. The Workspace tab returns to the previous page
and project selection.

Project settings show readiness for the folder, goals, collection scope, and
first capture. Create a project with its name, folder, and goals. Set paths
and capture limits in the Capture scope disclosure. An empty path list uses
the default collection scope.

## Run a coding agent

Choose an enabled agent and select **Start agent**. The agent starts in
that project's folder with its own sign-in, permissions, models, and settings.
The panel can dock left or right inside the chat or float above the composer.
Type into the terminal for interactive prompts, or use the composer to send
a line. F6 returns focus from the terminal to the panel controls.

Each open tab keeps its own running process and draft input. Closing the tab
stops the process. Coderune saves the chat, its chosen agent, and composer
messages locally. Starting the agent again creates a fresh process. The agent
owns its session history, and Coderune keeps terminal output in memory while
the terminal stays open.

## Verify

Use `bun run test:desktop` for workspace navigation, chat persistence, and
terminal input tests. Use `bun run verify:ui` to render Home, Project settings,
and the docked and floating chat scenes.
