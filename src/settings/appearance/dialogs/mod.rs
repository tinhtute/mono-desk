//! Modal stack for the Appearance page, in zeron's priority order (upstream
//! `crates/ui/src/settings/appearance.rs:4089-4092`): background adjustment >
//! theme import > theme mapping review. The page host delegates its drag-move
//! and mouse-up plumbing here.
//!
//! Split into one file per dialog so the phase-4 and phase-5 owners never
//! write the same file; this dispatcher is Lead-owned.

mod background;
mod library;

use gpui::{AnyElement, Context, MouseUpEvent, Window};
use zeron_ui::theme::Theme;

use super::AppearancePage;

pub(super) use background::{BackgroundDrag, BackgroundZoomDrag};

/// One state slot per modal, as upstream keeps them on the page.
#[derive(Default)]
pub(super) struct State {
    pub(super) background: background::State,
    pub(super) library: library::State,
}

/// At most one modal renders, in zeron's order.
pub(super) fn render(
    page: &mut AppearancePage,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    background::render(page, theme, window, cx).or_else(|| library::render(page, theme, window, cx))
}

/// Host `on_drag_move<BackgroundDrag>` (upstream :4192-4196).
pub(super) fn drag_pan(
    page: &mut AppearancePage,
    event: &gpui::DragMoveEvent<BackgroundDrag>,
    cx: &mut Context<AppearancePage>,
) {
    background::drag_pan(page, event, cx);
}

/// Host `on_drag_move<BackgroundZoomDrag>` (upstream :4197-4201).
pub(super) fn drag_zoom(
    page: &mut AppearancePage,
    event: &gpui::DragMoveEvent<BackgroundZoomDrag>,
    cx: &mut Context<AppearancePage>,
) {
    background::drag_zoom(page, event, cx);
}

/// Host `on_mouse_up`/`on_mouse_up_out` (upstream :4202-4223).
pub(super) fn mouse_up(
    page: &mut AppearancePage,
    event: &MouseUpEvent,
    _window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    background::mouse_up(page, event, cx);
}
