# Native Quartz components

Date: 2026-10-26
Status: accepted for this implementation

## Context
The desktop application renders with GPUIX. Quartz registry components depend on DOM, CSS, Tailwind, and browser portals, which GPUIX cannot execute. Replacing the renderer would change packaging and native integrations beyond the requested redesign.

## Decision
Retain GPUIX and provide native adapters for Quartz controls, typography, frames, and the workspace shell. Store the native token projection in the Quartz package. Register bundled Geist and Paper Mono TrueType fonts from the tokens package before rendering. Keep the official registry sources in the Quartz package's web entry and use them as the reference.

The desktop uses neutral black and white channels and the existing blue accent. The web marketing palette remains independently scoped.

## Consequences
Native controls preserve native focus, input, and keyboard behavior. Browser portals and body classes apply only to the DOM workspace. Native adapters require native screenshot and accessibility verification in addition to the official Quartz lint, which targets DOM classes.
