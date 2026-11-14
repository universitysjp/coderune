# 0005 — Desktop UI framework: GPUI now, Rune as the alternative

Status: accepted on 2026-11-15. The developer chose Rune.

## Context

The native desktop rewrite currently uses crates.io `gpui 0.2.2`. That follows
the stated direction to take Cap's architecture: Rust engine crates, one
process, a GPU-drawn interface, and no web view. The first port pass is working:
onboarding, profiles, projects, documents, agents, capture, and data screens
build and run; the service crates are tested.

Two neighbouring options exist:

- **GPUI Kit** — a component library for GPUI. It reduces hand-built control
  work but does not add a headless UI test surface.
- **Rune** (`anomalyco/rune`) — an independent native GPU UI framework with its
  own renderer, layout, text, animation, `rune-kit` components, and an agent
  tooling mode. Debug builds can render PNG snapshots, dump the element tree,
  run timestamped input scripts, and produce frame strips without a window.

Rune is not GPUI. Moving to it means replacing `crates/ui` and the view layer,
not changing the domain, storage, collector, agent, or application crates.

## Decision

Switch the desktop UI layer to Rune and keep the service crates unchanged.

Reasons:

1. Rune's headless tooling renders PNG snapshots, dumps the element tree, and
   drives scripted pointer and keyboard input with a virtual clock. That is the
   verification path this project needs and GPUI does not provide.
2. `rune-kit` supplies the controls a desktop app needs — text input, fields,
   buttons, badges, scroll views, accessibility roles — instead of hand-building
   them in GPUI.
3. The domain, storage, collectors, agents, and application operations are
   framework-independent; the switch replaces `crates/ui` and the view layer only.
4. Rune is pre-release and pinned to `8fc71fbf21db41920409bf6fa20ffb41ddb2f99d`.
   Its API churn is an owned dependency risk, accepted for the verification and
   component benefits.

## Revisit when

- The UI needs components Rune already ships and GPUI would need a large
  hand-built set (tables, comboboxes, rich text, menus, popovers).
- Headless screenshot/tree/input verification becomes a release requirement.
- The product is willing to pin Rune and treat its pre-release API churn as an
  owned dependency.

## Consequences

- `apps/desktop` is a Rune app; `crates/ui` (GPUI controls) is removed.
- The app depends on a git revision of Rune and on Rune's vendored crates through
  `[patch.crates-io]`. `bun run setup:rune` fetches the checkout.
- The Rust toolchain is pinned to 1.99.1, which Rune requires.
- The Cap reference remains architectural, not literal: Rust service crates, one
  process, native GPU drawing, and no web view.
- Headless snapshots and scripted input become part of UI review.
