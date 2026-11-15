> Status: historical. Cap used GPUI; Coderune chose Rune after this review. The engineering lessons below still guided the port.

# Lessons from Cap's GPUI migration

Source: [A technical deep dive into Cap's migration to GPUI](https://cap.so/blog/rebuilding-cap-in-gpui), 21 October 2026.

These are the points that change how Coderune's native rewrite is built. The
blog describes Cap 0.6.1 moving from Tauri web views to a native Rust/GPUI app.

## What we are copying

1. **Rewrite the interface, keep the engine.** Cap kept fourteen Rust engine
   crates and replaced only the UI. Coderune does the same: domain, storage,
   collectors, agents, and application operations are Rust crates; GPUI is the
   only UI layer. The old TypeScript desktop surface is removed rather than
   kept beside it.

2. **One process, shared types.** The old Tauri boundary serialized every
   command to JSON. The native app calls service methods directly. Our UI holds
   `Workspace` and calls typed operations; no IPC and no JSON bridge.

3. **No browser engine on any platform.** A GPUI app removes the three-webview
   class of bugs: Linux blank windows, Windows text-size zoom overflow, DPI
   scale changes, WebKit pointer capture cancellation, and lost clicks between
   web views.

4. **The visual language ports because GPUI reads like Tailwind.** Cap found
   width, height, padding, flex, radius, and fill map almost one-to-one. Our
   theme and controls are a direct transcription of `packages/ui/src/tokens.ts`
   and the native component specs, not a redesign.

5. **GPU drawing has a small shape vocabulary.** Shadows, quads, glyphs,
   icons, and images. GPUI handles batching and platform backends (Metal,
   Direct3D, Vulkan). We stay inside that vocabulary: flat fills, 1px lines, ten
   radius values, no CSS effects.

6. **Compatibility of local data matters.** Cap reads the same projects and
   settings after the rewrite. Coderune keeps the legacy `Context Bridge`
   database directory, filename, schema version, and JSON shapes so existing
   local data opens unchanged.

## What we must budget for

7. **Text fields are the largest hidden cost.** Cap's field is about 2,700
   lines because a browser normally provides typing, selection, copy/paste,
   undo, IME, and accents. Our field implements the GPUI input protocol, IME
   composition, grapheme-aware deletion, UTF-16 ranges, clipboard, and
   undo/redo. Still to add: word-wise movement, double-click selection, and
   horizontal scrolling for long values. Do not treat the field as "done"
   because typing works.

8. **Keyboard navigation is per-window work.** Tab is not automatic in GPUI.
   We bind Tab and Shift-Tab to `focus_next`/`focus_prev` and give buttons and
   choices a tab stop, but every new surface must be checked with the keyboard.

9. **No hot reload.** The old Vite loop is gone. The native loop is
   `cargo run -p coderune-desktop`, a few seconds to rebuild, then relaunch.
   Keep the UI state reproducible so returning to a screen is cheap.

10. **Idling is a framework concern.** GPUI checks for redraws on the display
    refresh. Cap patched it to stop after 100 ms of no changes. Coderune has no
    looping animation, so a static window should settle; revisit an upstream
    patch only if profiling shows idle wake-ups.

11. **The ecosystem is smaller.** Avoid dependencies that only exist for the
    browser. We use a small set: GPUI, rusqlite, serde, sha2, walkdir,
    globset, and unicode-segmentation.

12. **Platform-specific window tricks are deferred.** Cap needed native panels,
    click-through regions, materials, and a macOS 26 visibility patch. Coderune's
    current windows are ordinary app windows, so none of that is in scope yet.

## What we are not copying

- Cap's video frame path, IOSurface handoff, editor timeline, and export engine.
  Coderune has no media pipeline.
- Cap's dark/material theme, glass surfaces, and Radix palette. Coderune keeps
  its fixed light neutral/ink/blue-wash design system.
- Cap's custom Zed fork. We build against crates.io `gpui 0.2.2` with
  `runtime_shaders` because the machine only has Command Line Tools, not full
  Xcode's Metal compiler.
