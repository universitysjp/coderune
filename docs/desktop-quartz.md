# Quartz in Coderune

> Planned architecture: Quartz will move to `packages/quartz` with separate
> `web` and `native` entries, and its Oxlint rules will move to
> `scripts/quartz-lint`. See
> [ADR 0002](./adr/0002-quartz-package.md). The paths below describe the
> current implementation until that extraction lands.

The desktop uses native Quartz adapters in apps/desktop/src/quartz.tsx. GPUIX does not execute DOM components, CSS custom properties, Tailwind classes, or browser portals. The official registry is installed under apps/web/src/components/quartz and apps/web/src/lib/quartz as the reference and for browser UI. Read the root Quartz skill before UI edits.

## Desktop roles
Import quartz from @coderune/tokens/quartz for every aesthetic value. Native resolved sRGB colors project the Quartz roles into neutral black/white surfaces; blue marks selection and focusable accents. Primary actions use ink, and destructive actions use a fail edge and word. Cream web roles stay in tokens.css. Geist is for UI and prose; Paper Mono is for code, paths, IDs, hashes, and JSON.

Use View/Text for layout and type; Button/Input/Textarea for controls; Field for accessible labels; Card/Alert/PageHeader/Sidebar/Badge for structure. The adapters retain native keyboard handling and expose explicit accessible names. Register bundled TTF fonts before first layout. Both variable fonts and their OFL licenses ship in the tokens package; the desktop build emits its font assets beside the entry point.

`mountDesktop` uses GPUIX's `render` to initialize and reuse the native window and run its frame loop. The mounted `QuartzFonts` gate registers fonts through that initialized renderer before mounting the application UI. Creating and injecting a renderer bypasses GPUIX's window initialization and automatic frame loop. The startup regression test launches this same entry path in a subprocess with disposable storage and exercises the live window, rather than an already initialized offscreen renderer.

Geist source: https://github.com/google/fonts/tree/main/ofl/geist
Paper Mono source: https://github.com/paper-design/paper-mono
Quartz source: https://design.how/r/quartz/registry.json

## Checks
Run `bun run check` from the root. This checks the desktop, installed registry, and touched browser bootstrap through the repo's Vite+ Oxlint config. Tests and Quartz sources keep the Quartz rules but are exempt from the repository's general lint set. The legacy marketing screen bodies are outside this desktop migration. The official no-inline-style rule is replaced for native TSX by tokens-only and components-only checks because GPUIX requires StyleDesc objects. Window/test viewport dimensions are geometry rather than design roles.

Use bun check in apps/desktop and bun test for the suites. Native UI tests need macOS/Windows graphics access; GPUIX has no offscreen test renderer on Linux. Browser and native builds have separate scripts. Existing ApplicationOperations tests protect the local data and consent workflows.

## Task tracking
Specification: https://github.com/universitysjp/coderune/issues/3
Foundations: https://github.com/universitysjp/coderune/issues/4
Onboarding and shell: https://github.com/universitysjp/coderune/issues/5
Workspace sections and checks: https://github.com/universitysjp/coderune/issues/6
