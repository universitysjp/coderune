> Status: historical. This maps Cap's GPUI app; Coderune now uses Rune (ADR 0005). The architecture lessons still apply.

# Cap's GPUI desktop architecture

Reference: `vendor/cap/apps/desktop-gpui` (Cap 0.6.x, AGPL-3.0, vendored and ignored by git).
Target: a new Rust GPUI app built against crates.io `gpui 0.2.2`.

This is an implementation map, not a feature inventory. Every path below is
relative to `vendor/cap/apps/desktop-gpui` unless it starts with `/`.

The short version: Cap does not use a router, a reactive store, or a component
framework. It uses **one root entity per window**, **plain `Global`s to hold
window handles and app-scoped entities**, **`Copy` theme values resolved at
render time**, and **three explicit executors** (GPUI foreground, GPUI
background, tokio). The discipline is in the ownership boundaries, not in
abstraction layers.

---

## 1. Workspace and crate boundaries

### 1.1 A deliberately standalone workspace

`Cargo.toml:1-10`:

```toml
# Standalone workspace on purpose: the gpui dependency tree is enormous ...
[workspace]
resolver = "2"
members = [".", "kernels", "i18n-build", "glass"]
```

Reasons given in `Cargo.toml` and `README.md:70-83`:

- GPUI pins its own git revisions of wgpu/font stacks. Keeping it out of the
  root workspace keeps the root `Cargo.lock` and `cargo hakari` workspace-hack
  free of it.
- The root workspace's `[patch.crates-io]` forks (`reqwest`, `cidre`) are not
  wanted by GPUI's tree.
- `rust-toolchain.toml` pins `stable` locally because GPUI's dependencies need
  1.89+, while the root workspace pins 1.88.0. The nearest toolchain file wins.

Members:

| Member | Path | Purpose |
|---|---|---|
| `cap-desktop-gpui` | `.` | The binary `cap-gpui`; all UI and app logic. |
| `cap-gpui-kernels` | `kernels/` | Pixel/sample hot loops compiled at `opt-level = 3`. |
| `cap-i18n-build` | `i18n-build/` | Build script that generates typed translation tables. |
| `cap-glass` | `glass/` | CPU model of the macOS glass material used by the probe. |

The app itself is **one binary crate with modules**, not a library + thin main.
`src/main.rs:6-114` is a wall of `mod` declarations. Window roots, components,
platform shims, engine adapters, and pure helpers all live in the same crate.

### 1.2 Dependency pins that matter

`Cargo.toml`:

| Dependency | Pin | Why it matters |
|---|---|---|
| `gpui`, `gpui_platform`, `gpui_tokio` | `git = wingleeio/zed`, rev `5d1f83d9…` | Cap's fork; includes fixes the transparent shell needs. |
| `kameo` | `0.17.2` | Must match `cap-recording`'s actor version exactly. |
| `image` | `0.25.1` | Must match GPUI's `RenderImage` frame type. |
| `wgpu` | `25.0.0` | Editor/rendering line; GPUI uses a separate wgpu 29 tree. |
| `tokio` | `1`, features `rt`, `time` | Runtime handle only; the fork's `gpui_tokio` owns the bridge. |
| `flume` | `0.11.0` | Lock-free channels between tokio workers and GPUI threads. |
| `cpal` | `CapSoftware/cpal`, rev `6013cb5f8bd3` | Microphone feed identity. |
| `ffmpeg` (`ffmpeg-next`) | `CapSoftware/rust-ffmpeg`, rev `49db1fed…` | Media/export. |

`[patch."https://github.com/wingleeio/zed"]` points at a local
`../../../zed-cap` checkout. That means the committed build is not reproducible
without that sibling checkout. A new app should not copy this; depend on
crates.io `gpui = "0.2.2"` and avoid fork-only APIs.

### 1.3 Profiles

`Cargo.toml` bottom:

```toml
[profile.dev]
opt-level = 0
incremental = false

[profile.dev.package."*"]
opt-level = 2
debug = "line-tables-only"

[profile.dev.package.cap-gpui-kernels]
opt-level = 3

[profile.dev.package.cap-desktop-gpui]
incremental = true
debug = "line-tables-only"

[profile.release]
codegen-units = 1
lto = "thin"
strip = "symbols"
```

Rationale in `README.md:18-39`: GPUI is unusably slow when its dependency tree
is unoptimised. The app crate stays at 0 so one-line edits rebuild fast; hot
loops move to `kernels/`. Do not optimise the app crate itself.

### 1.4 Build script

`build.rs` does three things:

1. Emits diagnostic build metadata (`scripts/diagnostic-build.rs`).
2. Generates i18n tables via `cap_i18n_build::build("locales")`.
3. On macOS compiles `native/exception_guard.m` with `-fobjc-exceptions`;
   on Windows embeds the icon and raises the stack to 16 MiB with
   `/STACK:16777216` because media futures are constructed on the UI thread.

---

## 2. Boot sequence (`src/main.rs`)

The order is load-bearing. Copy the order, not every platform branch.

### 2.1 Before the app exists

`src/main.rs:295-381`:

1. Linux environment fixups (`LP_NUM_THREADS`, `ALSA_CONFIG_PATH`, stale
   `WAYLAND_DISPLAY`) before logging starts worker threads.
2. AppImage CLI dispatch and Linux dialog helper exit early.
3. `init_logging()` returns a `WorkerGuard` that must outlive `main`.
4. `crash_capture::install(&diagnostics::logs_dir())`.
5. `single_instance::acquire()` — a second launch forwards a deep link/reopen
   and exits.
6. `crash_capture::begin_session(...)` returns the previous session marker, fed
   to encoder/blur recovery.
7. macOS AppKit exception guards and URL-scheme handler; scan argv for
   `token`/`api_key` deep links.
8. `app_runtime()` creates the tokio runtime **before** GPUI:

```rust
// src/main.rs:286-293
fn app_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(MEDIA_THREAD_STACK_SIZE)
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio")
}
```

`MEDIA_THREAD_STACK_SIZE` is 16 MiB (`src/main.rs:282`). Media/render/export
futures compile at the app crate's dev opt-level and need more stack than the
2 MiB default.

### 2.2 Application builder

`src/main.rs:369-380`:

```rust
let app = gpui_platform::application().with_assets(Assets);
app.on_reopen(crate::app_windows::handle_dock_reopen);
#[cfg(target_os = "macos")]
app.on_open_urls(|urls| {
    for url in urls {
        crate::deeplink::submit_deep_link(&url);
    }
});
let runtime_handle = runtime.handle().clone();
```

`on_reopen` must be registered on the builder. Its handler guards against
firing before the window registry exists. `Application::on_reopen` /
`on_open_urls` exist in crates.io 0.2.2; only the constructor differs (see §11).

### 2.3 Inside `app.run`

`src/main.rs:382-844` runs in this order:

1. `gpui_tokio::init_from_handle(cx, runtime_handle)` — install the runtime
   bridge before any task can spawn.
2. Spawn long-lived diagnostic checkpoint task.
3. Set the dock icon on unbundled dev builds.
4. **Load fonts before the first window**:
   `Assets.load_fonts(cx)` (`src/assets.rs:299-311`).
5. Bind text-input keys (`ui::bind_text_input_keys(cx)`), scoped to the
   `TextInput` key context.
6. Read persisted settings once:
   `GeneralSettings::load()`.
7. Initialise i18n, then theme, then reduce-motion policy.
8. Create app-scoped globals **in dependency order**:

```rust
let session = RecordingSession::init(cx);
crate::sign_in::SignIn::init(cx);
crate::feeds::Feeds::init(cx);
crate::media_access::MediaAccess::init(cx);
crate::target_overlay::TargetSelect::init(cx);
```

9. Compute `first_run` from the store (not from TCC queries), then open the
   main window hidden on first run so onboarding does not flash it.
10. Install `cx.on_app_quit(...)` to mark a clean exit.
11. `app_windows::init(window_handle, session, cx)` — installs `AppWindows` and
    the session observer.
12. Power/sleep hooks, native quit handler, update check, upload queues,
    launch render warm-up.
13. Menus, then tray, then hotkeys — all after the window registry exists.
14. Open onboarding directly on first run, or after a background gate query.
15. Test/debug `CAP_GPUI_AUTO_*` hooks (skip in a new app).
16. `cx.activate(true)` unless the dev-restore loop is enabled.

The rule behind steps 11-13: **nothing that dispatches into windows is
installed before `AppWindows` exists**. Menus and tray both reach the registry.

### 2.4 Main window options

`src/main.rs:426-479`:

```rust
let window_handle = cx.open_window(
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,                       // no titlebar, no traffic lights
        kind: WindowKind::Floating,           // macOS panel-like
        app_owns_titlebar_drag: true,         // fork-only; see §11
        window_background: WindowBackgroundAppearance::Transparent,
        is_resizable: false,
        focus: !first_run,
        show: !first_run,
        ..Default::default()
    },
    {
        let session = session.clone();
        move |window, cx| cx.new(|cx| MainWindow::new(session, window, cx))
    },
)
```

Do not do real work inside the builder closure. `MainWindow::new` only wires
observers, focus handles and input entities. Device enumeration, library scans
and native window-material installation all start **after** the handle exists
(`src/main.rs:591-654`, `src/app_windows.rs:1395-1448`).

---

## 3. Window and navigation model

There is no router and no shared navigation stack. Every window is its own
root entity implementing `Render`. Cross-window navigation is a set of free
functions in `src/app_windows.rs` that read and mutate a `Global` registry.

### 3.1 `AppWindows`: the registry

`src/app_windows.rs:254-313`:

```rust
pub struct AppWindows {
    pub main: WindowHandle<MainWindow>,
    pub controls: Option<WindowHandle<ControlsWindow>>,
    pub camera: Option<WindowHandle<CameraWindow>>,
    pub settings: Option<WindowHandle<SettingsWindow>>,
    pub onboarding: Option<WindowHandle<OnboardingWindow>>,
    pub mode_select: Option<WindowHandle<ModeSelectWindow>>,
    pub teleprompter: Option<WindowHandle<TeleprompterWindow>>,
    pub overlays: Vec<(DisplayId, WindowHandle<OverlayWindow>)>,
    pub editors: Vec<(PathBuf, WindowHandle<EditorWindow>)>,
    pub screenshot_editors: Vec<(PathBuf, WindowHandle<ScreenshotEditorWindow>)>,
    // lifecycle flags: hidden_by_app, closed_by_user, hidden_for_picker, ...
}

impl Global for AppWindows {}
```

`src/app_windows.rs:558-761` installs it and wires the one observer that drives
the recording flow:

```rust
pub fn init(main: WindowHandle<MainWindow>, session: Entity<RecordingSession>, cx: &mut App) {
    cx.set_global(AppWindows { main, controls: None, /* ... */ });

    let mut last_phase = Phase::Idle;
    cx.observe(&session, move |session, cx| {
        let phase = session.read(cx).phase;
        // ... open the controls bar, close overlays, finalize, open editor ...
        last_phase = phase;
    })
    .detach();
}
```

The registry is a plain global because it is never rendered. It holds only
`WindowHandle`s and scalar flags.

### 3.2 Open / focus / close functions

Every window has a small set of free functions, all in `app_windows.rs`:

| Concern | Functions |
|---|---|
| Main | `show_main_window`, `hide_main_window`, `request_close_main`, `minimize_main_window`, `heal_main_window_style` |
| Settings | `open_settings(page)`, `close_settings`, `settings_closed`, `restore_after_settings` |
| Onboarding | `open_onboarding`, `onboarding_finished`, `close_onboarding`, `onboarding_closed` |
| Mode select | `open_mode_select`, `mode_select_closed`, `choose_mode_in_mode_select` |
| Editor | `open_editor(path)`, `editor_closed`, `restore_after_editor_close`, `reload_editor` |
| Overlays | `open_target_overlays`, `dismiss_target_overlays`, `close_target_overlays`, `refresh_target_overlays` |
| Camera/controls/teleprompter | `open_camera_window`, `close_camera_window`, `open_teleprompter`, `play_teleprompter` |

The standard open shape (`src/app_windows.rs:1310-1451`):

```rust
pub fn open_settings(page: Page, cx: &mut App) {
    if let Some(handle) = cx.global::<AppWindows>().settings {
        // Reuse the live window: show, focus, re-target the page.
        let native = handle.update(cx, |view, window, cx| {
            view.set_page(page, window, cx);
            platform::native_window(window)
        }).ok().flatten();
        cx.spawn(async move |_| { /* platform show */ }).detach();
        hide_main_and_park_camera_preview(cx);
        return;
    }

    let handle = cx.open_window(WindowOptions { /* ... */ },
        move |window, cx| cx.new(|cx| SettingsWindow::new(page, window, cx)));

    cx.global_mut::<AppWindows>().settings = Some(handle);
    handle.update(cx, |view, window, cx| {
        view.start_enumeration(window, cx);
        view.page_shown(window, cx);
        view.focus_root(window, cx);
        platform::native_window(window)
    }).ok();
    hide_main_and_park_camera_preview(cx);
}
```

Copy these three properties:

1. **A live window is reused and re-targeted**, never duplicated.
2. **The registry is updated immediately after `open_window` returns.**
3. **Open windows from `cx.defer`, never from inside an entity update.** The
   doc comment at `src/app_windows.rs:1303-1309` says opening a window paints it
   synchronously and double-leases the caller.

### 3.3 Deferred work and deferred focus

The codebase has a repeated "do it after the current lease" rule:

- `cx.defer(...)` in `show_main_window`/`heal_main_window_style` and in every
  key dispatch that opens a window.
- `window.defer(cx, ...)` for appearance/native work
  (`src/theme.rs:132-137`).
- `cx.defer_in(window, ...)` in `ControlsWindow::new` for its first
  `fit_to_issue`.
- Focus after open is done inside a spawned task, after the window is on
  screen: `open_overlay` focuses the overlay's root handle only after
  `place_overlay_panel` (`src/app_windows.rs:2847-2882`).
- `open_settings` focuses its root handle inside the post-open update
  (`src/app_windows.rs:1397-1405`). `open_panel` focuses the search field before
  returning (`src/main_window.rs:4402-4425`).

The rule is: **mutating native window state, or opening/focusing a window,
never happens inside an entity update or `open_window` builder.** Read the
native handle inside the update, act on it from a task or a deferred closure.

### 3.4 Screen composition inside a window

Each root owns its own "router":

- `MainWindow`: `Panel` enum replaces the body
  (`src/main_window.rs:807-818`). `render_body` matches `self.panel` and draws
  either the home screen or the active device/target/library panel
  (`src/main_window.rs:4054-4090`).
- `SettingsWindow`: `Page` enum (`src/settings_window.rs:90-160`) with `ALL`,
  `label()`, `slug()`, `from_slug()`. `render_content` is a single match
  (`src/settings_window.rs:2383-2397`).
- `OnboardingWindow`: `Step` enum (`src/onboarding_window.rs:113-176`) with
  `ALL`, `first`, `next`, `previous`; `Render` matches the step and applies a
  slide/fade using `step_revision` (`src/onboarding_window.rs:2763-2803`).
- Teleprompter/editor/screenshot editor: their own root state; no shared
  navigation types.

There is no route enum shared across windows. `Page` is not a global router:
it is passed into `open_settings`, stored on `SettingsWindow`, and re-applied by
`set_page`.

### 3.5 Close lifecycle

Every root registers `on_window_should_close` and defers the registry cleanup:

```rust
// src/settings_window.rs:789-792
window.on_window_should_close(cx, |_window, cx| {
    cx.defer(crate::app_windows::settings_closed);
    true
});
```

`settings_closed` takes the handle out of `AppWindows` and calls
`restore_after_settings`, which brings the main window back when appropriate
(`src/app_windows.rs:1455-1472`, `1610-1636`). Main window close is different:
it hides to the tray unless the platform's native quit path is in play
(`src/main_window.rs:1051-1058`, `src/app_windows.rs:1143-1187`).

`handle_dock_reopen` (`src/app_windows.rs:1189-1265`) is the canonical "focus
the right existing window, otherwise show main" routine:

```rust
let live = cx.windows().into_iter().map(|h| h.window_id()).collect::<HashSet<_>>();
windows.editors.retain(|(_, h)| live.contains(&h.window_id())); // prune dead handles
let focus = first_registered_reopen_target(candidates, &live);
match focus {
    Some(handle) => cx.defer(move |cx| { /* show native */ }),
    None => show_main_window(cx),
}
cx.activate(true);
```

---

## 4. State ownership

Cap has four distinct state tiers. Do not collapse them.

### 4.1 Tier 1: widget state as an `Entity`

`ui::TextInputState` is the model; `ui::TextInput` is the element
(`src/ui/text_input.rs:522-558`, `1637-1662`):

```rust
pub struct TextInputState {
    focus: FocusHandle,
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    multi_line: bool,
    disabled: bool,
    layout: Rc<RefCell<Option<InputLayout>>>,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    _blur: gpui::Subscription,
}

pub struct TextInput {
    state: Entity<TextInputState>,
    id: ElementId,
    /* paint fields copied from &Theme */
}
```

The owner creates it with `cx.new(|cx| ui::TextInputState::single_line(window, cx))`
and stores `Entity<TextInputState>`. The element is a stateless `RenderOnce`
that reads the entity. The state emits `Changed` / `Confirmed` / `Cancelled` /
`Blurred`; the **window** decides what those mean.

This is the safest reusable-state pattern in the codebase: one entity per field,
owned by the window, element borrows a clone.

### 4.2 Tier 2: window state as root fields

`MainWindow` has ~80 fields (`src/main_window.rs:943-1021`), `SettingsWindow`
~30 (`src/settings_window.rs:714-765`). They are all view-owned: selected tab,
menu, drafts, scroll handles, task handles, subscriptions. None of this is
global. `SettingsWindow` is also split by extension modules
(`src/settings_pages.rs`), which keep their page state in a `PagesState` field.

Cross-window state never lives here. The window holds `Entity<RecordingSession>`
and `Entity<Feeds>` clones.

### 4.3 Tier 3: app-scoped entities in `Global`

The pattern is mechanical:

```rust
struct SessionGlobal(Entity<RecordingSession>);
impl Global for SessionGlobal {}

impl RecordingSession {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let session = cx.new(|_| Self { phase: Phase::Idle, /* ... */ });
        cx.set_global(SessionGlobal(session.clone()));
        session
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<SessionGlobal>().0.clone()
    }
}
```

`src/session.rs:416-553`. The same shape appears in:

| State | File | Global wrapper |
|---|---|---|
| Recording session | `src/session.rs` | `SessionGlobal(Entity<RecordingSession>)` |
| Device feeds | `src/feeds.rs:733-871` | `FeedsGlobal(Entity<Feeds>)` |
| Sign-in flow | `src/sign_in.rs:47-65` | `SignInGlobal(Entity<SignIn>)` |
| Media permissions | `src/media_access.rs` | `MediaAccessGlobal(Entity<MediaAccess>)` |
| Target picker | `src/target_overlay.rs:482-514` | `TargetSelectGlobal(Entity<TargetSelect>)` |
| Window registry | `src/app_windows.rs:499` | `AppWindows` (plain data, not an entity) |
| Theme preference | `src/theme.rs:82-98` | `ThemePreference(AppTheme)` |
| Native material | `src/platform.rs:528-530` | `WindowMaterial(Option<MaterialKind>)` |
| Keyboard mode | `src/ui/keyboard_nav.rs:50-58` | `KeyboardWindows(HashSet<WindowId>)` |

The global is a thin wrapper so `cx.global::<T>()` does not collide with other
`Global` impls on the entity. Never put a `WindowHandle` in an entity that
windows observe.

### 4.4 Tier 4: persisted state is not reactive

`src/store.rs` is a plain file store, not a global entity. Reads happen on
demand; writes are explicit functions.

Two files (`src/store.rs:1-22`):

- `gpui-state.json` — app-owned, whole-file rewrite, atomic rename.
- `store` — the tauri-plugin-store JSON shared with the Tauri app.

`GeneralSettings::load()` reads the cached raw JSON and maps fields with
defaults (`src/store.rs:1388-1495`). There is no subscription. Windows reload
when they are shown/re-targeted (`SettingsWindow::set_page`,
`src/settings_window.rs:935-953`).

Writes are read-modify-write on the raw JSON to preserve unknown keys
(`src/store.rs:457-535`):

```rust
pub fn set_store_setting(section: &str, key: &str, value: Value) -> bool {
    update_store(|store| {
        let entry = store.entry(section.to_string()).or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() { *entry = Value::Object(Map::new()); }
        entry.as_object_mut().unwrap().insert(key.to_string(), value);
        true
    })
}
```

`update_store` locks, reads on-disk state, mutates exactly the requested path,
writes to a sibling temp file, and renames (`src/store.rs:518-579`). This is the
contract that lets two apps share a settings file. Copy the contract even if
your new app owns its own file: never serialize a typed struct over a store you
did not exclusively create.

### 4.5 Observation and event edges

- `cx.observe(&session, ...)` for durable cross-window reactions
  (`app_windows::init`).
- `cx.observe_in(&session, window, ...)` when the callback needs a `&mut Window`
  (`src/main_window.rs:1074-1108`, `src/settings_window.rs:845-861`).
- `cx.subscribe(&input, ...)` for one-shot typed events
  (`TextInputEvent`, `SignInEvent`).
- `cx.observe_window_activation/...` for focus/blur policy
  (`src/settings_window.rs:400-410`).
- `window.on_window_should_close` for close policy.
- Raw `flume`/`tokio::sync::watch` for cross-thread edges, never callbacks.

The distinction is explicit: **observers are state edges; subscriptions are
event edges.** A text field's `Changed` is an event; the session's phase is
state.

---

## 5. Component API shape (`src/ui/`)

`src/ui/mod.rs:1-35` states the contract: every component is hand-transcribed
from the Solid app, every component takes `&Theme`, and every component uses
`impl Fn(&ClickEvent, &mut Window, &mut App)` handlers so it is
window-agnostic without a generic parameter.

### 5.1 The shared handler aliases

```rust
// src/ui/button.rs:17
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

// src/ui/menu.rs:29
pub(crate) type OpenHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;

// index-dispatching components (Menu, TabRail, SegmentedControl)
type SelectHandler = Box<dyn Fn(&usize, &mut Window, &mut App) + 'static>;
```

The index is passed **by reference** (`&usize`) so `cx.listener` can build the
handler directly without an extra clone layer.

### 5.2 Stateless component: `Button`

`src/ui/button.rs:113-135`, `191-251`, `427-524`:

```rust
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    size: ButtonSize,
    paint: ButtonPaint,
    radius: Option<Pixels>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    on_open: Option<OpenHandler>,
}

impl Button {
    /// One named constructor per surface.
    pub fn plain(theme: &Theme, id: impl Into<ElementId>, variant: ButtonVariant, size: ButtonSize) -> Self { /* ... */ }
    pub fn body(theme: &Theme, id: impl Into<ElementId>, variant: ButtonVariant, size: ButtonSize) -> Self { /* ... */ }
    pub fn settings(theme: &Theme, id: impl Into<ElementId>, variant: ButtonVariant, size: ButtonSize) -> Self { /* ... */ }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self { self.label = Some(label.into()); self }
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler)); self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Button { id, label, paint, size, /* ... */ } = self;
        div()
            .id(id)
            .tab_index(0)
            .tab_stop(!disabled)
            .flex().items_center().justify_center()
            .px(size.padding_x())
            /* ... */
            .when_some(on_click, |this, handler| this.on_click(handler))
    }
}
```

Key conventions:

- Fields are fully resolved at construction (`ButtonPaint`), so render has no
  variant branching.
- Every visual knob has a builder (`bg`, `text_color`, `radius`, `width`,
  `full_width`, `height`, `gap`, `font_weight`).
- The disabled/enabled branches are `.when(...)` chains, not separate types.
- `#[derive(IntoElement)]` + `RenderOnce`, not `Render`; stateless components
  do not need an entity.

### 5.3 Stateful component: `TextInput`

`src/ui/text_input.rs:522-600`, `1637-1711`:

```rust
pub struct TextInputState { /* focus, content, selection, undo, layout */ }

impl TextInputState {
    pub fn single_line(window: &mut Window, cx: &mut Context<Self>) -> Self { Self::new(false, window, cx) }
    pub fn multi_line(window: &mut Window, cx: &mut Context<Self>) -> Self { Self::new(true, window, cx) }
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) { /* ... */ }
    pub fn focus_and_select_all(&mut self, window: &mut Window, cx: &mut Context<Self>) { /* ... */ }
}

pub struct TextInput {
    state: Entity<TextInputState>,
    id: ElementId,
    /* paint fields */
}

impl RenderOnce for TextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement { /* ... */ }
}
```

The element never mutates the state directly. It calls
`state.update(cx, |state, cx| ...)`, and the state emits events. The window
subscribes and decides what ESC/Enter/blur mean.

### 5.4 Menus: state machine separated from rendering

`src/ui/menu.rs:33-155`:

```rust
pub struct MenuItem { pub label: SharedString, pub checked: bool }

pub enum MenuKey { Moved, Commit(usize), Dismiss, Ignored }

pub struct MenuState {
    pub origin: Point<Pixels>,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    highlighted: Option<usize>,
    items: usize,
}

impl MenuState {
    pub fn new(origin: Point<Pixels>, items: &[MenuItem]) -> Self { /* ... */ }
    pub fn on_key(&mut self, key: &str) -> MenuKey { /* arrows, Home/End, Enter, Escape */ }
}
```

`MenuState` is pure and unit-tested without a window. `Menu` is a `RenderOnce`
that draws the absolute panel. `Select` is just the trigger; it exposes
`on_open` rather than owning a menu (`src/ui/select.rs:117-131`). The window
owns `Option<OpenMenu>` and renders the menu as the last child so it paints over
the page (`src/settings_window.rs:5026-5044`, `1865-1869`).

Open/focus flow (`src/settings_window.rs:1495-1549`):

```rust
pub(crate) fn open_menu(&mut self, kind: MenuKind, origin: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
    let return_focus = ui::keyboard_nav::active(window, cx).then(|| window.focused(cx)).flatten();
    let focus = self.focus.clone();
    window.focus(&focus, cx);       // take focus from the text field
    self.menu = Some(OpenMenu { kind, return_focus, state: ui::MenuState::new(origin, &items) });
    cx.notify();
}
```

`close_menu` restores `return_focus` only when keyboard navigation is active.

### 5.5 Surfaces and layout primitives

`src/ui/surface.rs`:

- `Card::settings(theme, padded)` / `Card::settings_rows(...)`
- `Card::liquid_glass(theme)`
- `Popover::glass(theme, width)` / `Popover::plain(theme)`
- `SettingRow::settings(theme, label, description, control)` — 46px min row
- `Section::settings(theme, title, description, right, children)` — section
  header + optional Pro pill

`src/ui/field.rs` adds `Field` / `Subfield` for labelled control containers
(`plain`, `section`, `stacked`, `inline`). These are layout-only; they do not
own state.

### 5.6 Tooltip, Kbd, Progress, TabRail

- `Tooltip` implements `Render` and is built into an `AnyView` via
  `Tooltip::view(cx)`; call sites use gpui's own
  `.tooltip_show_delay(TOOLTIP_SHOW_DELAY).tooltip(...)`
  (`src/ui/tooltip.rs:87-123`, `src/ui/tab_rail.rs:174-175`).
- `KbdChip` is a stateless `RenderOnce` with `tooltip` and `row` constructors;
  `kbd_symbol` maps W3C key names to ⌘⌃⇧⌥ or words
  (`src/ui/kbd.rs:16-128`).
- `CircularProgress` is a `RenderOnce` with a `canvas` for the arc
  (`src/ui/progress.rs:24-148`).
- `TabRail` is an index-dispatching `RenderOnce` with `TabRailItem`
  (`src/ui/tab_rail.rs:26-187`).

### 5.7 Which component to reach for

| Need | Use |
|---|---|
| Primary/secondary/ghost/destructive action | `Button` |
| Icon-only action with tooltip | `IconButton` |
| Toggle | `Toggle` |
| Dropdown trigger | `Select` + window-owned `MenuState` |
| Text field | `Entity<TextInputState>` + `TextInput` |
| Settings card/section/row | `Card`, `Section`, `SettingRow` |
| Label + control layout | `Field` / `Subfield` |
| Editor action | `EditorButton` |
| Radio-as-card | `RadioCards` |
| Numeric input | `NumberFieldState` + `NumberField` |
| 2D position pad | `PositionPad` |
| Slider | `Slider` + `SliderDrag` |
| Progress ring | `CircularProgress` |
| Keycap | `KbdChip` |

---

## 6. Theme token shape (`src/theme.rs`)

### 6.1 The resolved palette is a value, not a global

`src/theme.rs:377-468`:

```rust
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub appearance: Appearance,
    pub material: Option<MaterialTokens>,
    pub gray_1: Rgba,
    /* gray_2..gray_12 */
    pub blue_3: Rgba,
    /* blue_4..blue_12 */
    pub red_2: Rgba,
    /* red_3..red_11 */
    pub amber_3: Rgba,
    /* amber_6, amber_11 */
    pub blue_100: Rgba,
    pub blue_500: Rgba,
    pub red_300: Rgba,
    pub gray_200_legacy: Rgba,
    /* legacy grays */
    pub text_primary: Rgba,
    pub editor: EditorPalette,
}
```

`Theme` is `Copy` and cheap. There is no palette global. Each window stores one
`Theme` field and refreshes it at the top of `render`:

```rust
// src/main_window.rs:1955-1957
fn sync_appearance(&mut self, window: &Window, cx: &gpui::App) {
    self.theme.refresh(window, cx, true);
}
```

`Theme::for_window` (`src/theme.rs:699-716`) resolves preference + OS appearance
+ native material into a fresh value:

```rust
pub fn for_window(window: &Window, cx: &App, with_material: bool) -> Self {
    let preference = current_preference(cx);
    let system = platform::window_is_dark(window)
        .map(|dark| if dark { Appearance::Dark } else { Appearance::Light })
        .unwrap_or_else(|| Appearance::from_window(window.appearance()));
    let theme = Self::new(resolve_appearance(preference, system));
    if with_material { theme.with_material(platform::active_material(cx)) } else { theme }
}

pub fn refresh(&mut self, window: &Window, cx: &App, with_material: bool) -> bool {
    let next = Self::for_window(window, cx, with_material);
    if next.appearance != self.appearance || next.material_kind() != self.material_kind() {
        *self = next;
        true
    } else { false }
}
```

### 6.2 Preference global vs resolved palette

Only the **preference** is global (`ThemePreference(AppTheme)`,
`src/theme.rs:82-98`). The resolved palette is per-window, per-frame. That
avoids a global invalidation cascade every time a window draws.

`theme::init(preference, cx)` is called once at boot; `set_preference` updates
the global; `app_windows::broadcast_theme` loops every open window and calls
`cx.notify()` so each `render` resolves the new palette
(`src/app_windows.rs:800-882`).

### 6.3 Semantic helpers

Rather than call sites doing `theme.gray_3`, Cap exposes helpers that fold the
native-material remaps into the token read:

- `shell_bg()` / `shell_border()`
- `header_bg()` / `header_border()`
- `body_text()` / `body_fill(n)` / `body_hover_fill(n)` / `body_border(n)`
- `settings_*()` (`settings_window_radius`, `settings_sidebar_bg`,
  `settings_content_bg`, `settings_card_bg`, `settings_fill`, `settings_hover`,
  `settings_selection`, `settings_border`, `settings_text`, `settings_muted`)
- `teleprompter_*()`

`src/theme.rs:874-1152`. This is the right place for your token semantics.

### 6.4 Fixed light theme for Coderune

Coderune's own spec is `docs/rust-gpui/01-design-system.md`. It is a **fixed
light theme**, not Cap's Radix/dark/material system:

- one `Theme` (or `Palette`) `Copy` struct built from `theme::color::*`;
- no `Appearance`, no `MaterialTokens`, no vibrancy/liquid-glass;
- `TextRole` enum owns size/line-height/family;
- no raw hex outside the theme module.

Borrow the **shape** (Copy token value resolved at render, semantic helpers,
named per-surface constructors), not the values.

---

## 7. Async boundaries

Cap uses three executors, and the choice is deliberate.

| Work | Executor | API |
|---|---|---|
| UI updates, timers, observers, channel pumps | GPUI foreground | `cx.spawn`, `cx.spawn_in(window, ...)` |
| Filesystem scans, thumbnail decode, CPU work | GPUI background | `cx.background_executor().spawn(...)` |
| Recording engine, kameo actors, ffmpeg, HTTP, transcription | tokio | `gpui_tokio::Tokio::spawn(cx, ...)` or `Tokio::handle(cx).spawn(...)` |
| Escape a current borrow / key dispatch | GPUI deferred | `cx.defer`, `cx.defer_in`, `window.defer` |

### 7.1 Never block the foreground

`src/library.rs:14-17`:

```rust
//! Everything in this module runs on the background executor --
//! [`spawn_decode_pool`] is the one function called from the foreground, and
//! all it does is fan jobs out to that executor. Nothing here touches gpui
//! state.
```

`MainWindow` explicitly starts device enumeration outside `new` because
AVFoundation/window-server enumeration costs ~180ms of a blank window
(`src/main_window.rs:1171-1174`, `src/main_window.rs:1321`).

### 7.2 Tokio → GPUI handoff

`src/feeds.rs:1089-1156` is the canonical shape:

```rust
let set = gpui_tokio::Tokio::spawn(cx, async move {
    // tokio-only work: kameo asks, permission checks, device configuration
    let ready = camera_input_operation(&gate, &current_epoch, epoch, async {
        attach_camera_preview_sender(&actor, sender).await?;
        actor.ask(camera::SetInput { id: selection.id, settings }).await
            .map_err(|e| e.to_string())
    }).await?;
    Ok::<_, String>(/* ... */)
});

let ready = owned_input_readiness(async move {
    set.await.unwrap_or_else(|e| Err(e.to_string()))
}, readiness_epoch, epoch);
self.camera_ready = Some(ready.clone());

cx.spawn(async move |this, cx| {
    let result = ready.await;
    this.update(cx, |this, cx| {
        if this.camera_epoch != epoch { return; }   // stale result guard
        this.camera_input_pending = false;
        match result { /* ... */ }
        cx.notify();
    }).ok();
}).detach();
```

Two invariants:

1. **Return to GPUI through `cx.update`/`this.update`**, never touch entities
   from a tokio thread.
2. **Check an epoch/generation before applying a result.** `camera_epoch`,
   `mic_epoch`, `recording_generation`, `device_format_generation`,
   `input_poll_generation` all exist for this reason.

### 7.3 Kameo actors

Actors are spawned onto the ambient tokio runtime. Because selection happens on
the GPUI main thread, Cap enters the runtime first
(`src/feeds.rs:1351-1355`):

```rust
let actor = actor.unwrap_or_else(|| {
    let _runtime = gpui_tokio::Tokio::handle(cx).enter();
    CameraFeed::spawn(CameraFeed::default())
});
```

Actor messages always go through `gpui_tokio::Tokio::spawn`, never directly from
the UI thread.

### 7.4 Channels

- `flume` for bounded/unbounded cross-thread queues.
- Camera preview is `flume::bounded(4)` so a stalled UI drops frames instead of
  ballooning (`src/feeds.rs:1362-1363`).
- Mic meter is `bounded(16)` plus a `bounded(1)` wake channel so audio callbacks
  only wake the UI when the level changes (`src/feeds.rs:1429-1455`).
- `tokio::sync::watch` for start-gate/primed state inside the engine
  (`src/session.rs:770`, `src/recording.rs`).
- `flume::Receiver::recv_async().await` inside `cx.spawn` is the standard pump:

```rust
cx.spawn(async move |_this, cx| {
    while let Ok(frame) = frame_rx.recv_async().await {
        cx.update(|cx| app_windows::deliver_camera_frame(frame, cx));
    }
})
```

### 7.5 Cancellation

Dropping a `gpui::Task` cancels it. Cap relies on this heavily:

- `Option<Task<()>>` fields for polls, ticks, scans, drags, debounces.
- Assigning over a task aborts the previous one.
- `src/settings_window.rs:747-750` documents the collapsible ticker: "Dropping
  it cancels, so a re-toggle mid-flight replaces the ticker rather than racing
  it."
- `ActiveRecording`/`discarding` tasks wait for teardown before a new start
  (`src/session.rs:476`, `1067`).

Prefer `Option<Task<()>>` + replace over manual cancellation flags. Use a
generation only when a result can arrive after the task is dropped.

---

## 8. Screen organization in detail

### 8.1 Main window (`src/main_window.rs`, 7.8k lines)

- Fixed 330×435 shell, custom header, hand-drawn traffic lights, body scroll.
- `Mode { Instant, Studio, Screenshot }` (`:104-172`), persisted through the
  shared store.
- `Panel { Device(DeviceMenu), Target(TargetType), Library(LibraryKind), ModeInfo }`
  (`:807-818`). One panel at a time; opening replaces the body.
- `render` (`:2990-3094`) stacks:
  header → body → mode hover → recording overlay → recovery toast →
  preflight → microphone warning → sign-in prompt → idle error.
- Cross-window actions call `app_windows` and `RecordingSession::global(cx)`;
  they do not own the recording lifecycle.
- Device state comes from `Feeds::global(cx)`. The main window observes it and
  mirrors selection into its own fields (`:1126-1168`).
- Library rows are scanned only while the library panel is open; the scan task
  is dropped on close (`src/main_window.rs:4501-4717`).

### 8.2 Settings window (`src/settings_window.rs`, 5.5k lines)

- 782×775, resizable to 780×560 (`:50-53`), native traffic lights at `(22, 15)`
  (`:64-69`).
- `Page` enum + `ALL` + `label()` + `slug()` + `from_slug()`
  (`:90-180`).
- Sidebar is a vertical nav list, not `TabRail`: profile, nav, account
  (`:1901-1929`).
- `render_content` is the router (`:2352-2399`):

```rust
.children(match self.page {
    Page::General => self.render_general(window, cx),
    Page::Quality => self.render_recording_quality(cx),
    Page::Recordings => self.render_recordings(cx),
    Page::Screenshots => self.render_screenshots(cx),
    // ...
})
```

- Pages beyond General/Recordings/Screenshots live in
  `src/settings_pages.rs` as `impl SettingsWindow` extensions, with their state
  in `PagesState`. This keeps the shell file readable.
- `set_page` re-targets a live window and reloads settings from disk
  (`:935-953`).
- `page_shown` starts page-specific fetches and cancels the other page's poll
  (`:984-1010`).
- Every input is an `Entity<TextInputState>` built in `new` so its focus handle
  survives page changes (`:794-841`).

### 8.3 Onboarding (`src/onboarding_window.rs`, 3k lines)

- `Step { Permissions, Overview, Instant, Studio, Screenshot, Toggle, Shortcuts, Faq }`
  (`:113-176`).
- `Step::first(macos, permissions_only)` and `next/previous` encode the flow.
- `welcome` is a separate overlay state on top of the first step.
- `Render` picks the step then applies a slide/fade keyed by `step_revision`
  (`:2763-2803`).
- The window is dynamic-size (58% of display width, clamped 860–1080;
  `src/app_windows.rs:1494-1522`), transparent, `titlebar: None`.

### 8.4 Library (`src/library.rs`, 4.5k lines)

- No GPUI UI at all. Pure `std::fs` scan + metadata parsing + thumbnail cache.
- `RecordingItem`, `ScreenshotItem`, `RecentItem`, `MediaKind`, `RecordingMode`,
  `RecordingStatus` (`:44-268`).
- `known_recordings_dirs()` merges current + default + previous paths, deduped
  by canonical path (`:100-118`).
- `recent_media()` sorts by bundle ctime, caps at 9 (`:1320-1334`).
- Thumbnails are downsampled to 392×224 and cached as JPEG q80 under the OS
  cache dir (`:1340-1441`).
- `spawn_decode_pool` fans decode jobs to `BackgroundExecutor` with at most 8
  workers (`:1369-1425`).

This is the correct separation: **filesystem/domain code never imports a
`Window` or `Context<T>`** (it only uses `App` for `open_url` helpers).

---

## 9. Testing patterns

### 9.1 The test-support feature

`Cargo.toml` dev-dependencies:

```toml
[dev-dependencies]
gpui = { git = "...", rev = "...", features = ["test-support"] }
tokio = { version = "1", features = ["rt-multi-thread", "time", "test-util", "net", "io-util", "macros"] }
tempfile = "3"
```

`test-support` is deliberately dev-only so the shipped binary's feature set is
unchanged. The multi-thread tokio dev feature exists because editor smoke tests
block the test thread while tokio workers run.

### 9.2 Pure unit tests

Prefer pure structs with no window:

- `MenuState` keyboard contract (`src/ui/menu.rs` tests).
- Slider snapping (`src/ui/slider.rs`).
- Progress labels (`src/ui/progress.rs:150-171`).
- Store read/write/atomicity (`src/store.rs` tests).
- Session phase transitions and quit policy (`src/session.rs`,
  `src/menus.rs:1027-1190`).
- `kbd_symbol` mapping (`src/ui/kbd.rs:130-153`).
- `keyboard_nav::direction` (`src/ui/keyboard_nav.rs:162-201`).

These tests give the highest value per line and do not need a graphics context.

### 9.3 GPUI visual/interaction tests

`src/settings_window/recording_action_tests.rs:1-133` is the template:

```rust
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, size};

struct RecordingActions { settings: Entity<SettingsWindow> }

impl Render for RecordingActions {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.settings.update(cx, |settings, cx| {
            settings.render_recordings_list(cx).into_any_element()
        })
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| { window.refresh(); window.draw(cx).clear(); });
}

#[gpui::test]
fn held_recording_action_does_not_move_to_a_different_recording(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(600.), px(300.)), |window, cx| { /* root */ });
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    draw(&mut visual);
    visual.simulate_mouse_move(link, None, Modifiers::none());
    visual.simulate_mouse_down(link, MouseButton::Left, Modifiers::none());
    // mutate model between down and up
    draw(&mut visual);
    visual.simulate_mouse_up(link, MouseButton::Left, Modifiers::none());
    assert_eq!(visual.opened_url(), None);
}
```

Use `TestAppContext` for entities and `VisualTestContext` for input. `draw`
forces a frame; `simulate_*` drives real dispatch.

### 9.4 Integration tests (`tests/`)

- `tests/editor_frame0.rs` links the real media/edit stack and renders a real
  frame headlessly. It **skips** if no fixture is available and accepts a
  `CAP_GPUI_E0_PROJECT` override.
- `tests/camera_convert.rs` is a pixel-exactness guard for an IOSurface
  conversion; gated `#![cfg(target_os = "macos")]`.
- Tests use `CAP_GPUI_APP_DATA_DIR`, `CAP_GPUI_TAURI_STORE`,
  `CAP_GPUI_RECORDINGS_DIR` to sandbox writes.

### 9.5 Asset tests

`src/assets.rs:378-454` asserts both directions:

- every `"icons/…"` string in the source is embedded;
- every embedded icon is referenced;
- same for images;
- fonts/icons resolve.

This catches GPUI's silent failure mode: an unregistered `svg()` path draws
nothing and logs nothing.

### 9.6 What to test in the new app

1. Pure theme/token mapping and text roles.
2. Menu keyboard state machine and any pure reducer.
3. Store read-modify-write and atomic rename.
4. One `#[gpui::test]` per interactive contract (menu open/commit/dismiss;
   field ESC/Enter; tab focus).
5. Asset completeness both directions.
6. A smoke test that opens the main window in `TestAppContext` and draws one
   frame.

---

## 10. Logging and crash evidence

`src/main.rs:184-274`:

- Default filter:
  `warn,cap_gpui=info,wgpu_core=error,wgpu_hal=error,zbus::proxy=error`.
- `tracing_subscriber::fmt` to stdout plus a **daily rolling file** under the
  app's log dir, prefixed `cap-gpui.log` (deliberately different from the Tauri
  app's prefix so the two never interleave).
- The writer is `tracing_appender::non_blocking`; keep its `WorkerGuard` alive
  for the whole of `main` or the last lines never reach disk.
- `crash_capture::install` runs immediately after logging:
  panic hook, Unix fatal-signal handlers, Windows unhandled-exception and
  stack-overflow handlers, stderr redirection, a session marker for unclean
  exits (`src/crash_capture.rs:1-140`).
- `crash_capture::begin_session` is called **after** the single-instance lock,
  so a second launch cannot touch the running instance's marker.

A new app should keep: `tracing` + env filter, one rolling file, a panic hook,
and a clean-exit marker. It can skip platform signal handlers initially.

---

## 11. What depends on Cap's Zed fork, and how to adapt to crates.io `gpui 0.2.2`

Cap builds against `wingleeio/zed` rev `5d1f83d9…` through three crates:
`gpui`, `gpui_platform`, `gpui_tokio`. crates.io `gpui 0.2.2` is one crate with
the platform inside it. The following differences are verified by reading
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2`.

### 11.1 Boot and application

| Cap (fork) | crates.io 0.2.2 | Adapt |
|---|---|---|
| `gpui_platform::application()` | `gpui::Application::new()` | Replace constructor. |
| `gpui_platform` feature flags (`wayland`, `x11`, `font-kit`, `runtime_shaders`) | `gpui` default features already include `font-kit`, `wayland`, `x11`, `windows-manifest`; `runtime_shaders` is a feature | Use `gpui = "0.2.2"`; enable `runtime_shaders` only if needed. |
| `gpui_tokio::init_from_handle` / `Tokio::spawn` | **No `gpui_tokio` crate on crates.io** | Write a small tokio bridge (§11.2). |
| `app.on_reopen(...)` | Exists: `Application::on_reopen` (`app.rs:198`) | Same call. |
| `app.on_open_urls(...)` | Exists (`app.rs:174-190`) | Same call. |
| `Application::with_assets` | Exists (`app.rs:155`) | Same call. |

### 11.2 Tokio bridge for 0.2.2

`gpui_tokio` is a thin adapter: runtime handle in a `Global`, spawn on the tokio
runtime, and a `Task` that keeps the handle alive. A minimal replacement:

```rust
use gpui::{App, Global};
use tokio::runtime::Handle;

struct TokioRuntime(Handle);
impl Global for TokioRuntime {}

pub fn init(cx: &mut App, handle: Handle) {
    cx.set_global(TokioRuntime(handle));
}

pub fn spawn<F>(cx: &App, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    cx.global::<TokioRuntime>().0.spawn(future)
}
```

For kameo, enter the handle before `Actor::spawn`:

```rust
let _guard = cx.global::<TokioRuntime>().0.enter();
CameraFeed::spawn(CameraFeed::default())
```

For actor allows, use `tokio::task::JoinHandle` and convert to a GPUI task only
when you need cancellation:

```rust
let join = spawn(cx, async move { actor.ask(msg).await });
cx.spawn(async move |this, cx| {
    let result = join.await;
    this.update(cx, |this, cx| { /* apply */ cx.notify(); }).ok();
}).detach();
```

Do not run a tokio runtime on the GPUI main thread. Do not call
`Handle::block_on` from an entity update.

### 11.3 Window APIs

| Cap | 0.2.2 | Adapt |
|---|---|---|
| `WindowOptions::app_owns_titlebar_drag` | **Absent** | Drop it. Use `div().window_control_area(WindowControlArea::Drag)` on the header and/or call `window.start_window_move()` from `on_mouse_down`. |
| `Window::retained_visibility()` / `set_retained_visibility(...)` | **Absent** | Linux-only Cap feature. For a new app, keep windows alive and hide/show via the platform layer, or accept that hiding a window is a close/reopen. Do not port this unless you need Wayland retained visibility. |
| `Window::kick_display_link(window)` | **Absent** | Cap fork fix for macOS 26 hidden-window display links. Skip unless you reproduce transparent/hidden windows on macOS 26. |
| `Window::show_native` / `hide_native` | **Absent** (they are Cap's `platform` helpers over `objc2`) | Use `Window::minimize_window`, `Window::zoom_window`, `Window::activate_window`, `Window::remove_window`, `Window::set_window_title`; write your own native shim only if needed. |
| `Window::refresh()` | Exists (`window.rs:1367`) | Same. |
| `Window::remove_window()` | Exists (`window.rs:1375`) | Same. |
| `Window::activate_window()` | Exists (`window.rs:4112`) | Same. |
| `Window::start_window_move()` | Exists (`window.rs:1753`) | Same. |
| `Window::on_window_should_close(...)` | Exists (`window.rs:4329`) | Same. |
| `Window::window_control_area(...)` | Exists (`elements/div.rs:581`) | Same. |
| `Window::observe_window_appearance(...)` | Exists (`window.rs:1321`) | Same. |
| `Window::handle_input(...)` | Exists (`window.rs:3400`) | Same. |
| `Window::has_active_prompt()` | Absent | Track a `prompt_open: bool` on your root, or use the prompt's own state. |
| `Window::set_reduce_motion(...)` | Absent | Cap's fork feeds the OS Reduce Motion flag into GPUI. In 0.2.2, either query the OS yourself and gate your animations, or always animate. |
| `Window::occlude()` | `occlude_mouse()` exists (`elements/div.rs:575`) | Rename. |
| `role`, `aria_label`, `aria_toggled`, `Role`, `Toggled`, `focus_visible` | **Absent** in published 0.2.2 (no accessibility module) | This is the biggest fork-only area. Do not copy Cap's accessibility calls. Either vendor/patch GPUI for accessibility, or implement your own native accessibility bridge and keep labels in your own component metadata. |

### 11.4 Menu APIs

`cx.bind_keys`, `cx.on_action`, `cx.set_menus` and `Menu`/`MenuItem` all exist in
0.2.2 (`src/platform/app_menu.rs`). Differences:

- `gpui::Menu` in 0.2.2 has only `name` and `items`; Cap's fork adds `disabled`.
  Remove `disabled: false`.
- `MenuItem::action` has no `.disabled(bool)` builder. Gate the *action
  handler*, or rebuild the menu when the row must be greyed.
- `MenuItem::os_action` and `MenuItem::os_submenu` exist; the Edit menu pattern
  (Undo/Redo/Cut/Copy/Paste/SelectAll with `OsAction`) ports directly.
- `App::set_menus` is available; `App::on_action` is available.
- Cap patches standard macOS menu rows by position (`platform::patch_standard_application_menu`).
  That is platform-specific and should not be copied.

### 11.5 Fork-only window behaviour to avoid

Cap's Cargo comment says the pinned rev carries a transparent-window
destination-alpha fix. Related fork patches in `patches/` cover closed-window
events, idle frames, streaming images, device loss, Linux/Windows window
behaviour, and glass. If your new app uses **opaque windows and ordinary
titlebars**, none of this matters. If you need a transparent rounded macOS
shell, expect to patch or vendor GPUI; crates.io 0.2.2 will not match Cap's
rendering.

### 11.6 Recommended Cargo shape for the new app

```toml
[workspace]
resolver = "2"
members = [".", "kernels"]

[package]
name = "coderune-desktop"
edition = "2024"
publish = false

[dependencies]
gpui = "0.2.2"
tokio = { version = "1", features = ["rt-multi-thread", "time", "sync"] }
flume = "0.11"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

[dev-dependencies]
gpui = { version = "0.2.2", features = ["test-support"] }

[profile.dev]
opt-level = 0
incremental = true

[profile.dev.package."*"]
opt-level = 2
debug = "line-tables-only"

[profile.dev.package.coderune-desktop-kernels]
opt-level = 3

[profile.release]
codegen-units = 1
lto = "thin"
strip = "symbols"
```

Pin `image` to the version GPUI's `RenderImage` uses if you feed decoded frames
into GPUI. Check the resolved `image` version in `gpui 0.2.2`'s lockfile before
adding an image dependency.

---

## 12. Ten rules for the new app

1. **Use a standalone Cargo workspace with a thin binary crate and a kernels
   crate.** Keep GPUI's dependency tree out of any root workspace; set
   `resolver = "2"`; build dependencies at `opt-level = 2`, app at 0, hot loops
   at 3. Depend on crates.io `gpui = "0.2.2"`, not a fork, unless you accept
   maintaining patches.

2. **Boot in one place, in dependency order.** Logging/crash/single-instance →
   runtime → `Application::new().with_assets(...)` → fonts → settings → theme →
   app-scoped globals → main window → window registry → menus/tray/hotkeys.
   Nothing that dispatches into windows may be installed before the registry
   exists.

3. **Keep one `AppWindows` global registry and all open/focus/close functions
   in one module.** Reuse a live window by re-targeting it. Prune dead handles
   on reopen. Never let a window own a handle to another window.

4. **Cross-window state is an `Entity<T>` wrapped in a private `Global`.**
   Use the `struct XGlobal(Entity<X>); impl Global for XGlobal` pattern and a
   `global(cx) -> Entity<X>` accessor. Window-private state stays in the root
   struct. Persisted settings stay a plain file store, not a global entity.

5. **Open and focus windows only from `cx.defer` or a spawned task.** Never
   open a window inside an entity update. Never mutate native window state
   inside a GPUI update. Read the handle inside, act outside.

6. **Every window is a root entity implementing `Render`, and every screen is
   an enum inside that root.** `Page` for settings, `Step` for onboarding,
   `Panel` for the main window. No global router, no navigation stack. The
   `Render` body is a single match.

7. **Resolve the theme into a `Copy` value at render time.** Store one `Theme`
   on each root; call `theme.refresh(...)` at the top of `render`. Use semantic
   helpers (`body_fill`, `settings_text`, `text_primary`) rather than raw scales
   at call sites. For Coderune, use the fixed light tokens in
   `docs/rust-gpui/01-design-system.md`; do not port Cap's dark/material system.

8. **Components are `IntoElement` + `RenderOnce` builders that take `&Theme`
   and named per-surface constructors.** Handlers are
   `impl Fn(&ClickEvent, &mut Window, &mut App)`, index handlers take `&usize`.
   Stateful widgets expose a separate `Entity<State>` model that emits typed
   events; the element only reads it.

9. **Separate the three executors and always return to GPUI through
   `cx.update`.** UI-bound work on `cx.spawn`, CPU/fs on
   `background_executor().spawn`, tokio for media/actors/network. Guard results
   with an epoch/generation. Prefer `Option<Task<()>>` + replace for
   cancellation, and bounded `flume` channels for frame/sample pumps.

10. **Persist with read-modify-write and atomic rename, and test the contracts,
    not the pixels.** Preserve unknown keys. Load on show. Write pure unit tests
    for state machines and token mapping; add `#[gpui::test]` visual tests for
    interaction; add both-direction asset completeness tests; sandbox every
    path through environment variables in tests.

---

## Appendix: file map worth reading in order

| Order | File | Why |
|---|---|---|
| 1 | `Cargo.toml` | Workspace, pins, profiles, patches. |
| 2 | `src/main.rs` | Boot order, runtime, window options, lifecycle. |
| 3 | `src/app_windows.rs` | Window registry, open/close, observers, focus. |
| 4 | `src/theme.rs` | Token shape and per-window resolution. |
| 5 | `src/ui/mod.rs` | Component contract and surface list. |
| 6 | `src/ui/button.rs` | Canonical stateless component. |
| 7 | `src/ui/text_input.rs` | Canonical stateful component + IME. |
| 8 | `src/ui/menu.rs` | Pure state machine + overlay rendering. |
| 9 | `src/session.rs` | Entity-in-global, async lifecycle, epochs. |
| 10 | `src/feeds.rs` | Kameo actors, tokio bridge, channel pumps. |
| 11 | `src/store.rs` | Shared-file persistence contract. |
| 12 | `src/settings_window.rs` | Page router, live-window re-target. |
| 13 | `src/onboarding_window.rs` | Step router, transition pattern. |
| 14 | `src/assets.rs` | Embedded assets, font registration, asset tests. |
| 15 | `src/library.rs` | Pure domain/filesystem layer. |
| 16 | `src/menus.rs` | App menu actions, key bindings, quit policy. |
| 17 | `src/settings_window/recording_action_tests.rs` | Visual test template. |

Do not port: Cap's recording engine adapters, ffmpeg/editor stack, native
macOS material/traffic-light shims, global-hotkey/tray code, auto-screenshot
environment hooks, i18n build, or its Radix palette. Do port: workspace shape,
boot order, `AppWindows` registry, `Entity`+`Global` state pattern, `Copy`
theme, component builder conventions, executor boundaries, store contract, and
the test layering.
