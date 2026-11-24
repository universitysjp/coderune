//! Workspace chrome and in-window page navigation.

use coderune_ui::components::{self as ui, WhenExt};
use coderune_ui::icons;
use coderune_ui::theme::{self, LABEL, LABEL_SM};
use rune::{col, el, row, text, Color, Cursor, Cx, Drag, El, Phase, ScrollSpec};
use rune_kit::shell::{
    PaneState, Sidebar, TabItem, TabOrientation, TabOverflow, TabStrip, WindowTitlebar,
};
use rune_kit::Kit;

use crate::app::{CoderuneApp, SettingsSection, View};
use crate::screens;

const SIDEBAR_BREAKPOINT: f32 = 1080.0;
const SIDEBAR_STEP: f32 = 8.0;

pub(crate) struct SidebarPane {
    wide: PaneState,
    narrow: Option<PaneState>,
    grab: Option<PaneState>,
}

impl Default for SidebarPane {
    fn default() -> Self {
        Self {
            wide: PaneState::new(theme::SIDEBAR).range(176.0, 296.0),
            narrow: None,
            grab: None,
        }
    }
}

impl SidebarPane {
    fn current(&self) -> &PaneState {
        self.narrow.as_ref().unwrap_or(&self.wide)
    }

    fn current_mut(&mut self) -> &mut PaneState {
        self.narrow.as_mut().unwrap_or(&mut self.wide)
    }

    fn set_narrow(&mut self, narrow: bool) -> bool {
        if narrow == self.narrow.is_some() {
            return false;
        }
        self.narrow = narrow.then(|| self.wide.collapsed());
        self.grab = None;
        true
    }

    pub(crate) fn toggle(&mut self) {
        self.current_mut().toggle();
    }

    fn step(&mut self, steps: i32) {
        let pane = self.current_mut();
        if steps > 0 && !pane.is_open() {
            pane.set_open(true);
        } else if steps < 0 && pane.width() == pane.min_width() {
            pane.set_open(false);
        } else if pane.is_open() {
            pane.set_width(pane.width() + steps as f32 * SIDEBAR_STEP);
        }
    }

    fn drag(&mut self, drag: &Drag) {
        match drag.phase {
            Phase::Down => {
                self.grab = Some(*self.current());
                if drag.clicks == 2 {
                    self.current_mut().set_open(true);
                    self.current_mut().reset_width();
                }
            }
            Phase::Move => {
                if let Some(start) = self.grab {
                    self.current_mut().drag_to(start.shown_width() + drag.dx);
                }
            }
            Phase::Up => {
                if let Some(start) = self.grab.take() {
                    let pane = self.current_mut();
                    if drag.cancelled() {
                        *pane = start;
                    } else if !pane.is_open() {
                        pane.set_width(start.width());
                    }
                }
            }
        }
    }
}

const TAB_STRIP_KEY: &str = "workspace-tabs";
const WORKSPACE_KEY: &str = "workspace";

pub(crate) fn workspace(s: &CoderuneApp, k: &Kit, cx: &mut Cx<CoderuneApp>) -> El {
    let active = s.view;
    let page = match active {
        View::Home => screens::home(s, k),
        View::Profile => screens::profile(s, k),
        View::Projects => screens::projects(s, k),
        View::Documents => screens::documents(s, k),
        View::Subagents => text("Subagents").font(theme::HEADING).color(theme::ink()),
        View::Capture => screens::capture(s, k),
        View::Avatars => screens::avatar_gallery(s),
    };
    let content = ui::surface_frame(k, theme::panel(), k.radius.lg)
        .key("workspace-panel")
        .role("group")
        .label(active.label())
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(
            col()
                .key(active.id())
                .w_full()
                .h_full()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .scroll_edge_fade(16.0)
                .child(
                    col()
                        .w_full()
                        .px(theme::GUTTER)
                        .py(24.0)
                        .items_center()
                        .child(col().w_full().max_w(theme::CONTAINER).child(page)),
                ),
        );

    let mut layout = row()
        .key("workspace-layout")
        .role("tabpanel")
        .labelled_by("t-workspace")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .pr(theme::FRAME)
        .pb(theme::FRAME);
    let narrow = cx
        .layout_with(WORKSPACE_KEY, |rect| rect[2] < SIDEBAR_BREAKPOINT)
        .unwrap_or(false);
    let pane = if narrow {
        s.sidebar
            .narrow
            .unwrap_or_else(|| s.sidebar.wide.collapsed())
    } else {
        s.sidebar.wide
    };
    if pane.is_open() {
        layout = layout.child(
            col()
                .key("workspace-sidebar")
                .w(pane.shown_width())
                .h_full()
                .child(sidebar(s, k)),
        );
    }
    // Keep the same hit target at the window's edge after the pane closes.
    let grip = el()
        .key("sidebar-resize")
        .w(10.0)
        .h_full()
        .hit()
        .cursor(Cursor::ResizeColumn)
        .role("separator")
        .label("Resize sidebar")
        .range_value(
            f64::from(pane.shown_width()),
            0.0,
            f64::from(pane.max_width()),
        )
        .range_step(f64::from(SIDEBAR_STEP))
        .on_step(move |s: &mut CoderuneApp, steps, tx| {
            s.sidebar.set_narrow(narrow);
            s.sidebar.step(steps);
            tx.instant();
        })
        .on_drag(move |s: &mut CoderuneApp, drag, tx| {
            s.sidebar.set_narrow(narrow);
            s.sidebar.drag(drag);
            tx.instant();
        });

    // Remount the watcher when the width class changes, including resizes
    // smaller than Rune's threshold for reporting a changed layout rect.
    let width_class = el()
        .key(if narrow {
            "sidebar-narrow"
        } else {
            "sidebar-wide"
        })
        .absolute(0.0, 0.0)
        .size(0.0, 0.0)
        .a11y_hidden()
        .on_layout_tx(move |s: &mut CoderuneApp, _, tx| {
            if s.sidebar.set_narrow(narrow) {
                tx.instant();
            } else {
                tx.live_only();
            }
        });
    col()
        .key(WORKSPACE_KEY)
        .w_full()
        .h_full()
        .fill(theme::surface())
        .instant_changes()
        .child(titlebar(cx, k))
        .child(layout.child(grip).child(content))
        .child(width_class)
}

fn titlebar(cx: &Cx<CoderuneApp>, k: &Kit) -> El {
    let mut tab_kit = k.clone();
    tab_kit.c.layer_03 = Color::TRANSPARENT;
    tab_kit.c.pressed = theme::accent_soft();
    let tabs = TabStrip::new(TAB_STRIP_KEY, TabOrientation::Horizontal)
        .label("Window tabs")
        .active(Some("workspace"))
        .row_height(32.0)
        .overflow(TabOverflow::ShrinkThenScroll {
            min_width: 96.0,
            max_width: 144.0,
            pinned_width: 36.0,
            fade_width: 16.0,
        })
        .fill(theme::surface())
        .tabs([TabItem::<CoderuneApp>::new("workspace", "Workspace")
            .icon(View::Home.icon())
            .closable(false)])
        .build(&tab_kit);
    WindowTitlebar::from_insets(cx.chrome_insets())
        .height(theme::TOPBAR)
        .fill(theme::surface())
        .tabs(tabs)
        .build(k)
}

fn sidebar(s: &CoderuneApp, k: &Kit) -> El {
    let mut nav = Sidebar::new()
        .label("Workspace navigation")
        .inset(10.0)
        .header_padding(0.0, 8.0)
        .row_gap(2.0);
    for view in View::NAV {
        nav = nav.child(nav_item(s, view));
    }
    let settings = ui::nav_item(
        "nav-data-footer",
        "Settings",
        &icons::GEAR,
        s.settings.is_some(),
        |s: &mut CoderuneApp| s.select_settings(SettingsSection::General),
    );
    nav.footer(settings).footer(profile_item(s)).build(k)
}

fn nav_item(s: &CoderuneApp, view: View) -> El {
    ui::nav_item(
        view.nav_key(),
        view.label(),
        view.icon(),
        s.view == view,
        move |s: &mut CoderuneApp| s.navigate(view),
    )
}

fn profile_item(s: &CoderuneApp) -> El {
    let profile = s
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.profile());
    let name = profile.map_or("No baseline", |profile| profile.display_name.as_str());
    let baseline = profile.map_or_else(
        || "Developer baseline".to_string(),
        |profile| format!("Baseline v{}", profile.version),
    );
    let selected = s.view == View::Profile;
    row()
        .key(View::Profile.nav_key())
        .hit()
        .focusable(true)
        .focus_ring(2.0, -2.0, theme::ink())
        .role("button")
        .label(format!("Profile, {name}"))
        .h(40.0)
        .w_full()
        .items_center()
        .gap(10.0)
        .px(8.0)
        .radius(6.0)
        .when(selected, |this| this.fill(theme::accent_soft()))
        .when(!selected, |this| this.hover(theme::code()))
        .on_press(|s: &mut CoderuneApp| s.navigate(View::Profile))
        .child(
            row()
                .size(22.0, 22.0)
                .center()
                .radius(11.0)
                .fill(theme::pressed())
                .a11y_hidden()
                .child(
                    text(name.chars().next().unwrap_or('?').to_string())
                        .font(LABEL)
                        .color(theme::ink()),
                ),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .a11y_hidden()
                .child(text(name).font(LABEL).color(theme::ink()).truncate())
                .child(
                    text(baseline)
                        .font(LABEL_SM)
                        .color(theme::faint())
                        .truncate(),
                ),
        )
}
