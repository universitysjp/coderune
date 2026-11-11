# Quartz in Coderune

Quartz is a private workspace package at `packages/quartz` with separate `web`
and `native` entries. Its Oxlint rules live in `scripts/quartz-lint`. See
[ADR 0002](./adr/0002-quartz-package.md).

The desktop uses native Quartz adapters in packages/quartz/src/native/index.tsx. GPUIX does not execute DOM components, CSS custom properties, Tailwind classes, or browser portals. The official registry is installed under packages/quartz/src/web as the reference and for browser UI. Read the root Quartz skill before UI edits.

## Desktop roles
Import quartz from @coderune/quartz/tokens for every aesthetic value. Native resolved sRGB colors project the Quartz roles into neutral black/white surfaces; blue marks selection and focusable accents. Primary actions use ink, and destructive actions use a fail edge and word. Cream web roles stay in tokens.css. Geist is for UI and prose; Paper Mono is for code, paths, IDs, hashes, and JSON.

Use View/Text for layout and type; Button/Input/Textarea for controls; Field for accessible labels; Card/Alert/PageHeader/Sidebar/Badge for structure. The adapters retain native keyboard handling and expose explicit accessible names. Register bundled TTF fonts before first layout. Both variable fonts and their OFL licenses ship in the tokens package; the desktop build emits its font assets beside the entry point.

Use the native `Tabs` adapter for workspace tabs. It exposes a labelled tab list,
uses Left/Right and Home/End to select tabs, and gives each closable tab a named
close action. The caller owns tab contents and their lifetime. The native
adapter is separate from the Quartz web registry because GPUIX cannot render
DOM components.

Use `IconButton` for icon-only rail actions; it supplies a name and a tooltip on
hover or focus. Use `DropdownMenu` for anchored action lists such as opening an
editor; the floating menu leaves workspace layout unchanged.

`mountDesktop` uses GPUIX's `render` to initialize and reuse the native window and run its frame loop. The mounted `QuartzFonts` gate registers fonts through that initialized renderer before mounting the application UI. Creating and injecting a renderer bypasses GPUIX's window initialization and automatic frame loop. The startup regression test launches this same entry path in a subprocess with disposable storage and exercises the live window, rather than an already initialized offscreen renderer.

Geist source: https://github.com/google/fonts/tree/main/ofl/geist
Paper Mono source: https://github.com/paper-design/paper-mono
Quartz source: https://design.how/r/quartz/registry.json

## Checks
Run `bun run check` from the root. This checks the desktop, native Quartz adapters, and touched browser bootstrap through the repo's Vite+ Oxlint config. Test folders and the Quartz DOM sources are excluded from linting; the GPUIX native entry stays in scope. The legacy marketing screen bodies are outside this desktop migration. The official no-inline-style rule is replaced for native TSX by tokens-only and components-only checks because GPUIX requires StyleDesc objects. Window/test viewport dimensions are geometry rather than design roles.

Use bun check in apps/desktop and bun test for the suites. Native UI tests need macOS/Windows graphics access; GPUIX has no offscreen test renderer on Linux. Browser and native builds have separate scripts. Existing ApplicationOperations tests protect the local data and consent workflows.

## Task tracking
Specification: https://github.com/universitysjp/coderune/issues/3
Foundations: https://github.com/universitysjp/coderune/issues/4
Onboarding and shell: https://github.com/universitysjp/coderune/issues/5
Workspace sections and checks: https://github.com/universitysjp/coderune/issues/6
