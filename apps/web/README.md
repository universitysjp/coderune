# Coderune website

Research project site for Coderune, built with React, TanStack Router, Tailwind CSS v4, Base UI, and Phosphor icons.

## Stack

- React 19 with the React Compiler
- TanStack Router (file-based routes, no TanStack Start)
- Tailwind CSS v4 via `@tailwindcss/vite`
- Shared component registry from `@coderune/ui/web`, backed by Base UI
- Phosphor icons
- Shared design tokens from `@coderune/tokens`
- TypeScript 6, Vite+, Oxlint, Oxfmt

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

## GitHub Pages

The web workflow builds and publishes changes from `main` to
[Coderune on GitHub Pages](https://universitysjp.github.io/coderune/).
It passes the configured Pages base path to Vite. The build emits a static
entry point for every navigation route so shared links work on a direct visit.

## Routes

- `/` overview and verification demo
- `/research` problem, research questions, objectives
- `/design` architecture, data boundary, module scope
- `/evaluation` three-condition study plan
- `/team` team, institution, milestones

Routes live in `src/routes`. The router plugin generates `src/routeTree.gen.ts`.

## Design tokens

`@coderune/tokens` provides the Geist and Paper Mono variable fonts and the light cream, ink, and blue palette in OKLCH. The package exposes:

- `@coderune/tokens` for `tokens.css`
- `@coderune/tokens/tokens.css` for an explicit stylesheet import
- `packages/tokens/fonts/PaperMono-Variable.ttf` for the full Paper Mono font

Semantic names follow shadcn conventions (`--background`, `--foreground`, `--primary`, `--accent`, and so on) and are mapped into Tailwind through `@theme inline` in `src/index.css`.

Geist is the body and ordinary heading face. Paper Mono is used for code, the
index, section side titles, and the `₪` glyph logo. The wordmark, index, and section
side titles are uppercase. Both fonts are bundled locally.
Typography and layout decisions are recorded in
[`docs/web-typography-layout.md`](../../docs/web-typography-layout.md).
