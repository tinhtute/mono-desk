//! The app shell: zeron's Settings surface and its window chrome.
//!
//! Ported from zeron's `crates/ui/src/shell.rs` (MIT, Copyright 2026 Wing).
//! The app has a single surface — zeron's *Settings* route — so this is that
//! route's shell, not the chat shell: the root paints the theme's glass tint
//! over the blurred window background (`Shell::render`'s `div().bg(frost)`
//! where `frost = theme.glass()`), the sidebar column stands on zeron's
//! `sidebar_tone` layer (`wash(0.05)` + 1px right hairline), and a 38px
//! overlay strip drags the window (`titlebar_drag_region`) without painting a
//! control cluster — zeron's settings route has none. The chat-only chrome
//! (cluster at 88px, sidebar resize/collapse, centered title) is deliberately
//! not ported; the plan's R2 records that decision.
//!
//! Landed by phase `chrome`: the shell surface, the drag strip, the sidebar
//! tone/column, the page's titlebar clearance, the section list (`render_nav`,
//! zeron `render_settings_nav`) and the footer identity row (`render_footer`,
//! zeron's `render_sidebar_footer_row` / user-menu trigger). Phase
//! `settings-surface` added the settings search field (`render_search`, zeron
//! `shell/settings_search.rs` layout), the section filter and the page pane.

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
/// resizable sidebar (the settings route has no drag handle), so the column is
/// fixed at the default. Bounds for reference: `SIDEBAR_MIN` 242,
/// `SIDEBAR_MAX` 400.
pub const SIDEBAR_WIDTH: f32 = 256.0;
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

    actions!(shell_actions, [FocusSearch]);
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
    /// Armed by a left press on the titlebar strip: the next pointer move with
    /// the button down hands the drag to the compositor (zeron's
    /// `titlebar_should_move`).
    titlebar_should_move: bool,
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
        Self {
            settings_search: Some(settings_search),
            section: section_from_env().unwrap_or_default(),
            nav_focus: SettingsSection::ALL
                .iter()
                .map(|_| cx.focus_handle())
                .collect(),
            titlebar_should_move: false,
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
    /// same field).
    fn focus_search(
        &mut self,
        _: &shell_actions::FocusSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(input) = self.settings_search.clone() {
            let focus = input.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
        }
        cx.stop_propagation();
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

    /// The settings sidebar column (zeron `render_settings_page`): full height
    /// at the fixed width, content starting below the titlebar strip, the
    /// section list filling the column and the footer pinned at its bottom.
    fn render_sidebar(
        &mut self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .pt(px(Theme::TITLEBAR_HEIGHT))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_nav(theme, window, cx)),
            )
            .child(self.render_footer(theme, cx))
            .into_any_element()
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
        // zeron publishes the sidebar width every frame so a page's dropdown
        // can measure the content pane (`settings::set_sidebar_width`).
        menu::set_sidebar_width(SIDEBAR_WIDTH, cx);
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
        let sidebar = self.render_sidebar(&theme, window, cx);
        let page = self.render_page(cx);
        // zeron `Shell::render`'s root: `id("shell-root")`, row layout,
        // `bg(theme.glass())`, theme text colour and the 14px rem scale.
        div()
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
            .child(Self::sidebar_tone(SIDEBAR_WIDTH, theme.border))
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
                    .child(drag),
            )
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
