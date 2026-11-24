---
description: Coderune monorepo guide. Use Bun for tooling, Rust/Rune for the desktop app, and follow the skills workflow.
globs: "*.ts, *.tsx, *.html, *.css, *.js, *.jsx, *.rs, *.toml, package.json"
alwaysApply: false
---

Instructions in this document appear in no particular order, so read and cache them in your working memory. One thing not to do is erase work while you are working.

# Preserve user changes

- Do not overwrite, restore, reset, or discard user-authored changes.
- Before editing a file, inspect its current contents and working-tree diff. Use narrow patches that preserve unrelated edits.
- Recheck the working tree before each write. Treat newly appearing changes as active user work and preserve them; if an edit conflicts, stop and ask how to proceed.

# AI Development Skills & Workflows (Strictly follow this, do not let a feature be implemented without going through these steps and making docs and tickets)

## 1. Getting Started

- **`/setup-matt-pocock-skills`**  
  Configures your repository once. It sets up your issue tracker, triage configurations, and documentation guidelines so the AI works seamlessly with your existing tech stack.

- **`/ask-matt`**  
  An intelligent routing skill. If you don't know what to do next, invoke it to analyze your current context and recommend the correct skill or workflow.

## 2. The Main Flow: Idea → Ship

- **`/grill-with-docs`**  
  Interviews you about your feature plan one question at a time while examining your codebase. It ensures your intent aligns with your system and updates `GLOSSARY.md` and Architecture Decision Records (ADRs) inline.

- **`/to-spec`**  
  Converts the active conversation into a durable technical specification and maps it to your issue tracker without further interviews.

- **`/to-tickets`**  
  Breaks a specification or plan into tightly isolated, “tracer-bullet” developer tickets while tracking exact cross-task dependencies.

- **`/implement`**  
  The core execution engine. It reads a ticket or specification, triggers individual test-driven cycles, and concludes with a rigorous code review before committing.

### Task-sized commits

- Use `/to-tickets` to split approved work into small, independently reviewable tasks.
- In `/implement`, finish one task, its required documentation and checks, and `/code-review` before committing it.
- Make one focused commit per task. Keep each commit to the smallest coherent set of files needed for that task; split independent work instead of combining it into a feature-wide or catch-all commit.
- Use a descriptive commit subject that names the task and scope, such as `feat(storage): add capture history`.
- After the task commits are ready, check the current branch and upstream, push them to that upstream, and report the commit hashes and push result.

- **`/code-review`**  
  Uses parallel sub-agents to audit the code diff across two axes:
  - **Standards:** Fowler code smells and repository guidelines
  - **Specification:** Whether the implementation fulfills the requested feature

- **`/retro`**  
  Runs at the end of a session to evaluate workspace health and suggest improvements to the environment, testing tools, or steering flags.

## 3. Shaping: Managing Complex Ambiguity

- **`/wayfinder`**  
  Used when a project is too large for a single AI context window. It constructs an overarching map of decision tickets in your issue tracker so you can systematically navigate the “fog of war.”

- **`/prototype`**  
  Instructs the agent to rapidly assemble throwaway code—such as single-file HTML state maps or toggleable UI branches—to answer isolated design questions.

- **`/research`**  
  Spawns a background agent to investigate open questions using high-trust primary resources, returning a structured and cited Markdown log.

## 4. Engineering: Codebase Health & Maintenance

- **`/improve-codebase-architecture`**  
  Scans your project for architectural weaknesses and deep-module opportunities, visualizes them in an HTML report, and guides you through fixing the issue you select.

- **`/diagnosing-bugs`**  
  Applies a disciplined debugging loop:

  1. Build a failing test loop.
  2. Minimize the reproducible surface area.
  3. Form a hypothesis.
  4. Instrument the system.
  5. Apply the fix.
  6. Verify against regressions.

- **`/triage`**  
  Acts as a state machine for managing inbound bugs or tracker issues, systematically moving them through predefined organizational roles and labels.

- **`/implement-spec`**  
  Handles large tasks by generating independent, concurrently running sub-agents that work across available project tickets.

## 5. Productivity: Human-to-AI Workflows

- **`/grill-me`**  
  A lightweight, aggressive interrogation tool that challenges raw technical ideas through immediate back-and-forth questioning to uncover edge-case requirements before code is generated.

- **`/handoff`**  
  Compresses active session memory into a concise Markdown artifact so a new AI agent can continue exactly where the previous one stopped.

- **`/to-questionnaire`**  
  Converts questions that the developer cannot answer alone into a clean Markdown format suitable for emailing or messaging an asynchronous team member.

- **`/teach`**  
  Turns the agent into an interactive, step-by-step mentor that uses active codebase files as a live workspace for teaching concepts or frameworks.

- **`/wait-what`**  
  Triggered when a model response feels misaligned or introduces confusion. It immediately pauses the workflow to recalibrate intentions.

## 6. Reference Skills: Background Enforcers

- **`/tdd`**  
  Enforces strict Red–Green–Refactor cycles during implementation.

- **`/codebase-design`**  
  Provides background guidelines focused on “deep modules,” ensuring the agent writes dense behavior behind simple, isolated interfaces.

- **`/domain-modeling`**  
  Prevents the AI from using generic terminology by aligning code structure directly with the defined glossary.

## 7. Other installed skills

Skills live in `.agents/skills/` and are recorded in `skills-lock.json`. Besides the ones above, the repository also has design-system skills (`build-design-system`, `component-docs`, `design-system-boss`, `migrate-design-system`, `token-mapping`, `ui-review`, `oklch-skill`), `untangle-codebase`, `rust-best-practices`, `turborepo`, `setup-ts-deep-modules`, `pr`, `wizard`, and writing skills (`writing-for-agents`, `writing-beats`, `writing-fragments`, `writing-shape`). Use `rust-best-practices` for Rust changes and `turborepo` for `turbo.json` or task-pipeline changes.

# Project overview

Coderune researches local shared context and assumption verification for AI coding assistants. It is a Bun + Turborepo monorepo with a native Rust desktop app and a React website. The proposal and architecture are in `docs/presentation.pdf`.

| Path | What it is |
| --- | --- |
| `apps/desktop` | Native Rust desktop app built with [Rune](https://github.com/anomalyco/rune) (`coderune-desktop`, binary `coderune`). `src/app.rs` holds state and actions, `src/screens/` has one view module per screen, `src/widgets.rs` has shared controls. |
| `apps/site` | Website: React 19, Vite+, Tailwind v4, and ui-sdk. `src/routes` holds the five pages, `src/components` the site's own frame and sections, and `src/app.tsx` picks a page from the pathname. |
| `crates/ui` | Rune design system for the desktop: tokens, icons, cards, fields, alerts. |
| `crates/avatars` | Animated bot avatars for agents, ported from `bot-avatars` onto Rune's vector canvas. `src/engine.rs` is the rig, `src/render.rs` draws a pose as `rune::Shape`. |
| `crates/domain`, `crates/storage`, `crates/collectors`, `crates/agents`, `crates/app` | Framework-independent domain and application operations (SeaORM/SQLite storage, context collectors, coding-agent connections). |
| `packages/tokens` | Design tokens and fonts shared by the website and the desktop. The site imports `src/tokens.css`; the TTF files are bundled locally. |
| `scripts/` | Repo scripts: `setup-rune.sh`, `verify-ui.sh`, `render-wbs.ts`, and `skills-to-typescript.ts`. |
| `docs/` | `adr/`, `rune/` (desktop notes), `wbs/` (work breakdown), `agents/` (skills conventions), plus design notes. |

Architecture rules:

- Screens never own data, and service crates never import the UI. Keep the desktop views thin over `coderune-app`.
- Keep desktop UI logic in `crates/ui` and `apps/desktop`. The website keeps its own React components in `apps/site` and reads `packages/tokens`; do not add a shared React UI package.
- Local-only directories are git-ignored: `.rune/` (pinned Rune checkout), `.hex/`, `target/`, `.design-system/`, `.ui-review/`, `.playwright-cli/`, `.turbo/`. Never commit them and never edit the vendored crates under `.rune/vendor/`.

# Commands

Run everything from the repository root. Use Bun, never npm, yarn, or pnpm.

```sh
bun install                 # install JS dependencies
bun run setup:rune          # clone the pinned Rune checkout into .rune (required before any cargo command)
bun run dev                 # serve the website
bun run dev:desktop         # desktop only (cargo run -p coderune-desktop also works)
bun run build               # turbo build
bun run build:site          # static website into apps/site/dist
bun run build:desktop       # release desktop build
bun run check               # vp check: format, lint, and types for JS/TS
bun run fix                 # vp check --fix
bun run check-types         # tsc through turbo
bun run check:desktop       # cargo check for the desktop app
bun run check:site          # TypeScript check for the website
bun run lint                # lint apps and crates (oxlint and clippy)
bun run format              # format apps and crates (scoped, so vendored Rune members are untouched)
bun run test                # tests for apps and crates
bun run test:desktop        # desktop tests only
bun run verify:ui           # render every desktop scene headlessly into target/ui-snapshots
bun run wbs:render          # re-render the work breakdown structure
bun run skills:ts           # convert installed skill scripts to TypeScript
```

Notes:

- `rust-toolchain.toml` pins Rust (`1.99.1`, with `rustfmt` and `clippy`). Cargo cannot resolve the workspace without `.rune`, so run `bun run setup:rune` first on a fresh clone.
- Run `cargo fmt` only through `bun run format`. Calling it directly rewrites the vendored Rune workspace members.
- Turborepo drives the Cargo workspace (`experimentalCargoWorkspaces`). Vendored Rune patches are workspace members only so Turbo can hash them; they are not default members.
- A pre-commit hook runs `vp check --fix` on staged `css, js, jsx, json, jsonc, md, ts, tsx` files. Do not bypass it.
- Markdown under `docs/`, `.agents/`, `AGENTS.md`, `GLOSSARY.md`, and `README.md` is excluded from formatting.
- Before finishing a task, run the narrowest relevant checks (`bun run check`, `bun run check-types`, `bun run test`, `bun run check:desktop`) and report what you ran.

# Bun conventions

Default to using Bun instead of Node.js.

- Use `bun <file>` instead of `node <file>` or `ts-node <file>`
- Use `bun test` instead of `jest` or `vitest` for standalone scripts; workspace tests go through `bun run test`
- Use `bun install` instead of `npm install`, `yarn install`, or `pnpm install`
- Use `bun run <script>` instead of `npm run <script>` or equivalents
- Use `bunx <package> <command>` instead of `npx <package> <command>`
- Bun automatically loads `.env`, so don't use dotenv.
- Use `Bun.file` over `node:fs`'s readFile/writeFile, and Bun.$`ls` instead of execa.
- Prefer built-in Bun APIs (`bun:sqlite`, `Bun.sql`, `Bun.redis`, `WebSocket`, `Bun.serve()`) over `better-sqlite3`, `pg`, `ioredis`, `ws`, or `express`. Coderune's persistent storage is Rust-side SeaORM/SQLite, so these mostly apply to scripts.

For standalone scripts:

```ts
import { test, expect } from "bun:test";

test("hello world", () => {
  expect(1).toBe(1);
});
```

## Website tooling

The website is React, built with Vite+ (`vp`) and deployed as static files to GitHub Pages.

- Run the site through `bun run dev` (or `bun run dev:site`), not by invoking `vite` directly.
- `bun run build:site` runs the site's TypeScript check and Vite build into `apps/site/dist`; the build emits every route's `index.html` and a `404.html`.
- The site keeps its components in `apps/site/src/components`; there is no shared React UI package.
- Use [ui-sdk](https://ui-sdk.dev) components (`@ui-sdk/react/<component>`) for behavior-heavy parts instead of Base UI. Before using them, read `node_modules/@ui-sdk/react/docs/llms.txt`; the `ui-sdk` skill in `.agents/skills/ui-sdk` has the workflow.
- Colors and fonts come from `packages/tokens`; map semantic colors through the existing Tailwind theme in `src/index.css` instead of adding one-off values.
- TypeScript still runs the agent skills and repository scripts; do not remove Bun or the workspace packages for those.

For Bun API details, read the docs in `node_modules/bun-types/docs/**.mdx`.

# Agent skills

## Issue tracker

Track issues and specs in `universitysjp/coderune` using GitHub Issues with the `gh` CLI. See `docs/agents/issue-tracker.md`.

## Triage labels

Use the five standard triage labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

## Domain docs

Use a single project glossary (`GLOSSARY.md`) and ADR directory (`docs/adr/`). See `docs/agents/domain.md`. Use the glossary's terms in code, tests, and issues, and record decisions with meaningful reversal cost as ADRs.
