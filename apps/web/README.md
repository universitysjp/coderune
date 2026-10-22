# Context Bridge website

Research project site for Context Bridge, built with React, TanStack Router, Tailwind CSS v4, Base UI, and Phosphor icons.

## Stack

- React 19 with the React Compiler
- TanStack Router (file-based routes, no TanStack Start)
- Tailwind CSS v4 via `@tailwindcss/vite`
- shadcn/ui components backed by Base UI
- Phosphor icons
- Shared design tokens from `@context-bridge/tokens`
- TypeScript 7, Vite+, Oxlint, Oxfmt

## Commands

```sh
bun run dev
bun run build
bun run check-types
bun run lint
bun run fix
bun run preview
```

Run commands from `apps/web`.

## Routes

- `/` overview and verification demo
- `/research` problem, research questions, objectives
- `/design` architecture, data boundary, module scope
- `/evaluation` three-condition study plan
- `/team` team, institution, milestones

Routes live in `src/routes`. The router plugin generates `src/routeTree.gen.ts`.

## Design tokens

`@context-bridge/tokens` provides the Paper Mono variable font and the light cream, ink, and blue palette in OKLCH. The package exposes:

- `@context-bridge/tokens` for `tokens.css`
- `@context-bridge/tokens/PaperMono-Variable.woff2` for the font

Semantic names follow shadcn conventions (`--background`, `--foreground`, `--primary`, `--accent`, and so on) and are mapped into Tailwind through `@theme inline` in `src/index.css`.
