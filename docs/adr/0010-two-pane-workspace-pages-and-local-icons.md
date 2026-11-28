# Workspace pages share one two-pane frame, and icons are local

Home, Profile, and Settings are two-pane pages: a 240 pt rail of destinations
beside the open page, capped at a 728 pt line length, both scrolling together at
the same 24 pt inset. `coderune_ui::components` owns the frame (`two_pane`,
`pane_rail`, `pane_nav`, `pane_page`, `pane_heading`, `pane_body`) so the three
pages cannot drift apart, and the rail row reuses the 28 pt navigation metrics
with the shared 120 ms hover wash.

Home's rail lists projects and links to Settings. The developer profile is the
only sidebar footer destination; Settings is reached from Home and the top-bar
layout. The chat hover card uses the same 16 pt card inset as Settings rows.

Every glyph is embedded from the Phosphor Regular set in
`crates/ui/src/icons.rs`. The app previously re-exported a few glyphs from the
reference desktop's theme crate; that crate stays a dependency of Rune's theme
only, so the app's icon language is one library rather than a mix of borrowed
and hand-drawn paths. The interface and code faces stay the two installed
defaults.

[Local settings reference](../rune/README.md)
