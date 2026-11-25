//! Workspace chrome and in-window page navigation.

use coderune_ui::components::{self as ui, WhenExt};
use coderune_ui::icons;
use coderune_ui::theme::{self, LABEL, LABEL_SM};
use rune::{col, el, row, text, Color, Cursor, Cx, Drag, El, Phase, ScrollSpec};
use rune_kit::shell::{
    PaneState, Sidebar, SidebarRow, SidebarSection, TabItem, TabOrientation, TabOverflow, TabStrip,
    WindowTitlebar,
};
use rune_kit::{ControlSize, IconButton, IconButtonVariant, Kit};

use crate::app::{CoderuneApp, SettingsSection, View};
use crate::screens;

const SIDEBAR_BREAKPOINT: f32 = 1080.0;
const SIDEBAR_STEP: f32 = 8.0;
/// Every top-level navigation tab shares this height and inset, so the four
/// destinations read as one set.
const NAV_ROW_HEIGHT: f32 = 32.0;

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
    let chat = s.active_chat();
    let page = match active {
        View::Home => screens::home(s, k),
        View::Profile => screens::profile(s, k),
        View::Projects => screens::projects(s, k),
        View::Documents => screens::documents(s, k),
        View::Subagents => screens::subagents(s),
        View::Capture => screens::capture(s, k),
        View::Avatars => screens::avatar_gallery(s),
    };
    let content = ui::surface_frame(k, theme::canvas(), k.radius.lg)
        .key("workspace-panel")
        .role("group")
        .label(if chat.is_some() {
            "Chat"
        } else {
            active.label()
        })
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(if let Some(chat) = chat {
            col()
                .key(format!("t-chat-{}", chat.id))
                .w_full()
                .h_full()
                .child(screens::chat(s, k))
        } else {
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
                )
        });

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
        .child(titlebar(s, cx, k, narrow, pane.is_open()))
        .child(layout.child(grip).child(content))
        .child(width_class)
}

fn titlebar(
    s: &CoderuneApp,
    cx: &Cx<CoderuneApp>,
    k: &Kit,
    narrow: bool,
    sidebar_open: bool,
) -> El {
    let mut tab_kit = k.clone();
    tab_kit.c.layer_03 = Color::TRANSPARENT;
    tab_kit.c.pressed = theme::accent_soft();
    let mut tabs = vec![TabItem::<CoderuneApp>::new("workspace", "Workspace")
        .icon(s.view.icon())
        .closable(false)];
    for id in &s.chats.open {
        if let Some(chat) = s.chats.chat(id) {
            tabs.push(
                TabItem::<CoderuneApp>::new(chat.id.as_str(), chat.title.as_str())
                    .icon(&icons::MESSAGE)
                    .closable(true),
            );
        }
    }
    let strip = TabStrip::new(TAB_STRIP_KEY, TabOrientation::Horizontal)
        .label("Chat tabs")
        .active(Some(s.chats.active.as_deref().unwrap_or("workspace")))
        .row_height(32.0)
        .overflow(TabOverflow::ShrinkThenScroll {
            min_width: 96.0,
            max_width: 168.0,
            pinned_width: 36.0,
            fade_width: 16.0,
        })
        .fill(theme::surface())
        .tabs(tabs)
        .on_select(|s: &mut CoderuneApp, id: &str, _: &mut rune::Tx| {
            if id == "workspace" {
                s.show_workspace();
            } else {
                s.open_chat(id);
            }
        })
        .on_close(|s: &mut CoderuneApp, id: &str, _: &mut rune::Tx| s.close_chat(id))
        .build(&tab_kit);
    let mut titlebar = WindowTitlebar::from_insets(cx.chrome_insets())
        .height(theme::TOPBAR)
        .fill(theme::surface())
        .tabs(strip);
    if narrow || !sidebar_open {
        titlebar = titlebar.leading(
            IconButton::new(&icons::PANEL_LEFT)
                .variant(IconButtonVariant::Ghost)
                .control_size(ControlSize::Small)
                .label(if sidebar_open {
                    "Hide sidebar"
                } else {
                    "Show sidebar"
                })
                .on_press(move |s: &mut CoderuneApp| {
                    s.sidebar.set_narrow(narrow);
                    s.sidebar.toggle();
                })
                .build(k)
                .key("sidebar-toggle"),
        );
    }
    titlebar.build(k)
}

fn sidebar(s: &CoderuneApp, k: &Kit) -> El {
    let mut nav = Sidebar::new()
        .label("Workspace navigation")
        .inset(8.0)
        .header_padding(0.0, 6.0)
        .row_gap(1.0);
    // The four destinations sit together at the top, with no group label.
    for view in [View::Home, View::Documents, View::Subagents, View::Capture] {
        nav = nav.child(nav_item(s, view, k));
    }

    // Projects come below the tabs. The header carries only the add button: the
    // list is always shown, so a project is never hidden behind a disclosure.
    let mut projects = SidebarSection::new("Projects").key("nav-projects").action(
        IconButton::new(&icons::PLUS)
            .variant(IconButtonVariant::Ghost)
            .control_size(ControlSize::Small)
            .label("Add a project folder")
            .on_press_tx(|s: &mut CoderuneApp, tx: &mut rune::Tx| s.choose_project_folder(tx))
            .build(k)
            .key("sidebar-new-project"),
    );
    for project in s.workspace.iter().flat_map(|workspace| &workspace.projects) {
        projects = projects.row(project_row(s, project, k));
        if s.project_expanded(&project.id) {
            for chat in s.chats.chats_for(&project.id) {
                projects = projects.row(chat_row(s, chat, k));
            }
            let project_id = project.id.clone();
            projects = projects.row(
                SidebarRow::new("New chat")
                    .icon(&icons::PLUS)
                    .height(NAV_ROW_HEIGHT)
                    .depth(1)
                    .on_select(move |s: &mut CoderuneApp, tx: &mut rune::Tx| {
                        s.new_chat(&project_id);
                        tx.instant();
                    })
                    .build(k)
                    .key(format!("new-chat-{}", project.id)),
            );
        }
    }
    nav = nav.section(projects);

    let settings = ui::nav_item(
        "nav-data-footer",
        "Settings",
        &icons::GEAR,
        s.settings.is_some(),
        |s: &mut CoderuneApp| s.select_settings(SettingsSection::General),
    );
    nav.footer(settings).footer(profile_item(s)).build(k)
}

fn project_row(s: &CoderuneApp, project: &coderune_domain::Project, k: &Kit) -> El {
    let id = project.id.clone();
    let select = id.clone();
    let chevron = id.clone();
    let open = s.project_expanded(&id);
    let selected = s.active_chat().is_none()
        && s.view == View::Projects
        && s.project_form.editing.as_deref() == Some(id.as_str());
    let (done, total) = s.context_done(&id);
    SidebarRow::new(&project.name)
        .icon(&icons::FOLDER)
        .subtitle(format!("{done} of {total} context items"))
        .height(46.0)
        .selected(selected)
        .trailing(
            row()
                .gap(4.0)
                .items_center()
                .child(progress_bar(done, total))
                .child(
                    IconButton::new(if open {
                        k.glyph.triangle_down
                    } else {
                        k.glyph.triangle_right
                    })
                    .variant(IconButtonVariant::Ghost)
                    .control_size(ControlSize::Mini)
                    .label(if open { "Hide chats" } else { "Show chats" })
                    .on_press(move |s: &mut CoderuneApp| s.toggle_project_chats(&chevron))
                    .build(k)
                    .key(format!("project-chats-{id}")),
                ),
        )
        .on_select(move |s: &mut CoderuneApp, tx: &mut rune::Tx| {
            s.load_project_form(&select);
            s.navigate(View::Projects);
            tx.instant();
        })
        .build(k)
        .key(format!("project-{id}"))
}

fn chat_row(s: &CoderuneApp, chat: &crate::chats::Chat, k: &Kit) -> El {
    let id = chat.id.clone();
    let select = id.clone();
    let close = id.clone();
    SidebarRow::new(&chat.title)
        .icon(&icons::MESSAGE)
        .depth(1)
        .selected(s.chats.active.as_deref() == Some(id.as_str()))
        .on_select(move |s: &mut CoderuneApp, tx: &mut rune::Tx| {
            s.open_chat(&select);
            tx.instant();
        })
        .on_close(move |s: &mut CoderuneApp, _: &mut rune::Tx| s.close_chat(&close))
        .build(k)
        .key(format!("chat-{id}"))
}

fn progress_bar(done: u8, total: u8) -> El {
    let fraction = if total == 0 {
        0.0
    } else {
        f32::from(done) / f32::from(total)
    };
    row()
        .w(36.0)
        .h(3.0)
        .radius(2.0)
        .clip_rounded(2.0)
        .fill(theme::line())
        .a11y_hidden()
        .child(el().h_full().w_frac(fraction).fill(if fraction >= 1.0 {
            theme::muted()
        } else {
            theme::ink()
        }))
}

fn nav_item(s: &CoderuneApp, view: View, k: &Kit) -> El {
    SidebarRow::new(view.label())
        .icon(view.icon())
        .height(NAV_ROW_HEIGHT)
        .selected(s.view == view && s.active_chat().is_none())
        .on_select(move |s: &mut CoderuneApp, _: &mut rune::Tx| s.navigate(view))
        .build(k)
        .key(view.nav_key())
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
