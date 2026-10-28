# Desktop Quartz migration

## Problem
The native desktop interface uses ad hoc purple styling instead of Quartz. The developer wants a black-and-white desktop workspace with blue accents, Geist UI text, and Paper Mono code, paths, and IDs.

## Solution
Keep the native GPUIX renderer and application operations, adapt the installed Quartz workspace block and controls to native host elements, and share desktop tokens and bundled font assets through the tokens package. Install the official DOM registry in the web workspace for reuse without porting the marketing site in this task.

## User stories
1. As a developer, I can confirm or resume my developer baseline in a clear onboarding form.
2. As a developer, I can navigate Profile, Projects, Documents, Agents, Capture, and Data without losing their existing operations.
3. As a developer, I can distinguish selected rows and focus with blue accents on neutral surfaces.
4. As a developer, I can read prose in Geist and copy paths, code, IDs, and JSON in Paper Mono.
5. As a developer, I can preview and approve imported or captured context before saving or sharing it.
6. As a developer, I retain explicit confirmation for destructive operations and can export my data.

## Implementation decisions
- Preserve native GPUIX and local SQLite operations; native Quartz adapters replace browser-only primitives at the renderer boundary.
- Quartz provides the spacing, typography, control, frame, and page-shell roles. Desktop chromatic accents use the existing blue palette; cream marketing tokens stay scoped to the web.
- Bundle TrueType desktop fonts and register them before first layout. The web retains its WOFF2 assets.
- Use the installed Workspace home block as the layout reference, replacing sample content with developer/project context.
- Keep registry sources and the Quartz skill available to future UI changes.
- Keep all work uncommitted at the user’s explicit request.

## Testing decisions
Run existing ApplicationOperations tests for behavior. Capture native onboarding and all six workspace sections at narrow and wide widths using isolated fixture storage; exercise keyboard focus, form input, confirmations, long text, and input feedback through native automation. Run Quartz lint, native token checks, type checking, and build checks. Do not introduce tests that merely compare token constants with themselves.

## Tasks
1. Install Quartz and expose native tokens, font loading, and controls. No blockers.
2. Migrate onboarding and the workspace shell. Blocked by task 1.
3. Migrate every workspace section and review panel; verify all touched UI and checks. Blocked by task 2.

## Out of scope
New application features, storage/schema changes, replacing GPUIX with a browser shell, and redesigning the marketing website.
