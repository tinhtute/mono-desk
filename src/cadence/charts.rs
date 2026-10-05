//! Cadence data marks — the reference's micro-charts, drawn from plain
//! divs: histograms and sparkline strips, stacked bars over gridlines, a
//! waffle grid, horizontal aging bars, a dual-handle range slider and
//! gantt-style row bars. GPUI cannot dash a border, so dashed strokes
//! render as hairline solids at reduced alpha.

use gpui_kit::component::{ActiveTheme, StyledExt as _, h_flex, v_flex};
use gpui_kit::{
    App, Div, Hsla, IntoElement, ParentElement as _, Pixels, RenderOnce, SharedString, Styled as _,
    Window, div, prelude::FluentBuilder as _, px, relative,
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
            .child(
                h_flex()
                    .absolute()
                    .inset_0()
                    .items_end()
                    .justify_between()
                    .children(self.values.iter().enumerate().map(|(ix, v)| {
                        let accent = self
                            .accent_range
                            .is_some_and(|(from, to)| ix >= from && ix <= to);
                        div()
                            .w(self.bar_w)
                            .h(self.height * v.clamp(0.02, 1.))
                            .rounded(px(1.5))
                            .bg(if accent {
                                super::Accent::solid(cx)
                            } else {
                                cx.theme().border
                            })
                    })),
            )
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

/// Dual-handle range slider (visual): hairline track, solid accent span,
/// white thumbs ringed in accent. Positions are 0..1 track fractions.
#[derive(IntoElement)]
pub struct RangeSlider {
    lo: f32,
    hi: f32,
}

impl RangeSlider {
    pub fn new(lo: f32, hi: f32) -> Self {
        Self {
            lo: lo.clamp(0., 1.),
            hi: hi.clamp(0., 1.).max(lo),
        }
    }
}

impl RenderOnce for RangeSlider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let thumb = |at: f32| {
            div()
                .absolute()
                .top(px(2.))
                .left(relative(at))
                .ml(px(-8.))
                .size(px(16.))
                .rounded_full()
                .bg(cx.theme().popover)
                .border_2()
                .border_color(super::Accent::solid(cx))
                .shadow_xs()
        };
        div()
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
                    .bg(super::Accent::faint(cx)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(8.5))
                    .left(relative(self.lo))
                    .w(relative(self.hi - self.lo))
                    .h(px(3.))
                    .rounded_full()
                    .bg(super::Accent::solid(cx)),
            )
            .child(thumb(self.lo))
            .child(thumb(self.hi))
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

/// Gantt-style row bar: sequential pill segments with 2px gaps.
#[derive(IntoElement)]
pub struct Gantt {
    segments: Vec<GanttSeg>,
}

impl Gantt {
    pub fn new(segments: Vec<GanttSeg>) -> Self {
        Self { segments }
    }
}

impl RenderOnce for Gantt {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap(px(2.))
            .children(self.segments.into_iter().map(|seg| {
                match seg {
                    GanttSeg::Solid(f) => pill(f).bg(super::Accent::solid(cx)),
                    GanttSeg::Faint(f) => pill(f).bg(super::Accent::faint(cx)),
                    GanttSeg::Dashed(f) => pill(f)
                        .border_1()
                        .border_color(cx.theme().muted_foreground.alpha(0.5)),
                    GanttSeg::Delivery => div()
                        .size(px(10.))
                        .rounded_full()
                        .border_2()
                        .border_color(super::Accent::solid(cx))
                        .bg(cx.theme().popover)
                        .flex_shrink_0(),
                }
            }))
    }
}

fn pill(frac: f32) -> Div {
    div()
        .w(relative(frac.clamp(0., 1.)))
        .h(px(8.))
        .rounded_full()
        .flex_shrink_0()
}
