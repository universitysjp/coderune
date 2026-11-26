# Native project chat uses structured provider sessions

Project chat owns its locally displayed transcript, agent choice, model, and
reasoning effort. The coding agent runs in the project folder using its existing
sign-in and settings. Coderune stores the provider session ID with the chat so
the coding agent can continue across turns and app restarts.

Use the coding agent's structured interface to receive replies and approval
requests. Codex uses App Server, Claude Code uses stream JSON, and OpenCode
and Antigravity use the Agent Client Protocol. These interfaces let the app
distinguish text, session identity, and permissions without parsing terminal
output. Each adapter is responsible for preserving its provider's permissions
and sending explicit one-time decisions.

We considered keeping a PTY and reading text back from it. Terminal redraws
erase message boundaries and make model discovery and approval unreliable.
Structured adapters carry an ongoing integration cost as coding-agent protocols
change. The terminal remains available for separate shell work. Later context
capture and assumption verification can consume project chat state through
Coderune's application layer.

[Project chat specification](https://github.com/universitysjp/coderune/issues/47)
