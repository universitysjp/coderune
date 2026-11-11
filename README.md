# Coderune

Research into local shared context and assumption verification for AI coding assistants.

## Current stage

Project setup and requirements discussion. The proposal and architecture are in
[docs/presentation.pdf](docs/presentation.pdf).

The first planned milestone is collecting developer and project context and storing
it locally. Architecture and the work breakdown structure will be developed through
`/grill-with-docs` before implementation.

## Setup

```sh
bun install
```

To make the desktop command available on your PATH while developing, run
`bun link` in `apps/desktop` and ensure Bun's global bin directory is on your
PATH. Then run `coderune .` to open the current directory, or `coderune` to
open the workspace. The editor chooser detects the `code`, `cursor`,
`antigravity`, and `zed` commands on PATH, and standard installations under
`/Applications` on macOS. Install the editor's shell command if it does not
appear.

Pele is installed for diagrams. Matt Pocock's skills are installed in `.agents/skills`
and recorded in `skills-lock.json`.

Issues and specs will use [GitHub Issues](https://github.com/universitysjp/coderune/issues).
Skills conventions are documented in [docs/agents/](docs/agents/).
