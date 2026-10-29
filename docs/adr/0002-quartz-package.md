# Quartz as a workspace package

Date: 2026-10-28
Status: accepted; implemented
Related: [0001-native-quartz.md](./0001-native-quartz.md)

## Context

Quartz currently exists in two places:

- The DOM design system lives under `apps/web/src/components/quartz` and
  `apps/web/src/lib/quartz`.
- The native GPUIX adaptation lives in `apps/desktop/src/quartz.tsx`.

This makes Quartz look like a web feature with a desktop copy. It also keeps
the design system coupled to one application's source tree, even though the
tokens, component contract, and lint rules belong to both applications.

The web implementation cannot render in GPUIX: it depends on DOM elements, CSS
custom properties, Tailwind classes, Base UI, and browser portals. The native
implementation cannot render in the browser: it depends on GPUIX `StyleDesc`
objects and native controls. The correct boundary is therefore one design
system package with platform-specific entries, not one component
implementation shared across platforms.

The lint rules are different from the runtime design system. They are build
tooling: Oxlint loads them from the root Vite+ configuration, they are not
imported by either application, and they should not be part of the package's
runtime surface. They belong under `scripts/`, not inside the design system
package.

## Decision

Create a private workspace package, `@coderune/quartz`, that owns the Quartz
design system runtime:

```txt
packages/quartz/
  package.json
  tsconfig.json
  src/
    tokens.ts              Native Quartz role projection
    assets.d.ts            Font module declarations for the native entry
    web/
      index.ts             Public web barrel (no icon collision)
      icon.tsx             Icon set, exported separately
      lib/                 Web-only helpers and hooks
      ui/                  React DOM components
      base.css             Scoped Quartz base
      tokens.css           Quartz CSS variables
      styles.css           Tailwind entry for package sources
    native/
      index.tsx            GPUIX Quartz adapters
```

Keep the Oxlint plugin under a root scripts directory:

```txt
scripts/quartz-lint/
  plugin.mjs               Quartz DOM/app rules
  core.mjs                 Shared rule engine
  value-rules.mjs          Raw color and arbitrary value checks
  classes.json             Quartz class vocabulary
  native.mjs               Native TSX rules
```

The root `vite.config.ts` remains the single lint configuration. It imports
`plugin.mjs` from `scripts/quartz-lint` and registers both plugins there. No
separate Vite config is introduced.

### Package contract

`packages/quartz/package.json` exposes platform entries explicitly. There is no
ambiguous root export:

| Export | Target | Consumer |
| --- | --- | --- |
| `@coderune/quartz/web` | `src/web/index.ts` | Web application |
| `@coderune/quartz/web/icon` | `src/web/icon.tsx` | Web application, when icons are needed |
| `@coderune/quartz/web/styles.css` | `src/web/styles.css` | Web Tailwind entry |
| `@coderune/quartz/web/tokens.css` | `src/web/tokens.css` | Web CSS, if imported directly |
| `@coderune/quartz/web/base.css` | `src/web/base.css` | Web CSS, if imported directly |
| `@coderune/quartz/native` | `src/native/index.tsx` | GPUIX desktop application |
| `@coderune/quartz/tokens` | `src/tokens.ts` | Both applications |

The web barrel exports components and lib helpers, but not `icon.tsx`. The icon
module exports a `DataTable` icon and the table module exports a `DataTable`
component; re-exporting both from one barrel would collide. Icons stay
available through `@coderune/quartz/web/icon`.

The package exports raw TypeScript and TSX, matching the other workspace
packages. Vite and Bun transpile it; there is no package build step. Set
`"sideEffects": ["**/*.css"]` so JavaScript exports remain tree-shakeable while
CSS imports are preserved.

### Ownership boundaries

Inside the package:

- Quartz primitives: buttons, inputs, fields, chips, panels, tables, charts,
  status, and the other registry components.
- Quartz web helpers: class merging, portals, direction, formatting, overlay,
  scroll focus, and offscreen helpers.
- Quartz CSS: base, tokens, and the Tailwind entry.
- Quartz native adapters: the GPUIX implementation of the same visual roles.
- Shared Quartz token projection.

Outside the package:

- `apps/web` owns its routes, marketing site, and app-specific blocks such as
  `ProjectHome`.
- `apps/desktop` owns its screens, application operations, and native
  composition.
- `scripts/quartz-lint` owns lint tooling only.

Neither app should import the other platform's entry. Web must not import
`@gpuix/react` through `@coderune/quartz/web`; desktop must not import Base UI or
`react-dom` through `@coderune/quartz/native`.

### Token strategy

Keep `@coderune/tokens` as the shared foundation for fonts and the existing
global site palette. Move Quartz-specific material into the Quartz package:

- `packages/tokens/src/quartz.ts` becomes `packages/quartz/src/tokens.ts`.
- `apps/web/src/components/quartz/{base,tokens,styles}.css` becomes
  `packages/quartz/src/web/{base,tokens,styles}.css`.
- `@coderune/quartz/tokens` becomes the canonical Quartz token import for both
  applications.
- Remove the `@coderune/tokens/quartz` export after updating its consumers.
  Do not leave a re-export in `@coderune/tokens`: the Quartz package still needs
  `@coderune/tokens` for bundled font files, and a re-export in the opposite
  direction would create a workspace dependency cycle.

Fonts and their licenses remain in `@coderune/tokens`. The native Quartz entry
continues to import the font files from there.

### Tailwind and CSS

The package owns its own Tailwind source discovery:

```css
/* packages/quartz/src/web/styles.css */
@import "./tokens.css";
@import "./base.css";

:where(.quartz, [data-system="quartz"]) {
  @tailwind utilities source(none);
}

@source "./";
```

The web application keeps a small composition stylesheet:

```css
/* apps/web/src/quartz.css */
@import "@coderune/quartz/web/styles.css";

@source "./components";
@source "./lib";
@source "./routes";
```

`apps/web/src/routes/__root.tsx` imports `@/quartz.css` instead of the old
`@/components/quartz/styles.css`. The package stylesheet scans package
components; the app stylesheet adds app-owned classes. Both remain scoped by
`:where(.quartz, [data-system="quartz"])`.

### Lint configuration

The root `vite.config.ts` keeps one configuration and points at the scripts
directory:

```ts
import {
  appRules as quartzAppRules,
  sourceRules as quartzSourceRules,
} from "./scripts/quartz-lint/plugin.mjs";

// jsPlugins
{ name: "quartz", specifier: "./scripts/quartz-lint/plugin.mjs" }

// native override
{ name: "quartz-native", specifier: "./scripts/quartz-lint/native.mjs" }
```

Add `scripts/quartz-lint/**` to the lint and format ignore patterns. The lint
folder is tooling, not application source, and its rule files are not formatted
like runtime code.

Rule scopes:

| Scope | Files | Rules |
| --- | --- | --- |
| Web app | `apps/web/src/main.tsx`, `apps/web/src/routes/__root.tsx`, `apps/web/src/components/project-home.tsx` | `appRules` |
| Native app | `apps/desktop/**/*.{js,jsx,ts,tsx}` | `appRules`, with `quartz/no-inline-style` off for TSX |
| Package sources | `packages/quartz/src/web/**`, `packages/quartz/src/native/**` | `sourceRules` |
| Native rules | `apps/desktop/**/*.tsx` | `quartz-native/*` |
| Ignored | `scripts/quartz-lint/**` | none |

The marketing site remains outside the Quartz app-rule scope. Package native
sources get source rules, not native app rules, so the adapters themselves are
not required to use `aria-label` on every internal control.

Update the native rule message from `./src/quartz` to
`@coderune/quartz/native`, and make the raw-host-element exception recognize the
package native entry if the rule is ever applied to it.

## Migration plan

Each phase should leave the workspace buildable.

### Phase 1: Package scaffold and web runtime

1. Create `packages/quartz` with `package.json`, `tsconfig.json`, and the
   source tree above.
2. Move `apps/web/src/components/quartz/ui`, `icon.tsx`, and the web CSS into
   `packages/quartz/src/web`.
3. Move `apps/web/src/lib/quartz` into `packages/quartz/src/web/lib`.
4. Rewrite internal `@/lib/quartz/*` and `@/components/quartz/*` imports to
   relative paths inside the package.
5. Add `src/web/index.ts`. Export components and lib helpers; exclude icons
   from the barrel.
6. Keep `ProjectHome` in the app and update it to import from
   `@coderune/quartz/web`.
7. Add the app composition stylesheet and update the root route import.

### Phase 2: Tokens

1. Move the native projection to `packages/quartz/src/tokens.ts`.
2. Move the web Quartz CSS files into the package.
3. Update native and app token imports to `@coderune/quartz/tokens`.
4. Remove `packages/tokens/src/quartz.ts` and the `./quartz` export.
5. Update `packages/tokens/README.md` and `docs/desktop-quartz.md`.

### Phase 3: Native runtime

1. Move `apps/desktop/src/quartz.tsx` to
   `packages/quartz/src/native/index.tsx`.
2. Add the native JSX runtime declaration. Prefer
   `/** @jsxImportSource @gpuix/react */` at the top of the native entry. If the
   package type checker cannot handle mixed runtimes, use separate
   `tsconfig.web.json` and `tsconfig.native.json` files; do not duplicate the
   implementation.
3. Update `apps/desktop/app.tsx` and the native tests to import from
   `@coderune/quartz/native`.
4. Delete the old desktop adapter file.

### Phase 4: Lint tooling

1. Move `lint/quartz/*` and `lint/quartz-native.mjs` to `scripts/quartz-lint/`.
2. Update root `vite.config.ts` imports, plugin specifiers, ignore patterns, and
   rule scopes.
3. Update the native lint message and path exception.
4. Remove `lint/**` from ignore patterns and add
   `scripts/quartz-lint/**`.

### Phase 5: Dependency cleanup

Web `package.json`:

- Add `@coderune/quartz`.
- Remove dependencies used only by the moved components after confirming no
  site code imports them: `@base-ui/react`,
  `@fortawesome/free-solid-svg-icons`, `@tanstack/react-table`, and
  `@tanstack/react-virtual`.
- Keep `class-variance-authority`, `clsx`, and `tailwind-merge` if site code
  still uses them.

Desktop `package.json`:

- Add `@coderune/quartz`.
- Keep `@gpuix/react` and `@gpuix/native` for the desktop application.
- Remove `@coderune/tokens` only if no direct app import remains.

Package dependencies:

- `@base-ui/react`, `@fortawesome/free-solid-svg-icons`,
  `class-variance-authority`, `clsx`, `react`, `react-dom`,
  `tailwind-merge`, `@tanstack/react-table`, and `@tanstack/react-virtual` for
  the web entry.
- `@gpuix/react` for the native entry.
- `@coderune/tokens` for fonts and the foundation stylesheet.
- `@types/react` and `@types/react-dom` as dev dependencies.

Run `bun install` after the package and dependency changes. Do not hand-edit
`bun.lock`.

## Verification

Run these after the extraction:

1. `bun install` and confirm the lockfile only changes for the workspace link
   and moved dependencies.
2. `bun check` in `packages/quartz` to type-check the package.
3. `bun run check-types` / `bun check` in `apps/web` and `apps/desktop`.
4. `bun run build` in `apps/web`; confirm Vite resolves the package and emits
   Tailwind classes used by package components.
5. `bun run build` in `apps/desktop`; confirm Bun bundles the native entry and
   emits the font assets.
6. `bun test` in `apps/desktop`.
7. Lint probes:
   - Raw color in a web app component reports `quartz/no-raw-color`.
   - Raw color in a package web component does not report an app-only rule.
   - A raw host element in a desktop screen reports
     `quartz-native/components-only`.
   - A hardcoded padding in native TSX reports `quartz-native/tokens-only`.
   - A native control without a name reports
     `quartz-native/controls-named`.
8. Bundle inspection:
   - Web output does not contain `@gpuix/react`.
   - Desktop output does not contain Base UI or `react-dom`.
9. Search for stale paths: `@coderune/tokens/quartz`,
   `apps/web/src/lib/quartz`, `apps/desktop/src/quartz.tsx`, and `lint/quartz`.
10. `git diff --check`.

The repository may still report pre-existing general lint findings from
Ultracite, Anti-Slop, and React Doctor in Quartz files. Those are outside this
architecture change; the acceptance condition is that Quartz rules still fire
in the intended scopes and no new Quartz-rule violations appear.

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Mixed JSX runtimes in one package | Native entry uses `@gpuix/react`; web uses React. Prefer a per-file JSX pragma; fall back to separate package tsconfigs. |
| Web bundle pulls GPUIX | Explicit `./web` and `./native` exports; no root export; verify the web bundle. |
| Desktop bundle pulls Base UI or `react-dom` | Desktop imports only `./native`; verify the desktop bundle. |
| Tailwind misses package classes | Package `styles.css` owns `@source "./"`; app composition CSS scans app-owned files; verify built CSS. |
| Lint rules leak into runtime | Lint stays in `scripts/quartz-lint`; no package export; ignore the folder. |
| App-only rules flag design-system source | Package sources use `sourceRules`; app screens use `appRules`. |
| Nickname collision in the web barrel | Do not export icons from `src/web/index.ts`; expose `./web/icon`. |
| Token package cycle | Move Quartz tokens into the Quartz package; do not re-export from `@coderune/tokens`. |
| Font imports break in the desktop bundle | Keep font files in `@coderune/tokens`, keep the `*.ttf` declaration in the package, and verify the Bun build output. |

## Consequences

- Quartz becomes a real design system boundary instead of a web feature with a
  desktop copy.
- Web and desktop share the component contract, tokens, and lint rules without
  sharing DOM or native implementations.
- The package remains private and buildless, so the change is mostly file
  movement and import rewiring.
- Lint tooling remains a repository concern and can change without changing the
  package's runtime API.
- The root Vite+ configuration remains the only lint configuration.

## Out of scope

- Rewriting native components as DOM components or vice versa.
- Sharing one component implementation between React DOM and GPUIX.
- Moving application screens, routes, or marketing pages into the package.
- Publishing `@coderune/quartz` to a registry or adding package versioning.
- Changing Quartz's visual design or lint rule semantics.
- Adding a second Vite lint configuration for Quartz-only runs.

If a React Native or other mobile client is added later, it needs its own
platform entry and renderer. It should not be forced into the GPUIX native
entry.
