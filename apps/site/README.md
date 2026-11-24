# Coderune website

Research project site for Coderune, built with React, Vite, Tailwind CSS v4, and
ui-sdk. The site owns its components; there is no shared React UI package.

## Stack

- React 19 with the React Compiler
- Tailwind CSS v4 via `@tailwindcss/vite`
- [ui-sdk](https://ui-sdk.dev) behaviors (`@ui-sdk/react`) for the separator and
  member avatars, installed with the `ui-sdk` skill
- Shared design tokens and fonts from `@coderune/tokens`
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

Run commands from `apps/site`.

## GitHub Pages

The web workflow builds and publishes changes from `main` to
[Coderune on GitHub Pages](https://universitysjp.github.io/coderune/).
It passes the configured Pages base path to Vite. The build emits a static
entry point for every navigation route and a `404.html`, so shared links work on
a direct visit.

## Routes

- `/` overview and verification demo
- `/research` problem, research questions, objectives
- `/design` architecture, data boundary, module scope
- `/evaluation` three-condition study plan
- `/team` team, institution, milestones

`src/app.tsx` picks the page from the pathname; the route components live in
`src/routes`. There is no client-side router.

## Design tokens

`@coderune/tokens` provides the Geist and Paper Mono fonts and the neutral
OpenCode v2 palette: white surfaces, near-black ink, and a near-black primary
fill. The package exposes:

- `@coderune/tokens` for `tokens.css`
- `@coderune/tokens/tokens.css` for an explicit stylesheet import
- `packages/tokens/fonts/PaperMono-Variable.ttf` for the full Paper Mono font

Semantic names follow shadcn conventions (`--background`, `--foreground`,
`--primary`, `--accent`, and so on) and are mapped into Tailwind through
`@theme inline` in `src/index.css`.

Geist is the body and ordinary heading face. Paper Mono is used for code, the
index, section side titles, and the `₪` glyph logo. The wordmark, index, and section
side titles are uppercase. Both fonts are bundled locally.
Typography and layout decisions are recorded in
[`docs/web-typography-layout.md`](../../docs/web-typography-layout.md).
