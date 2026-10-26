# Website typography and layout

## Request

Keep the desktop index on the left and move it farther left.
Use Geist as the main font and Paper Mono as the mono font in the shared tokens package.
Preserve every existing color, page, link, and research claim. Use
dashed dividers and section titles beside the content. Index and side titles use
Paper Mono; page titles and other headings use Geist. The GitHub link in the index
is text only, and the logo uses Paper Mono's `₪` glyph. The `CODERUNE` wordmark,
index links, and section side titles are uppercase; other headings retain their case.
The wordmark uses `0.01em` tracking; the small uppercase labels use `0.04em`.

## Implementation

- Use locally bundled variable WOFF2 fonts: Geist from the npm registry for
  prose and ordinary headings, full Paper Mono v1.000 for the index, side titles,
  and code. Use its supported `₪` glyph (U+20AA) for the site logo.
- Keep the existing font-family token interface so app consumers inherit the
  new fonts without separate setup. Retain the Paper Mono asset/export and remove
  the unused Geist Mono dependency.
- Use a compact reading column, restrained headings, and consistent vertical rhythm.
  Tune tracking and unitless leading for Geist rather than copying font metrics.
- Keep one shared type scale across routes. Use comfortable body leading, slightly
  tighter headings, and neutral tracking for small monospace text.
- Keep the desktop index fixed with a wider gutter to the reading column.
  Collapse navigation into the existing wrapping top navigation when both columns
  no longer fit, using the same breakpoint for navigation and layout.
- Put each section title in a narrow side column, aligned with its first content
  line. Align the page introduction with the content column. Stack side titles
  above their content on narrow screens. Use a shared section component to retain
  anchors and associate each section with its heading.
- Separate successive sections with dashed rules, using the
  existing border color token. Expand the overall reading frame to accommodate
  labels without squeezing the prose column.
- Tighten hero and section spacing, and use simple stacked page links instead of
  a large card grid. Preserve the existing palette and all color declarations.

## Acceptance criteria

- [x] Geist and Paper Mono load locally; the unused Geist Mono package is removed.
- [x] The reading column, headings, and vertical rhythm form a consistent layout.
- [x] The left index is moved farther left with space reserved beside the content.
- [x] All five routes and the not-found page use the shared typography.
- [x] All 18 content sections have responsive side titles and dashed dividers.
- [x] Only index and side titles use Mono; ordinary headings remain in Geist.
- [x] The index GitHub link is text only, and the logo uses a real Paper Mono glyph.
- [x] The wordmark, index, and section side titles are uppercase with appropriate tracking.
- [x] Colors and existing content remain unchanged, apart from font documentation.
- [x] Navigation, examples, tables, and team entries have responsive layouts.

## Validation

Inspect the rendered overview and secondary routes at desktop and mobile widths,
including long claims and labels. Check production build, TypeScript, and lint.
Compare all color-related declarations with the pre-task snapshot. This is a
presentation-only change; no behavioral test seam or new unit tests are needed.

Completed: production build and TypeScript check, formatting and lint, all 24
existing tests, unchanged palette comparison, and both Standards and Spec reviews
with no actionable findings. The build bundles Geist and Paper Mono locally.

Browser checks covered the initial desktop overview, 390px overview and team
layouts, production research page, and desktop side-title layout. The final
uppercase labels and glyph logo could not be rechecked in the browser because
computer-use calls timed out; these final details were checked in source and the
production build. The favicon outline was extracted from Paper Mono's U+20AA
glyph, avoiding reliance on a system fallback font.

## Scope

No backend changes, new features, palette changes, deployment, or unrelated
workspace edits. This document also serves as the implementation ticket scope.
