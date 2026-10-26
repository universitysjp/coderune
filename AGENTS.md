---
description: Use Bun instead of Node.js, npm, pnpm, or vite.
globs: "*.ts, *.tsx, *.html, *.css, *.js, *.jsx, package.json"
alwaysApply: false
---

Instructions in this document appear without a order so read and cache them on your temp. One thing not to do is erase work while you are working. 

# Preserve user changes

- Do not overwrite, restore, reset, or discard user-authored changes.
- Before editing a file, inspect its current contents and working-tree diff. Use narrow patches that preserve unrelated edits.
- Recheck the working tree before each write. Treat newly appearing changes as active user work and preserve them; if an edit conflicts, stop and ask how to proceed.

# AI Development Skills & Workflows (Strickly follow this, do not let a featuere implement without going through these steps and making docs and tickets)

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

Default to using Bun instead of Node.js.

- Use `bun <file>` instead of `node <file>` or `ts-node <file>`
- Use `bun test` instead of `jest` or `vitest`
- Use `bun check` instead of `tsc --noEmit`
- Use `bun build <file.html|file.ts|file.css>` instead of `webpack` or `esbuild`
- Use `bun install` instead of `npm install` or `yarn install` or `pnpm install`
- Use `bun run <script>` instead of `npm run <script>` or `yarn run <script>` or `pnpm run <script>`
- Use `bunx <package> <command>` instead of `npx <package> <command>`
- Bun automatically loads .env, so don't use dotenv.

## APIs

- `Bun.serve()` supports WebSockets, HTTPS, and routes. Don't use `express`.
- `bun:sqlite` for SQLite. Don't use `better-sqlite3`.
- `Bun.redis` for Redis. Don't use `ioredis`.
- `Bun.sql` for Postgres. Don't use `pg` or `postgres.js`.
- `WebSocket` is built-in. Don't use `ws`.
- Prefer `Bun.file` over `node:fs`'s readFile/writeFile
- Bun.$`ls` instead of execa.

## Testing

Use `bun test` to run tests.

```ts#index.test.ts
import { test, expect } from "bun:test";

test("hello world", () => {
  expect(1).toBe(1);
});
```

## Type checking

Use `bun check` to type check the project. It reads `tsconfig.json` and reports the same errors as `tsc`.

- `bun check src/index.ts` checks one file and what it imports
- `bun --check src/index.ts`, `bun test --check` and `bun build --check` type check first, and do nothing else if there is an error

## Frontend

Use HTML imports with `Bun.serve()`. Don't use `vite`. HTML imports fully support React, CSS, Tailwind.

Server:

```ts#index.ts
import index from "./index.html"

Bun.serve({
  routes: {
    "/": index,
    "/api/users/:id": {
      GET: (req) => {
        return new Response(JSON.stringify({ id: req.params.id }));
      },
    },
  },
  // optional websocket support
  websocket: {
    open: (ws) => {
      ws.send("Hello, world!");
    },
    message: (ws, message) => {
      ws.send(message);
    },
    close: (ws) => {
      // handle close
    }
  },
  development: {
    hmr: true,
    console: true,
  }
})
```

HTML files can import .tsx, .jsx or .js files directly and Bun's bundler will transpile & bundle automatically. `<link>` tags can point to stylesheets and Bun's CSS bundler will bundle.

```html#index.html
<html>
  <body>
    <h1>Hello, world!</h1>
    <script type="module" src="./frontend.tsx"></script>
  </body>
</html>
```

With the following `frontend.tsx`:

```tsx#frontend.tsx
import React from "react";
import { createRoot } from "react-dom/client";

// import .css files directly and it works
import './index.css';

const root = createRoot(document.body);

export default function Frontend() {
  return <h1>Hello, world!</h1>;
}

root.render(<Frontend />);
```

Then, run index.ts

```sh
bun --hot ./index.ts
```

For more information, read the Bun API docs in `node_modules/bun-types/docs/**.mdx`.

## Agent skills

### Issue tracker

Track issues and specs in `universitysjp/coderune` using GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five standard triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use a single project glossary and ADR directory. See `docs/agents/domain.md`.
