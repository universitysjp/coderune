# 03 — GPUI 0.2.2 API guide

Status: reference for the native Rust/GPUI rewrite.
Target: **crates.io `gpui 0.2.2`** (Zed's GPU UI framework, published from the
`zed-industries/zed` monorepo snapshot). This is *not* the `@gpuix` React wrapper.

Everything below was read out of the real crate source and, where marked
**verified**, compiled and/or executed against it. Anything I could not verify is
called out in [§18](#18-what-i-could-not-verify).

---

## 0. How this guide was verified

| Check | Command | Result |
| --- | --- | --- |
| Crate metadata + features | `cargo info gpui` | resolved 0.2.2 |
| Source on disk | `~/.cargo/registry/src/index.crates.io-*/gpui-0.2.2` | 27k LOC `src/`, `examples/`, `docs/`, `tests/` |
| Default build on macOS (CLT only, no full Xcode) | `cargo build` | **fails**: build script needs `xcrun metal` (see §1.3) |
| Build with `runtime_shaders` | `cargo build` | **ok** (9.7 s incremental, deps warm) |
| Hello world (§2) | `cargo build` | compiles |
| Stateful app + text field (§16) | `cargo build --example doc_app` | compiles |
| API sweep (lists, svg, img, assets, fonts, scroll, groups) | `cargo build --example sink` | compiles |
| Action dispatch, clicks, `#[gpui::test]` args | `cargo test` (5 tests) | passes |
| Binary launch | run hello world 3 s, then kill | no panic, no stderr |
| MSRV | crate sets no `rust-version`; built here with Rust 1.91.0 | — |

The probe crate lives outside the repo at
`/private/var/folders/wk/.../T/opencode/gpui-probe` and is not part of the
workspace. Nothing in the repository was modified except this file.

Source locations cited below are relative to the extracted crate root, e.g.
`src/app.rs:943` means `~/.cargo/registry/src/*/gpui-0.2.2/src/app.rs:943`.

---

## 1. Crate, features, toolchain

### 1.1 Cargo setup

```toml
# Cargo.toml
[package]
name = "coderune-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = "0.2.2"
# Only needed by the hand-written text field in §16 (grapheme-aware cursor
# movement). GPUI does not re-export it.
unicode-segmentation = "1"

[dev-dependencies]
# `#[gpui::test]`, TestAppContext and VisualTestContext are behind this feature.
gpui = { version = "0.2.2", features = ["test-support"] }
```

If your machine cannot compile Metal shaders (see §1.3), use:

```toml
gpui = { version = "0.2.2", features = ["runtime_shaders"] }
```

`gpui` re-exports `anyhow::Result` as `gpui::Result`, so `gpui::Result<T>` ==
`anyhow::Result<T>`; that is the error type of `open_window` and `AssetSource`.

### 1.2 Feature flags

From `Cargo.toml`:

| Feature | Effect |
| --- | --- |
| `default` | `font-kit`, `wayland`, `x11`, `windows-manifest` |
| `font-kit` | system font enumeration (zed fork `zed-font-kit`) |
| `wayland`, `x11` | Linux windowing/GPU backends. These are **features of `gpui` itself**; there is no separate `gpui_platform` crate in 0.2.2. Their deps are `cfg(target_os = "linux"/"freebsd")`-gated, so leaving them on (the default) is harmless on macOS — verified: the macOS build above ran with default features enabled. |
| `windows-manifest` | Windows resource manifest only (`embed-resource` build dep) |
| `macos-blade` | use the blade/wgpu-style backend on macOS instead of Metal |
| `runtime_shaders` | do not compile `shaders.metal` at build time; ship the source and compile at runtime |
| `test-support` | test harness: `gpui::test`, `TestAppContext`, `VisualTestContext`, debug selectors, extra `#[cfg]` APIs. Also turns on `leak-detection`, `rand`, `collections/test-support`, `util/test-support`, `http_client/test-support`. |
| `inspector` | devtools inspector (`gpui_macros/inspector`) |
| `leak-detection` | backtraces for leaked entities |
| `screen-capture` | `scap` based screen capture |

There is **no** feature that adds widgets. See §1.5.

### 1.3 macOS prerequisites (read this first)

A default `cargo build` on macOS requires **full Xcode**, not just the Command
Line Tools, because `build.rs` shells out to the Metal compiler:

```
cargo::error=metal shader compilation failed:
xcrun: error: unable to find utility "metal", not a developer tool or in PATH
```

`build.rs` also runs `bindgen` (needs `libclang`, satisfied by the CLT here) and
`cbindgen` to generate the scene header. Options, in order of preference:

1. **`features = ["runtime_shaders"]`** — build script skips `xcrun metal` /
   `metallib` and embeds the stitched shader source; Metal compiles at runtime
   on first use. Verified: builds and launches on a machine with CLT only.
   Cost: slightly slower startup and shader errors surface at runtime.
2. **`features = ["macos-blade"]`** — swaps the macOS renderer for the blade
   backend. *Not verified here.*
3. Install full Xcode and select it (`xcode-select -s /Applications/Xcode.app`)
   so `xcrun metal` exists.

Build time is significant: the dependency tree is ~700 crates (Metal, CoreText,
resvg, taffy, image, reqwest via `http_client`, …). Budget a cold build of
5–15 minutes.

### 1.4 Is `gpui` 0.2.2 sufficient on macOS?

Yes. macOS (Metal) and Windows are first-class in this snapshot, Linux has
x11/wayland. The published crate is the whole framework: app/window lifecycle,
layout (taffy), text shaping (CoreText + a fallback stack), rendering, actions
and keymaps, test harness. There is no companion crate you must add.

Two practical caveats:

* It is a **snapshot**, not a stable release: API differs from older blog posts
  and from Zed's current `main`. In particular `docs/key_dispatch.md` *inside the
  crate* is stale (it shows `#[gpui::action]`, which does not exist in 0.2.2 —
  verified compile error) and shows a 4-argument `on_action` closure that is not
  the 0.2.2 signature.
* Semver-wise it is `0.2.x`; pin the exact version.

### 1.5 There is no component library

Verified by enumerating `src/elements/` and grepping for `pub struct Button`,
`Input`, `TextField`, `Checkbox`, `Label`: **none exist**. The public element
constructors are:

`div()`, `list()`, `uniform_list()`, `img()`, `svg()`, plus `StyledText` /
`InteractiveText` (text is normally just `.child("str")`). `Anchored`,
`Deferred`, `Surface`, `Canvas`, `ImageCacheElement`, `Animation` exist as
low-level helpers.

Consequences for this project:

* Button, text field, checkbox, select, tabs, dialog, toast, menu *widgets*,
  tooltip content, etc. must be built from `div()` + interaction handlers. GPUI
  gives you `.hover()`, `.active()`, `.on_click()`, `.focusable()`, cursor
  styles, shadows/borders/radii — the visual layer — but no semantics, no ARIA,
  no default keyboard behaviour. Note `.active()`/`.on_click()`/`.focusable()`
  need `.id(..)` first (see §6.3).
* Tab *ordering* exists (`.tab_index(isize)`, `.tab_stop(bool)`, `.tab_group()`,
  `window.focus_next()` / `focus_prev()`), but the Tab key is **not** bound for
  you: bind an action to `"tab"` that calls `window.focus_next()`, or nothing
  will move focus.
* Menus (native menu bar) *are* built in: `cx.set_menus(vec![Menu { .. }])` and
  `MenuItem::action/separator/submenu/os_submenu`.

---

## 2. Hello world (verified)

```rust
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
```

Line-by-line:

* `Application::new()` builds the app but does not run it. `.run(cb)` takes
  `'static + FnOnce(&mut App)` and blocks for the lifetime of the app
  (`src/app.rs:132`, `:174`). It returns `()`.
* The callback runs on the main thread *after* platform init. Create windows
  here, bind keys here, set menus here.
* `Bounds::centered(None, size(px(500.), px(500.)), cx)` — `None` means "primary
  display". Returns `Bounds<Pixels>`.
* `WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }`.
  If `window_bounds` is `None`, `Window::new` calls `default_bounds()`
  (`src/window.rs:905`) which offsets from the current active window, or uses the
  primary display's default bounds (`DEFAULT_WINDOW_SIZE` = 1536×864). Always
  pass explicit bounds for a predictable layout.
* The `|_, cx| cx.new(|_| HelloWorld { .. })` closure receives
  `(&mut Window, &mut App)` and must return `Entity<V>` where
  `V: Render + 'static`.
* `cx.activate(true)` brings the app forward (`ignoring_other_apps`).
* `div()` is a *function*, not a macro. `.child(format!(..))` works because
  `String`/`&'static str`/`SharedString` implement `IntoElement`
  (`src/elements/text.rs:69,77,135`).

---

## 3. Application / App (exact signatures)

```rust
impl Application {
    pub fn new() -> Self;
    pub fn headless() -> Self;                                  // no windows; SSH/CI
    pub fn with_assets(self, asset_source: impl AssetSource) -> Self;
    pub fn with_http_client(self, http_client: Arc<dyn HttpClient>) -> Self;
    pub fn run<F>(self, on_finish_launching: F) where F: 'static + FnOnce(&mut App); // -> ()
    pub fn on_open_urls<F>(&self, cb: F) -> &Self;
    pub fn on_reopen<F>(&self, cb: F) -> &Self;                 // macOS dock re-open
    pub fn background_executor(&self) -> BackgroundExecutor;
    pub fn foreground_executor(&self) -> ForegroundExecutor;
    pub fn text_system(&self) -> Arc<TextSystem>;
}
```

`App` is the root context. The parts you will use:

```rust
impl App {
    pub fn open_window<V: 'static + Render>(
        &mut self,
        options: WindowOptions,
        build_root_view: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> anyhow::Result<WindowHandle<V>>;                        // src/app.rs:943

    pub fn new<T: 'static>(&mut self, build: impl FnOnce(&mut Context<T>) -> T) -> Entity<T>;
    pub fn activate(&self, ignoring_other_apps: bool);
    pub fn quit(&self);
    pub fn windows(&self) -> Vec<AnyWindowHandle>;
    pub fn active_window(&self) -> Option<AnyWindowHandle>;

    pub fn bind_keys(&mut self, bindings: impl IntoIterator<Item = KeyBinding>);
    pub fn clear_key_bindings(&mut self);
    pub fn on_action<A: Action>(&mut self, listener: impl Fn(&A, &mut Self) + 'static);

    pub fn set_menus(&self, menus: Vec<Menu>);
    pub fn on_window_closed(&self, cb: impl FnMut(&mut App) + 'static) -> Subscription;
    pub fn on_app_quit<Fut>(&self, cb: impl FnMut(&mut App) -> Fut + 'static) -> Subscription
        where Fut: 'static + Future<Output = ()>;

    pub fn observe<W>(&mut self, entity: &Entity<W>,
        on_notify: impl FnMut(Entity<W>, &mut App) + 'static) -> Subscription;
    pub fn subscribe<T, Event>(&mut self, entity: &Entity<T>,
        on_event: impl FnMut(Entity<T>, &Event, &mut App) + 'static) -> Subscription
        where T: EventEmitter<Event>, Event: 'static;

    pub fn focus_handle(&self) -> FocusHandle;                   // allocate a new handle
    pub fn text_system(&self) -> &Arc<TextSystem>;
    pub fn read_from_clipboard(&self) -> Option<ClipboardItem>;
    pub fn write_to_clipboard(&self, item: ClipboardItem);
    pub fn to_async(&self) -> AsyncApp;
    pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
        where AsyncFn: AsyncFnOnce(&mut AsyncApp) -> R + 'static, R: 'static;
}
```

`App` is `!Send`; everything UI happens on the main thread. `Subscription` is
`#[must_use]`-ish: call `.detach()` to keep an observer alive for the app's life,
or store it to cancel on drop.

Typical shutdown wiring (from `examples/on_window_close_quit.rs`):

```rust
cx.on_window_closed(|cx| {
    if cx.windows().is_empty() {
        cx.quit();
    }
})
.detach();
```

---

## 4. Windows

```rust
#[derive(Debug)]
pub struct WindowOptions {
    pub window_bounds: Option<WindowBounds>,
    pub titlebar: Option<TitlebarOptions>,
    pub focus: bool,          // default true
    pub show: bool,           // default true
    pub kind: WindowKind,     // Normal | PopUp | Floating
    pub is_movable: bool,     // true
    pub is_resizable: bool,   // true
    pub is_minimizable: bool, // true
    pub display_id: Option<DisplayId>,
    pub window_background: WindowBackgroundAppearance, // Opaque | Transparent | Blurred
    pub app_id: Option<String>,
    pub window_min_size: Option<Size<Pixels>>,
    pub window_decorations: Option<WindowDecorations>, // Linux only
    pub tabbing_identifier: Option<String>,            // macOS native tabs
}

pub enum WindowBounds {
    Windowed(Bounds<Pixels>),
    Maximized(Bounds<Pixels>),   // bounds = restore size
    Fullscreen(Bounds<Pixels>),  // bounds = restore size
}

impl WindowBounds {
    pub fn get_bounds(&self) -> Bounds<Pixels>;
    pub fn centered(size: Size<Pixels>, cx: &App) -> Self;
}

#[derive(Debug, Default)]
pub struct TitlebarOptions {
    pub title: Option<SharedString>,
    pub appears_transparent: bool,           // macOS + Windows: hide system titlebar
    pub traffic_light_position: Option<Point<Pixels>>, // macOS
}
```

`WindowOptions::default()` is `window_bounds: None`, `titlebar: Some(default)`,
`focus/show/movable/resizable/minimizable: true`, `kind: Normal`.

`WindowHandle<V>` is `Copy + Clone`, type-parameterised by the **root view type**:

```rust
impl<V: 'static + Render> WindowHandle<V> {
    pub fn update<C, R>(&self, cx: &mut C,
        f: impl FnOnce(&mut V, &mut Window, &mut Context<V>) -> R) -> anyhow::Result<R>
        where C: AppContext;
    pub fn read<'a>(&self, cx: &'a App) -> anyhow::Result<&'a V>;
    pub fn entity<C: AppContext>(&self, cx: &C) -> anyhow::Result<Entity<V>>;
    pub fn is_active(&self, cx: &mut App) -> Option<bool>;
}
```

`AnyWindowHandle` is the type-erased variant (`cx.windows()`,
`cx.active_window()`), and `AnyWindowHandle::update(self, cx, |AnyView, &mut Window, &mut App| ..)`
(consumes the handle).

Useful `Window` methods (all `&mut self` unless noted):

```rust
window.focus(&FocusHandle);            // focus moves, keystrokes cleared
window.focused(cx: &App) -> Option<FocusHandle>;
window.focus_next(); window.focus_prev();
window.handle_input(&FocusHandle, impl InputHandler, &App);   // paint-time IME registration
window.paint_quad(PaintQuad);
window.request_layout(style: Style, children: impl IntoIterator<Item = LayoutId>, cx: &mut App) -> LayoutId;
window.text_style() -> TextStyle;      // inherited text style for this element
window.text_system() -> &Arc<WindowTextSystem>;  // Derefs to TextSystem
window.line_height() -> Pixels;
window.rem_size() -> Pixels;
window.bounds() -> Bounds<Pixels>;
window.window_bounds() -> WindowBounds;
window.mouse_position() -> Point<Pixels>;
window.is_window_hovered() -> bool;
window.remove_window();                // closes; triggers on_window_closed
window.refresh();                      // request redraw
window.dispatch_action(Box<dyn Action>, &mut App);
```

Opening/activating multiple windows is plain: call `open_window` again. On macOS
there is one `NSApplication` and windows are independent; `cx.activate(false)`
inside the builder is what `examples/on_window_close_quit.rs` does when it does
not want to steal focus per window.

**Custom titlebar** (macOS/Windows): hide the system bar, then mark a drag
region. `WindowControlArea` is `Drag | Close | Max | Min`.

```rust
WindowOptions {
    titlebar: Some(TitlebarOptions {
        title: Some("Coderune".into()),
        appears_transparent: true,                  // hide the system titlebar
        traffic_light_position: Some(point(px(12.), px(12.))),
    }),
    ..Default::default()
}

// in render(): the custom bar drags the window
div()
    .id("titlebar")
    .h(px(36.))
    .w_full()
    .window_control_area(WindowControlArea::Drag)
    .flex().items_center().child("Coderune")
```

Without `window_control_area(Drag)` the window cannot be moved once the system
titlebar is hidden. Snippet compiled; drag behaviour not exercised.

---

## 5. Geometry and units

```rust
pub struct Pixels(/* private f32 */);       // NOT `Pixels(4.0)`
pub const fn px(pixels: f32) -> Pixels;
pub const fn size<T>(width: T, height: T) -> Size<T>;
pub const fn point<T>(x: T, y: T) -> Point<T>;
pub fn rems(rems: f32) -> Rems;
pub const fn relative(fraction: f32) -> DefiniteLength;   // 1.0 = 100%
pub const fn percentage(value: f32) -> Percentage;
pub fn auto() -> Length;
pub fn phi() -> DefiniteLength;                           // golden ratio
```

* `Pixels`' field is private. Construct with `px(4.)`, convert out with
  `f32::from(p)` / `p.into()`, or `Pixels::from(4.0_f32)`. It implements
  `Add/Sub/Mul<f32>/Div<Pixels>`, `Display` prints `"4px"`.
* `Size<T> { width, height }`, `Point<T> { x, y }` and `Bounds<T> { origin, size }`
  are plain structs with public fields, plus `Bounds::new(origin, size)`,
  `Bounds::from_corners(tl, br)`, `Bounds::centered(display_id, size, cx)`,
  `Bounds::centered_at(center, size)`, `.left()/.right()/.top()/.bottom()`,
  `.center()`, `.localize(&point) -> Option<Point<T>>`, `.contains(&point)`.
* Sizes in the style system are `Length` (`Definite(Pixels|Rems)` or `Auto`);
  `px()`/`rems()`/`relative()` convert via `Into`. Style setters take
  `impl Into<Length>` / `impl Into<DefiniteLength>`, so `.w(px(200.))` and
  `.w(relative(0.5))` both compile.
* Font sizes use `AbsoluteLength`; `.text_size(px(14.))` works, as does
  `rems(1.)`. `TextStyle::font_size.to_pixels(window.rem_size())` converts.

Gotcha: `DefiniteLength` is not `Length`; padding/gap take `DefiniteLength`
(no `auto`), margins and sizes take `Length` (allow `auto`).

---

## 6. Elements

### 6.1 Constructors

| Call | Type | Notes |
| --- | --- | --- |
| `div()` | `Div` | the only layout container. `Styled + InteractiveElement + ParentElement + Element`. |
| `.child("text")`, `.child(String)`, `.child(SharedString)` | text | there is **no `text()` function** in 0.2.2. |
| `StyledText::new("…")` | `StyledText` | explicit styled text; `.with_runs(Vec<TextRun>)`, `.with_default_highlights(style, ranges)`. |
| `svg()` | `Svg` | then `.path("icons/x.svg")` (resolved through `AssetSource`), `.with_transformation(..)`. |
| `img(source)` | `Img` | `source: impl Into<ImageSource>`: `&str`, `String`, `SharedString`, `PathBuf`, `Arc<RenderImage>`, `Arc<Image>`, or a closure. `&str` starting with a URI scheme is fetched over HTTP, otherwise it is treated as an embedded/app-relative resource. `StyledImage`: `.object_fit(ObjectFit::*)`, `.grayscale(bool)`, `.with_loading(..)`, `.with_fallback(..)`. |
| `uniform_list(id, count, f)` | `UniformList` | §11. |
| `list(id, count, f)` | `List` | virtualised, variable-height. |

`ParentElement`:

```rust
fn child(self, child: impl IntoElement) -> Self;
fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self;
fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>);
```

`Render::render` returns `impl IntoElement` — one concrete type. When branches
return different element types, convert with `.into_any_element()` (`AnyElement`).

### 6.2 Styling: `Styled`

`Styled` (`src/styled.rs:21`) is implemented by `Div`, `Stateful<E>`, `Img`,
`Svg`, `UniformList`, … Its methods are a Tailwind-flavoured subset generated by
`gpui_macros::style_helpers!` from `gpui-macros/src/styles.rs`. Important: the
generated names are *not* discoverable from `styled.rs` alone — the macro emits
them. Confirmed names:

**Layout / flex** — `.flex() .block() .grid() .flex_col() .flex_col_reverse()
.flex_row() .flex_row_reverse() .flex_wrap() .flex_nowrap() .flex_1()
.flex_auto() .flex_initial() .flex_none() .flex_grow() .flex_shrink()
.flex_basis(impl Into<Length>) .items_start() .items_center() .items_end()
.items_baseline() .justify_start() .justify_center() .justify_end()
.justify_between() .justify_around() .content_*() .grid_cols(u16) .grid_rows(u16)
.col_span(u16) .row_span(u16) .col_start(i16) .row_start(i16)`

**Sizing** — `.w(impl Into<Length>) .h(..) .size(..) .min_w(..) .min_h(..)
.max_w(..) .max_h(..)` plus the numeric ramp and keywords:
`.w_full() .h_full() .size_full() .w_auto() .size_8() .w_64()`.
Ramp suffixes: `0, 0p5, 1, 1p5, 2, 2p5, 3, 3p5, 4, 5, 6, 7, 8, 9, 10, 11, 12,
14, 16, 20, 24, 32, 40, 48, 56, 64, 72, 80, 96, 112, 128, full, auto` where
allowed (`0p5` = 0.125rem = 2px, `4` = 1rem = 16px, `full` = 100%).
Negative variants exist for margins: `.m_neg_2()`.

**Spacing** — `.p .px .py .pt .pb .pl .pr` (DefiniteLength), `.m .mx .my .mt .mb
.ml .mr` (Length, `auto` allowed), `.gap .gap_x .gap_y` (DefiniteLength).
Examples: `.p_4() .px_3() .py_1() .mt_auto() .gap_2() .gap_3()`.

**Borders** — `.border(impl Into<AbsoluteLength>) .border_1() .border_2()
.border_t_1() .border_b_1() .border_l_* .border_r_* .border_x_* .border_y_*
.border_color(impl Into<Hsla>) .border_dashed()` (solid is the default; there is
no `border_solid()`).

**Radii** — custom: `.rounded(impl Into<AbsoluteLength>)`, plus per-corner
`.rounded_t(..) .rounded_b(..) .rounded_l(..) .rounded_r(..) .rounded_tl(..)
.rounded_tr(..) .rounded_bl(..) .rounded_br(..)`. Keywords: `.rounded_none()
.rounded_xs() .rounded_sm() .rounded_md() .rounded_lg() .rounded_xl()
.rounded_2xl() .rounded_3xl() .rounded_full()` (none = 0, sm = 4px, md = 6px,
lg = 8px, xl = 12px, full = 9999px). Per-corner keyword forms also exist, e.g.
`.rounded_t_md()`, `.rounded_bl_full()`.

**Shadows** — `.shadow(Vec<BoxShadow>) .shadow_none() .shadow_2xs() .shadow_xs()
.shadow_sm() .shadow_md() .shadow_lg() .shadow_xl() .shadow_2xl()`.

```rust
BoxShadow {
    color: Hsla,
    offset: Point<Pixels>,
    blur_radius: Pixels,
    spread_radius: Pixels,
}
```

**Colour / paint** — `.bg(impl Into<Fill>) .text_color(impl Into<Hsla>)
.text_bg(impl Into<Hsla>) .opacity(f32) .cursor(CursorStyle::*)`. `Fill` is
currently only `Fill::Color(Background)`; `Rgba`, `Hsla`, `Background` all
convert, which is why `.bg(rgb(0x101010))` and `.bg(gpui::red())` both work.

**Text** — `.text_size(impl Into<AbsoluteLength>) .text_xs() .text_sm()
.text_base() .text_lg() .text_xl() .font_family(impl Into<SharedString>)
.font(Font) .font_weight(FontWeight) .italic() .not_italic() .underline()
.line_through() .line_height(impl Into<DefiniteLength>) .text_align(TextAlign)
.text_left() .text_center() .text_right() .truncate() .line_clamp(usize)
.whitespace_nowrap() .whitespace_normal() .text_ellipsis()`

**Positioning / overflow** — `.relative() .absolute() .top() .bottom() .left()
.right() .inset() .overflow_hidden() .visible() .invisible()`. The *scrolling*
variants (`.overflow_scroll() .overflow_x_scroll() .overflow_y_scroll()
.scrollbar_width(..) .track_scroll(..)`) live on `StatefulInteractiveElement`, so
they need `.id(..)` (see §7).

**Conditional** — via `FluentBuilder` in the prelude: `.map(f)`,
`.when(cond, f)`, `.when_else(cond, f, else_f)`, `.when_some(opt, f)`.

### 6.3 `.id()` and `Stateful`

```rust
fn id(self, id: impl Into<ElementId>) -> Stateful<Self>;   // InteractiveElement
```

`ElementId` is an enum with `From` impls for `usize`, `i32`, `&'static str`,
`SharedString`, `Arc<Path>`, `&FocusHandle`, `(&'static str, EntityId)`,
`(&'static str, usize)`, `(&'static str, u64)`, `(SharedString, usize)`.
So `.id("save")`, `.id(ix)`, `.id(("row", ix))` all work.

`Stateful<E>` is a thin wrapper that adds `StatefulInteractiveElement`, which is
where `.on_click`, `.on_hover`, `.tooltip`, `.on_drag` live. **`.on_click` on a
plain `div()` does not compile** (verified: `E0599 no method named on_click
found for struct gpui::Div`) — you must call `.id(...)` first. This is the single
most common GPUI 0.2 mistake.

`.id()` is also required for scroll state, hover/active state that must persist
across frames, and `.debug_selector()` in tests.

---

## 7. Events

```rust
// InteractiveElement — available on a plain div()
fn on_mouse_down(self, button: MouseButton, l: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_mouse_up(self, button: MouseButton, l: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_mouse_up_out(self, button: MouseButton, l: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_mouse_move(self, l: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_scroll_wheel(self, l: impl Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_key_down(self, l: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_key_up(self, l: impl Fn(&KeyUpEvent, &mut Window, &mut App) + 'static) -> Self;
fn capture_key_down(self, l: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_action<A: Action>(self, l: impl Fn(&A, &mut Window, &mut App) + 'static) -> Self;
fn capture_action<A: Action>(self, l: impl Fn(&A, &mut Window, &mut App) + 'static) -> Self;
fn on_drop<T: 'static>(self, l: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self;
fn group(self, name: impl Into<SharedString>) -> Self;
fn group_hover(self, name: impl Into<SharedString>, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;
fn hover(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;
fn focus(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;  // "focused" style, not "focus this"
fn track_focus(self, &FocusHandle) -> Self;
fn key_context<C, E>(self, key_context: C) -> Self where C: TryInto<KeyContext, Error = E>, E: Debug;
```

```rust
// StatefulInteractiveElement — only after `.id(..)`
fn on_click(self, l: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;
fn on_hover(self, l: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self;   // true = enter
fn tooltip(self, build: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self;
fn active(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;
fn group_active(self, name: impl Into<SharedString>, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;
fn focusable(self) -> Self;
fn overflow_scroll(self) -> Self;
fn overflow_x_scroll(self) -> Self;
fn overflow_y_scroll(self) -> Self;
fn scrollbar_width(self, width: impl Into<AbsoluteLength>) -> Self;
fn track_scroll(self, &ScrollHandle) -> Self;
fn on_drag<T, W>(self, value: T, ctor: impl Fn(&T, Point<Pixels>, &mut Window, &mut App) -> Entity<W> + 'static) -> Self;
```

Handlers are `Fn(&Event, &mut Window, &mut App)` — they do **not** get your
entity state. `cx.listener(..)` adapts a method into that shape by re-entering
the entity:

```rust
.on_click(cx.listener(|this: &mut Self, _ev: &ClickEvent, window, cx| {
    this.count += 1;
    cx.notify();
}))
```

`.listener` accepts either `(&mut T, &E, &mut Window, &mut Context<T>)` for
event-with-payload handlers or, because closures can ignore args, you can pass a
method like `Self::add_clicked`. `.processor` is the value-returning variant.

Event payloads:

```rust
pub struct MouseDownEvent { pub position: Point<Pixels>, pub modifiers: Modifiers,
                            pub button: MouseButton, pub click_count: usize, pub first_mouse: bool }
pub struct KeyDownEvent  { pub keystroke: Keystroke, pub is_held: bool }
pub struct Keystroke { pub modifiers: Modifiers, pub key: String, pub key_char: Option<String> }
```

`KeyDownEvent` is a *raw* key event: it fires for every key, including keys that
are also bound to actions. Use it for "type into my own buffer" — but for
anything involving IME, clipboard, or actual text, implement `EntityInputHandler`
(§12). Raw `on_key_down` sees `keystroke.key_char == None` when a modifier
swallows the character (e.g. Cmd-S).

---

## 8. Actions and keybindings (verified end-to-end)

An **action** is a typed message. Declare unit actions with the macro:

```rust
actions!(todo, [Submit, ClearAll, Quit]);   // names become "todo::Submit", ...
```

or a namespace-less form `actions!([Save, Open])`. For payloads, derive:

```rust
#[derive(Clone, PartialEq, gpui::Action)]
#[action(namespace = todo)]
struct SetFilter { only_open: bool }
```

`#[derive(Action)]` also requires `serde::Deserialize` + `schemars::JsonSchema`
unless you add `#[action(no_json)]`. `#[gpui::action]` (the attribute macro shown
in the crate's own `docs/key_dispatch.md`) **does not exist** in 0.2.2.

Bind keys globally in the `run` callback:

```rust
cx.bind_keys([
    KeyBinding::new("enter", Submit, Some("TodoApp")),
    KeyBinding::new("cmd-k", ClearAll, Some("TodoApp")),
    KeyBinding::new("cmd-q", Quit, None),
]);
```

```rust
impl KeyBinding {
    pub fn new<A: Action>(keystrokes: &str, action: A, context: Option<&str>) -> Self;
}

fn key_context<C, E>(self, ctx: C) -> Self  // InteractiveElement, e.g. .key_context("TodoApp")
```

* `keystrokes` is space-separated ("ctrl-shift-p"), parsed by `Keystroke::parse`.
  `KeyBinding::new` **panics** on a bad string; `KeyBinding::load` returns
  `Result`.
* The third argument is a **context predicate** matched against the chain of
  `.key_context(..)` values from the focused element upwards. `None` = global.
  Strings can be predicates: `Some("TodoApp && !TextInput")`, `Some("Editor")`.
* Attach handlers with `.on_action(cx.listener(Self::submit))` on any element in
  the focused subtree. Handlers **bubble** from the focused element to the root;
  `cx.stop_propagation()` in a handler stops it; `.capture_action(..)` runs first.
* Dispatch requires **focus**. Verified in a test: with `track_focus` +
  `window.focus(&handle)` the bound action fires once per
  `simulate_keystrokes("enter")`; without focusing nothing fires. Tab order
  comes from `.tab_index(isize)`, `.tab_stop(bool)`, `.tab_group()`.
* App-level (non-element) handlers: `cx.on_action(|action: &Quit, cx: &mut App| cx.quit())`.

`App::key_bindings() -> Rc<RefCell<Keymap>>` exists if you need to inspect or
mutate bindings dynamically. `Keymap` is an in-memory registry in 0.2.2 (no
serde derives), so there is **no** built-in JSON keymap loader for a standalone
app — build bindings with `KeyBinding::new` (or `Action::build` from JSON).

---

## 9. Focus

```rust
pub struct FocusHandle { /* private */ }
impl FocusHandle {
    pub fn focus(&self, window: &mut Window);
    pub fn is_focused(&self, window: &Window) -> bool;
    pub fn contains_focused(&self, window: &Window, cx: &App) -> bool;
    pub fn dispatch_action(&self, action: &dyn Action, window: &mut Window, cx: &mut App);
    pub fn tab_index(self, index: isize) -> Self;
    pub fn tab_stop(self, tab_stop: bool) -> Self;
    pub fn downgrade(&self) -> WeakFocusHandle;
}

pub trait Focusable: 'static {
    fn focus_handle(&self, cx: &App) -> FocusHandle;
}
impl<V: Focusable> Focusable for Entity<V> {}     // exists
```

Creating and wiring:

```rust
struct MyView { focus_handle: FocusHandle }

fn new(cx: &mut Context<Self>) -> Self { Self { focus_handle: cx.focus_handle() } }

impl Focusable for MyView {
    fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

// render:
div().track_focus(&self.focus_handle(cx)).key_context("MyView")
```

`cx.focus_handle()` (`App::focus_handle`) allocates a handle owned by the app;
store it in your entity. `track_focus` makes the element focusable *and* applies
the `.focus(..)` style when focused. `.focusable()` (no handle) is the
hook-only variant when you don't need a handle, and `.tab_index(i)` makes it
tab-reachable. `window.focus(&handle)` moves focus programmatically — do it in
the window builder (as `examples/on_window_close_quit.rs` does with
`focus_handle.focus(window)`) or in a handler.

Note the naming collision: `.focus(f)` on an element sets the *focused style*,
while `FocusHandle::focus(&self, window)` actually moves focus.

---

## 10. State: `Entity`, `Context`, async

```rust
pub struct Entity<T>(/* private */);   // Copy for AnyEntity; Entity<T> is Clone + PartialEq by id
impl<T: 'static> Entity<T> {
    pub fn entity_id(&self) -> EntityId;
    pub fn read<'a>(&self, cx: &'a App) -> &'a T;
    pub fn read_with<R, C: AppContext>(&self, cx: &C, f: impl FnOnce(&T, &App) -> R) -> C::Result<R>;
    pub fn update<R, C: AppContext>(&self, cx: &mut C,
        f: impl FnOnce(&mut T, &mut Context<T>) -> R) -> C::Result<R>;
    pub fn write<C: AppContext>(&self, cx: &mut C, value: T) -> C::Result<()>;
    pub fn update_in<R, C: VisualContext>(&self, cx: &mut C,
        f: impl FnOnce(&mut T, &mut Window, &mut Context<T>) -> R) -> anyhow::Result<R>;
    pub fn downgrade(&self) -> WeakEntity<T>;
    pub fn into_any(self) -> AnyEntity;
}
```

`EntityId` is an opaque copyable id (`EntityId::as_u64()`; `.id(("row", entity_id))` uses
`From<(&'static str, EntityId)> for ElementId`). For `C = App`,
`C::Result<R> = R`; for async/test contexts it is `anyhow::Result<R>`.

```rust
pub struct Context<'a, T> { /* &mut App + WeakEntity<T> */ }
impl<'a, T: 'static> Context<'a, T> {
    pub fn entity(&self) -> Entity<T>;
    pub fn weak_entity(&self) -> WeakEntity<T>;
    pub fn entity_id(&self) -> EntityId;
    pub fn notify(&mut self);                      // mark dirty, schedule redraw, notify observers
    pub fn observe<W>(&mut self, &Entity<W>, impl FnMut(&mut T, Entity<W>, &mut Context<T>) + 'static) -> Subscription;
    pub fn subscribe<T2, Evt>(&mut self, &Entity<T2>, impl FnMut(&mut T, Entity<T2>, &Evt, &mut Context<T>) + 'static) -> Subscription;
    pub fn emit<Evt>(&mut self, event: Evt);       // requires impl EventEmitter<Evt> for T
    pub fn listener<E: ?Sized>(&self, impl Fn(&mut T, &E, &mut Window, &mut Context<T>) + 'static)
        -> impl Fn(&E, &mut Window, &mut App) + 'static;
    pub fn processor<E, R>(&self, impl Fn(&mut T, E, &mut Window, &mut Context<T>) -> R + 'static)
        -> impl Fn(E, &mut Window, &mut App) -> R + 'static;
    pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
        where AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R + 'static;
    pub fn focus_self(&mut self, window: &mut Window) where T: Focusable;
    pub fn on_focus(&self, impl Fn(&mut T, &FocusEvent, &mut Window, &mut Context<T>) + 'static) -> Subscription;
    pub fn on_blur(...) -> Subscription;
    pub fn emit(...);
}
```

`Context<T>` derefs to `App` (both `Deref` and `DerefMut`), so every `App` method
is callable on `cx`.

**Redraw model.** There is no reactive diffing. Mutating entity state does
nothing until you call `cx.notify()`; that marks the entity dirty, schedules a
frame, and re-runs `Render::render`. A window redraws at most once per frame.
If you forget `cx.notify()`, the UI is stale — this is the #2 most common GPUI
mistake.

**Async.** `cx.spawn` gives you a `WeakEntity<T>` and `&mut AsyncApp`, and
returns `Task<R>` which must be held or `.detach()`ed:

```rust
cx.spawn(async move |this, cx| {
    cx.background_executor().timer(Duration::from_millis(250)).await;
    this.update(cx, |this, cx| { this.ticks += 1; cx.notify(); }).ok();
})
.detach();
```

`this.update` returns `Result` (entity may have been released); `.ok()` or `?`.
`AsyncApp` has `update(|&mut App|) -> Result`, `read_global`, `open_window`,
`spawn`, `background_executor()`, `foreground_executor()`. CPU-heavy work goes
to `cx.background_spawn(async { .. })` (requires `Send + 'static`).

**Globals.** `impl Global for MyType {}` + `cx.set_global(x)`, `cx.global::<G>()`,
`cx.update_global(|g, cx| ..)`, `cx.default_global::<G>()`,
`cx.observe_global::<G>(|cx| ..)`. Use for app-wide services (theme, config,
window registry) — this is what Cap does.

---

## 11. Lists and scrolling

**A stateful `div` can scroll** — `.id(..)` first (scroll state must be keyed to
an element id), then overflow and a bounded size; content is clipped:

```rust
div()
    .id("scroller")
    .h(px(200.))
    .overflow_y_scroll()
    .track_scroll(&self.scroll_handle)   // optional; lets you read/control offset
    .child(content)
```

`ScrollHandle` (`src/elements/div.rs:3068`): `new()`, `offset() -> Point<Pixels>`,
`max_offset() -> Size<Pixels>`, `bounds()`, `top_item()`, `bottom_item()`,
`scroll_to_item(..)`, `set_offset(..)`. Clone it (it is `Rc<RefCell<..>>`) and
store it in the entity.

**Virtualised uniform-height list** (`examples/uniform_list.rs`):

```rust
pub fn uniform_list<R>(
    id: impl Into<ElementId>,
    item_count: usize,
    f: impl 'static + Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>,
) -> UniformList where R: IntoElement;
```

Use `cx.processor(...)` to get entity access. Only the visible range is built,
but every row must be the same height. `UniformListScrollHandle` adds
`scroll_to_item(ix, ScrollStrategy)`.

**Variable-height virtualised list**: `list(id, item_count, f)` with
`ListAlignment`, `ListState`, `ListSizingBehavior`; heavier, needs accurate
measurement. For a task list of tens/hundreds of rows, a plain
`div().overflow_y_scroll().children(..)` is fine and much simpler.

---

## 12. Text input: there is no `Input` widget

A usable single-line field in 0.2.2 = **three pieces**:

1. **`EntityInputHandler`** (`src/input.rs:7`) — the platform IME protocol.
   All 8 methods are required (no defaults):
   `text_for_range`, `selected_text_range`, `marked_text_range`, `unmark_text`,
   `replace_text_in_range`, `replace_and_mark_text_in_range`,
   `bounds_for_range`, `character_index_for_point`.
   Ranges are **UTF-16** offsets (NSTextInputClient semantics), while Rust
   strings are UTF-8 — you must convert both ways (helpers in §16).
2. **A custom `Element`** that lays out and paints the text: `request_layout`,
   `prepaint` (shape the line(s) with
   `window.text_system().shape_line(text, font_size, &runs, None)`, compute caret
   and selection quads), and `paint`, which must call
   ```rust
   window.handle_input(&focus_handle, ElementInputHandler::new(bounds, entity.clone()), cx);
   ```
   every frame while the field exists (`src/window.rs:3400`). Without that call
   the field receives no keyboard/clipboard/IME input at all.
3. **Focus plumbing**: `.track_focus(&focus_handle)` on the field container,
   `.cursor(CursorStyle::IBeam)`, and focus it when clicked
   (`focus_handle.focus(window)` or `window.focus(&handle)`).

Plus your own **actions** for editing: `backspace`, `delete`, `left`, `right`,
`select_all`, `paste`, `cut`, `copy`, `home`, `end` — GPUI defines none of
these, and you bind them yourself (see §8). `backspace` etc. must be bound with
the field's key context (e.g. `Some("TextInput")`) so they don't fire while
another widget is focused.

Clipboard is on `App`: `cx.read_from_clipboard() -> Option<ClipboardItem>`
(`ClipboardItem::text() -> Option<String>`), `cx.write_to_clipboard(ClipboardItem::new_string(s))`.

Simplifications that work but lose behaviour: **ignore IME** and just read
`event.keystroke.key_char` in `on_key_down` into a `String`. That compiles, is
~30 lines, and breaks CJK/emoji composition, dead keys, paste, and the OS caret
rect for the IME candidate window. §16 shows the full version (about 400 lines,
trimmed from the crate's `examples/input.rs`, with mouse-selection removed).

Text rendering for read-only text is much simpler: `.child("string")` or
`StyledText::new(..)` with `.text_color/.font_weight/..`; no `Element` needed.

---

## 13. Menus

```rust
pub struct Menu { pub name: SharedString, pub items: Vec<MenuItem> }

impl MenuItem {
    pub fn action(name: impl Into<SharedString>, action: impl Action) -> Self;
    pub fn separator() -> Self;
    pub fn submenu(menu: Menu) -> Self;
    pub fn os_submenu(name: impl Into<SharedString>, ty: SystemMenuType) -> Self; // e.g. Services
}
```

```rust
cx.set_menus(vec![Menu {
    name: "Todo".into(),
    items: vec![
        MenuItem::action("Add item", Submit),
        MenuItem::separator(),
        MenuItem::action("Clear all", ClearAll),
        MenuItem::separator(),
        MenuItem::action("Quit", Quit),
    ],
}]);
cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
```

Menu items dispatch the *action value*, so you need an `on_action` handler in the
focused tree (or app-level `cx.on_action`). On macOS this populates the real menu
bar, so include the standard app menu yourself (`os_submenu("Services",
SystemMenuType::Services)`, Quit, etc.) — GPUI does not build a default macOS
menu for you. `cx.get_menus()` / `cx.set_dock_menu(..)` also exist.

Widget-level context menus are **not** built in: build a popover with
`Anchored`/`Deferred` + `.absolute()` yourself, or use a `WindowKind::PopUp`
window.

---

## 14. Assets, images, SVG, fonts

```rust
pub trait AssetSource: 'static + Send + Sync {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>>;
    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>>;
}
impl AssetSource for () { /* returns None / empty */ }
```

Wire it once:

```rust
Application::new().with_assets(MyAssets).run(|cx| { .. });
```

* `svg().path("icons/plus.svg")` resolves `path` through the asset source each
  frame (cached). `img("...")` likewise for embedded resources; `img("https://…")`
  goes through `App`'s HTTP client (`Application::with_http_client`).
* For a filesystem-backed source, `std::fs::read(path).map(Into::into)` is the
  pattern the crate's `examples/animation.rs` uses. `rust-embed`/`include_bytes!`
  work too — return `Cow::Borrowed`.
* A missing asset is a soft failure: the loader logs and the image/svg simply
  does not appear. There is no compile-time check that a path exists — typo'd
  icon paths fail silently at runtime, so `list()`-based tests or a startup
  assertion are worth adding.

**Fonts.** System fonts are enumerated by `font-kit`; use the family name
directly:

```rust
div().font_family("SF Pro Text")
```

`TextStyle` defaults come from the platform; read them with
`window.text_style()` (`.font_family`, `.font_size`, `.color`, `.font()`).
To register bundled fonts:

```rust
cx.text_system()
    .add_fonts(vec![
        std::borrow::Cow::Borrowed(include_bytes!("../assets/Inter-Regular.ttf")),
    ])
    .expect("failed to register bundled fonts");
```

`TextSystem::add_fonts(&self, Vec<Cow<'static, [u8]>>) -> gpui::Result<()>`
(`src/text_system.rs:104`). It is process-global (not per window), so call it
once in `run`. Family name resolution then works via `.font_family("Inter")`.
`cx.text_system().all_font_names() -> Vec<String>` lists what is available
(useful to confirm a font loaded). Verified compiling; the *runtime* effect on
glyph selection was not exercised.

---

## 15. Testing

```toml
[dev-dependencies]
gpui = { version = "0.2.2", features = ["test-support"] }
```

```rust
use gpui::{div, point, prelude::*, px, rgb, ClickEvent, Context, Modifiers,
           Render, TestAppContext, Window};

struct Counter { count: usize }

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("counter")
            .size(px(200.))
            .bg(rgb(0xff0000))
            .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                this.count += 1;
                cx.notify();
            }))
            .child(format!("{}", self.count))
    }
}

#[gpui::test]
async fn counter_increments_on_click(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Counter { count: 0 });
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    cx.run_until_parked();
    let count = cx.update(|_, cx| view.read(cx).count);
    assert_eq!(count, 1);
}
```

Verified: this test passes (`cargo test`), as does a keybinding test that binds
`enter` to an action, focuses the view, and asserts the handler ran once.

* `#[gpui::test]` runs the body on a deterministic executor. Arguments may be
  `&TestAppContext` (or `&mut TestAppContext`), `&mut App`, `StdRng`,
  `BackgroundExecutor`. Multiple `TestAppContext` args = multiple simulated apps
  (for collaboration tests). Options (verified syntax):
  `#[gpui::test(iterations = 10, seed = 7, seeds(1, 2, 3), retries = 2,
  on_failure = "path::to::fn")]`. Note `seeds(..)` is call-style, not
  `seeds = [..]`. Env vars `SEED` and `ITERATIONS` also apply.
* `TestAppContext`: `build`, `add_window_view(builder) -> (Entity<V>, &mut VisualTestContext)`,
  `add_window`, `add_empty_window`, `update(|&mut App|)`, `read(|&App|)`,
  `run_until_parked()`, `spawn`, `set_global`, `dispatch_action`,
  `simulate_keystrokes(&str)`, `simulate_input(&str)`, `read_from_clipboard`,
  `opened_url`, `has_pending_prompt`, `notifications`, `events`, `condition`.
* `VisualTestContext` (derefs to `TestAppContext`) is "Window + App": `update(|&mut Window, &mut App|)`,
  `simulate_click(point, Modifiers)`, `simulate_mouse_down/up/move`,
  `simulate_modifiers_change`, `simulate_keystrokes`, `simulate_input`,
  `simulate_resize`, `dispatch_action(action)`, `debug_bounds(selector)`,
  `draw`, `simulate_close`, `window_title`.
* `debug_bounds("sel")` requires `.debug_selector(|| "sel".to_string())` on the element
  (only in `test-support`/test builds) — this is how you assert layout.
* `gpui::test::observe(&entity, cx)` turns `notify` into a `Stream`.

There is **no** snapshot/screenshot testing helper in the crate.

---

## 16. Example 2 — entity state, text input, button, shortcut

This is one file, `examples/doc_app.rs` in the probe crate, compiled with
`cargo build --example doc_app`. It is the crate's `examples/input.rs` trimmed
(mouse selection dropped) plus a stateful root view, a mutating button, a bound
shortcut, and a native menu.

Reading order:

* `TextInput` — entity: `content`, selection, focus handle, cached layout.
* `impl EntityInputHandler for TextInput` — IME protocol, UTF-16 ↔ UTF-8.
* `impl Render for TextInput` + `TextElement` — paint and register input.
* `TodoApp` — root view; owns `Entity<TextInput>` and `Vec<SharedString>`;
  `submit` is a method used by *both* the button (`on_click`) and the action
  handler (`on_action`) — one code path, two inputs.
* `fn main` — keybindings, menu, window.

```rust
use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, rgb, rgba, size, App, Application, Bounds,
    ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, InspectorElementId, KeyBinding,
    LayoutId, Menu, MenuItem, PaintQuad, Pixels, Point, ShapedLine, SharedString,
    Style, TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions,
};
use unicode_segmentation::UnicodeSegmentation;

// Namespace the action names ("todo::Submit" etc).
actions!(todo, [Submit, ClearAll, Quit]);
// gpui ships no text-input actions, so define your own.
actions!(
    text_input,
    [Backspace, Delete, Left, Right, SelectAll, Paste, Cut, Copy]
);

// ===========================================================================
// 1. A hand-written single-line text field.
//    GPUI has no Input widget: you implement `EntityInputHandler` (IME/clipboard
//    protocol) and paint the text yourself in an `Element`.
// ===========================================================================

pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "What needs doing?".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
        }
    }

    pub fn text(&self) -> String {
        self.content.to_string()
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        cx.notify();
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.selected_range.end), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let offset = self.previous_boundary(self.cursor_offset());
        self.move_to(offset, cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let offset = self.next_boundary(self.cursor_offset());
        self.move_to(offset, cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .key_context("TextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .h(px(34.))
            .w_full()
            .px(px(8.))
            .py(px(4.))
            .rounded_md()
            .border_1()
            .border_color(rgb(0xc0c0c0))
            .bg(rgb(0xffffff))
            .child(TextElement {
                input: cx.entity(),
            })
    }
}

/// The `Element` that draws the text and registers the input handler.
struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), gpui::hsla(0., 0., 0., 0.5))
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    rgb(0x000000),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x3311ff30),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        // Register with the platform IME for this frame.
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line = prepaint.line.take().unwrap();
        line.paint(bounds.origin, window.line_height(), window, cx)
            .unwrap();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

// ===========================================================================
// 2. The root view: Entity-owned state, a button, keyboard shortcuts.
// ===========================================================================

struct TodoApp {
    input: Entity<TextInput>,
    items: Vec<SharedString>,
    focus_handle: FocusHandle,
}

impl Focusable for TodoApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TodoApp {
    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().trim().to_string();
        if !text.is_empty() {
            self.items.push(text.into());
            self.input.update(cx, |input, cx| input.reset(cx));
        }
        window.focus(&self.input.focus_handle(cx));
        cx.notify();
    }

    fn clear_all(&mut self, _: &ClearAll, _: &mut Window, cx: &mut Context<Self>) {
        self.items.clear();
        cx.notify();
    }

    fn add_clicked(&mut self, _: &gpui::ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.submit(&Submit, window, cx);
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .p(px(24.))
            .bg(rgb(0xf5f5f5))
            .text_color(rgb(0x1f1f1f))
            .font_family("Helvetica")
            .track_focus(&self.focus_handle(cx))
            .key_context("TodoApp")
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::clear_all))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(div().flex_1().child(self.input.clone()))
                    .child(
                        div()
                            .id("add-button")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x2563eb))
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x1d4ed8)))
                            .active(|style| style.bg(rgb(0x1e40af)))
                            .on_click(cx.listener(Self::add_clicked))
                            .child("Add"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.items.iter().enumerate().map(|(ix, item)| {
                        div()
                            .id(("item", ix))
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(rgb(0xffffff))
                            .child(item.clone())
                    })),
            )
            .child(
                div()
                    .mt_auto()
                    .text_sm()
                    .text_color(rgb(0x666666))
                    .child(format!("{} item(s) · Enter adds · Cmd-K clears", self.items.len())),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Keybinding context strings must match `.key_context(...)` in the tree.
        cx.bind_keys([
            KeyBinding::new("enter", Submit, Some("TodoApp")),
            KeyBinding::new("cmd-k", ClearAll, Some("TodoApp")),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("backspace", Backspace, Some("TextInput")),
            KeyBinding::new("delete", Delete, Some("TextInput")),
            KeyBinding::new("left", Left, Some("TextInput")),
            KeyBinding::new("right", Right, Some("TextInput")),
            KeyBinding::new("cmd-a", SelectAll, Some("TextInput")),
            KeyBinding::new("cmd-v", Paste, Some("TextInput")),
            KeyBinding::new("cmd-c", Copy, Some("TextInput")),
            KeyBinding::new("cmd-x", Cut, Some("TextInput")),
        ]);

        // macOS menu bar. Actions are matched by type.
        cx.set_menus(vec![Menu {
            name: "Todo".into(),
            items: vec![
                MenuItem::action("Add item", Submit),
                MenuItem::separator(),
                MenuItem::action("Clear all", ClearAll),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        }]);
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        let bounds = Bounds::centered(None, size(px(520.), px(520.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("GPUI Todo".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let input = cx.new(TextInput::new);
                window.focus(&input.read(cx).focus_handle.clone());
                cx.new(|cx| TodoApp {
                    input,
                    items: Vec::new(),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
```

Notes on this code:

* Action names must be unique per namespace: `Submit` in namespace `todo` and
  `Backspace` in `text_input` coexist fine, but declaring the same name twice
  (or re-declaring a gpui action) panics at startup during registration.
* `cx.listener(Self::backspace)` works because the method takes
  `(&mut Self, &Backspace, &mut Window, &mut Context<Self>)` — exactly the
  `on_action` shape after the listener adapter.
* `key_context("TextInput")` on the field container and
  `Some("TextInput")` on the bindings is what keeps `backspace` from firing in
  the rest of the app. Remove the context and every backspace anywhere becomes
  an edit.
* `self.input.update(cx, |input, cx| input.reset(cx))` — child-entity mutation
  from the parent; `cx` here is `&mut Context<TodoApp>`, which is an `AppContext`.
* `window.focus(&self.input.focus_handle(cx))` after submitting returns focus to
  the field; `Entity<V>: Focusable` makes `focus_handle(cx)` available.
* `.mt_auto()` on the footer is the flexbox `margin-top: auto` trick to pin it
  to the bottom.

---

## 17. Pitfalls checklist

1. `.on_click`, `.on_hover`, `.tooltip`, `.active`, `.focusable`,
   `.overflow_*_scroll`, `.track_scroll` and `.on_drag` are on
   `StatefulInteractiveElement` and require `.id(..)` first — without an id you
   get `E0599 no method named ... found for struct gpui::Div` (verified).
2. Mutating state without `cx.notify()` = no repaint.
3. `Render::render` returns one concrete type; use `.into_any_element()` for
   branches.
4. `Pixels` cannot be constructed literally; use `px(..)`.
5. No `text()` element constructor and no widgets (§1.5).
6. Key dispatch needs focus (`track_focus` + `window.focus`), and bindings need
   a matching `key_context` or `None`.
7. `KeyBinding::new` panics on malformed key strings.
8. `.focus(style)` ≠ moving focus. Use `window.focus(&handle)`.
9. `open_window` returns `Result`; `unwrap()` is normal for startup but should be
   handled if you open windows from user actions.
10. Custom `Element`s: text fields must call `window.handle_input(..)` in
    `paint` **every frame**, and `prepaint` must cache what `paint` needs
    (`last_layout`, `last_bounds` in §16) because `paint` cannot re-shape text.
11. `Subscription`s from `observe`/`subscribe` do nothing until stored or
    `.detach()`ed.
12. Default macOS build needs full Xcode (§1.3).
13. The crate's own `docs/key_dispatch.md` is stale relative to 0.2.2; trust the
    source and this document.

---

## 18. What I could not verify

* **Runtime behaviour of the input field with a real IME** (CJK composition, dead
  keys), clipboard round-trip, and menu-bar clicks. The code compiles; the
  interactive paths were not driven by hand or by `simulate_input`.
* **`runtime_shaders` under load**: the binary launched and idled without error,
  but GPU rendering correctness on this machine was not visually inspected.
* **`macos-blade`** backend: not built.
* **Linux (X11/Wayland) and Windows**: not built or run.
* **`cx.text_system().add_fonts(..)`** effect on actual glyph selection (call
  site compiles; no font file was really loaded in the probe).
* **`uniform_list` scroll performance/behaviour** beyond compilation.
* **`list()` (variable-height)** was not exercised at all.
* **`#[derive(Action)]`** was not compiled (needs serde/schemars in the probe);
  signature/requirements were read from `src/action.rs` docs.
* **`Animation`/`AnimationExt`/`bounce`/`ease_in_out`** were not compiled.
* **Multiple windows / native macOS tabs / `WindowKind::PopUp`** were not run.
* **Custom transparent titlebar + `WindowControlArea::Drag`** compiles; the
  window-drag behaviour was not exercised.
* **`Element` paint-time APIs** other than `paint_quad`, `handle_input`,
  `request_layout`, `text_system`, `text_style`, `line_height`, `rem_size` were
  not exercised.
* The crate sets no `rust-version`; MSRV is unknown (1.91.0 worked, and the
  source uses `let ... && let ...` let-chains, so **Rust ≥ 1.88** is the practical
  floor).
