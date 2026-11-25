# Desktop code structure

`apps/desktop` is one binary. `main.rs` registers the window, fonts, scenes,
and keyboard/poll hooks; everything else is grouped by responsibility so a
change lands in one place.

## Module map

| Path | Responsibility |
| --- | --- |
| `src/app.rs` | `CoderuneApp` state, `init`, and app-wide navigation. |
| `src/app/actions/` | `impl CoderuneApp` blocks for chats, capture, documents, profile, projects, settings, and local storage. |
| `src/app/root.rs` | Root view, overlays, keyboard shortcuts, and the once-per-frame poll. |
| `src/app/scenes.rs` | Headless fixture scenes used by `--scene` and `verify:ui`. |
| `src/forms/` | Editable forms and the line-oriented codecs that turn them into domain drafts. |
| `src/screens/` | One view module per destination. Screens read state and call app methods. |
| `src/shell.rs` | Window chrome, sidebar, title bar tabs, and workspace layout. |
| `src/widgets.rs` | App-aware builders: buttons, text fields, choice chips, status alerts. |
| `src/chats.rs`, `src/chat_terminal.rs` | Chat persistence and PTY-backed coding-agent sessions. |
| `src/preferences.rs`, `src/agent_detection.rs`, `src/updates.rs`, `src/update_install.rs` | Desktop-local services. |

## Rules

- **State changes live in `actions`.** A screen queues a method; it never writes
  a field or a preference directly. Methods that cross action modules use
  `pub(in crate::app)` rather than widening the public surface.
- **Forms convert, they do not persist.** A form owns `TextEdit` values and a
  `to_draft` method. Storage rejection comes back as `AppError`, which the
  action records once.
- **Views are pure reads.** `screens/*` and `app/root.rs` take `&CoderuneApp`
  and return `El`; only callbacks mutate state.
- **`root.rs` owns the frame.** Polling, font-family changes, keyboard routing,
  and overlay ordering stay together so a new screen does not invent its own
  tick.
- **Scenes seed through the same actions.** A fixture calls `confirm_profile`,
  `save_project`, or `start_new_project`; it never builds a fake workspace.

## Components

Use the smallest layer that fits:

1. `rune-kit` for editable, focusable, animated, or accessible controls.
2. `coderune_ui::components` for the desktop design language:
   `layout` (cards, rows, page headers, empty states), `text` (section titles,
   code wells, supporting copy), and `controls` (buttons, badges, alerts,
   fields, navigation rows, keycaps).
3. `widgets` when the builder needs `CoderuneApp`, such as a primary button that
   calls an action or the shared error/notice stack.

Pages compose `ui::page`, `ui::section`, and `ui::empty_state`. A section is a
muted label over one raised card of rows; rows are separated by the card, not
by the page.

## Adding a destination or a feature

1. Add the `View` variant, label, icon, and nav key in `app.rs`.
2. Add one `screens/<name>.rs` with `fn <name>(s: &CoderuneApp, k: &Kit) -> El`.
3. Add the matching `actions/<name>.rs` methods and declare the module in
   `app/actions/mod.rs`.
4. Add a `scenes::scene_<name>` and register it in `main.rs` if it needs a
   headless render.
5. Cover the flow with a `Runtime` test in `app.rs` and run
   `bun run test:desktop`.
