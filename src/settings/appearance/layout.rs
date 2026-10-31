//! Conversation width + math rows (phase `fonts-layout`): the 240x28
//! transcript-width slider with its hover/keyboard details, Render math,
//! Math font and Math size, ported from zeron's
//! `crates/ui/src/settings/appearance.rs` :398-678 and :4161-4162.
//!
//! Upstream line map: `render_transcript_width` :515-678,
//! `render_math_rows` :398-513, `queue_width`/`apply_pending_width`/
//! `drag_width` :349-396, `TranscriptWidthDrag` :289-296, the width state
//! fields :299-330, host drag/mouse-up hooks :4185-4225, `format_px`
//! :1518-1524, `MATH_SIZES`/`nearest_math_size_ix` :267-280.
//!
//! Adaptations: zeron's `settings::set_transcript_width`/`set_render_math`/
//! `set_math_font`/`set_math_size` write zeron's settings service — mono-desk
//! has no such service, so every handler writes the `AppSettings` global and
//! persists through `crate::settings::persist_now` (the same funnel the other
//! Appearance rows use). Row strings come from `crate::i18n::tr`. The width
//! slider keeps zeron's per-frame coalescing: pointer moves only queue a
//! pending width, one `window.on_next_frame` publishes it, and the page
//! host's `on_mouse_up`/`on_mouse_up_out` flush guarantees the final value.
//! Widths stay a persisted preference only (NG3: no transcript exists to
//! apply them to).

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, Bounds, Context, DragMoveEvent, FocusHandle, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, Pixels, Render, Role, Toggled, Window, div, px, relative,
};

use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;
use zeron_ui::typography;

use crate::app_settings::{
    AppSettings, MATH_SIZE_DEFAULT, MathFont, TRANSCRIPT_WIDTH_DEFAULT, TRANSCRIPT_WIDTH_MAX,
    TRANSCRIPT_WIDTH_MIN, TRANSCRIPT_WIDTH_STEP, normalize_transcript_width,
};
use crate::i18n::tr;

use super::AppearancePage;
use super::font_model::{MATH_SIZES, nearest_math_size_ix};

/// The slider's track: 240x28 like zeron's, a 4px rail, a 14px knob, and the
/// 7px end inset that lets the knob's centre reach both extremes.
const SLIDER_WIDTH: f32 = 240.0;
const SLIDER_HEIGHT: f32 = 28.0;
const TRACK_HEIGHT: f32 = 4.0;
const KNOB_SIZE: f32 = 14.0;
const KNOB_INSET: f32 = 7.0;

/// zeron's zero-sized width drag (`crates/ui/src/settings/appearance.rs:289-296`).
#[derive(Clone)]
pub(super) struct TranscriptWidthDrag;

impl Render for TranscriptWidthDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// View state the layout rows own (`pending_width`/`width_frame_pending`,
/// the hover/press/keyboard flags and the width focus handle; upstream
/// `AppearancePage` fields :299-305) plus the two math selects (upstream
/// `math_font_select`/`math_size_select` :329-330).
#[derive(Default)]
pub(super) struct State {
    /// The width's focus handle, created lazily: the page constructor has no
    /// `cx` argument in the skeleton the page base phase landed.
    width_focus: Option<FocusHandle>,
    width_hovered: bool,
    width_pressed: bool,
    width_keyboard_active: bool,
    /// The slider's painted bounds, captured by the canvas in the track so
    /// pointer coordinates can be turned into a width.
    width_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// A width queued by this frame's pointer/keyboard events; published by
    /// [`apply_pending_width`] on the next frame or on mouse-up.
    pending_width: Option<f64>,
    width_frame_pending: bool,
    /// Open state of the Math font / Math size selects.
    math_font: SelectState,
    math_size: SelectState,
}

/// Upstream `format_px`'s number part (:1518-1524): whole values print
/// without a decimal, fractional values keep one.
fn format_px_number(size: f64) -> String {
    if size.fract().abs() < f64::EPSILON {
        format!("{size:.0}")
    } else {
        format!("{size:.1}")
    }
}

/// The live width's word: the number plus the localized pixel unit.
fn format_width(width: f64, lang: &str) -> String {
    format!("{} {}", format_px_number(width), tr("unit.px", lang))
}

/// The width a pointer at `x` inside the painted `bounds` picks (upstream
/// `drag_width`'s coordinate math :380-396): the knob's 7px inset on each end
/// is subtracted so its centre follows the pointer to both extremes.
fn width_from_x(bounds: Bounds<Pixels>, x: Pixels) -> f64 {
    let track = (f32::from(bounds.size.width) - KNOB_INSET * 2.0).max(1.0);
    let fraction = ((f32::from(x - bounds.left()) - KNOB_INSET) / track).clamp(0.0, 1.0);
    TRANSCRIPT_WIDTH_MIN + f64::from(fraction) * (TRANSCRIPT_WIDTH_MAX - TRANSCRIPT_WIDTH_MIN)
}

/// Enter/Space activate a focused action, like zeron's own interactive rows.
fn is_activation(event: &gpui::KeyDownEvent) -> bool {
    !event.is_held && matches!(event.keystroke.key.as_str(), "enter" | "space")
}

/// zeron `queue_width` (:349-371): normalize, keep the last pending value,
/// and coalesce pointer/keyboard events into one publish per frame.
fn queue_width(
    page: &mut AppearancePage,
    width: f64,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    let width = normalize_transcript_width(width);
    let current = page
        .layout
        .pending_width
        .unwrap_or_else(|| AppSettings::global(cx).transcript_width);
    if (current - width).abs() < f64::EPSILON {
        return;
    }
    page.layout.pending_width = Some(width);
    if !page.layout.width_frame_pending {
        page.layout.width_frame_pending = true;
        let page_handle = cx.weak_entity();
        window.on_next_frame(move |_, cx| {
            let _ = page_handle.update(cx, |page, cx| {
                page.layout.width_frame_pending = false;
                apply_pending_width(page, cx);
            });
        });
    }
    cx.notify();
}

/// zeron `apply_pending_width` (:373-378): publish the queued width into the
/// settings store. mono-desk persists immediately; zeron's store debounced.
fn apply_pending_width(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    if let Some(width) = page.layout.pending_width.take() {
        AppSettings::global_mut(cx).transcript_width = width;
        crate::settings::persist_now(cx);
        cx.notify();
    }
}

/// Width under a pointer x, normalized and queued (the `drag_width` body
/// upstream :380-396).
fn drag_width_at(
    page: &mut AppearancePage,
    x: Pixels,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    let Some(bounds) = page.layout.width_bounds.get() else {
        return;
    };
    queue_width(page, width_from_x(bounds, x), window, cx);
}

/// The width's focus handle, created on first paint.
fn width_focus(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) -> FocusHandle {
    if let Some(focus) = page.layout.width_focus.clone() {
        return focus;
    }
    let focus = cx.focus_handle();
    page.layout.width_focus = Some(focus.clone());
    focus
}

/// "Reset" in the width details: back to zeron's 736px default.
fn reset_width(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    page.layout.pending_width = None;
    AppSettings::global_mut(cx).transcript_width = TRANSCRIPT_WIDTH_DEFAULT;
    crate::settings::persist_now(cx);
    cx.notify();
}

/// "Render math" toggle handler.
fn set_render_math(value: bool, cx: &mut Context<AppearancePage>) {
    AppSettings::global_mut(cx).render_math = value;
    crate::settings::persist_now(cx);
    cx.notify();
}

/// The short style note beside a math face's name (upstream
/// `MathFont::style()`), localized so no English leaks into the other three
/// locales.
fn math_font_style(font: MathFont, lang: &str) -> &'static str {
    match font {
        MathFont::Fira => tr("appearance.layout.math_style.sans", lang),
        MathFont::StixTwo | MathFont::Libertinus | MathFont::Pagella => {
            tr("appearance.layout.math_style.serif", lang)
        }
        MathFont::ComputerModern => tr("appearance.layout.math_style.latex", lang),
        MathFont::Euler => tr("appearance.layout.math_style.upright", lang),
    }
}

/// The Conversation width row plus the three math rows, appended inside the
/// Fonts and layout card (upstream :4161-4163).
pub(super) fn section(
    page: &mut AppearancePage,
    theme: &Theme,
    lang: &str,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let settings = AppSettings::global(cx);
    let current_width = settings.transcript_width;
    let render_math = settings.render_math;
    let math_font = settings.math_font;
    let math_size = settings.math_size;

    // The slider reads the un-published pending value so its knob tracks the
    // pointer within the same frame the event arrived.
    let width = page.layout.pending_width.unwrap_or(current_width);
    let fraction =
        ((width - TRANSCRIPT_WIDTH_MIN) / (TRANSCRIPT_WIDTH_MAX - TRANSCRIPT_WIDTH_MIN)) as f32;
    let fraction = fraction.clamp(0.0, 1.0);

    let focus_handle = width_focus(page, cx);
    let show_details = page.layout.width_hovered
        || page.layout.width_pressed
        || (page.layout.width_keyboard_active && focus_handle.is_focused(window));

    let bounds = page.layout.width_bounds.clone();
    let accent = theme.accent;
    let border = theme.border;
    let muted = theme.text_muted;
    let text = theme.text;

    let width_title = tr("appearance.layout.conversation_width", lang);
    let width_value = format_width(width, lang);
    let reset_label = tr("action.reset", lang);
    let min_label = tr("appearance.layout.width_min", lang);
    let max_label = tr("appearance.layout.width_max", lang);

    // The 240x28 control (upstream :589-621): the canvas captures the track's
    // bounds for the drag math, the rail + knob are painted from `fraction`.
    let slider = div()
        .id("transcript-width-slider")
        .track_focus(&focus_handle)
        .tab_index(0)
        .role(Role::Slider)
        .aria_label(width_title)
        .aria_numeric_value(width)
        .aria_numeric_value_step(TRANSCRIPT_WIDTH_STEP)
        .aria_min_numeric_value(TRANSCRIPT_WIDTH_MIN)
        .aria_max_numeric_value(TRANSCRIPT_WIDTH_MAX)
        .focus_visible(move |style| style.border_2().border_color(accent))
        .relative()
        .w(px(SLIDER_WIDTH))
        .h(px(SLIDER_HEIGHT))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|page, event: &MouseDownEvent, window, cx| {
                let focus = width_focus(page, cx);
                window.focus(&focus, cx);
                page.layout.width_pressed = true;
                page.layout.width_keyboard_active = false;
                cx.notify();
                cx.stop_propagation();
                drag_width_at(page, event.position.x, window, cx);
            }),
        )
        .on_drag(TranscriptWidthDrag, |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_key_down(cx.listener(|page, event: &gpui::KeyDownEvent, window, cx| {
            let width = page
                .layout
                .pending_width
                .unwrap_or_else(|| AppSettings::global(cx).transcript_width);
            let next = match event.keystroke.key.as_str() {
                "left" | "down" => width - TRANSCRIPT_WIDTH_STEP,
                "right" | "up" => width + TRANSCRIPT_WIDTH_STEP,
                "home" => TRANSCRIPT_WIDTH_MIN,
                "end" => TRANSCRIPT_WIDTH_MAX,
                _ => return,
            };
            page.layout.width_keyboard_active = true;
            cx.notify();
            cx.stop_propagation();
            window.prevent_default();
            queue_width(page, next, window, cx);
        }))
        .child(
            gpui::canvas(move |rect, _, _| bounds.set(Some(rect)), |_, _, _, _| {})
                .absolute()
                .size_full(),
        )
        .child(
            div()
                .absolute()
                .left(px(KNOB_INSET))
                .right(px(KNOB_INSET))
                .top(px((SLIDER_HEIGHT - TRACK_HEIGHT) / 2.0))
                .h(px(TRACK_HEIGHT))
                .rounded_full()
                .bg(border)
                .child(
                    div()
                        .h_full()
                        .w(relative(fraction))
                        .rounded_full()
                        .bg(accent),
                )
                .child(
                    div()
                        .absolute()
                        .left(relative(fraction))
                        .ml(px(-KNOB_SIZE / 2.0))
                        .top(px(-(KNOB_SIZE - TRACK_HEIGHT) / 2.0))
                        .size(px(KNOB_SIZE))
                        .rounded_full()
                        .bg(accent),
                ),
        );

    // The detail rows occupy the surrounding whitespace, so the slider keeps
    // the same row rhythm as the font controls (upstream :622-676).
    let reset = div()
        .id("reset-transcript-width")
        .tab_index(0)
        .role(Role::Button)
        .aria_label(format!("{reset_label} {width_title}"))
        .focus_visible(move |style| style.border_2().border_color(accent))
        .cursor_pointer()
        .text_color(muted)
        .hover(move |style| style.text_color(text))
        .on_key_down(cx.listener(|page, event: &gpui::KeyDownEvent, _, cx| {
            if is_activation(event) {
                cx.stop_propagation();
                reset_width(page, cx);
            }
        }))
        .on_click(cx.listener(|page, _, _, cx| reset_width(page, cx)))
        .child(reset_label);

    let width_control = div()
        .id("transcript-width-control")
        .on_hover(cx.listener(|page, hovered: &bool, _, cx| {
            page.layout.width_hovered = *hovered;
            if *hovered {
                page.layout.width_keyboard_active = false;
            }
            cx.notify();
        }))
        .my(px(-12.0))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .when(!show_details, |el| el.invisible())
                .flex()
                .justify_between()
                .text_size(typography::ui_rems(12.0))
                .line_height(px(16.0))
                .child(width_value)
                .child(reset),
        )
        .child(slider)
        .child(
            div()
                .when(!show_details, |el| el.invisible())
                .flex()
                .justify_between()
                .text_size(typography::ui_rems(11.0))
                .line_height(px(14.0))
                .text_color(muted)
                .child(min_label)
                .child(max_label),
        );

    let width_row = widgets::card_row(theme, false)
        .gap(px(20.0))
        .child(
            div()
                .flex_1()
                .min_w(px(200.0))
                .child(widgets::row_title(theme, width_title))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(tr("appearance.layout.conversation_width.desc", lang))
                            .into_any_element(),
                    ],
                )),
        )
        .child(width_control);

    // Render math: the toggle keeps the row readable but inert.
    let render_math_control = widgets::toggle_switch(theme, render_math, "render-math")
        .id("render-math-toggle")
        .tab_index(0)
        .role(Role::Switch)
        .aria_label(tr("appearance.layout.render_math", lang))
        .aria_toggled(if render_math {
            Toggled::True
        } else {
            Toggled::False
        })
        .focus_visible(move |style| style.border_2().border_color(accent))
        .cursor_pointer()
        .on_click(cx.listener(move |_, _, _, cx| set_render_math(!render_math, cx)));
    let render_math_row = widgets::card_row(theme, false)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(widgets::row_title(
                    theme,
                    tr("appearance.layout.render_math", lang),
                ))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(tr("appearance.layout.render_math.desc", lang))
                            .into_any_element(),
                    ],
                )),
        )
        .child(render_math_control);

    // Math font: zeron's face list with the style note as the option detail.
    let font_index = MathFont::ALL
        .into_iter()
        .position(|font| font == math_font)
        .unwrap_or_default();
    let font_control = select(
        "math-font",
        tr("appearance.layout.math_font", lang),
        theme,
        |page: &mut AppearancePage| &mut page.layout.math_font,
    )
    .options(
        MathFont::ALL
            .into_iter()
            .map(|font| SelectOption::new(font.label()).detail(math_font_style(font, lang))),
        font_index,
    )
    .width(160.0)
    .menu_width(220.0)
    .heading(tr("appearance.layout.math_font", lang))
    .on_select(|_page: &mut AppearancePage, ix, _window, cx| {
        AppSettings::global_mut(cx).math_font = MathFont::ALL[ix];
        crate::settings::persist_now(cx);
        cx.notify();
    })
    .render(&page.layout.math_font, cx);
    let math_font_row = widgets::card_row(theme, false)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(
                    theme,
                    tr("appearance.layout.math_font", lang),
                ))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(tr("appearance.layout.math_font.desc", lang))
                            .into_any_element(),
                    ],
                )),
        )
        .child(font_control);

    // Math size: the percent ladder, with "Default" noted on the 100% rung.
    let size_index = nearest_math_size_ix(math_size);
    let default_detail = tr("appearance.layout.math_size.default", lang);
    let percent = tr("unit.percent", lang);
    let size_control = select(
        "math-size",
        tr("appearance.layout.math_size", lang),
        theme,
        |page: &mut AppearancePage| &mut page.layout.math_size,
    )
    .options(
        MATH_SIZES.into_iter().map(|size| {
            let option = SelectOption::new(format!("{size}{percent}"));
            if size == MATH_SIZE_DEFAULT {
                option.detail(default_detail)
            } else {
                option
            }
        }),
        size_index,
    )
    .width(160.0)
    .on_select(|_page: &mut AppearancePage, ix, _window, cx| {
        AppSettings::global_mut(cx).math_size = MATH_SIZES[ix];
        crate::settings::persist_now(cx);
        cx.notify();
    })
    .render(&page.layout.math_size, cx);
    let math_size_row = widgets::card_row(theme, false)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(
                    theme,
                    tr("appearance.layout.math_size", lang),
                ))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(tr("appearance.layout.math_size.desc", lang))
                            .into_any_element(),
                    ],
                )),
        )
        .child(size_control);

    // One column so the rows nest under the Fonts and layout card's
    // `section_card` exactly like zeron appends them to `font_rows`.
    div()
        .w_full()
        .flex()
        .flex_col()
        .child(width_row)
        .child(render_math_row)
        .child(math_font_row)
        .child(math_size_row)
        .into_any_element()
}

/// Host `on_drag_move<TranscriptWidthDrag>` hook (upstream :4186-4191).
pub(super) fn drag_width(
    page: &mut AppearancePage,
    event: &DragMoveEvent<TranscriptWidthDrag>,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    drag_width_at(page, event.event.position.x, window, cx);
}

/// Host `on_mouse_up`/`on_mouse_up_out` flush of the pending width (upstream
/// :4202-4225): the release always publishes the last queued value.
pub(super) fn mouse_up(
    page: &mut AppearancePage,
    _event: &MouseUpEvent,
    _window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    if page.layout.width_pressed {
        page.layout.width_pressed = false;
        cx.notify();
    }
    apply_pending_width(page, cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};

    /// The slider's painted bounds in the upstream drag test: a 240x28 track
    /// starting at x=100.
    fn track_bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(100.0), px(0.0)), size(px(240.0), px(28.0)))
    }

    #[test]
    fn math_size_ladder_holds_the_default_inside_the_legal_range() {
        assert!(MATH_SIZES.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(MATH_SIZES.iter().all(|size| {
            (crate::app_settings::MATH_SIZE_MIN..=crate::app_settings::MATH_SIZE_MAX).contains(size)
        }));
        assert_eq!(MATH_SIZES[nearest_math_size_ix(MATH_SIZE_DEFAULT)], 100);
        assert_eq!(
            MATH_SIZES[nearest_math_size_ix(crate::app_settings::MATH_SIZE_MIN)],
            90
        );
        assert_eq!(
            MATH_SIZES[nearest_math_size_ix(crate::app_settings::MATH_SIZE_MAX)],
            150
        );
        // Off-ladder values snap, never drop.
        assert_eq!(MATH_SIZES[nearest_math_size_ix(133)], 135);
        assert_eq!(MATH_SIZES[nearest_math_size_ix(0)], 90);
    }

    #[test]
    fn width_from_x_matches_the_upstream_coordinate_math() {
        // Both extremes and the centre, after the 16px grid normalization.
        assert_eq!(
            normalize_transcript_width(width_from_x(track_bounds(), px(100.0))),
            TRANSCRIPT_WIDTH_MIN
        );
        assert_eq!(
            normalize_transcript_width(width_from_x(track_bounds(), px(340.0))),
            TRANSCRIPT_WIDTH_MAX
        );
        assert_eq!(
            normalize_transcript_width(width_from_x(track_bounds(), px(220.0))),
            880.0
        );
        // Pointer positions outside the track clamp instead of escaping.
        assert_eq!(
            normalize_transcript_width(width_from_x(track_bounds(), px(0.0))),
            TRANSCRIPT_WIDTH_MIN
        );
        assert_eq!(
            normalize_transcript_width(width_from_x(track_bounds(), px(2000.0))),
            TRANSCRIPT_WIDTH_MAX
        );
    }

    #[test]
    fn degenerate_track_bounds_cannot_divide_by_zero() {
        let degenerate = Bounds::new(point(px(0.0), px(0.0)), size(px(0.0), px(28.0)));
        let width = normalize_transcript_width(width_from_x(degenerate, px(0.0)));
        assert_eq!(width, TRANSCRIPT_WIDTH_MIN);
    }

    #[test]
    fn width_words_print_whole_and_fractional_pixels() {
        assert_eq!(format_px_number(736.0), "736");
        assert_eq!(format_px_number(12.5), "12.5");
        assert_eq!(format_px_number(TRANSCRIPT_WIDTH_DEFAULT), "736");
        // The unit comes from the i18n table; the number is always there.
        assert!(format_width(736.0, "en").starts_with("736 "));
    }

    #[test]
    fn every_math_face_has_a_localized_style_note() {
        for font in MathFont::ALL {
            let style = math_font_style(font, "en");
            assert!(!style.is_empty(), "{font:?}");
        }
        // The style word distinguishes the sans face from the serif family.
        assert_ne!(
            math_font_style(MathFont::Fira, "en"),
            math_font_style(MathFont::StixTwo, "en")
        );
        assert_eq!(
            math_font_style(MathFont::StixTwo, "en"),
            math_font_style(MathFont::Pagella, "en")
        );
    }
}
