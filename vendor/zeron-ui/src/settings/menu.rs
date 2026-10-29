//! zeron's settings dropdown plumbing, ported from
//! `crates/ui/src/settings/widgets.rs` (MIT, Copyright 2026 Wing, pinned rev
//! 8b7afb3) in the plan's `settings-surface` phase (R8).
//!
//! The section-column width published by the shell, the page-pane bounds and
//! width math, and the anchored dropdown mounted inside zeron's frosted menu
//! layer plus its scrolling option list. Ported verbatim from upstream's
//! `widgets.rs` lines 14-166 (upstream kept them in the same file as the page
//! scaffold, so the port splits them into this module).
//!
//! Adaptations from upstream:
//! - `SIDEBAR_DEFAULT` lives in zeron's `crates/ui/src/settings.rs` (that
//!   module is not vendored); it is defined below with upstream's value and
//!   cited doc comment. `sidebar_width` resolves it locally.
//! - `PAGE_MAX_WIDTH`/`PAGE_PAD_X` are private consts in upstream's
//!   `widgets.rs` (lines 314-315) next to `column_width`; because the split
//!   puts `column_width` here, the two values are restated below with
//!   upstream's values. The page scaffold in [`super::widgets`] keeps its own
//!   private copies of the same upstream constants.
//! - `SettingsDropdown` and its fields are `pub(crate)`: upstream constructs
//!   the struct from the same file as [`crate::settings::select`], our split
//!   makes it cross-module.
//! - imports are trimmed to what this file defines (upstream's `widgets.rs`
//!   header imports what its whole file needs).
//! - `dropdown_rows`'s edge fade uses the vendored [`crate::edge_fade`], which
//!   upstream also keeps as its own module.

use gpui::{AnyElement, Pixels, ScrollHandle, SharedString, div, prelude::*, px};

use crate::popover;
use crate::theme::Theme;

/// Sidebar default width (px). Upstream's `crates/ui/src/settings.rs` line 45:
/// `pub const SIDEBAR_DEFAULT: f32 = 256.0;` — zeron's settings column default,
/// which the ported chrome also paints the sidebar at.
pub const SIDEBAR_DEFAULT: f32 = 256.0;

/// The page column's outer max width and side padding, restated from upstream
/// `settings/widgets.rs` lines 314-315 (`const PAGE_MAX_WIDTH`/`PAGE_PAD_X`).
const PAGE_MAX_WIDTH: f32 = 760.0;
const PAGE_PAD_X: f32 = 40.0;

/// Width of the section column beside the settings page, published by the
/// shell each frame so dropdowns and responsive pages measure the page pane.
struct SettingsSidebarWidth(f32);

impl gpui::Global for SettingsSidebarWidth {}

pub fn set_sidebar_width(width: f32, cx: &mut gpui::App) {
    if cx.try_global::<SettingsSidebarWidth>().map(|w| w.0) != Some(width) {
        cx.set_global(SettingsSidebarWidth(width));
    }
}

fn sidebar_width(cx: &gpui::App) -> f32 {
    cx.try_global::<SettingsSidebarWidth>()
        .map_or(SIDEBAR_DEFAULT, |w| w.0)
}

/// The page pane: right of the section column, below the titlebar strip.
pub fn pane_bounds(viewport: gpui::Size<Pixels>, sidebar_width: f32) -> gpui::Bounds<Pixels> {
    let left = px(sidebar_width).min(viewport.width);
    let top = px(Theme::TITLEBAR_HEIGHT).min(viewport.height);
    gpui::Bounds::new(
        gpui::point(left, top),
        gpui::size(viewport.width - left, viewport.height - top),
    )
}

/// Inner width of the [`page_column`] at the current window size, for pages
/// that choose a layout by the room they get (the Devices grid).
pub fn column_width(window: &gpui::Window, cx: &gpui::App) -> f32 {
    let pane = f32::from(
        pane_bounds(window.viewport_size(), sidebar_width(cx))
            .size
            .width,
    );
    pane.min(PAGE_MAX_WIDTH) - 2.0 * PAGE_PAD_X
}

pub fn dropdown(
    id: impl Into<SharedString>,
    content: gpui::Div,
    closing: Option<std::time::Instant>,
    trigger_height: f32,
) -> AnyElement {
    SettingsDropdown {
        id: id.into(),
        content,
        closing,
        trigger_height,
        align: popover::MenuAlign::End,
    }
    .into_any_element()
}

#[derive(IntoElement)]
pub(crate) struct SettingsDropdown {
    pub(crate) id: SharedString,
    pub(crate) content: gpui::Div,
    pub(crate) closing: Option<std::time::Instant>,
    pub(crate) trigger_height: f32,
    pub(crate) align: popover::MenuAlign,
}

impl RenderOnce for SettingsDropdown {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let limits = dropdown_limits(window.viewport_size(), sidebar_width(cx));
        popover::contained_menu(
            self.id,
            self.content,
            self.closing,
            self.trigger_height,
            limits,
            self.align,
        )
    }
}

fn dropdown_limits(viewport: gpui::Size<Pixels>, sidebar_width: f32) -> gpui::Bounds<Pixels> {
    let pane = pane_bounds(viewport, sidebar_width);
    gpui::Bounds::new(
        pane.origin + gpui::point(px(8.0), px(8.0)),
        gpui::size(
            (pane.size.width - px(16.0)).max(px(1.0)),
            (pane.size.height - px(16.0)).max(px(1.0)),
        ),
    )
}

/// Scroll only a dropdown's options, keeping its frame and optional heading
/// fixed. The available height follows the same placement budget as the card.
pub fn dropdown_rows(
    id: impl Into<SharedString>,
    rows: impl IntoIterator<Item = AnyElement>,
    trigger_height: f32,
    chrome_height: f32,
) -> AnyElement {
    DropdownRows {
        id: id.into(),
        rows: rows.into_iter().collect(),
        trigger_height,
        chrome_height,
    }
    .into_any_element()
}

pub fn dropdown_list_height(
    viewport: gpui::Size<Pixels>,
    trigger_height: f32,
    chrome_height: f32,
) -> f32 {
    // Height alone matters here, and the pane's does not depend on the
    // section column's width.
    let limits = pane_bounds(viewport, 0.0);
    let card_height = ((f32::from(limits.size.height) - 16.0 - trigger_height) / 2.0 - 6.0)
        .max(1.0)
        .min(320.0);
    (card_height - chrome_height).max(1.0)
}

#[derive(IntoElement)]
struct DropdownRows {
    id: SharedString,
    rows: Vec<AnyElement>,
    trigger_height: f32,
    chrome_height: f32,
}

impl RenderOnce for DropdownRows {
    fn render(self, window: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        let key: SharedString = format!("{}-list-scroll", self.id).into();
        let scroll = window.with_global_id(key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<ScrollHandle>, _| {
                let scroll = previous.unwrap_or_default();
                (scroll.clone(), scroll)
            })
        });
        let list = div()
            .id(format!("{}-list", self.id))
            .debug_selector(|| "settings-dropdown-list".into())
            .max_h(px(dropdown_list_height(
                window.viewport_size(),
                self.trigger_height,
                self.chrome_height,
            )))
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .flex()
            .flex_col()
            .gap(px(2.0))
            .children(self.rows);
        crate::edge_fade::edge_faded(12.0, true, true, list).fade_overflow_y(&scroll)
    }
}
