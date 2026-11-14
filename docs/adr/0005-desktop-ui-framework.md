# 0005 — Desktop UI framework: GPUI now, Rune as the alternative

Status: proposed. Awaiting the developer's decision.

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

## Decision (proposed)

Keep GPUI for this port and do not switch to Rune in the same increment.

Reasons:

1. The requirement is Cap's GPUI architecture. The current app is the first
   working native surface, and replacing the framework now would restart the
   UI layer after it has been built.
2. The expensive correctness work — onboarding gate, versioned records,
   reviewed captures, stale-preview rejection, SQLite compatibility, agent
   detection — lives below the UI and is framework-independent.
3. GPUI's smaller platform surface is enough for ordinary windows, text input,
   and form controls. The current gaps are known and bounded.
4. Rune is pre-release, does not accept pull requests, and would require
   committing to a pinned `dev` revision and its own text/layout/rendering
   behaviour.

Rune's agent tooling is still the strongest argument for switching. Adopt the
parts that do not require a framework change:

- Keep the service and UI boundaries separate so the view layer is replaceable.
- Prefer deterministic, reviewable screens and state over free-form drawing.
- Add a headless or scripted UI verification path when GPUI makes one practical.

## Revisit when

- The UI needs components Rune already ships and GPUI would need a large
  hand-built set (tables, comboboxes, rich text, menus, popovers).
- Headless screenshot/tree/input verification becomes a release requirement.
- The product is willing to pin Rune and treat its pre-release API churn as an
  owned dependency.

## Consequences

Keeping GPUI: fastest path to the requested Cap-shaped app; more hand-built
controls; UI verification stays manual or screenshot-based.

Switching to Rune: stronger component coverage and agent-verifiable UI; a
second UI rewrite; divergence from Cap's implementation and the original
direction.
