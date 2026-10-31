//! The Appearance page: zeron's own custom Appearance page
//! (`crates/ui/src/settings/appearance.rs`, MIT, rev 8b7afb3) rebuilt on the
//! vendored kit.
//!
//! The page is split one module per zeron section: [`color`] (mode cards,
//! Light/Dark registry-variant selectors with palette previews, the accent row,
//! Match wallpaper colors), [`material`] (Glass, New-thread background image +
//! actions/effect, Wallpaper folder), [`motion`] (Reduce motion, Pause
//! animations in background, Smooth scrolling), [`library`] (the theme-library
//! card + warning line), [`fonts`] (Interface/Terminal/Code pickers and sizes,
//! then [`layout`]'s Conversation width + Render math + Math font/size rows),
//! [`preview`] (shared miniatures) and [`dialogs`] (the single modal stack:
//! background adjustment → theme import → mapping review). This module owns the
//! scroll host, the assembly order and the shared page state; each section
//! module owns its `State` and returns `section(...) -> AnyElement`.
//!
//! Upstream line map: page host + scroller assembly :4185-4289, list-hover
//! scroll hook :1392-1396, `ScrollRailHost` impl :1399-1409, page state
//! :298-330, section order :4238-4281, modal priority :4089-4092,
//! `reduce_motion_helper`/`smooth_scrolling_helper` :1549-1566, motion rows
//! :3958-4083.
//!
//! Wiring differs from upstream on purpose: zeron's `appearance::set_*` helpers
//! write into zeron's settings service, which mono-desk does not vendor. Every
//! control writes the `AppSettings` global, persists it, and funnels through
//! [`crate::settings::apply_appearance`] — the single theme path also used by
//! the shell's footer toggle and the window's OS-appearance sync.

mod color;
mod dialogs;
mod font_model;
mod fonts;
mod layout;
mod library;
mod material;
mod motion;
mod preview;

use gpui::prelude::*;
use gpui::{
    Context, DragMoveEvent, IntoElement, MouseButton, MouseUpEvent, Render, ScrollHandle,
    SharedString, Window, div, px,
};

use zeron_ui::motion as zeron_motion;
use zeron_ui::motion::ReduceMotion;
use zeron_ui::popover;
use zeron_ui::settings::select::SelectState;
use zeron_ui::settings::widgets::{self, PageScroll};
use zeron_ui::theme::{Appearance as ThemeAppearance, Theme};

use crate::app_settings::{AppSettings, Appearance as ModeChoice};
use crate::i18n::tr;

/// The Appearance page view.
pub struct AppearancePage {
    /// The page's scroll container: zeron's owned `ScrollHandle` plus the
    /// floating-rail state, driven by `popover::rail` (upstream `scroll`,
    /// :306).
    scroll: PageScroll,
    /// Open state of the Light / Dark theme selectors (upstream
    /// `light_theme_select` / `dark_theme_select`, :322-323).
    light_theme: SelectState,
    dark_theme: SelectState,
    /// Open state of the Reduce motion select (upstream
    /// `reduce_motion_select`, :326).
    motion: SelectState,
    /// Per-section view state. Each section module owns its struct; the page
    /// only reserves the slot (upstream keeps the fields flat on
    /// `AppearancePage`, :298-330).
    material: material::State,
    library: library::State,
    fonts: fonts::State,
    layout: layout::State,
    dialogs: dialogs::State,
}

impl AppearancePage {
    pub fn new() -> Self {
        Self {
            scroll: PageScroll::default(),
            light_theme: SelectState::default(),
            dark_theme: SelectState::default(),
            motion: SelectState::default(),
            material: material::State::default(),
            library: library::State::default(),
            fonts: fonts::State::default(),
            layout: layout::State::default(),
            dialogs: dialogs::State::default(),
        }
    }

    /// zeron `appearance::set_mode` (upstream :3584-3586 calls it).
    fn set_mode(&mut self, mode: ModeChoice, window: &mut Window, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).appearance = mode;
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    /// zeron `appearance::set_accent` (upstream :3626-3629); the persisted
    /// shape stays the pre-port `#RRGGBB` accent hex (NG4).
    fn set_accent(
        &mut self,
        selection: zeron_theme::AccentSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let appearance = Theme::of(cx).appearance;
        let stored = super::accent_stored(selection, appearance);
        AppSettings::global_mut(cx).accent = stored;
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    /// zeron `appearance::set_theme` (upstream :2189-2193): persist the picked
    /// registry variant for its appearance, then reinstall the theme.
    fn set_theme_variant(
        &mut self,
        appearance: ThemeAppearance,
        variant_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        {
            let settings = AppSettings::global_mut(cx);
            if appearance.is_dark() {
                settings.theme_dark = variant_id.to_string();
            } else {
                settings.theme_light = variant_id.to_string();
            }
        }
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    /// "Match wallpaper colors" (upstream :3698-3700). The vendored
    /// [`crate::settings::wallpaper_colors::apply_enabled`] persists the toggle
    /// and samples the installed background on the background executor; it
    /// returns the task so this page can re-run its appearance funnel once the
    /// sampled color has landed (upstream called `appearance::apply(cx)`
    /// itself, which needs no `Window`).
    fn set_wallpaper_colors(&mut self, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        let sampled = super::wallpaper_colors::apply_enabled(enabled, cx);
        cx.spawn_in(window, async move |page, cx| {
            let _ = sampled.await;
            page.update_in(cx, |_page, window, cx| {
                super::apply_appearance(window, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// zeron `motion::set_preference` (upstream :3975-3978), persisted so the
    /// boot path restores it.
    fn set_reduce_motion(&mut self, preference: ReduceMotion, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).reduce_motion = preference;
        super::persist_now(cx);
        zeron_motion::set_preference(preference, cx);
        cx.notify();
    }

    /// zeron `motion::set_pause_in_background` (upstream :4038-4041), persisted.
    fn set_pause_in_background(&mut self, enabled: bool, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).pause_in_background = enabled;
        super::persist_now(cx);
        zeron_motion::set_pause_in_background(enabled, cx);
        cx.notify();
    }

    /// Smooth scrolling is a persisted preference only (NG2: mono-desk has no
    /// scroll animation runtime to drive).
    fn set_smooth_scrolling(&mut self, enabled: bool, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).smooth_scrolling = enabled;
        super::persist_now(cx);
        cx.notify();
    }

    /// The host's list-hover hook for the floating rail (upstream
    /// :1392-1396).
    fn on_scroll_hovered(&mut self, hovered: &bool, _: &mut Window, cx: &mut Context<Self>) {
        if self.scroll.set_list_hovered(*hovered) {
            cx.notify();
        }
    }
}

impl popover::ScrollRailHost for AppearancePage {
    // The page's rail; a font dropdown's rail goes through `widgets::rail`,
    // which can serve further scroll hosts on the same view.
    fn rail_bar(&mut self) -> &mut popover::MenuScrollbarState {
        self.scroll.rail_bar()
    }

    fn rail_scroll(&self) -> Option<ScrollHandle> {
        self.scroll.rail_scroll()
    }
}

impl Render for AppearancePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).for_settings_surface();
        let current_appearance = Theme::of(cx).appearance;
        let settings = AppSettings::global(cx);
        let lang = settings.language.clone();
        let mode = settings.appearance;
        let accent = super::accent_selection(settings.accent, current_appearance);
        let surface = super::surface_preference(settings.background_translucent);
        let themes = super::theme_selection(settings);
        let wallpaper_colors = settings.wallpaper_theme_colors;
        let smooth_scrolling = settings.smooth_scrolling;

        let color_state = color::ColorState {
            mode,
            accent,
            surface,
            themes,
            wallpaper_colors,
        };
        let motion_state = motion::MotionState {
            reduce: zeron_motion::preference(cx),
            system_reduces: zeron_motion::system_reduces_motion(cx),
            pause_in_background: zeron_motion::pause_in_background(cx),
            smooth_scrolling,
            reduced_motion: zeron_motion::reduced_motion(cx),
        };

        let cards = color::mode_cards(&theme, &color_state, &lang, window, cx);
        let color_rows = color::color_rows(self, &theme, &color_state, &lang, cx);
        let motion_rows = motion::motion_rows(self, &theme, &motion_state, &lang, cx);
        let scrollbar = popover::rail(self, "appearance-page-scrollbar", &theme, cx);

        // zeron's page host + edge-faded scroller (:4185-4285). Later phases
        // insert the Material, theme-library and Fonts sections between the
        // color card and the Motion section, in zeron's order.
        div()
            .id("appearance-page-host")
            .relative()
            .size_full()
            .on_hover(cx.listener(Self::on_scroll_hovered))
            .on_drag_move(cx.listener(
                |page, event: &DragMoveEvent<layout::TranscriptWidthDrag>, window, cx| {
                    layout::drag_width(page, event, window, cx);
                },
            ))
            .on_drag_move(cx.listener(
                |page, event: &DragMoveEvent<dialogs::BackgroundDrag>, _window, cx| {
                    dialogs::drag_pan(page, event, cx);
                },
            ))
            .on_drag_move(cx.listener(
                |page, event: &DragMoveEvent<dialogs::BackgroundZoomDrag>, _window, cx| {
                    dialogs::drag_zoom(page, event, cx);
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|page, event: &MouseUpEvent, window, cx| {
                    layout::mouse_up(page, event, window, cx);
                    dialogs::mouse_up(page, event, window, cx);
                }),
            )
            .child(
                zeron_ui::edge_fade::edge_faded(
                    16.0,
                    true,
                    true,
                    div()
                        .id("appearance-page")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll.scroll)
                        .child(
                            widgets::page_column()
                                .child(widgets::page_header(
                                    &theme,
                                    tr("page.appearance", &lang),
                                    None,
                                ))
                                .child(
                                    widgets::section(
                                        &theme,
                                        tr("appearance.scheme", &lang),
                                        // No `.block()` here: upstream's call is
                                        // `search_anchor(..).block()` on the search
                                        // anchor wrapper, and search anchors are not
                                        // ported — on the row itself `.block()` would
                                        // resolve to gpui `Styled::block` (display:
                                        // block), overriding the row's `flex_row` and
                                        // stacking the three mode cards.
                                        widgets::option_card_row().children(cards),
                                    )
                                    .mt(px(24.0))
                                    .gap(px(12.0)),
                                )
                                .child(
                                    widgets::section_card(&theme)
                                        .children(color_rows)
                                        .mt(px(16.0)),
                                )
                                .child(material::section(self, &theme, &lang, window, cx))
                                .child(widgets::section(
                                    &theme,
                                    tr("appearance.motion", &lang),
                                    widgets::section_card(&theme).mt_0().children(motion_rows),
                                ))
                                .child(library::section(self, &theme, &lang, window, cx))
                                .child(fonts::section(self, &theme, &lang, window, cx))
                                .children(library::warning(self, &theme, &lang, cx)),
                        ),
                )
                .fade_overflow_y(&self.scroll.scroll),
            )
            .children(scrollbar)
            .children(dialogs::render(self, &theme, window, cx))
    }
}
