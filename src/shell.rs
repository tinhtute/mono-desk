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
//! Phase `chrome` T1 landed the shell surface, the drag strip, the sidebar
//! tone/column and the page's titlebar clearance; T2 adds the section list
//! (`render_nav`, zeron `render_settings_nav`) and the footer identity row
//! (`render_footer`, zeron's `render_sidebar_footer_row` / user-menu trigger).

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Div, FocusHandle, Hsla, MouseButton, Render, SharedString, Stateful,
    Window, WindowControlArea, div, px,
};

use zeron_ui::icons;
use zeron_ui::motion;
use zeron_ui::settings::{SettingsSection, menu, widgets};
use zeron_ui::theme::{self, Appearance, Theme};
use zeron_ui::typography;

use crate::app_settings::AppSettings;
use crate::settings::{GeneralPage, NotificationsPage};

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

pub struct Shell {
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
    notifications_page: Option<gpui::Entity<NotificationsPage>>,
}

impl Shell {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            section: section_from_env().unwrap_or_default(),
            nav_focus: SettingsSection::ALL
                .iter()
                .map(|_| cx.focus_handle())
                .collect(),
            titlebar_should_move: false,
            general_page: None,
            notifications_page: None,
        }
    }

    fn select_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        if self.section != section {
            self.section = section;
            cx.notify();
        }
    }

    /// Flip light/dark and re-point the native window background at the new
    /// surface (zeron re-installs the theme through the same
    /// `Theme::install_preferences` path; the window background follows from
    /// `Theme::window_background_appearance`). The Appearance page owns the
    /// persisted mode in `settings-surface`; this footer action is the quick
    /// toggle the demo window has always had.
    fn toggle_appearance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (appearance, accent) = {
            let theme = Theme::of(cx);
            (theme.appearance, theme.accent_color)
        };
        let next = match appearance {
            Appearance::Dark => Appearance::Light,
            Appearance::Light => Appearance::Dark,
        };
        Theme::install_preferences(next, accent, cx);
        window.set_background_appearance(Theme::of(cx).window_background_appearance());
        cx.refresh_windows();
        cx.notify();
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
    /// label + icon rows with one group break at the workspace rows. zeron's
    /// nav also carries the settings search row above it — that lands with
    /// `settings-surface` T4 (it needs a text-input primitive).
    fn render_nav(
        &mut self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let reduced_motion = motion::reduced_motion(cx);
        let selected = self.section;
        let mut rows = Vec::with_capacity(SettingsSection::ALL.len());
        for (index, item) in SettingsSection::ALL.into_iter().enumerate() {
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
            rows.push(
                widgets::section_tab(theme, is_selected, selection_t, key, hover_key)
                    .when(item.starts_nav_group(), |el| el.mt(px(Theme::SPACE_LG)))
                    .role(gpui::Role::Tab)
                    .aria_label(item.label())
                    .aria_selected(is_selected)
                    .track_focus(&focus)
                    .tab_index(0)
                    .tab_stop(is_selected)
                    .on_key_down(cx.listener(
                        move |this, event: &gpui::KeyDownEvent, window, cx| {
                            let items = SettingsSection::ALL;
                            let current = items
                                .iter()
                                .position(|section| *section == item)
                                .unwrap_or(0);
                            let next = match event.keystroke.key.as_str() {
                                "up" | "left" => (current + items.len() - 1) % items.len(),
                                "down" | "right" => (current + 1) % items.len(),
                                "home" => 0,
                                "end" => items.len() - 1,
                                _ => return,
                            };
                            this.select_section(items[next], cx);
                            let focus = this.nav_focus[next].clone();
                            window.focus(&focus, cx);
                            cx.stop_propagation();
                        },
                    ))
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| this.select_section(item, cx)))
                    .child(icons::icon(item.icon()).size(px(16.0)).text_color(tab_text))
                    .child(SharedString::from(item.label())),
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
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
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

    /// The page pane: the selected section's page view, or the placeholder for
    /// the sections whose pages land in the later `settings-surface` waves.
    /// zeron's pages own their scroll (`page_column` carries the titlebar
    /// clearance inside it), so the pane is only the host: `flex_1`, full
    /// height, no scroll of its own. Page views are created lazily and kept,
    /// like zeron's shell keeps one entity per settings page, so open select
    /// menus and scroll offsets survive a re-render.
    fn render_page(&mut self, theme: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let page: AnyElement = match self.section {
            SettingsSection::General => {
                if self.general_page.is_none() {
                    self.general_page = Some(cx.new(|_| GeneralPage::new()));
                }
                self.general_page
                    .as_ref()
                    .map(|page| page.clone().into_any_element())
                    .unwrap_or_else(|| self.render_placeholder(theme))
            }
            SettingsSection::Notifications => {
                if self.notifications_page.is_none() {
                    self.notifications_page = Some(cx.new(|_| NotificationsPage::new()));
                }
                self.notifications_page
                    .as_ref()
                    .map(|page| page.clone().into_any_element())
                    .unwrap_or_else(|| self.render_placeholder(theme))
            }
            _ => self.render_placeholder(theme),
        };
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(page)
            .into_any_element()
    }

    /// The page pane's placeholder for sections whose pages have not landed yet
    /// (Appearance is `settings-surface` T3; Profile, Finance UI, Reset and
    /// About are T2).
    fn render_placeholder(&self, theme: &Theme) -> AnyElement {
        div()
            .id("settings-page-scroll")
            .size_full()
            .overflow_y_scroll()
            .pt(px(Theme::TITLEBAR_HEIGHT + Theme::SPACE_LG))
            .px(px(Theme::SPACE_LG * 2.0))
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(SharedString::from(self.section.label())),
            )
            .child(
                div()
                    .mt_2()
                    .text_sm()
                    .text_color(theme.text_muted)
                    .child("page lands in the next `settings-surface` wave"),
            )
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
        let page = self.render_page(&theme, cx);
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
