# 0005 — Desktop UI framework: Rune

Status: accepted.

## Context

The desktop needs a native, GPU-drawn Rust interface with local text input,
accessible controls, and a headless way to review screens. The web app remains
React.

## Decision

Build the desktop on Rune with `rune-kit` controls and a Coderune design system in
`crates/ui`. Keep the domain, storage, collectors, agents, and application
operations framework-independent. Keep the shared web registry in
`packages/ui/src/web` for React.

Reasons:

- Rune renders deterministic PNG snapshots, dumps the element tree, and drives
  scripted pointer and keyboard input without opening a window.
- `rune-kit` supplies text input, fields, buttons, badges, scrolling, and
  accessibility roles instead of hand-building them.
- The service crates do not care which UI framework renders them.
- The desktop and web can share a palette and type ramp without sharing a runtime.

## Consequences

- The desktop is a Rune app; `crates/ui` is its Rust design system.
- The toolchain is pinned to Rune's required Rust version.
- Rune is fetched at a pinned revision by `bun run setup:rune`.
- Headless snapshots and scripted input are part of UI review.
- The web keeps React components in `packages/ui`; the desktop keeps the same
  look in Rust.
