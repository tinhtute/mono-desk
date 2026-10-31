//! Background adjustment dialog: zeron's pan/zoom framing editor over the
//! installed new-thread background image.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, Copyright
//! 2026 Wing, pinned rev 8b7afb3) :2747-3265 (the modal render), :61-77 (the
//! two drag payloads, `BACKGROUND_ZOOM_STEP`), :100-112 (`background_zoom_at_x`
//! and `background_preview_size`), :1212-1260 (open/dismiss/apply) and
//! :1262-1360 (reset, zoom, pan and drag helpers).
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - The preview paints through the vendored
//!   [`zeron_ui::new_thread_background_mask`] geometry with the vendored
//!   effect raster from [`zeron_ui::new_thread_background_effects`]; the
//!   app's persisted `NewThreadBackgroundAdjustment` converts to/from the
//!   vendored `Framing` at this boundary (the vendored crate cannot depend on
//!   the app's settings model).
//! - The preview well is a fixed 260px-tall crop of the dialog card (no chat
//!   hero exists in mono-desk to derive zeron's `background_preview_size` from,
//!   plan NG4); width follows the card's inner measure.
//! - Apply writes through `settings::background::set_adjustment` (T4), which
//!   persists and repaints; zeron called its settings service and toast.
//! - Strings are i18n keys under `appearance.background.adjust.*`; the
//!   appearance page owner adds the dialog copy (phase 4 T3, finance-ui).
//! - The drag payloads keep the skeleton's names (`BackgroundDrag`,
//!   `BackgroundZoomDrag`) as the Lead's host wiring expects.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    AnyElement, Bounds, Context, Corners, DragMoveEvent, FocusHandle, IntoElement, KeyDownEvent,
    MouseDownEvent, MouseUpEvent, PinchEvent, Pixels, Point, RenderImage, Role, ScrollDelta,
    ScrollWheelEvent, SharedString, TouchPhase, Window, div, point, px, size,
};
use zeron_ui::icons;
use zeron_ui::new_thread_background_effects::{self, BackgroundEffect};
use zeron_ui::new_thread_background_mask::{self as mask, Framing};
use zeron_ui::popover;
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::{AppSettings, NewThreadBackgroundAdjustment, NewThreadBackgroundEffect};
use crate::i18n::tr;

use super::super::AppearancePage;

/// zeron's zoom step for the slider, the +/- actions and the keyboard
/// (`BACKGROUND_ZOOM_STEP`, upstream :78).
const BACKGROUND_ZOOM_STEP: f32 = 0.1;

/// zeron's zero-sized pan drag (`appearance.rs:61-68`); the host's
/// `on_drag_move<BackgroundDrag>` feeds pointer deltas back here.
#[derive(Clone)]
pub(crate) struct BackgroundDrag;

impl Render for BackgroundDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// zeron's zero-sized zoom drag (`appearance.rs:70-77`).
#[derive(Clone)]
pub(crate) struct BackgroundZoomDrag;

impl Render for BackgroundZoomDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// zeron's dialog view state (`BackgroundAdjustmentDialog`, upstream :44-59):
/// focus ring handles for the modal and slider, the image path a draft belongs
/// to, the prepared preview raster, the measured preview/zoom bounds and the
/// in-flight drag/pinch.
pub(crate) struct Dialog {
    focus: FocusHandle,
    zoom_focus: FocusHandle,
    end_focus: FocusHandle,
    return_focus: Option<FocusHandle>,
    path: PathBuf,
    source: Option<Arc<RenderImage>>,
    draft: NewThreadBackgroundAdjustment,
    preview_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    zoom_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    drag: Option<(Point<Pixels>, NewThreadBackgroundAdjustment)>,
    pinch: Option<(f32, f32)>,
}

/// The modal slot the page owns (upstream kept the dialog directly on the
/// page; the phase split gives it its own state).
#[derive(Default)]
pub(crate) struct State {
    dialog: Option<Dialog>,
}

impl State {
    /// zeron's `open_background_adjustment` (upstream :1212-1231): seed the
    /// draft from the installed background and take focus, remembering the
    /// focus to restore on close.
    pub(crate) fn open(&mut self, window: &mut Window, cx: &mut Context<AppearancePage>) {
        let Some(background) = AppSettings::global(cx).new_thread_background.clone() else {
            return;
        };
        let focus = cx.focus_handle();
        let zoom_focus = cx.focus_handle();
        let return_focus = window.focused(cx);
        window.focus(&focus, cx);
        self.dialog = Some(Dialog {
            focus,
            zoom_focus,
            end_focus: cx.focus_handle(),
            return_focus,
            path: background.path,
            source: None,
            draft: background.adjustment.normalized(),
            preview_bounds: Rc::default(),
            zoom_bounds: Rc::default(),
            drag: None,
            pinch: None,
        });
    }
}

/// Render the modal when open (upstream :2747-3265).
pub(super) fn render(
    page: &mut AppearancePage,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    let viewport = window.viewport_size();
    let lang = AppSettings::global(cx).language.clone();
    let (focus, zoom_focus, end_focus, path, draft, preview_bounds, zoom_bounds, dragging) = {
        let dialog = page.dialogs.background.dialog.as_ref()?;
        (
            dialog.focus.clone(),
            dialog.zoom_focus.clone(),
            dialog.end_focus.clone(),
            dialog.path.clone(),
            dialog.draft,
            dialog.preview_bounds.clone(),
            dialog.zoom_bounds.clone(),
            dialog.drag.is_some(),
        )
    };
    let tab_start = focus.clone();
    let tab_end = end_focus.clone();
    let (same_image, available, effect) = {
        let settings = AppSettings::global(cx);
        (
            settings
                .new_thread_background
                .as_ref()
                .is_some_and(|background| background.path == path),
            path.is_file(),
            settings.new_thread_background_effect,
        )
    };
    let source = (same_image && available)
        .then(|| new_thread_background_effects::prepare(vendored_effect(effect), theme, &path, cx))
        .flatten();
    let ready = source.is_some();
    if let Some(dialog) = page.dialogs.background.dialog.as_mut() {
        dialog.source = source.clone();
    }
    let preview_size = preview_size(viewport);
    let card_width = (f32::from(viewport.width) - 40.0).clamp(1.0, 600.0);

    let paint_source = source.clone();
    let measured_bounds = preview_bounds.clone();
    let preview = div()
        .id("new-thread-background-adjustment-preview")
        .relative()
        .w(preview_size.width + px(2.0))
        .h(preview_size.height + px(2.0))
        .overflow_hidden()
        .rounded(px(12.0))
        .border_1()
        .border_color(zeron_ui::theme::hairline(0.12))
        .bg(theme.bg)
        .cursor(if !ready {
            gpui::CursorStyle::Arrow
        } else if dragging {
            gpui::CursorStyle::ClosedHand
        } else {
            gpui::CursorStyle::OpenHand
        })
        .on_scroll_wheel(cx.listener(|page, event: &ScrollWheelEvent, window, cx| {
            let delta = match event.delta {
                ScrollDelta::Pixels(delta) => f32::from(delta.y),
                ScrollDelta::Lines(delta) => delta.y * 40.0,
            };
            let Some(zoom) = page.background_draft().map(|draft| draft.zoom) else {
                return;
            };
            page.zoom_background_around(
                zoom * (delta * 0.0025).clamp(-2.0, 2.0).exp(),
                Some(event.position),
                cx,
            );
            cx.stop_propagation();
            window.prevent_default();
        }))
        .on_pinch(cx.listener(|page, event: &PinchEvent, window, cx| {
            if event.phase == TouchPhase::Ended {
                page.set_background_pinch(None);
                return;
            }
            if event.phase == TouchPhase::Started {
                let zoom = page.background_draft().map_or(1.0, |draft| draft.zoom);
                page.set_background_pinch(Some((zoom, 1.0)));
            }
            let Some((start, factor)) = page.background_pinch() else {
                return;
            };
            let factor = background_pinch_factor(factor, event.delta, cfg!(target_os = "windows"));
            page.set_background_pinch(Some((start, factor)));
            page.zoom_background_around(start * factor, Some(event.position), cx);
            cx.stop_propagation();
            window.prevent_default();
        }))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|page, event: &MouseDownEvent, window, cx| {
                page.begin_background_drag(event, window, cx);
            }),
        )
        .on_drag(BackgroundDrag, |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .child(
            gpui::canvas(
                move |bounds, _, _| measured_bounds.set(Some(bounds)),
                move |bounds, _, window, _| {
                    if let Some(source) = paint_source.clone() {
                        mask::paint_adjusted(
                            source,
                            bounds,
                            framing(draft),
                            // Canvas content masks are rectangular; round the image
                            // itself to the frame's 12px radius inside its 1px border.
                            Corners::all(px(11.0)),
                            window,
                        );
                    }
                },
            )
            .w(preview_size.width)
            .h(preview_size.height),
        )
        .when(source.is_none(), |preview| {
            preview.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(zeron_ui::typography::ui_rems(11.5))
                    .text_color(theme.text_muted)
                    .child(if !same_image {
                        tr("appearance.background.adjust.changed", &lang)
                    } else if !available {
                        tr("appearance.background.unavailable.desc", &lang)
                    } else {
                        tr("appearance.background.adjust.preparing", &lang)
                    }),
            )
        })
        .when(dragging, |preview| {
            preview
                .child(
                    div()
                        .absolute()
                        .left(gpui::relative(1.0 / 3.0))
                        .top_0()
                        .bottom_0()
                        .w(px(1.0))
                        .bg(gpui::white().opacity(0.22)),
                )
                .child(
                    div()
                        .absolute()
                        .left(gpui::relative(2.0 / 3.0))
                        .top_0()
                        .bottom_0()
                        .w(px(1.0))
                        .bg(gpui::white().opacity(0.22)),
                )
                .child(
                    div()
                        .absolute()
                        .top(gpui::relative(1.0 / 3.0))
                        .left_0()
                        .right_0()
                        .h(px(1.0))
                        .bg(gpui::white().opacity(0.22)),
                )
                .child(
                    div()
                        .absolute()
                        .top(gpui::relative(2.0 / 3.0))
                        .left_0()
                        .right_0()
                        .h(px(1.0))
                        .bg(gpui::white().opacity(0.22)),
                )
        });

    let zoom_fraction = background_zoom_fraction(draft.zoom);
    let measured_zoom_bounds = zoom_bounds.clone();
    let zoom_slider = div()
        .id("new-thread-background-adjustment-zoom")
        .track_focus(&zoom_focus)
        .tab_index(0)
        .role(Role::Slider)
        .aria_label(tr("appearance.background.adjust.zoom", &lang))
        .aria_numeric_value(f64::from(draft.zoom * 100.0))
        .aria_min_numeric_value(f64::from(NewThreadBackgroundAdjustment::MIN_ZOOM * 100.0))
        .aria_max_numeric_value(f64::from(NewThreadBackgroundAdjustment::MAX_ZOOM * 100.0))
        .aria_numeric_value_step(f64::from(BACKGROUND_ZOOM_STEP * 100.0))
        .aria_value(format!("{:.0}%", draft.zoom * 100.0))
        .focus_visible(|style| style.border_2().border_color(theme.accent))
        .relative()
        .w(px(168.0))
        .h(px(28.0))
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|page, event: &MouseDownEvent, window, cx| {
                page.begin_zoom_drag(event, window, cx);
            }),
        )
        .on_drag(BackgroundZoomDrag, |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_key_down(cx.listener(|page, event: &KeyDownEvent, window, cx| {
            let Some(current) = page.background_draft().map(|draft| draft.zoom) else {
                return;
            };
            let next = match event.keystroke.key.as_str() {
                "left" | "down" => current - BACKGROUND_ZOOM_STEP,
                "right" | "up" => current + BACKGROUND_ZOOM_STEP,
                "home" => NewThreadBackgroundAdjustment::MIN_ZOOM,
                "end" => NewThreadBackgroundAdjustment::MAX_ZOOM,
                _ => return,
            };
            page.set_background_zoom(next, cx);
            cx.stop_propagation();
            window.prevent_default();
        }))
        .child(
            gpui::canvas(
                move |bounds, _, _| measured_zoom_bounds.set(Some(bounds)),
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .child(
            div()
                .absolute()
                .left(px(7.0))
                .right(px(7.0))
                .top(px(12.0))
                .h(px(4.0))
                .rounded_full()
                .bg(theme.border)
                .child(
                    div()
                        .h_full()
                        .w(gpui::relative(zoom_fraction))
                        .rounded_full()
                        .bg(theme.accent),
                )
                .child(
                    div()
                        .absolute()
                        .left(gpui::relative(zoom_fraction))
                        .ml(px(-7.0))
                        .top(px(-5.0))
                        .size(px(14.0))
                        .rounded_full()
                        .bg(theme.accent),
                ),
        );

    let controls = div()
        .mt(px(14.0))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            compact_action(
                theme,
                tr("appearance.background.adjust.reset", &lang),
                "new-thread-background-adjustment-reset",
            )
            .on_click(cx.listener(|page, _, _, cx| page.reset_background_dialog(cx))),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    compact_action(theme, "−", "new-thread-background-adjustment-zoom-out")
                        .aria_label(tr("appearance.background.adjust.zoom_out", &lang))
                        .tooltip(widgets::text_tooltip(tr(
                            "appearance.background.adjust.zoom_out",
                            &lang,
                        )))
                        .on_click(cx.listener(|page, _, _, cx| {
                            let Some(zoom) = page.background_draft().map(|draft| draft.zoom) else {
                                return;
                            };
                            page.set_background_zoom(zoom - BACKGROUND_ZOOM_STEP, cx);
                        })),
                )
                .child(zoom_slider)
                .child(
                    div()
                        .w(px(52.0))
                        .text_center()
                        .text_size(zeron_ui::typography::ui_rems(11.5))
                        .text_color(theme.text_muted)
                        .child(format!("{:.0}%", draft.zoom * 100.0)),
                )
                .child(
                    compact_action(theme, "+", "new-thread-background-adjustment-zoom-in")
                        .aria_label(tr("appearance.background.adjust.zoom_in", &lang))
                        .tooltip(widgets::text_tooltip(tr(
                            "appearance.background.adjust.zoom_in",
                            &lang,
                        )))
                        .on_click(cx.listener(|page, _, _, cx| {
                            let Some(zoom) = page.background_draft().map(|draft| draft.zoom) else {
                                return;
                            };
                            page.set_background_zoom(zoom + BACKGROUND_ZOOM_STEP, cx);
                        })),
                ),
        );

    let footer = div()
        .mt(px(18.0))
        .pt(px(12.0))
        .border_t_1()
        .border_color(theme.border)
        .flex()
        .items_center()
        .justify_end()
        .gap(px(8.0))
        .child(
            compact_action(
                theme,
                tr("appearance.background.adjust.cancel", &lang),
                "new-thread-background-adjustment-cancel",
            )
            .h(px(34.0))
            .px(px(13.0))
            .on_click(
                cx.listener(|page, _, window, cx| page.dismiss_background_dialog(window, cx)),
            ),
        )
        .child(
            widgets::text_action(
                theme,
                widgets::ActionTone::Solid,
                tr("appearance.background.adjust.apply", &lang),
            )
            .id("new-thread-background-adjustment-apply")
            .h(px(34.0))
            .px(px(14.0))
            .py(px(0.0))
            .flex()
            .items_center()
            .when(!ready, |button| button.opacity(0.45))
            .when(ready, |button| {
                button
                    .tab_index(0)
                    .role(Role::Button)
                    .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                    .on_click(
                        cx.listener(|page, _, window, cx| page.apply_background_dialog(window, cx)),
                    )
            }),
        );

    let header = div()
        .flex()
        .items_start()
        .gap(px(16.0))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(popover::dialog_title(
                    theme,
                    tr("appearance.background.adjust.title", &lang),
                ))
                .child(
                    popover::dialog_body(theme, tr("appearance.background.adjust.body", &lang))
                        .mt(px(4.0)),
                ),
        )
        .child(
            div()
                .id("new-thread-background-adjustment-close")
                .size(px(28.0))
                .rounded(px(7.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.surface_raised.opacity(0.28))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|style| style.bg(theme.surface_raised_hover))
                .tab_index(0)
                .role(Role::Button)
                .aria_label(tr("appearance.background.adjust.close", &lang))
                .tooltip(widgets::text_tooltip(tr(
                    "appearance.background.adjust.close",
                    &lang,
                )))
                .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                .on_click(
                    cx.listener(|page, _, window, cx| page.dismiss_background_dialog(window, cx)),
                )
                .child(
                    icons::icon(icons::CLOSE)
                        .size(px(12.0))
                        .text_color(theme.text_muted),
                ),
        );

    let card = popover::dialog_card(theme)
        .id("new-thread-background-adjustment-card")
        .w(px(card_width))
        .p(px(20.0))
        .track_focus(&focus)
        .tab_group()
        .tab_index(0)
        .tab_stop(false)
        .on_key_down(cx.listener(move |page, event: &KeyDownEvent, window, cx| {
            let step = if event.keystroke.modifiers.shift {
                24.0
            } else {
                8.0
            };
            match popover::classify_key(
                event.keystroke.key.as_str(),
                event.keystroke.modifiers.platform,
                event.keystroke.modifiers.control,
            ) {
                popover::MenuKey::Escape => page.dismiss_background_dialog(window, cx),
                popover::MenuKey::ModEnter => page.apply_background_dialog(window, cx),
                _ => match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev(cx);
                        } else {
                            window.focus_next(cx);
                        }
                        if !tab_start.contains_focused(window, cx) {
                            if event.keystroke.modifiers.shift {
                                window.focus(&tab_end, cx);
                                window.focus_prev(cx);
                            } else {
                                window.focus(&tab_start, cx);
                                window.focus_next(cx);
                            }
                        }
                    }
                    "left" => page.pan_background(point(step, 0.0), cx),
                    "right" => page.pan_background(point(-step, 0.0), cx),
                    "up" => page.pan_background(point(0.0, step), cx),
                    "down" => page.pan_background(point(0.0, -step), cx),
                    "+" | "=" => {
                        let zoom = page
                            .background_draft()
                            .map_or(1.0, |draft| draft.zoom + BACKGROUND_ZOOM_STEP);
                        page.set_background_zoom(zoom, cx);
                    }
                    "-" => {
                        let zoom = page
                            .background_draft()
                            .map_or(1.0, |draft| draft.zoom - BACKGROUND_ZOOM_STEP);
                        page.set_background_zoom(zoom, cx);
                    }
                    _ => return,
                },
            }
            cx.stop_propagation();
            window.prevent_default();
        }))
        .on_mouse_down_out(
            cx.listener(|page, _, window, cx| page.dismiss_background_dialog(window, cx)),
        )
        .child(header)
        .child(div().h(px(16.0)))
        .child(
            div()
                .w_full()
                .h(px(260.0))
                .flex()
                .items_center()
                .justify_center()
                .child(preview),
        )
        .child(controls)
        .child(footer)
        .child(div().track_focus(&end_focus).tab_index(0).tab_stop(false))
        .into_any_element();

    Some(popover::modal(
        "new-thread-background-adjustment-dialog",
        viewport,
        card,
    ))
}

/// Host `on_drag_move<BackgroundDrag>` (upstream :4192-4196).
pub(super) fn drag_pan(
    page: &mut AppearancePage,
    event: &DragMoveEvent<BackgroundDrag>,
    cx: &mut Context<AppearancePage>,
) {
    page.drag_background(event.event.position, cx);
}

/// Host `on_drag_move<BackgroundZoomDrag>` (upstream :4197-4201).
pub(super) fn drag_zoom(
    page: &mut AppearancePage,
    event: &DragMoveEvent<BackgroundZoomDrag>,
    cx: &mut Context<AppearancePage>,
) {
    page.set_background_zoom_from_x(event.event.position.x, cx);
}

/// Host mouse-up flush of an in-flight pan (upstream :4202-4223).
pub(super) fn mouse_up(
    page: &mut AppearancePage,
    _event: &MouseUpEvent,
    cx: &mut Context<AppearancePage>,
) {
    page.finish_background_drag(cx);
}

/// zeron's `compact_action` (`appearance.rs:1707-1714`).
fn compact_action(
    theme: &Theme,
    label: impl Into<SharedString>,
    id: &'static str,
) -> gpui::Stateful<gpui::Div> {
    widgets::text_action(theme, widgets::ActionTone::Filled, label)
        .id(id)
        .tab_index(0)
        .role(Role::Button)
        .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
}

/// The preview well's size: the card's inner measure, 260px tall (see the
/// module's adaptation note; upstream derived this from the chat hero).
fn preview_size(viewport: gpui::Size<Pixels>) -> gpui::Size<Pixels> {
    let card_width = (f32::from(viewport.width) - 40.0).clamp(1.0, 600.0);
    let width = (card_width - 40.0).clamp(1.0, 556.0);
    size(px(width), px(260.0))
}

/// zeron's `background_pinch_factor` (upstream :79-91).
fn background_pinch_factor(factor: f32, delta: f32, relative_delta: bool) -> f32 {
    if !delta.is_finite() {
        return factor;
    }
    // Direct Manipulation reports a ratio to the last scale; the other
    // native backends report an additive change to the gesture's scale.
    let next = if relative_delta {
        factor * (1.0 + delta)
    } else {
        factor + delta
    };
    next.max(0.001)
}

/// zeron's `background_zoom_fraction` (upstream :93-97).
fn background_zoom_fraction(zoom: f32) -> f32 {
    let minimum = NewThreadBackgroundAdjustment::MIN_ZOOM;
    let maximum = NewThreadBackgroundAdjustment::MAX_ZOOM;
    ((zoom - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
}

/// zeron's `background_zoom_at_x` (upstream :100-107): the slider's track maps
/// its padded width onto the zoom range.
fn background_zoom_at_x(bounds: Bounds<Pixels>, x: Pixels) -> f32 {
    let minimum = NewThreadBackgroundAdjustment::MIN_ZOOM;
    let maximum = NewThreadBackgroundAdjustment::MAX_ZOOM;
    let fraction = ((f32::from(x - bounds.left()) - 7.0)
        / (f32::from(bounds.size.width) - 14.0).max(1.0))
    .clamp(0.0, 1.0);
    minimum + fraction * (maximum - minimum)
}

/// The vendored framing type from the app's persisted adjustment.
fn framing(adjustment: NewThreadBackgroundAdjustment) -> Framing {
    Framing {
        focal_x: adjustment.focal_x,
        focal_y: adjustment.focal_y,
        zoom: adjustment.zoom,
    }
}

/// The app's persisted adjustment from the vendored framing (re-clamped).
fn adjustment(framing: Framing) -> NewThreadBackgroundAdjustment {
    NewThreadBackgroundAdjustment {
        focal_x: framing.focal_x,
        focal_y: framing.focal_y,
        zoom: framing.zoom,
    }
    .normalized()
}

/// The vendored effect enum from the app's persisted effect.
fn vendored_effect(effect: NewThreadBackgroundEffect) -> BackgroundEffect {
    match effect {
        NewThreadBackgroundEffect::None => BackgroundEffect::None,
        NewThreadBackgroundEffect::Dither => BackgroundEffect::Dither,
        NewThreadBackgroundEffect::Ascii => BackgroundEffect::Ascii,
        NewThreadBackgroundEffect::Halftone => BackgroundEffect::Halftone,
        NewThreadBackgroundEffect::Scanlines => BackgroundEffect::Scanlines,
    }
}

impl AppearancePage {
    /// The current dialog draft, when the dialog is open.
    fn background_draft(&self) -> Option<NewThreadBackgroundAdjustment> {
        self.dialogs
            .background
            .dialog
            .as_ref()
            .map(|dialog| dialog.draft)
    }

    fn background_pinch(&self) -> Option<(f32, f32)> {
        self.dialogs
            .background
            .dialog
            .as_ref()
            .and_then(|dialog| dialog.pinch)
    }

    fn set_background_pinch(&mut self, pinch: Option<(f32, f32)>) {
        if let Some(dialog) = self.dialogs.background.dialog.as_mut() {
            dialog.pinch = pinch;
        }
    }

    fn begin_background_drag(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(dialog) = self.dialogs.background.dialog.as_mut() else {
            return;
        };
        if dialog.source.is_none() {
            return;
        }
        window.focus(&dialog.focus, cx);
        dialog.drag = Some((event.position, dialog.draft));
        cx.notify();
        cx.stop_propagation();
    }

    fn begin_zoom_drag(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(dialog) = self.dialogs.background.dialog.as_ref() {
            window.focus(&dialog.zoom_focus, cx);
        }
        self.set_background_zoom_from_x(event.position.x, cx);
        cx.stop_propagation();
        window.prevent_default();
    }

    /// zeron's `reset_background_adjustment` (upstream :1262-1269).
    fn reset_background_dialog(&mut self, cx: &mut Context<Self>) {
        if let Some(dialog) = self.dialogs.background.dialog.as_mut() {
            dialog.draft = NewThreadBackgroundAdjustment::default();
            dialog.drag = None;
            dialog.pinch = None;
        }
        cx.notify();
    }

    /// zeron's `set_background_zoom` (upstream :1271-1273).
    fn set_background_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        self.zoom_background_around(zoom, None, cx);
    }

    /// zeron's `set_background_zoom_from_x` (upstream :1275-1284).
    fn set_background_zoom_from_x(&mut self, x: Pixels, cx: &mut Context<Self>) {
        let Some(bounds) = self
            .dialogs
            .background
            .dialog
            .as_ref()
            .and_then(|dialog| dialog.zoom_bounds.get())
        else {
            return;
        };
        self.set_background_zoom(background_zoom_at_x(bounds, x), cx);
    }

    /// zeron's `zoom_background_around` (upstream :1286-1322): preserve the
    /// source pixel under the pointer/gesture anchor.
    fn zoom_background_around(
        &mut self,
        zoom: f32,
        anchor: Option<Point<Pixels>>,
        cx: &mut Context<Self>,
    ) {
        let Some(dialog) = self.dialogs.background.dialog.as_mut() else {
            return;
        };
        if dialog.source.is_none() {
            return;
        }
        let zoom = zoom.clamp(
            NewThreadBackgroundAdjustment::MIN_ZOOM,
            NewThreadBackgroundAdjustment::MAX_ZOOM,
        );
        let source = dialog.source.clone();
        let next = match (dialog.preview_bounds.get(), source) {
            (Some(bounds), Some(source)) => adjustment(mask::zoom_adjustment_around(
                &source,
                bounds,
                framing(dialog.draft),
                zoom,
                anchor.unwrap_or_else(|| bounds.center()),
            )),
            _ => NewThreadBackgroundAdjustment {
                zoom,
                ..dialog.draft
            }
            .normalized(),
        };
        if dialog.draft != next {
            dialog.draft = next;
            dialog.drag = None;
            cx.notify();
        }
    }

    /// zeron's `pan_background` (upstream :1324-1341).
    fn pan_background(&mut self, delta: Point<f32>, cx: &mut Context<Self>) {
        let Some(dialog) = self.dialogs.background.dialog.as_mut() else {
            return;
        };
        let source = dialog.source.clone();
        let (Some(bounds), Some(source)) = (dialog.preview_bounds.get(), source) else {
            return;
        };
        let before = dialog.draft;
        dialog.draft = adjustment(mask::pan_adjustment(
            &source,
            bounds,
            framing(dialog.draft),
            point(px(delta.x), px(delta.y)),
        ));
        if dialog.draft != before {
            cx.notify();
        }
    }

    /// zeron's `drag_background` (upstream :1343-1358): replay from the press
    /// position so a direct-manipulation drag stays absolute.
    fn drag_background(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(dialog) = self.dialogs.background.dialog.as_ref() else {
            return;
        };
        let Some((start, initial)) = dialog.drag else {
            return;
        };
        let delta = point(
            f32::from(position.x - start.x),
            f32::from(position.y - start.y),
        );
        if let Some(dialog) = self.dialogs.background.dialog.as_mut() {
            dialog.draft = initial;
        }
        self.pan_background(delta, cx);
    }

    /// zeron's `finish_background_drag` (upstream :1360-1366).
    fn finish_background_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(dialog) = self.dialogs.background.dialog.as_mut()
            && dialog.drag.take().is_some()
        {
            cx.notify();
        }
    }

    /// zeron's `dismiss_background_adjustment` (upstream :1233-1244): close
    /// and restore the focus that opened the dialog.
    fn dismiss_background_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(dialog) = self.dialogs.background.dialog.take()
            && let Some(focus) = dialog.return_focus
        {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    /// zeron's `apply_background_adjustment` (upstream :1246-1260): a shuffle or
    /// replacement must not receive a draft for another image.
    fn apply_background_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dialog) = self.dialogs.background.dialog.as_ref() else {
            return;
        };
        if dialog.source.is_none()
            || AppSettings::global(cx)
                .new_thread_background
                .as_ref()
                .is_none_or(|background| background.path != dialog.path)
        {
            return;
        }
        crate::settings::background::set_adjustment(dialog.draft, cx);
        self.dismiss_background_dialog(window, cx);
    }
}
