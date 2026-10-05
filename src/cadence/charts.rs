//! Cadence data marks — the reference's micro-charts, drawn from plain
//! divs: histograms and sparkline strips, stacked bars over gridlines, a
//! waffle grid, horizontal aging bars, a dual-handle range slider and
//! gantt-style row bars. GPUI cannot dash a border, so dashed strokes
//! render as hairline solids at reduced alpha.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::base::ElementExt as _;
use gpui_kit::component::{ActiveTheme, StyledExt as _, ThemeStyled as _, h_flex, v_flex};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, Div, ElementId, Empty, EntityId, FocusHandle, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px, relative,
};

/// Histogram / sparkline strip: fixed-width bars spread over the full
/// width. Bars inside `accent_range` (inclusive) take the solid accent,
/// the rest the hairline gray; `band` paints a wash behind the range.
#[derive(IntoElement)]
pub struct MiniBars {
    values: Vec<f32>,
    height: Pixels,
    bar_w: Pixels,
    accent_range: Option<(usize, usize)>,
    band: Option<(usize, usize)>,
}

impl MiniBars {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            values,
            height: px(48.),
            bar_w: px(4.),
            accent_range: None,
            band: None,
        }
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn bar_w(mut self, bar_w: Pixels) -> Self {
        self.bar_w = bar_w;
        self
    }

    /// Inclusive bar index range drawn in the solid accent.
    pub fn accent_range(mut self, range: (usize, usize)) -> Self {
        self.accent_range = Some(range);
        self
    }

    /// Inclusive bar index range with a wash band behind the bars.
    pub fn band(mut self, range: (usize, usize)) -> Self {
        self.band = Some(range);
        self
    }
}

impl RenderOnce for MiniBars {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let n = self.values.len().max(1) as f32;
        let slot = |ix: usize| ix as f32 / n;
        let span = |(from, to): (usize, usize)| (to - from + 1) as f32 / n;
        div()
            .relative()
            .w_full()
            .h(self.height)
            .children(self.band.map(|range| {
                div()
                    .absolute()
                    .left(relative(slot(range.0)))
                    .w(relative(span(range)))
                    .top_0()
                    .bottom_0()
                    .rounded(px(6.))
                    .bg(super::Accent::wash(cx))
            }))
            .child(h_flex().absolute().inset_0().items_end().children(
                self.values.iter().enumerate().map(|(ix, v)| {
                    let accent = self
                        .accent_range
                        .is_some_and(|(from, to)| ix >= from && ix <= to);
                    // One flex_1 slot per bar so bar centers line up with
                    // the `ix / n` band math (L2): `justify_between` would
                    // edge-align the first/last bars and skew the wash.
                    div()
                        .flex_1()
                        .h_full()
                        .flex()
                        .items_end()
                        .justify_center()
                        .child(
                            div()
                                .w(self.bar_w)
                                .h(self.height * v.clamp(0.02, 1.))
                                .rounded(px(1.5))
                                .bg(if accent {
                                    super::Accent::solid(cx)
                                } else {
                                    cx.theme().border
                                }),
                        )
                }),
            ))
    }
}

/// Stacked bar chart over gridlines: each bar is a faint "unscheduled"
/// segment under a solid "scheduled" one, with mono y labels on the left,
/// an optional tooltip pill over one bar, and mono x labels below.
#[derive(IntoElement)]
pub struct BarChartMini {
    bars: Vec<(f32, f32)>,
    height: Pixels,
    y_labels: Vec<SharedString>,
    tooltip: Option<(usize, SharedString)>,
    x_labels: Vec<SharedString>,
}

impl BarChartMini {
    pub fn new(bars: Vec<(f32, f32)>) -> Self {
        Self {
            bars,
            height: px(150.),
            y_labels: Vec::new(),
            tooltip: None,
            x_labels: Vec::new(),
        }
    }

    /// Mono labels top → bottom, one per gridline (last sits on the base).
    pub fn y_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.y_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Tooltip pill anchored over bar `ix`.
    pub fn tooltip(mut self, ix: usize, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some((ix, text.into()));
        self
    }

    pub fn x_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.x_labels = labels.into_iter().map(Into::into).collect();
        self
    }
}

impl RenderOnce for BarChartMini {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let height = self.height;
        let n = self.bars.len().max(1);
        let slot_w = |ix: usize| (ix as f32 + 0.5) / n as f32;
        let grid_color = cx.theme().border;

        // Tooltip pill: anchored over its bar, width estimated from the
        // mono glyph advance (~0.55em at text_sm) so it stays centered.
        let has_tooltip = self.tooltip.is_some();
        let tooltip_el = self.tooltip.map(|(ix, text)| {
            let est_w = text.len() as f32 * 7.5 + 20.;
            div()
                .absolute()
                .left(relative(slot_w(ix)))
                .top(px(4.))
                .ml(px(-est_w / 2.))
                .child(
                    h_flex()
                        .rounded(px(7.))
                        .bg(super::Accent::soft(cx))
                        .px_2()
                        .py(px(3.))
                        .text_color(super::Accent::ink(cx))
                        .child(super::blocks::mono(text, cx).text_sm()),
                )
        });

        div()
            .w_full()
            .child(
                h_flex()
                    .gap_2()
                    .when(has_tooltip, |this| this.mt(px(24.)))
                    .child(
                        // y labels, one per gridline, top → bottom.
                        v_flex().justify_between().h(height).items_end().children(
                            self.y_labels.iter().map(|label| {
                                super::blocks::mono(label.clone(), cx)
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                            }),
                        ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h(height)
                            .child(
                                // Gridlines: hairlines at even fractions,
                                // first on top, last on the baseline.
                                v_flex().absolute().inset_0().justify_between().children(
                                    (0..self.y_labels.len().max(2))
                                        .map(|_| div().h(px(1.)).w_full().bg(grid_color)),
                                ),
                            )
                            .child(
                                h_flex()
                                    .absolute()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .top_0()
                                    .items_end()
                                    .justify_between()
                                    .children(self.bars.iter().map(|(solid, faint)| {
                                        let solid_val = solid.clamp(0., 1.);
                                        let faint_val = faint.clamp(0., (1. - solid_val).max(0.));
                                        v_flex()
                                            .items_center()
                                            .justify_end()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .w(px(10.))
                                                    .h(height * faint_val)
                                                    .rounded(px(3.))
                                                    .bg(super::Accent::faint(cx)),
                                            )
                                            .child(
                                                div()
                                                    .w(px(10.))
                                                    .h(height * solid_val)
                                                    .rounded(px(3.))
                                                    .bg(super::Accent::solid(cx)),
                                            )
                                    })),
                            )
                            .children(tooltip_el),
                    ),
            )
            .when(!self.x_labels.is_empty(), |this| {
                this.child(h_flex().justify_between().mt_2().pl(px(36.)).children(
                    self.x_labels.iter().map(|label| {
                        super::blocks::mono(label.clone(), cx)
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                    }),
                ))
            })
    }
}

/// One waffle cell's fill level.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WaffleCell {
    Full,
    Mid,
    Light,
    /// Lighter than `Light` — the fourth legend color.
    Faint,
    /// Empty: hairline outline only (the reference's dashed cell).
    Empty,
}

/// Waffle chart: a `cols`-wide grid of rounded cells ramping from solid
/// accent to empty.
#[derive(IntoElement)]
pub struct Waffle {
    cells: Vec<WaffleCell>,
    cols: usize,
    cell: Pixels,
}

impl Waffle {
    pub fn new(cells: Vec<WaffleCell>) -> Self {
        Self {
            cells,
            cols: 10,
            cell: px(14.),
        }
    }

    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols.max(1);
        self
    }
}

impl RenderOnce for Waffle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let gap = px(4.);
        let cell_style = |cell: WaffleCell| match cell {
            WaffleCell::Full => div().bg(super::Accent::solid(cx)),
            WaffleCell::Mid => div().bg(super::Accent::mid(cx)),
            WaffleCell::Light => div().bg(super::Accent::faint(cx)),
            WaffleCell::Faint => div().bg(super::Accent::soft(cx)),
            WaffleCell::Empty => div().border_1().border_color(cx.theme().border),
        };
        v_flex().gap(gap).children(
            self.cells
                .chunks(self.cols)
                .map(|row| {
                    h_flex().gap(gap).children(row.iter().map(|cell| {
                        cell_style(*cell)
                            .size(self.cell)
                            .rounded(px(4.))
                            .flex_shrink_0()
                    }))
                })
                .collect::<Vec<_>>(),
        )
    }
}

/// One aging column: `filled` of 5 horizontal bars in `color`, labeled
/// below with its bucket and mono amount.
pub struct AgingGroup {
    filled: usize,
    color: Hsla,
    label: SharedString,
    amount: SharedString,
}

impl AgingGroup {
    pub fn new(
        filled: usize,
        color: Hsla,
        label: impl Into<SharedString>,
        amount: impl Into<SharedString>,
    ) -> Self {
        Self {
            filled: filled.clamp(0, 5),
            color,
            label: label.into(),
            amount: amount.into(),
        }
    }
}

/// The aging strip: columns of horizontal bars, fullest on the left.
#[derive(IntoElement)]
pub struct AgingBars {
    groups: Vec<AgingGroup>,
}

impl AgingBars {
    pub fn new(groups: Vec<AgingGroup>) -> Self {
        Self { groups }
    }
}

impl RenderOnce for AgingBars {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let track = cx.theme().border;
        h_flex()
            .gap_3()
            .items_start()
            .children(self.groups.into_iter().map(|group| {
                v_flex()
                    .flex_1()
                    .gap_1p5()
                    .children((0..5).map(|ix| {
                        div().w_full().h(px(6.)).rounded_full().mb(px(2.)).bg(
                            if ix < group.filled {
                                group.color
                            } else {
                                track
                            },
                        )
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(group.label),
                    )
                    .child(
                        super::blocks::mono(group.amount, cx)
                            .text_sm()
                            .font_medium()
                            .text_color(cx.theme().foreground),
                    )
            }))
    }
}

/// Which part of the slider a press grabbed.
#[derive(Clone, Copy)]
enum SliderGrab {
    Lo,
    Hi,
    /// The active span — panning shifts both thumbs, keeping the width.
    Span,
}

/// Press snapshot: what was grabbed, the pointer's start fraction, and the
/// (lo, hi) range at press time. Span pans are deltas off these values.
#[derive(Clone, Copy)]
struct SliderPress {
    grab: SliderGrab,
    start_frac: f32,
    start_range: (f32, f32),
}

/// Shared mutable state of one slider: the track's painted bounds (pointer
/// → fraction mapping) and the press snapshot. Held as `Rc<RefCell>` in
/// keyed element state — `on_prepaint` and mouse handlers both write it
/// without forcing re-renders (the `tab_bar` bounds pattern).
#[derive(Default)]
struct SliderInteraction {
    track: Bounds<Pixels>,
    press: Option<SliderPress>,
}

/// Invisible overlay entity required by `on_drag`'s payload type.
#[derive(Clone)]
struct SliderDrag {
    interaction: EntityId,
}

impl Render for SliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// Minimum gap kept between the two thumbs.
const SLIDER_MIN_GAP: f32 = 0.02;
/// Arrow-key step; `Shift` + arrow steps five times this.
const SLIDER_STEP: f32 = 0.01;

/// Dual-handle range slider: hairline track, solid accent span, white
/// thumbs ringed in accent. Positions are 0..1 track fractions. Thumbs
/// drag, the track click-seeks the nearest thumb, the active span pans,
/// and each thumb is a focus stop (arrows step, `Shift` accelerates,
/// Home/End snap).
#[derive(IntoElement)]
pub struct RangeSlider {
    id: ElementId,
    lo: f32,
    hi: f32,
    on_change: Option<super::RangeCallback>,
}

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, lo: f32, hi: f32) -> Self {
        let lo = lo.clamp(0., 1.);
        Self {
            id: id.into(),
            lo,
            hi: hi.clamp(0., 1.).max(lo),
            on_change: None,
        }
    }

    /// Called with the new `(lo, hi)` whenever pointer or keyboard moves
    /// either thumb.
    pub fn on_change(mut self, f: impl Fn(&(f32, f32), &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for RangeSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let RangeSlider {
            id,
            lo,
            hi,
            on_change,
        } = self;
        let interaction = window.use_keyed_state((id.clone(), "interaction"), cx, |_, _| {
            Rc::new(RefCell::new(SliderInteraction::default()))
        });
        let slider_id = interaction.entity_id();
        let interaction = interaction.read(cx).clone();
        let lo_focus = window
            .use_keyed_state((id.clone(), "lo-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let hi_focus = window
            .use_keyed_state((id.clone(), "hi-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();

        let frac_at = |track: Bounds<Pixels>, position: Point<Pixels>| -> f32 {
            if track.size.width <= px(0.) {
                return 0.;
            }
            ((position.x - track.left()) / track.size.width).clamp(0., 1.)
        };

        let thumb = |at: f32, grab: SliderGrab, focus: FocusHandle, focused: bool| {
            let (is_lo, tag) = match grab {
                SliderGrab::Lo => (true, "lo"),
                _ => (false, "hi"),
            };
            div()
                .id((id.clone(), tag))
                .absolute()
                .top(px(2.))
                .left(relative(at))
                // Center the 16px thumb on the fraction (H4).
                .ml(px(-8.))
                .size(px(16.))
                .rounded_full()
                .bg(cx.theme().popover)
                .border_2()
                .border_color(super::Accent::solid(cx))
                .shadow_xs()
                .cursor_ew_resize()
                .tab_index(0)
                .track_focus(&focus)
                .when(focused, |this| this.focus_ring_style(window, cx))
                .role(gpui_kit::Role::Slider)
                .aria_label(if is_lo {
                    "Minimum Amount"
                } else {
                    "Maximum Amount"
                })
                .aria_numeric_value(at as f64)
                .aria_min_numeric_value(0.)
                .aria_max_numeric_value(1.)
                .aria_orientation(gpui_kit::Orientation::Horizontal)
                .hover(|style| style.shadow_sm().border_color(super::Accent::solid(cx)))
                .on_mouse_down(MouseButton::Left, {
                    let interaction = interaction.clone();
                    move |event, window, cx| {
                        // Focus on press and own the gesture so the track's
                        // seek/pan handler below never sees it.
                        window.focus(&focus, cx);
                        let track = interaction.borrow().track;
                        interaction.borrow_mut().press = Some(SliderPress {
                            grab,
                            start_frac: frac_at(track, event.position),
                            start_range: (lo, hi),
                        });
                        cx.stop_propagation();
                    }
                })
                .on_drag(
                    SliderDrag {
                        interaction: slider_id,
                    },
                    |drag, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    },
                )
                .on_key_down({
                    let on_change = on_change.clone();
                    move |event, _window, cx| {
                        let key = event.keystroke.key.as_str();
                        let step = if event.keystroke.modifiers.shift {
                            SLIDER_STEP * 5.
                        } else {
                            SLIDER_STEP
                        };
                        let (mut nlo, mut nhi) = (lo, hi);
                        match key {
                            "left" | "down" => {
                                if is_lo {
                                    nlo = (lo - step).clamp(0., hi - SLIDER_MIN_GAP);
                                } else {
                                    nhi = (hi - step).clamp(lo + SLIDER_MIN_GAP, 1.);
                                }
                            }
                            "right" | "up" => {
                                if is_lo {
                                    nlo = (lo + step).clamp(0., hi - SLIDER_MIN_GAP);
                                } else {
                                    nhi = (hi + step).clamp(lo + SLIDER_MIN_GAP, 1.);
                                }
                            }
                            "home" => {
                                if is_lo {
                                    nlo = 0.;
                                } else {
                                    nhi = (lo + SLIDER_MIN_GAP).min(1.);
                                }
                            }
                            "end" => {
                                if is_lo {
                                    nlo = (hi - SLIDER_MIN_GAP).max(0.);
                                } else {
                                    nhi = 1.;
                                }
                            }
                            _ => return,
                        }
                        cx.stop_propagation();
                        if let Some(on_change) = &on_change {
                            on_change(&(nlo, nhi), _window, cx);
                        }
                    }
                })
        };

        div()
            .id(id.clone())
            .relative()
            .w_full()
            .h(px(20.))
            .child(
                div()
                    .absolute()
                    .top(px(8.5))
                    .left_0()
                    .right_0()
                    .h(px(3.))
                    .rounded_full()
                    .bg(super::Accent::faint(cx))
                    .on_prepaint({
                        let interaction = interaction.clone();
                        move |bounds, _, _| {
                            interaction.borrow_mut().track = bounds;
                        }
                    }),
            )
            .child(
                div()
                    .absolute()
                    .top(px(8.5))
                    .left(relative(lo))
                    .w(relative(hi - lo))
                    .h(px(3.))
                    .rounded_full()
                    .bg(super::Accent::solid(cx))
                    .cursor_grab(),
            )
            .child(thumb(
                lo,
                SliderGrab::Lo,
                lo_focus.clone(),
                lo_focus.is_focused(window),
            ))
            .child(thumb(
                hi,
                SliderGrab::Hi,
                hi_focus.clone(),
                hi_focus.is_focused(window),
            ))
            .on_mouse_down(MouseButton::Left, {
                let interaction = interaction.clone();
                let on_change = on_change.clone();
                move |event, window, cx| {
                    let track = interaction.borrow().track;
                    let frac = frac_at(track, event.position);
                    let grab = if frac >= lo && frac <= hi {
                        SliderGrab::Span
                    } else if (frac - lo).abs() <= (frac - hi).abs() {
                        SliderGrab::Lo
                    } else {
                        SliderGrab::Hi
                    };
                    interaction.borrow_mut().press = Some(SliderPress {
                        grab,
                        start_frac: frac,
                        start_range: (lo, hi),
                    });
                    match grab {
                        // Inside the span: no jump — the press arms a span
                        // pan that starts when the pointer moves.
                        SliderGrab::Span => {}
                        // Outside: the nearest thumb jumps to the click
                        // (seek) and keeps scrubbing on drag.
                        SliderGrab::Lo => {
                            let nlo = frac.clamp(0., hi - SLIDER_MIN_GAP);
                            if let Some(on_change) = &on_change {
                                on_change(&(nlo, hi), window, cx);
                            }
                        }
                        SliderGrab::Hi => {
                            let nhi = frac.clamp(lo + SLIDER_MIN_GAP, 1.);
                            if let Some(on_change) = &on_change {
                                on_change(&(lo, nhi), window, cx);
                            }
                        }
                    }
                }
            })
            .on_drag(
                SliderDrag {
                    interaction: slider_id,
                },
                |drag, _, _, cx| cx.new(|_| drag.clone()),
            )
            .on_drag_move::<SliderDrag>({
                let interaction = interaction.clone();
                let on_change = on_change.clone();
                move |event, _window, cx| {
                    if event.drag(cx).interaction != slider_id {
                        return;
                    }
                    let (track, press) = {
                        let i = interaction.borrow();
                        (i.track, i.press)
                    };
                    let Some(press) = press else { return };
                    let frac = frac_at(track, event.event.position);
                    let (slo, shi) = press.start_range;
                    let (nlo, nhi) = match press.grab {
                        SliderGrab::Lo => (frac.clamp(0., shi - SLIDER_MIN_GAP), shi),
                        SliderGrab::Hi => (slo, frac.clamp(slo + SLIDER_MIN_GAP, 1.)),
                        SliderGrab::Span => {
                            let delta = frac - press.start_frac;
                            let width = shi - slo;
                            let nlo = (slo + delta).clamp(0., 1. - width);
                            (nlo, nlo + width)
                        }
                    };
                    if let Some(on_change) = &on_change {
                        on_change(&(nlo, nhi), _window, cx);
                    }
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let interaction = interaction.clone();
                move |_, _, _| {
                    interaction.borrow_mut().press = None;
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let interaction = interaction.clone();
                move |_, _, _| {
                    interaction.borrow_mut().press = None;
                }
            })
    }
}

/// One gantt segment of a purchase-order row.
#[derive(Clone, Copy)]
pub enum GanttSeg {
    /// Billed: solid accent pill, `frac` of the track.
    Solid(f32),
    /// Ordered: faint pill.
    Faint(f32),
    /// Pending: hairline outline pill (the reference's dashed stroke).
    Dashed(f32),
    /// Delivery marker: ringed dot closing the bar.
    Delivery,
}

/// Gantt-style row bar: sequential pill segments with 2px gaps. Each
/// segment is a hover group: hovering raises it and reveals the matching
/// tooltip pill (`.tips`), centered above the segment slot.
#[derive(IntoElement)]
pub struct Gantt {
    id: SharedString,
    segments: Vec<GanttSeg>,
    tips: Vec<SharedString>,
}

impl Gantt {
    /// `id` bases the per-segment hover groups — it must be unique per row.
    pub fn new(id: impl Into<SharedString>, segments: Vec<GanttSeg>) -> Self {
        Self {
            id: id.into(),
            segments,
            tips: Vec::new(),
        }
    }

    /// Hover text per segment, in segment order. Missing entries render no
    /// tooltip; extras are ignored.
    pub fn tips(mut self, tips: Vec<impl Into<SharedString>>) -> Self {
        self.tips = tips.into_iter().map(Into::into).collect();
        self
    }
}

impl RenderOnce for Gantt {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id;
        h_flex()
            .items_center()
            .gap(px(2.))
            .children(self.segments.into_iter().enumerate().map(|(ix, seg)| {
                // The hover group name doubles as the segment's element id
                // (a hitbox is what registers the group).
                let hover = SharedString::from(format!("{base}-{ix}"));
                let slot = |body: Div, tip: Option<Div>| {
                    body.id(hover.clone())
                        .group(hover.clone())
                        .relative()
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.8))
                        .children(tip)
                };
                let tip_el = self.tips.get(ix).cloned().map(|text| {
                    div()
                        .absolute()
                        .bottom(px(12.))
                        .left_0()
                        .right_0()
                        .invisible()
                        .group_hover(hover.clone(), |s| s.visible())
                        .child(
                            h_flex().justify_center().child(
                                h_flex()
                                    .flex_shrink_0()
                                    .rounded(px(7.))
                                    .bg(super::Accent::soft(cx))
                                    .px_2()
                                    .py(px(3.))
                                    .text_color(super::Accent::ink(cx))
                                    .child(
                                        super::blocks::mono(text, cx).text_sm().whitespace_nowrap(),
                                    ),
                            ),
                        )
                });
                match seg {
                    GanttSeg::Solid(f) => slot(
                        div()
                            .w(relative(f.clamp(0., 1.)))
                            .h(px(8.))
                            .rounded_full()
                            .flex_shrink_0()
                            .bg(super::Accent::solid(cx)),
                        tip_el,
                    ),
                    GanttSeg::Faint(f) => slot(
                        div()
                            .w(relative(f.clamp(0., 1.)))
                            .h(px(8.))
                            .rounded_full()
                            .flex_shrink_0()
                            .bg(super::Accent::faint(cx)),
                        tip_el,
                    ),
                    GanttSeg::Dashed(f) => slot(
                        div()
                            .w(relative(f.clamp(0., 1.)))
                            .h(px(8.))
                            .rounded_full()
                            .flex_shrink_0()
                            .border_1()
                            .border_color(cx.theme().muted_foreground.alpha(0.5)),
                        tip_el,
                    ),
                    GanttSeg::Delivery => slot(
                        div()
                            .size(px(10.))
                            .rounded_full()
                            .border_2()
                            .border_color(super::Accent::solid(cx))
                            .bg(cx.theme().popover)
                            .flex_shrink_0(),
                        tip_el,
                    ),
                }
            }))
    }
}
