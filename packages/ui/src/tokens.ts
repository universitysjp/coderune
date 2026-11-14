/**
 * Native ui roles, adapted from https://design.how/r/ui/theme.json.
 * The native Rust/GPUI desktop consumes resolved sRGB values, not CSS custom properties.
 * The desktop deliberately removes chroma from ui's ink/paper/canvas/grey
 * channels: 20%/100%/99.1%/95.6% OKLCH become #161616/#fff/#fcfcfc/#f0f0f0.
 * Frame/control/row values composite that ink at the registry's 14%/16%/8%.
 * Blue is Coderune's existing --palette-blue-700; the selection wash is ui's
 * --hue-3-wash. Trouble uses --fail-text and --fail-wash, never a success hue.
 */
const fonts = { sans: "Geist", mono: "Paper Mono" } as const;

const typeRole = (
  fontSize: number,
  lineHeight: number,
  fontFamily: string = fonts.sans,
  fontWeight = 400
) => ({ fontSize, lineHeight, fontFamily, fontWeight });

export const ui = {
  colors: {
    bg: "#fcfcfc",
    panel: "#ffffff",
    ink: "#161616",
    muted: "#454545",
    faint: "#656565",
    line: "#dedede",
    rowLine: "#ececec",
    controlLine: "#dadada",
    controlHover: "#b4b4b4",
    accent: "#4c94fc",
    accentSoft: "#e3eefa",
    danger: "#b72728",
    dangerSoft: "#ffe6e3",
    success: "#161616",
    successSoft: "#f0f0f0",
    code: "#f0f0f0",
    transparent: "transparent",
    inverse: "#ffffff",
    hover: "#f3f3f3",
    pressed: "#ececec",
    disabled: "#a2a2a2",
    primaryHover: "#373737",
  },
  fonts,
  space: {
    0: 0,
    0.5: 2,
    1: 4,
    2: 8,
    3: 12,
    4: 16,
    5: 20,
    6: 24,
    8: 32,
    10: 40,
    15: 60,
    20: 80,
    30: 120,
  },
  radius: { none: 0, inner: 4, soft: 6, frame: 10 },
  line: { hair: 1, focus: 2, focusOffset: 2 },
  opacity: { disabled: 0.4 },
  type: {
    display: typeRole(40, 44),
    headingLg: typeRole(32, 36),
    heading: typeRole(24, 28),
    headingSm: typeRole(20, 24),
    headingXs: typeRole(16, 20),
    bodyLg: typeRole(16, 22),
    body: typeRole(14, 20),
    bodySm: typeRole(13, 18),
    label: typeRole(14, 16),
    labelSm: typeRole(12, 16),
    button: typeRole(16, 20),
    code: typeRole(13, 20, fonts.mono),
    codeSm: typeRole(12, 16, fonts.mono),
  },
  layout: {
    sidebar: 248,
    sidebarIcon: 48,
    topbar: 56,
    gutter: 24,
    gutterSm: 16,
    container: 1120,
    controlXs: 24,
    controlSm: 28,
    control: 32,
    controlLg: 44,
    row: 40,
    rowLg: 56,
    tag: 20,
    pageHeaderMax: 720,
    overlay: 480,
    overlayLg: 640,
    fieldMaxHeight: 176,
  },
} as const;

export type ui = typeof ui;
