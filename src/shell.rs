//! The app shell: zeron's Settings surface and its window chrome.
//!
//! Ported from zeron's `crates/ui/src/shell.rs` (MIT, Copyright 2026 Wing).
//! The app has a single surface — zeron's *Settings* route — so this is that
//! route's shell, not the chat shell: the root paints the theme's glass tint
//! over the blurred window background (`Shell::render`'s `div().bg(frost)`
//! where `frost = theme.glass()`), the sidebar column stands on zeron's
//! `sidebar_tone` layer (`wash(0.05)` + 1px right hairline), and a 38px
//! overlay strip drags the window (`titlebar_drag_region`). Plan R2 ported
//! zeron's settings route literally, i.e. without a titlebar cluster or a
//! collapsible sidebar; the `sidebar-collapse` amendment then ported the chat
//! shell's sidebar toggle onto this surface: the top-left `toggle-sidebar`
//! window-control button (zeron `window_control_button_with` around the
//! morphing `icons::sidebar_glyph`, at the shared 88px cluster start), the
//! `WidthTween` collapse/expand of the column and its `sidebar_collapsed`
//! state (⌘B, persisted). The centered title and the sidebar resize handle
//! stay unported.
//!
//! Landed by phase `chrome`: the shell surface, the drag strip, the sidebar
//! tone/column, the page's titlebar clearance, the section list (`render_nav`,
//! zeron `render_settings_nav`) and the footer identity row (`render_footer`,
//! zeron's `render_sidebar_footer_row` / user-menu trigger). Phase
//! `settings-surface` added the settings search field (`render_search`, zeron
//! `shell/settings_search.rs` layout), the section filter and the page pane.

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Div, FocusHandle, Focusable as _, Hsla, MouseButton, Render, SharedString,
    Stateful, Window, WindowControlArea, div, px,
};

use crate::settings;
use zeron_ui::icons;
use zeron_ui::motion;
use zeron_ui::popover;
use zeron_ui::settings::{SettingsSection, menu, widgets};
use zeron_ui::theme::{self, Appearance, Theme};
use zeron_ui::typography;

use crate::app_settings::{AppSettings, Appearance as AppAppearance};
use crate::i18n::tr;
use crate::settings::text_field::TextField;
use crate::settings::{
    AboutPage, AppearancePage, FinancePage, GeneralPage, NotificationsPage, ProfilePage, ResetPage,
};

/// zeron's default sidebar width (`settings.rs` `SIDEBAR_DEFAULT`). zeron's
/// settings route reads the persisted chat-sidebar width; mono-desk has no
/// resizable sidebar (no drag handle), so the column is fixed at the default
/// and only its *visibility* animates (`sidebar-collapse`). Bounds for
/// reference: `SIDEBAR_MIN` 242, `SIDEBAR_MAX` 400.
pub const SIDEBAR_WIDTH: f32 = 256.0;
/// Width the sidebar column collapses/expands between: zeron's chat shell
/// tween target, i.e. the settled content width or zero.
pub(crate) fn sidebar_target(collapsed: bool) -> f32 {
    if collapsed { 0.0 } else { SIDEBAR_WIDTH }
}
/// zeron `shell.rs` `TITLEBAR_CLUSTER_PAD`: the control row's own horizontal
/// inset. Paired with [`zeron_ui::chrome::titlebar_spacer_width`] so the row
/// padding plus the spacer lands on the declared cluster start (zeron's chrome
/// test asserts the same identity; the vendored `chrome.rs` copy stays private
/// like upstream).
const TITLEBAR_CLUSTER_PAD: f32 = 10.0;
/// Square action beside the account trigger in the sidebar footer (zeron
/// `SIDEBAR_FOOTER_ACTION_SIZE` = `SIDEBAR_FOOTER_BUTTON_SIZE`).
const SIDEBAR_FOOTER_ACTION_SIZE: f32 = 28.0;
/// Breathing room between the account trigger and its adjacent action.
const SIDEBAR_FOOTER_ACTION_GAP: f32 = 4.0;
/// The identity pill's height (zeron `SIDEBAR_FOOTER_BUTTON_SIZE`).
const SIDEBAR_FOOTER_BUTTON_SIZE: f32 = 28.0;
/// The avatar circle (zeron `SIDEBAR_FOOTER_AVATAR_SIZE`).
const SIDEBAR_FOOTER_AVATAR_SIZE: f32 = 16.0;
/// Identity shown in the footer when the Profile display name is empty.
const IDENTITY_NAME: &str = "Mono Desk";

/// A oneshot width tween, driven MANUALLY from render (zeron `shell.rs`
/// `WidthTween`). gpui keys an animation element's start time by its full
/// global element-id path, so a `with_animation` wrapper that mounts or
/// remounts silently replays the tween from t=0; evaluating the tween in
/// render keeps the element tree's shape constant, and a finished or stale
/// tween is exactly the steady state.
#[derive(Debug, Clone, Copy)]
struct WidthTween {
    from: f32,
    to: f32,
    started: Instant,
    duration: Duration,
    curve: motion::CubicBezier,
}

impl WidthTween {
    /// A column moving from `from` to `to` (zeron `WidthTween::panel`): longer
    /// moves take longer, so a wide pane travels at about the pace of the
    /// sidebar instead of covering twice the distance in the same time — 180ms
    /// up to the sidebar's width, growing with the square root of the distance
    /// to at most 240ms. The dialog entrance curve eases into the movement and
    /// settles without overshoot.
    fn panel(from: f32, to: f32) -> Self {
        Self {
            from,
            to,
            started: Instant::now(),
            duration: motion::PANEL_RESIZE
                .total()
                .mul_f32(motion::speed_scale() * panel_duration_scale(from, to)),
            curve: motion::PANEL_RESIZE.curve,
        }
    }

    /// The eased width at `now`, or `None` once the timeline has settled (the
    /// caller then paints its target exactly, like zeron `eval_tween`).
    fn eased_at(&self, now: Instant) -> Option<f32> {
        let total = self.duration.as_secs_f32();
        if total <= 0.0 {
            return None;
        }
        let raw = now.saturating_duration_since(self.started).as_secs_f32() / total;
        if raw >= 1.0 {
            return None;
        }
        Some(motion::lerp(self.from, self.to, self.curve.eval(raw)))
    }
}

/// [`WidthTween::panel`]'s distance factor: 1 for a move up to the sidebar's
/// width, growing with `sqrt(distance / SIDEBAR_WIDTH)` and clamped to `4/3`
/// (zeron's `WidthTween::panel` formula).
fn panel_duration_scale(from: f32, to: f32) -> f32 {
    ((to - from).abs() / SIDEBAR_WIDTH)
        .sqrt()
        .clamp(1.0, 4.0 / 3.0)
}

/// Dev hook (the same family as `MONO_DESK_APPEARANCE`/`MONO_DESK_SELFTEST`):
/// `MONO_DESK_SIDEBAR=collapsed|open` seeds the collapse state before the first
/// frame, so a headless selftest run can prove the collapsed shell without a
/// synthetic ⌘B. Launching with the seed performs no settings write; because it
/// seeds the same `AppSettings` global the toggle writes, a later settings
/// write in that dev run would persist it (the toggle itself is the normal
/// writer, not the seed).
pub(crate) fn sidebar_collapsed_from_env() -> Option<bool> {
    match std::env::var("MONO_DESK_SIDEBAR")
        .ok()?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "collapsed" | "closed" | "true" | "1" => Some(true),
        "open" | "expanded" | "false" | "0" => Some(false),
        _ => None,
    }
}

/// The nav label for a section in `lang`: the same `page.*` key the page
/// header uses, so the sidebar follows `AppSettings.language` like the pages do.
fn section_label(section: SettingsSection, lang: &str) -> &'static str {
    match section {
        SettingsSection::General => tr("page.general", lang),
        SettingsSection::Appearance => tr("page.appearance", lang),
        SettingsSection::Notifications => tr("page.notifications", lang),
        SettingsSection::Profile => tr("page.profile", lang),
        SettingsSection::FinanceUi => tr("page.finance", lang),
        SettingsSection::Reset => tr("page.reset", lang),
        SettingsSection::About => tr("page.about", lang),
    }
}

/// Whether a section row matches the sidebar query: case-insensitive substring
/// over both the localized and the English label, an empty query matching
/// everything (so both the current UI language and the upstream section names
/// stay searchable).
fn section_matches(section: SettingsSection, query: &str, lang: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    section.label().to_lowercase().contains(query)
        || section_label(section, lang).to_lowercase().contains(query)
}

/// The section rows the sidebar shows for `query` in `lang` (trimmed,
/// case-insensitive). Shared by the nav and the `MONO_DESK_SEARCH` probe.
pub(crate) fn sections_for_query(query: &str, lang: &str) -> Vec<SettingsSection> {
    let query = query.trim().to_lowercase();
    SettingsSection::ALL
        .into_iter()
        .filter(|section| section_matches(*section, &query, lang))
        .collect()
}

/// Dev hook (the same family as `MONO_DESK_APPEARANCE`/`MONO_DESK_SELFTEST`):
/// `MONO_DESK_SECTION=<slug>` opens the shell on that section, so a headless
/// run can render (and therefore prove) a page other than the default General.
/// Slugs are the section labels lowercased with spaces as dashes: `general`,
/// `appearance`, `notifications`, `profile`, `finance-ui`, `reset`, `about`.
pub(crate) fn section_from_env() -> Option<SettingsSection> {
    let slug = std::env::var("MONO_DESK_SECTION").ok()?;
    let wanted = slug.trim().to_ascii_lowercase();
    SettingsSection::ALL
        .iter()
        .copied()
        .find(|section| section.label().to_ascii_lowercase().replace(' ', "-") == wanted)
}

pub mod shell_actions {
    use gpui::actions;

    actions!(shell_actions, [FocusSearch, ToggleSidebar]);
}

pub struct Shell {
    /// The sidebar search field (zeron `shell/settings_search.rs` layout; the
    /// hit index is replaced by filtering the section rows, which is all a
    /// seven-page settings window needs).
    settings_search: Option<gpui::Entity<TextField>>,
    /// Selected settings section — the page pane follows it.
    section: SettingsSection,
    /// One roving focus handle per section row (zeron `settings_nav_focus`).
    nav_focus: Vec<FocusHandle>,
    /// The titlebar sidebar toggle's focus handle: gpui registers a tab stop (and
    /// refines `focus_visible`) only for a tracked handle, so the button needs
    /// one to be reachable by Tab like the other ported controls.
    sidebar_toggle_focus: FocusHandle,
    /// Armed by a left press on the titlebar strip: the next pointer move with
    /// the button down hands the drag to the compositor (zeron's
    /// `titlebar_should_move`).
    titlebar_should_move: bool,
    /// One-shot width animation for the sidebar collapse/expand (zeron
    /// `sidebar_tween`): the current width is evaluated from it each frame.
    sidebar_tween: Option<WidthTween>,
    /// Re-armed at the start of every render while a manually driven tween is
    /// mid-flight, so the render tail keeps frames coming (zeron's
    /// `motion_active` cell).
    motion_active: bool,
    /// Lazily created page views (zeron's shell keeps one entity per settings
    /// page); each owns its select/scroll state across frames.
    general_page: Option<gpui::Entity<GeneralPage>>,
    appearance_page: Option<gpui::Entity<AppearancePage>>,
    notifications_page: Option<gpui::Entity<NotificationsPage>>,
    profile_page: Option<gpui::Entity<ProfilePage>>,
    finance_page: Option<gpui::Entity<FinancePage>>,
    reset_page: Option<gpui::Entity<ResetPage>>,
    about_page: Option<gpui::Entity<AboutPage>>,
}

impl Shell {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let placeholder = tr("settings.search", &AppSettings::global(cx).language);
        let settings_search = cx.new(|cx| TextField::new("settings-search", placeholder, cx));
        // Dev hook: `MONO_DESK_SEARCH=<text>` seeds the query so a headless run
        // can prove the section filter without synthetic typing.
        if let Ok(query) = std::env::var("MONO_DESK_SEARCH") {
            settings_search.update(cx, |field, cx| field.set_value(query, cx));
        }
        // Any edit re-renders the shell so the section list can filter.
        cx.observe(&settings_search, |_, _, cx| cx.notify())
            .detach();
        // Dev hook: `MONO_DESK_SIDEBAR=<collapsed|open>` seeds the collapse
        // state (no settings write on launch; the toggle is the normal writer).
        if let Some(collapsed) = sidebar_collapsed_from_env() {
            AppSettings::global_mut(cx).sidebar_collapsed = collapsed;
        }
        Self {
            settings_search: Some(settings_search),
            section: section_from_env().unwrap_or_default(),
            nav_focus: SettingsSection::ALL
                .iter()
                .map(|_| cx.focus_handle())
                .collect(),
            sidebar_toggle_focus: cx.focus_handle(),
            titlebar_should_move: false,
            sidebar_tween: None,
            motion_active: false,
            general_page: None,
            appearance_page: None,
            notifications_page: None,
            profile_page: None,
            finance_page: None,
            reset_page: None,
            about_page: None,
        }
    }

    fn select_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        if self.section != section {
            self.section = section;
            cx.notify();
        }
    }

    /// Flip light/dark: persist the choice into `AppSettings` (the same key the
    /// pre-port footer wrote) and funnel it through the one theme path the
    /// Appearance page uses, so accent, surface material and font size all stay
    /// applied. The quick toggle the demo window has always had.
    fn toggle_appearance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = match Theme::of(cx).appearance {
            Appearance::Dark => AppAppearance::Light,
            Appearance::Light => AppAppearance::Dark,
        };
        AppSettings::global_mut(cx).appearance = next;
        settings::persist_now(cx);
        settings::apply_appearance(window, cx);
        cx.notify();
    }

    /// ⌘F: put the keyboard in the settings search (zeron's shortcut for the
    /// same field). A collapsed column has no search field on screen, so the
    /// shortcut opens the column first — zeron's settings route never collapses
    /// its sidebar, so there is no upstream behaviour to match here.
    fn focus_search(
        &mut self,
        _: &shell_actions::FocusSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if AppSettings::global(cx).sidebar_collapsed {
            self.set_sidebar_collapsed(false, cx);
        }
        if let Some(input) = self.settings_search.clone() {
            let focus = input.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
        }
        cx.stop_propagation();
    }

    /// The sidebar's width as painted right now (zeron `Shell::sidebar_now`):
    /// the eased tween while it runs, else the settled target.
    fn sidebar_now(&self, cx: &gpui::App) -> f32 {
        let target = sidebar_target(AppSettings::global(cx).sidebar_collapsed);
        if motion::reduced_motion(cx) {
            return target;
        }
        self.sidebar_tween
            .and_then(|tween| tween.eased_at(Instant::now()))
            .unwrap_or(target)
    }

    /// [`Self::sidebar_now`] at a frame time, flagging `motion_active` while
    /// the tween runs so render's tail requests the next frame (zeron
    /// `Shell::eval_tween`). Reduced motion snaps to the target.
    fn eval_sidebar_width(&mut self, now: Instant, target: f32, reduced: bool) -> f32 {
        if reduced {
            self.sidebar_tween = None;
            return target;
        }
        match self.sidebar_tween.and_then(|tween| tween.eased_at(now)) {
            Some(width) => {
                self.motion_active = true;
                width
            }
            None => {
                self.sidebar_tween = None;
                target
            }
        }
    }

    /// Persist `collapsed` and start the width tween from the width the column
    /// paints right now (shared by the toggle and the ⌘F open-before-focus
    /// path). The state write and the animation are one step so the two paths
    /// cannot disagree.
    fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        let from = self.sidebar_now(cx);
        AppSettings::global_mut(cx).sidebar_collapsed = collapsed;
        settings::persist_now(cx);
        self.sidebar_tween = Some(WidthTween::panel(from, sidebar_target(collapsed)));
        cx.notify();
    }

    /// Whether the keyboard sits on a control that lives inside the sidebar
    /// column (the search field or one of the nav rows). Those controls unmount
    /// once the column has closed, so the toggle must not leave the focus on
    /// one of them — keys would be dispatched to an element that is no longer
    /// painted. zeron's chat sidebar keeps its rows mounted (only the width
    /// clips) and its composer never unmounts, so upstream needs no equivalent.
    fn sidebar_owns_focus(&self, window: &Window, cx: &gpui::App) -> bool {
        if let Some(input) = self.settings_search.as_ref()
            && input.read(cx).focus_handle(cx).is_focused(window)
        {
            return true;
        }
        self.nav_focus.iter().any(|focus| focus.is_focused(window))
    }

    /// ⌘B / the titlebar toggle: flip the persisted collapse state, like zeron
    /// `Shell::toggle_sidebar` (minus its chat-only panel-fit work), and tween
    /// the column from the width it currently paints to the new target.
    fn toggle_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let collapsed = !AppSettings::global(cx).sidebar_collapsed;
        self.set_sidebar_collapsed(collapsed, cx);
        if collapsed && self.sidebar_owns_focus(window, cx) {
            window.blur();
        }
    }

    /// The query the section list filters on (trimmed, case-insensitive).
    fn search_query(&self, cx: &Context<Self>) -> String {
        self.settings_search
            .as_ref()
            .map(|input| input.read(cx).value().trim().to_lowercase())
            .unwrap_or_default()
    }

    /// zeron's settings search field (`shell/settings_search.rs`
    /// `render_settings_search_field`): a 32px rounded field with the
    /// magnifier in a nav glyph's slot, the query, and (idle) the ⌘F hint;
    /// once there is a query the clear button replaces the hint.
    fn render_search(
        &mut self,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(input) = self.settings_search.clone() else {
            return div().into_any_element();
        };
        let lang = AppSettings::global(cx).language.clone();
        let placeholder = tr("settings.search", &lang);
        if input.read(cx).placeholder() != placeholder {
            input.update(cx, |field, cx| field.set_placeholder(placeholder, cx));
        }
        let focus = input.read(cx).focus_handle(cx);
        let empty = input.read(cx).value().trim().is_empty();
        let idle = empty && !focus.is_focused(window);
        div()
            .id("settings-search")
            .debug_selector(|| "settings-search".into())
            .w_full()
            .h(px(32.0))
            .rounded(px(8.0))
            .px(px(8.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .cursor_text()
            .text_size(typography::ui_rems(13.0))
            .bg(theme.wash(0.03))
            .hover(|style| style.bg(theme.wash(0.06)))
            .on_mouse_down(MouseButton::Left, {
                let focus = focus.clone();
                move |_, window, cx| {
                    window.focus(&focus, cx);
                    cx.stop_propagation();
                }
            })
            .child(
                div().w(px(16.0)).flex_none().flex().justify_center().child(
                    icons::icon(icons::MAGNIFER)
                        .size(px(14.0))
                        .text_color(theme.text_muted),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h(px(17.0))
                    .text_color(theme.text)
                    .child(input.clone()),
            )
            .when(idle, |field| field.child(popover::kbd_hint(theme, "⌘F")))
            .when(!empty, |field| {
                field.child(
                    div()
                        .id("settings-search-clear")
                        .role(gpui::Role::Button)
                        .aria_label(tr("settings.clear", &lang))
                        .tab_index(0)
                        .focus_visible(|style| style.border_2().border_color(theme.accent))
                        .size(px(20.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|style| style.bg(theme.wash(0.08)))
                        .on_click(cx.listener(|this, _, window, cx| {
                            let Some(input) = this.settings_search.clone() else {
                                return;
                            };
                            let focus = input.read(cx).focus_handle(cx);
                            input.update(cx, |field, cx| field.set_value("", cx));
                            window.focus(&focus, cx);
                            cx.notify();
                        }))
                        .child(
                            icons::icon(icons::CLOSE)
                                .size(px(12.0))
                                .text_color(theme.text_muted),
                        ),
                )
            })
            .into_any_element()
    }

    /// zeron `Shell::sidebar_tone`: the absolute full-height column that paints
    /// the shell's 5% ink wash under the sidebar, with a 1px hairline on its
    /// inner edge. Upstream rounds/two-tone-trims this layer only for Linux
    /// client-side-decorated corners (NG3), so the macOS port paints it square
    /// — the window server rounds the window itself.
    fn sidebar_tone(width: f32, border_color: Hsla) -> Div {
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left_0()
            .w(px(width))
            .bg(theme::wash(0.05))
            .border_r_1()
            .border_color(border_color)
    }

    /// Make a titlebar strip drag the window — zed's platform-titlebar pattern
    /// (zeron's `.drag` region): mark it a [`WindowControlArea::Drag`] (macOS
    /// app-owned titlebar), hand the drag to the compositor once the pointer
    /// moves with the button down, and double-click zooms.
    fn titlebar_drag_region(
        &self,
        id: &'static str,
        el: Div,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        el.id(id)
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down_out(cx.listener(|this, _, _, _| this.titlebar_should_move = false))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = false),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = true),
            )
            // Hand the drag to the compositor only while the button is
            // actually held (`pressed_button` guard): on macOS
            // `start_window_move` runs AppKit's NATIVE drag session
            // (`performWindowDragWithEvent:`), and AppKit resolves a quick
            // second click inside that session as a titlebar double-click —
            // system zoom — natively, beyond gpui's reach.
            .on_mouse_move(
                cx.listener(|this, event: &gpui::MouseMoveEvent, window, _| {
                    if this.titlebar_should_move && event.pressed_button == Some(MouseButton::Left)
                    {
                        this.titlebar_should_move = false;
                        window.start_window_move();
                    }
                }),
            )
            .on_click(|event, window, _| {
                if event.click_count() == 2 {
                    if cfg!(target_os = "macos") {
                        // Native titlebar double-click action (zoom/minimize
                        // per system preference).
                        window.titlebar_double_click();
                    } else {
                        window.zoom_window();
                    }
                }
            })
    }

    /// The section list (zeron `render_settings_nav`): a roving tab list of
    /// label + icon rows with one group break at the workspace rows, filtered
    /// by the settings search above it. Labels come from the page i18n table
    /// (`section_label`), so the nav follows `AppSettings.language`; the
    /// filter matches both the localized and the English label.
    fn render_nav(
        &mut self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let reduced_motion = motion::reduced_motion(cx);
        let selected = self.section;
        let lang = AppSettings::global(cx).language.clone();
        // The search filters the section rows (zeron indexes every setting on
        // every page; a seven-page window only needs the section list).
        let query = self.search_query(cx);
        let visible: Vec<SettingsSection> = sections_for_query(&query, &lang);
        let search = self.render_search(theme, window, cx);
        let mut rows = Vec::with_capacity(visible.len());
        for (index, item) in SettingsSection::ALL.into_iter().enumerate() {
            if !visible.contains(&item) {
                continue;
            }
            let is_selected = item == selected;
            let key = format!("settings-nav-{}", item.label());
            let hover_key = format!("{key}-hover");
            let selection_t = widgets::tab_selection_t(
                window,
                format!("{key}-selection"),
                is_selected,
                reduced_motion,
            );
            let tab_text = motion::hover_blend(
                &hover_key,
                motion::mix(theme.text_muted, theme.text, selection_t),
                theme.text,
            );
            let focus = self.nav_focus[index].clone().tab_stop(is_selected);
            let items = visible.clone();
            rows.push(
                widgets::section_tab(theme, is_selected, selection_t, key, hover_key)
                    .when(item.starts_nav_group(), |el| el.mt(px(Theme::SPACE_LG)))
                    .role(gpui::Role::Tab)
                    .aria_label(section_label(item, &lang))
                    .aria_selected(is_selected)
                    .track_focus(&focus)
                    .tab_index(0)
                    .tab_stop(is_selected)
                    .on_key_down(cx.listener(
                        move |this, event: &gpui::KeyDownEvent, window, cx| {
                            let Some(current) = items.iter().position(|section| *section == item)
                            else {
                                return;
                            };
                            let next = match event.keystroke.key.as_str() {
                                "up" | "left" => (current + items.len() - 1) % items.len(),
                                "down" | "right" => (current + 1) % items.len(),
                                "home" => 0,
                                "end" => items.len() - 1,
                                _ => return,
                            };
                            let target = items[next];
                            this.select_section(target, cx);
                            let all_index = SettingsSection::ALL
                                .iter()
                                .position(|section| *section == target)
                                .unwrap_or(0);
                            let focus = this.nav_focus[all_index].clone();
                            window.focus(&focus, cx);
                            cx.stop_propagation();
                        },
                    ))
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| this.select_section(item, cx)))
                    .child(icons::icon(item.icon()).size(px(16.0)).text_color(tab_text))
                    .child(SharedString::from(section_label(item, &lang))),
            );
        }
        div()
            .id("settings-sections")
            .role(gpui::Role::TabList)
            .aria_label("Settings sections")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_none()
                    .px(px(Theme::SPACE_SM))
                    .pt(px(Theme::SPACE_MD))
                    .pb(px(Theme::SPACE_SM))
                    .child(search),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .px(px(Theme::SPACE_SM))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .when(visible.is_empty(), |list| {
                        list.child(
                            div()
                                .px(px(Theme::SPACE_SM))
                                .py(px(Theme::SPACE_SM))
                                .text_size(typography::ui_rems(12.5))
                                .text_color(theme.text_muted)
                                .child(tr("settings.empty", &lang)),
                        )
                    })
                    .children(rows),
            )
            .into_any_element()
    }

    /// The footer's account trigger (zeron's user-menu trigger): an initial
    /// avatar in the inverse text colour, the display name, and the appearance
    /// toggle as the footer's square action. The name is the Profile display
    /// name from `AppSettings` (settings-surface's known gap); the built-in
    /// identity is only the empty-name fallback.
    fn render_footer(&mut self, theme: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let name: SharedString = {
            let configured = AppSettings::global(cx).display_name.clone();
            if configured.trim().is_empty() {
                IDENTITY_NAME.into()
            } else {
                configured
            }
        };
        let initial: SharedString = name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".into())
            .into();
        div()
            .p(px(Theme::SPACE_SM))
            .flex_none()
            .child(
                div()
                    .id("sidebar-footer-row")
                    .debug_selector(|| "sidebar-footer-row".into())
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(SIDEBAR_FOOTER_ACTION_GAP))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h(px(SIDEBAR_FOOTER_BUTTON_SIZE))
                            .rounded(px(8.0))
                            .px(px(Theme::SPACE_SM))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(Theme::SPACE_SM))
                            .text_size(typography::ui_rems(13.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.text.opacity(0.8))
                            .child(
                                // Avatar: text-coloured circle, initial in the
                                // surface colour (zeron user-menu.tsx).
                                div()
                                    .size(px(SIDEBAR_FOOTER_AVATAR_SIZE))
                                    .flex_none()
                                    .rounded_full()
                                    .bg(theme.text)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .font_family(theme.font_mono.clone())
                                    .text_size(px(10.0))
                                    .line_height(px(SIDEBAR_FOOTER_AVATAR_SIZE))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.bg)
                                    .child(div().w_full().text_center().child(initial)),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .line_height(px(17.0))
                                    .child(name),
                            ),
                    )
                    .child(self.render_appearance_toggle(theme, cx)),
            )
            .into_any_element()
    }

    /// The footer's square action: a light/dark toggle. The glyph shows the
    /// appearance a click switches to (zeron's `appearance_icon` rule).
    fn render_appearance_toggle(&self, theme: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let target_light = matches!(theme.appearance, Appearance::Dark);
        let glyph_color = if target_light {
            theme.text
        } else {
            theme.text_muted
        };
        div()
            .id("appearance-toggle")
            .debug_selector(|| "appearance-toggle".into())
            .role(gpui::Role::Button)
            .aria_label(if target_light {
                "Switch to light appearance"
            } else {
                "Switch to dark appearance"
            })
            .tab_index(0)
            .focus_visible(|style| style.border_2().border_color(theme.accent))
            .size(px(SIDEBAR_FOOTER_ACTION_SIZE))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.0))
            .text_color(glyph_color)
            .bg(motion::hover_blend(
                "appearance-toggle",
                theme.glass_hover().opacity(0.0),
                theme.glass_hover(),
            ))
            .on_hover(motion::hover_listener("appearance-toggle"))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| this.toggle_appearance(window, cx)))
            .child(
                icons::icon(if target_light {
                    icons::SUN
                } else {
                    icons::MOON
                })
                .size(px(16.0))
                .text_color(glyph_color),
            )
            .into_any_element()
    }

    /// zeron's titlebar control cluster (`Shell::render_titlebar_cluster`),
    /// reduced to the one control this app has: the sidebar toggle. Painted as
    /// an overlay at the window's top-left ABOVE the drag strip and the
    /// sidebar, so the button keeps its element identity and never moves or
    /// remounts while the column width animates beneath it. The container has
    /// no listeners: the space between controls falls through to the drag
    /// strip below, like upstream. The macOS traffic-light spacer comes from
    /// the vendored geometry; fullscreen snaps instead of tweening because this
    /// shell keeps no fullscreen tween state.
    fn render_titlebar_cluster(
        &self,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = !AppSettings::global(cx).sidebar_collapsed;
        let spacer = zeron_ui::chrome::titlebar_spacer_width(
            cfg!(target_os = "macos"),
            window.is_fullscreen(),
            TITLEBAR_CLUSTER_PAD,
        );
        div()
            .absolute()
            .top_0()
            .left_0()
            .h(px(Theme::TITLEBAR_HEIGHT))
            .flex()
            .flex_row()
            .items_center()
            .pt(px(Theme::TITLEBAR_TOP_PAD))
            .px(px(TITLEBAR_CLUSTER_PAD))
            // Off macOS the spacer is zero; skip it entirely like upstream's
            // `titlebar_spacer` (which returns `None` there).
            .children((spacer > 0.0).then(|| div().flex_none().h_full().w(px(spacer))))
            .child(self.render_sidebar_toggle(theme, open, cx))
            .into_any_element()
    }

    /// zeron's `toggle-sidebar` window-control button (`window_control_button_with`
    /// around `icons::sidebar_glyph`): a 24px, rounded-6 button with the fading
    /// hover wash, the glyph whose panel narrows 5.5 → 1.75 while the column
    /// closes, and the macOS-occluded press handling (`.occlude()` +
    /// `prevent_default`) that keeps a press from becoming a window drag or a
    /// titlebar double-click. mono-desk adds the a11y metadata the other ported
    /// controls carry (role/aria/tab stop) and a localized tooltip label.
    fn render_sidebar_toggle(
        &self,
        theme: &Theme,
        open: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lang = AppSettings::global(cx).language.clone();
        let label = tr("sidebar.toggle", &lang);
        let fade_key = "window-control-toggle-sidebar";
        let reduced = motion::reduced_motion(cx);
        let focus = self.sidebar_toggle_focus.clone();
        div()
            .id("toggle-sidebar")
            .debug_selector(|| "toggle-sidebar".into())
            .role(gpui::Role::Button)
            .aria_label(label)
            .aria_expanded(open)
            .track_focus(&focus)
            .tab_index(0)
            .focus_visible(|style| style.border_2().border_color(theme.accent))
            .size(px(24.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(6.0))
            .cursor_pointer()
            .bg(motion::hover_blend(
                fade_key,
                theme.glass_hover().opacity(0.0),
                theme.glass_hover(),
            ))
            .on_hover(motion::hover_listener(fade_key))
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .on_click(cx.listener(|this, _, window, cx| {
                cx.stop_propagation();
                this.toggle_sidebar(window, cx);
            }))
            // Enter/Space activate the focused toggle like zeron's other
            // interactive rows (`is_activation`): a tracked tab stop must be
            // operable from the keyboard, not just focusable.
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if !event.is_held && matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    cx.stop_propagation();
                    this.toggle_sidebar(window, cx);
                }
            }))
            .tooltip(widgets::text_tooltip(label))
            .child(icons::sidebar_glyph(
                motion::state_t("toggle-sidebar", open, motion::GLYPH_STATE, reduced),
                false,
                16.0,
                theme.text_muted,
            ))
            .into_any_element()
    }

    /// The settings sidebar column (zeron `render_settings_page` + the chat
    /// shell's `render_sidebar` collapse mask): the outer column is the
    /// animated, clipped width and the content keeps its settled width, so a
    /// closing column clips instead of reflowing its rows (zeron's stable
    /// inner/clipped outer behaviour). The content unmounts once the column has
    /// fully closed, so its search field and nav rows leave the tab order.
    fn render_sidebar(
        &mut self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
        width: f32,
    ) -> AnyElement {
        let mut column = div()
            .id("sidebar-column")
            .debug_selector(|| "sidebar-column".into())
            .h_full()
            .flex_none()
            .overflow_hidden()
            .w(px(width));
        if width > 0.5 {
            column = column.child(
                div()
                    .w(px(SIDEBAR_WIDTH))
                    .h_full()
                    .flex()
                    .flex_col()
                    .pt(px(Theme::TITLEBAR_HEIGHT))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .child(self.render_nav(theme, window, cx)),
                    )
                    .child(self.render_footer(theme, cx)),
            );
        }
        column.into_any_element()
    }

    /// The page pane: the selected section's page view.
    /// zeron's pages own their scroll (`page_column` carries the titlebar
    /// clearance inside it), so the pane is only the host: `flex_1`, full
    /// height, no scroll of its own. Page views are created lazily and kept,
    /// like zeron's shell keeps one entity per settings page, so open select
    /// menus and scroll offsets survive a re-render.
    fn render_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // zeron's shell keeps one lazily created entity per settings page, so
        // open select menus and scroll offsets survive section switches.
        let page: AnyElement = match self.section {
            SettingsSection::General => {
                let page = self
                    .general_page
                    .get_or_insert_with(|| cx.new(|_| GeneralPage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::Appearance => {
                let page = self
                    .appearance_page
                    .get_or_insert_with(|| cx.new(|_| AppearancePage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::Notifications => {
                let page = self
                    .notifications_page
                    .get_or_insert_with(|| cx.new(|_| NotificationsPage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::Profile => {
                let page = self
                    .profile_page
                    .get_or_insert_with(|| cx.new(|_| ProfilePage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::FinanceUi => {
                let page = self
                    .finance_page
                    .get_or_insert_with(|| cx.new(|_| FinancePage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::Reset => {
                let page = self
                    .reset_page
                    .get_or_insert_with(|| cx.new(|_| ResetPage::new()));
                page.clone().into_any_element()
            }
            SettingsSection::About => {
                let page = self
                    .about_page
                    .get_or_insert_with(|| cx.new(|_| AboutPage::new()));
                page.clone().into_any_element()
            }
        };
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(page)
            .into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        // Manual tween drive bookkeeping for this pass (zeron `Shell::render`):
        // the width evaluation below re-arms `motion_active` while the collapse
        // tween runs, and the render tail requests the next frame.
        self.motion_active = false;
        let collapsed = AppSettings::global(cx).sidebar_collapsed;
        let settled_sidebar = sidebar_target(collapsed);
        let sidebar_width =
            self.eval_sidebar_width(Instant::now(), settled_sidebar, motion::reduced_motion(cx));
        // zeron publishes the sidebar width every frame so a page's dropdown can
        // measure the content pane (`settings::set_sidebar_width`). Publish the
        // settled target rather than the animated paint width: at rest that is
        // exactly where the pane starts, and `pane_bounds`/`dropdown_limits`
        // anchor popovers at `left = sidebar_width`. (Upstream's settings route
        // never animates its sidebar, so its constant is the same thing; mid-tween
        // a dropdown opened in the remaining ~100ms clamps to the settled edge
        // until the animation lands.)
        menu::set_sidebar_width(settled_sidebar, cx);
        let drag = self.titlebar_drag_region(
            "settings-titlebar",
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(Theme::TITLEBAR_HEIGHT)),
            cx,
        );
        let cluster = self.render_titlebar_cluster(&theme, window, cx);
        let sidebar = self.render_sidebar(&theme, window, cx, sidebar_width);
        let page = self.render_page(cx);
        // zeron `Shell::render`'s root: `id("shell-root")`, row layout,
        // `bg(theme.glass())`, theme text colour and the 14px rem scale.
        let root = div()
            .id("shell-root")
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.glass())
            .text_color(theme.text)
            .font_family(theme.font_sans.clone())
            .text_size(typography::ui_rems(14.0))
            .on_action(cx.listener(Self::focus_search))
            .on_action(
                cx.listener(|this, _: &shell_actions::ToggleSidebar, window, cx| {
                    this.toggle_sidebar(window, cx)
                }),
            )
            // The tone melts away with the collapse instead of vanishing in a
            // frame (zeron paints it at `sidebar_now`).
            .child(Self::sidebar_tone(sidebar_width, theme.border))
            .child(
                div()
                    .size_full()
                    .relative()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .flex_row()
                            .child(sidebar)
                            .child(page),
                    )
                    .child(drag)
                    // Above the drag strip and the sidebar: zeron's overlay
                    // cluster keeps the toggle live in both sidebar states.
                    .child(cluster),
            );
        // A manually driven tween is mid-flight: keep frames coming (the same
        // scheduling `with_animation` would have requested). Hover color fades
        // and glyph state morphs ride the same clock; their once-per-frame
        // tick lives here — this is the window's root render, so it runs
        // exactly once per frame. Both clocks tick on EVERY pass (zeron's
        // bitwise `|` tail): short-circuiting them while the collapse tween
        // runs would freeze their frame counters for those 180ms.
        let hover_fades_active = motion::hover_fades_active();
        let state_morphs_active = motion::state_morphs_active();
        if self.motion_active || hover_fades_active || state_morphs_active {
            window.request_animation_frame();
        }
        root
    }
}

#[cfg(test)]
mod search_tests {
    use super::{SettingsSection, sections_for_query};

    #[test]
    fn empty_query_matches_every_section() {
        assert_eq!(
            sections_for_query("", "en").len(),
            SettingsSection::ALL.len()
        );
        assert_eq!(
            sections_for_query("  ", "vi").len(),
            SettingsSection::ALL.len()
        );
    }

    #[test]
    fn query_filters_english_labels_case_insensitively() {
        assert_eq!(
            sections_for_query("fin", "en"),
            vec![SettingsSection::FinanceUi]
        );
        assert!(sections_for_query("ui", "en").contains(&SettingsSection::FinanceUi));
        assert_eq!(
            sections_for_query("gen", "en"),
            vec![SettingsSection::General]
        );
        assert!(sections_for_query("zzz", "en").is_empty());
    }

    #[test]
    fn query_also_matches_the_localized_label() {
        // Vietnamese "Chung" is General's page title in `src/i18n.rs`; the nav
        // shows that label, so the filter must find it.
        assert_eq!(
            sections_for_query("chung", "vi"),
            vec![SettingsSection::General]
        );
        // The English label stays searchable as well.
        assert_eq!(
            sections_for_query("general", "vi"),
            vec![SettingsSection::General]
        );
    }
}

#[cfg(test)]
mod sidebar_tests {
    use super::{
        SIDEBAR_WIDTH, TITLEBAR_CLUSTER_PAD, WidthTween, panel_duration_scale, sidebar_target,
    };
    use std::time::{Duration, Instant};

    /// The column heads for exactly two widths: its settled content width and
    /// zero — the two endpoints zeron's chat shell tweens between.
    #[test]
    fn target_is_the_settled_width_or_zero() {
        assert_eq!(sidebar_target(false), SIDEBAR_WIDTH);
        assert_eq!(sidebar_target(true), 0.0);
        assert_eq!(SIDEBAR_WIDTH, 256.0);
    }

    /// zeron `WidthTween::panel`'s distance factor: a collapse/expand move of
    /// the sidebar's own width stays on the 180ms base; wider moves stretch
    /// with the square root of the distance and clamp at `4/3`.
    #[test]
    fn panel_duration_scale_stretches_with_distance_and_clamps() {
        assert_eq!(panel_duration_scale(0.0, SIDEBAR_WIDTH), 1.0);
        assert_eq!(panel_duration_scale(SIDEBAR_WIDTH, 0.0), 1.0);
        // 400px (zeron's SIDEBAR_MAX): sqrt(400/256) = 1.25.
        assert!((panel_duration_scale(0.0, 400.0) - 1.25).abs() < 1e-6);
        // Very wide moves saturate at 4/3, and short moves never shrink below 1.
        assert_eq!(panel_duration_scale(0.0, 4096.0), 4.0 / 3.0);
        assert_eq!(panel_duration_scale(240.0, 256.0), 1.0);
    }

    /// `WidthTween::panel` actually applies the distance factor the docs claim:
    /// a move of the sidebar's own width is the `PANEL_RESIZE` base (180ms) and
    /// a 400px move is 1.25x it. Compared as ratios so `ZERON_MOTION_SCALE`
    /// (the measurement knob) cannot make the test flaky.
    #[test]
    fn panel_tween_duration_scales_with_distance() {
        let full = WidthTween::panel(0.0, SIDEBAR_WIDTH);
        let wide = WidthTween::panel(0.0, 400.0);
        let base =
            zeron_ui::motion::PANEL_RESIZE.total().as_secs_f32() * zeron_ui::motion::speed_scale();
        assert!(
            (full.duration.as_secs_f32() - base).abs() < 1e-4,
            "full-width move = {:?}, base = {base}s",
            full.duration
        );
        let ratio = wide.duration.as_secs_f32() / full.duration.as_secs_f32();
        assert!((ratio - 1.25).abs() < 1e-5, "400px move ratio = {ratio}");
        assert_eq!(full.from, 0.0);
        assert_eq!(full.to, SIDEBAR_WIDTH);
    }

    /// A running tween eases between its endpoints and reports settled once the
    /// timeline is over; a reversal evaluates its own endpoints in either
    /// direction.
    #[test]
    fn width_tween_eases_and_settles() {
        let tween = WidthTween {
            from: SIDEBAR_WIDTH,
            to: 0.0,
            started: Instant::now(),
            duration: Duration::from_millis(180),
            curve: zeron_ui::motion::PANEL_RESIZE.curve,
        };
        let at = |ms: u64| tween.eased_at(tween.started + Duration::from_millis(ms));
        // Mount frame: the tween starts from its `from` edge.
        assert_eq!(at(0), Some(SIDEBAR_WIDTH));
        // Mid-flight: strictly between the endpoints, and past the start.
        let early = at(45).expect("mid-flight");
        let mid = at(90).expect("mid-flight");
        assert!(mid < SIDEBAR_WIDTH && mid > 0.0, "mid = {mid}");
        assert!(mid < early && early < SIDEBAR_WIDTH, "early = {early}");
        // Settled exactly at the end of the timeline: the caller paints the
        // target (`None` = no longer mid-flight), never a stale width.
        assert_eq!(at(180), None);
        assert_eq!(at(10_000), None);
        // A reversal starts from whatever it painted and eases the other way.
        let reverse = WidthTween {
            from: 0.0,
            to: SIDEBAR_WIDTH,
            started: Instant::now(),
            duration: Duration::from_millis(180),
            curve: zeron_ui::motion::PANEL_RESIZE.curve,
        };
        let value = reverse
            .eased_at(reverse.started + Duration::from_millis(90))
            .expect("mid-flight");
        assert!(value > 0.0 && value < SIDEBAR_WIDTH, "reverse = {value}");
    }

    /// The vendored cluster geometry and this shell's own cluster padding must
    /// still land the toggle on the declared 88px cluster start (the identity
    /// zeron's chrome test pins).
    #[test]
    fn cluster_pad_lands_on_the_declared_cluster_start() {
        assert_eq!(
            TITLEBAR_CLUSTER_PAD
                + zeron_ui::chrome::titlebar_spacer_width(true, false, TITLEBAR_CLUSTER_PAD),
            zeron_ui::chrome::titlebar_cluster_start(false)
        );
        // Linux/Windows have no traffic lights: the cluster hugs the edge.
        assert_eq!(
            zeron_ui::chrome::titlebar_spacer_width(false, false, TITLEBAR_CLUSTER_PAD),
            0.0
        );
    }
}
