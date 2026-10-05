//! Cadence data marks — the reference's micro-charts, drawn from plain
//! divs: histograms and sparkline strips, stacked bars over gridlines, a
//! waffle grid, horizontal aging bars, a dual-handle range slider and
//! gantt-style row bars. GPUI cannot dash a border, so dashed strokes
//! render as hairline solids at reduced alpha.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::base::ElementExt as _;
use gpui_kit::component::{ActiveTheme, StyledExt as _, ThemeStyled as _, h_flex, v_flex};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, Div, ElementId, Empty, Entity, EntityId, FocusHandle,
    Hsla, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point,
    Render, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
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
/// The floating mono pill every chart tooltip shares. `anchor` is a 0..1
/// track fraction; `track` is the painted track width so the zero-width
/// centered container can be clamped by half the pill's estimated width
/// instead of spilling over the card edge. The estimate counts *chars*
/// (L3 — `len()` is bytes and `·` is two of them, skewing the clamp).
fn hover_pill(anchor: f32, track: Pixels, text: SharedString, cx: &App) -> Div {
    let est_w = text.chars().count() as f32 * 7.5 + 20.;
    let x = (track * anchor).clamp(px(est_w / 2.), (track - px(est_w / 2.)).max(px(0.)));
    h_flex()
        .absolute()
        .left(x)
        .top_0()
        .w(px(0.))
        .justify_center()
        .child(
            h_flex()
                .rounded(px(7.))
                .bg(super::Accent::soft(cx))
                .px_2()
                .py(px(3.))
                .whitespace_nowrap()
                .text_color(super::Accent::ink(cx))
                .child(super::blocks::mono(text, cx).text_sm()),
        )
}

/// Stacked bar chart over gridlines: each bar is a faint "unscheduled"
/// segment under a solid "scheduled" one, with mono y labels on the left,
/// a mono tooltip pill over the hovered/selected/pinned bar, and mono x
/// labels below. Columns hover (highlight + sibling dimming + tooltip),
/// click to toggle the pinned bar, and the chart is a focus stop whose
/// arrows walk the selection (Home/End snap, Escape clears).
#[derive(IntoElement)]
pub struct BarChartMini {
    id: ElementId,
    bars: Vec<(f32, f32)>,
    height: Pixels,
    y_labels: Vec<SharedString>,
    x_labels: Vec<SharedString>,
    selected: Option<usize>,
    pinned: Option<usize>,
    tip_label: Option<super::BarLabelCallback>,
    on_select: Option<super::OptPickCallback>,
}

impl BarChartMini {
    pub fn new(id: impl Into<ElementId>, bars: Vec<(f32, f32)>) -> Self {
        Self {
            id: id.into(),
            bars,
            height: px(150.),
            y_labels: Vec::new(),
            x_labels: Vec::new(),
            selected: None,
            pinned: None,
            tip_label: None,
            on_select: None,
        }
    }

    /// Mono labels top → bottom, one per gridline (last sits on the base).
    pub fn y_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.y_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn x_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.x_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// The currently selected bar (accent baseline dot + tooltip anchor).
    pub fn selected(mut self, ix: Option<usize>) -> Self {
        self.selected = ix;
        self
    }

    /// Fallback tooltip anchor when no bar is hovered or selected — the
    /// reference's pinned "Oct 1" pill.
    pub fn pinned(mut self, ix: usize) -> Self {
        self.pinned = Some(ix);
        self
    }

    /// Tooltip text for a bar: `(index, solid, faint)` — the fractions are
    /// already clamped the way the column paints them.
    pub fn tip_label(mut self, f: impl Fn(usize, f32, f32) -> SharedString + 'static) -> Self {
        self.tip_label = Some(Rc::new(f));
        self
    }

    /// Called when a bar click toggles or arrow keys move the selection;
    /// `None` clears it (click on the selected bar, Escape).
    pub fn on_select(mut self, f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for BarChartMini {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let height = self.height;
        let n = self.bars.len().max(1);
        let slot_w = |ix: usize| (ix as f32 + 0.5) / n as f32;
        let grid_color = cx.theme().border;
        let selectable = self.on_select.is_some();

        let hovered = window.use_keyed_state((id.clone(), "hovered"), cx, |_, _| None::<usize>);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus.is_focused(window);
        let track_w = window
            .use_keyed_state((id.clone(), "track-w"), cx, |_, _| {
                Rc::new(RefCell::new(px(0.)))
            })
            .read(cx)
            .clone();

        // The bar the tooltip rides: hovered wins, then the selection,
        // then the pinned fallback.
        let hovered_ix = *hovered.read(cx);
        let sel = self.selected.filter(|&ix| ix < n);
        let tip_label = self.tip_label.clone();
        let eff_ix = tip_label
            .as_ref()
            .and(hovered_ix.or(sel).or(self.pinned))
            .map(|ix| ix.min(n - 1));

        let tooltip_el = eff_ix.zip(tip_label.as_ref()).and_then(|(ix, tip)| {
            let tw = *track_w.borrow();
            (tw > px(0.)).then(|| {
                let (solid, faint) = self.bars.get(ix).copied().unwrap_or((0., 0.));
                let sv = solid.clamp(0., 1.);
                let fv = faint.clamp(0., (1. - sv).max(0.));
                hover_pill(slot_w(ix), tw, tip(ix, sv, fv), cx)
            })
        });

        // The y column's estimated width doubles as the x row's left
        // spacer, keeping x slots under the same flex_1 plot (H7).
        let y_w = px(self
            .y_labels
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as f32
            * 7.5
            + 4.);

        let on_select = self.on_select;

        div()
            .id(id.clone())
            .w_full()
            .when(selectable, |this| {
                this.tab_index(0)
                    .track_focus(&focus)
                    .when(is_focused, |this| this.focus_ring_style(window, cx))
                    .on_mouse_down(MouseButton::Left, {
                        let focus = focus.clone();
                        move |_, window, cx| {
                            window.focus(&focus, cx);
                        }
                    })
                    .on_key_down({
                        let on_select = on_select.clone();
                        move |event, window, cx| {
                            let Some(on_select) = &on_select else { return };
                            let cur = sel.or(self.pinned).unwrap_or(0).min(n - 1);
                            let next = match event.keystroke.key.as_str() {
                                "left" => Some(cur.saturating_sub(1)),
                                "right" => Some((cur + 1).min(n - 1)),
                                "home" => Some(0),
                                "end" => Some(n - 1),
                                "escape" => None,
                                _ => return,
                            };
                            cx.stop_propagation();
                            on_select(next, window, cx);
                        }
                    })
            })
            .child(
                h_flex()
                    .gap_2()
                    .when(eff_ix.is_some(), |this| this.mt(px(24.)))
                    .child(
                        // y labels, one per gridline: each sits in a 1px
                        // wrapper centered on its hairline so the glyph
                        // straddles the line instead of hanging below (M2).
                        v_flex()
                            .w(y_w)
                            .justify_between()
                            .h(height)
                            .items_end()
                            .children(self.y_labels.iter().map(|label| {
                                div().h(px(1.)).flex().items_center().justify_end().child(
                                    super::blocks::mono(label.clone(), cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                )
                            })),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h(height)
                            .on_prepaint({
                                let track_w = track_w.clone();
                                move |bounds, _, _| {
                                    *track_w.borrow_mut() = bounds.size.width;
                                }
                            })
                            .child(
                                // Gridlines: hairlines at even fractions,
                                // first on top, last on the baseline.
                                v_flex().absolute().inset_0().justify_between().children(
                                    (0..self.y_labels.len().max(2))
                                        .map(|_| div().h(px(1.)).w_full().bg(grid_color)),
                                ),
                            )
                            .child(
                                // One flex_1 slot per bar so columns share
                                // the plot evenly; the stack clamps to the
                                // row, which also clips any residual
                                // overflow (H3).
                                h_flex()
                                    .absolute()
                                    .inset_0()
                                    .items_end()
                                    .overflow_hidden()
                                    .children(self.bars.iter().enumerate().map(
                                        |(ix, &(solid, faint))| {
                                            let solid_val = solid.clamp(0., 1.);
                                            let faint_val =
                                                faint.clamp(0., (1. - solid_val).max(0.));
                                            let is_hover = hovered_ix == Some(ix);
                                            let is_sel = sel == Some(ix);
                                            div()
                                                .id((id.clone(), ix.to_string()))
                                                .flex_1()
                                                .h_full()
                                                .flex()
                                                .items_end()
                                                .justify_center()
                                                .relative()
                                                .cursor_pointer()
                                                .rounded(px(4.))
                                                .when(hovered_ix.is_some() && !is_hover, |this| {
                                                    this.opacity(0.70)
                                                })
                                                .when(is_hover, |this| {
                                                    this.bg(super::Accent::wash(cx))
                                                })
                                                .on_hover({
                                                    let hovered = hovered.clone();
                                                    move |entered, _window, cx| {
                                                        hovered.update(cx, |h, cx| {
                                                            *h = entered.then_some(ix);
                                                            cx.notify();
                                                        });
                                                    }
                                                })
                                                .on_click({
                                                    let on_select = on_select.clone();
                                                    move |_, window, cx| {
                                                        if let Some(on_select) = &on_select {
                                                            on_select(
                                                                (sel != Some(ix)).then_some(ix),
                                                                window,
                                                                cx,
                                                            );
                                                        }
                                                    }
                                                })
                                                .child(
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
                                                        ),
                                                )
                                                .when(is_sel, |this| {
                                                    // Selection anchor: an
                                                    // accent dot riding the
                                                    // baseline under the bar.
                                                    this.child(
                                                        div()
                                                            .absolute()
                                                            .bottom(px(1.))
                                                            .left(relative(0.5))
                                                            .ml(px(-3.))
                                                            .size(px(6.))
                                                            .rounded_full()
                                                            .bg(super::Accent::solid(cx)),
                                                    )
                                                })
                                        },
                                    )),
                            )
                            .children(tooltip_el),
                    ),
            )
            .when(!self.x_labels.is_empty(), |this| {
                // x labels ride the same flex_1 slot geometry as the bars:
                // label l anchors on bar l * n / len (week starts), the
                // spacer keeps the row aligned under the plot (H7).
                let xlen = self.x_labels.len();
                this.child(h_flex().mt_2().gap_2().child(div().w(y_w)).child(
                    h_flex().flex_1().children((0..n).map(|ix| {
                        div().flex_1().flex().items_center().justify_center().when(
                            (ix * xlen).is_multiple_of(n),
                            |this| {
                                let label_ix = ix * xlen / n;
                                this.child(
                                    super::blocks::mono(self.x_labels[label_ix].clone(), cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                )
                            },
                        )
                    })),
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
/// accent to empty. `cats`/`tips` map each cell to its legend row so hover
/// highlights the whole category (cells and legend share `hovered`), a
/// click toggles the category filter, and arrow keys walk the grid.
#[derive(IntoElement)]
pub struct Waffle {
    id: ElementId,
    cells: Vec<WaffleCell>,
    cols: usize,
    cell: Pixels,
    cats: Vec<usize>,
    tips: Vec<SharedString>,
    selected: Option<usize>,
    hovered: Option<Entity<Option<usize>>>,
    on_select: Option<super::OptPickCallback>,
}

impl Waffle {
    pub fn new(id: impl Into<ElementId>, cells: Vec<WaffleCell>) -> Self {
        Self {
            id: id.into(),
            cells,
            cols: 10,
            cell: px(14.),
            cats: Vec::new(),
            tips: Vec::new(),
            selected: None,
            hovered: None,
            on_select: None,
        }
    }

    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols.max(1);
        self
    }

    /// Explicit cell size (M4 — `~26px` with the 4px gap fills the card
    /// well's width at 10 columns).
    pub fn cell_size(mut self, cell: Pixels) -> Self {
        self.cell = cell;
        self
    }

    /// The legend-category index of every cell, parallel to `cells`.
    pub fn cats(mut self, cats: Vec<usize>) -> Self {
        self.cats = cats;
        self
    }

    /// Tooltip text per category index (shown on the hovered cell/cat).
    pub fn tips(mut self, tips: Vec<impl Into<SharedString>>) -> Self {
        self.tips = tips.into_iter().map(Into::into).collect();
        self
    }

    /// The selected category index (accent outline on its cells).
    pub fn selected(mut self, cat: Option<usize>) -> Self {
        self.selected = cat;
        self
    }

    /// Shared "hovered category" state — the legend writes it so its rows
    /// light up matching cells; cells write it on their own hover.
    pub fn hovered(mut self, state: Entity<Option<usize>>) -> Self {
        self.hovered = Some(state);
        self
    }

    /// Called when a cell click or Enter/Space toggles its category;
    /// `None` clears the filter (Escape).
    pub fn on_select(mut self, f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Waffle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let gap = px(4.);
        let cols = self.cols;
        let n = self.cells.len();
        let cat_of = |ix: usize| self.cats.get(ix).copied().unwrap_or(0);

        let cat_hover = self.hovered.unwrap_or_else(|| {
            window.use_keyed_state((id.clone(), "cat-hovered"), cx, |_, _| None::<usize>)
        });
        let cell_ix = window.use_keyed_state((id.clone(), "cell-ix"), cx, |_, _| usize::MAX);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus.is_focused(window);
        let track_w = window
            .use_keyed_state((id.clone(), "track-w"), cx, |_, _| {
                Rc::new(RefCell::new(px(0.)))
            })
            .read(cx)
            .clone();

        let hovered_cat = *cat_hover.read(cx);
        let sel = self.selected;
        let selectable = self.on_select.is_some();
        let on_select = self.on_select;
        // `cats` must live in the 'static listeners too — share it.
        let cats = self.cats.clone();

        // The pill rides the hovered cell; a legend hover (whose category
        // the pointed cell doesn't carry) anchors on the midpoint of the
        // category's cell run instead.
        let pill = hovered_cat.and_then(|cat| {
            let tw = *track_w.borrow();
            let text = self.tips.get(cat)?.clone();
            if tw <= px(0.) || n == 0 {
                return None;
            }
            let cell_ix = *cell_ix.read(cx);
            let anchor_ix = if cell_ix < n && cat_of(cell_ix) == cat {
                cell_ix
            } else {
                let first = (0..n).find(|&ix| cat_of(ix) == cat).unwrap_or(0);
                let last = (0..n).rev().find(|&ix| cat_of(ix) == cat).unwrap_or(first);
                (first + last) / 2
            };
            let frac = ((anchor_ix % cols) as f32 + 0.5) / cols as f32;
            Some(hover_pill(frac, tw, text, cx))
        });

        let cell_style = |cell: WaffleCell| match cell {
            WaffleCell::Full => div().bg(super::Accent::solid(cx)),
            WaffleCell::Mid => div().bg(super::Accent::mid(cx)),
            WaffleCell::Light => div().bg(super::Accent::faint(cx)),
            WaffleCell::Faint => div().bg(super::Accent::soft(cx)),
            WaffleCell::Empty => div().border_1().border_color(cx.theme().border),
        };
        // The cell the grid's arrow keys stand on: `cell_ix` doubles as
        // the pointer's last cell, so keyboard navigation picks up where
        // the mouse left off; usize::MAX = not set yet → first cell.
        let focus_cell = match *cell_ix.read(cx) {
            usize::MAX => 0,
            ix => ix.min(n.saturating_sub(1)),
        };

        v_flex()
            .id(id.clone())
            .gap_2()
            .child(div().relative().h(px(20.)).w_full().children(pill))
            .child(
                v_flex()
                    .when(selectable, |this| this.tab_index(0).track_focus(&focus))
                    .on_prepaint({
                        let track_w = track_w.clone();
                        move |bounds, _, _| {
                            *track_w.borrow_mut() = bounds.size.width;
                        }
                    })
                    .on_key_down({
                        let on_select = on_select.clone();
                        let cell_ix = cell_ix.clone();
                        let cats = cats.clone();
                        move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            let cur = match *cell_ix.read(cx) {
                                usize::MAX => 0,
                                ix => ix.min(n.saturating_sub(1)),
                            };
                            let next = match key {
                                "left" => Some(cur.saturating_sub(1)),
                                "right" => Some((cur + 1).min(n.saturating_sub(1))),
                                "up" => Some(cur.saturating_sub(cols)),
                                "down" => Some((cur + cols).min(n.saturating_sub(1))),
                                "home" => Some(0),
                                "end" => Some(n.saturating_sub(1)),
                                _ => None,
                            };
                            if let Some(next) = next {
                                cx.stop_propagation();
                                cell_ix.update(cx, |ix, cx| {
                                    *ix = next;
                                    cx.notify();
                                });
                                return;
                            }
                            let Some(on_select) = &on_select else { return };
                            match key {
                                "enter" | "space" => {
                                    cx.stop_propagation();
                                    let cat = cats.get(cur).copied().unwrap_or(0);
                                    on_select((sel != Some(cat)).then_some(cat), window, cx);
                                }
                                "escape" => {
                                    cx.stop_propagation();
                                    on_select(None, window, cx);
                                }
                                _ => {}
                            }
                        }
                    })
                    .gap(gap)
                    .children(self.cells.chunks(cols).enumerate().map(|(row_ix, row)| {
                        h_flex()
                            .when(row.len() == cols, |this| this.justify_between())
                            .when(row.len() != cols, |this| this.gap(gap))
                            .children(row.iter().enumerate().map(|(col_ix, cell)| {
                                let ix = row_ix * cols + col_ix;
                                let cat = cats.get(ix).copied().unwrap_or(0);
                                let dim = hovered_cat.is_some_and(|c| c != cat);
                                let is_focus_cell = is_focused && selectable && focus_cell == ix;
                                cell_style(*cell)
                                    .id((id.clone(), ix.to_string()))
                                    .size(self.cell)
                                    .rounded(px(4.))
                                    .flex_shrink_0()
                                    .cursor_pointer()
                                    .when(dim, |this| this.opacity(0.35))
                                    .when(hovered_cat == Some(cat), |this| {
                                        this.shadow_sm()
                                            .border_1()
                                            .border_color(super::Accent::solid(cx))
                                    })
                                    .when(sel == Some(cat), |this| {
                                        this.border_1().border_color(super::Accent::solid(cx))
                                    })
                                    .when(is_focus_cell, |this| this.focus_ring_style(window, cx))
                                    .on_hover({
                                        let cat_hover = cat_hover.clone();
                                        let cell_ix = cell_ix.clone();
                                        move |entered, _window, cx| {
                                            cat_hover.update(cx, |h, cx| {
                                                *h = entered.then_some(cat);
                                                cx.notify();
                                            });
                                            if *entered {
                                                cell_ix.update(cx, |ix_, cx| {
                                                    *ix_ = ix;
                                                    cx.notify();
                                                });
                                            }
                                        }
                                    })
                                    .on_click({
                                        let on_select = on_select.clone();
                                        move |_, window, cx| {
                                            if let Some(on_select) = &on_select {
                                                on_select(
                                                    (sel != Some(cat)).then_some(cat),
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }
                                    })
                            }))
                    })),
            )
    }
}

/// One aging column: `filled` of 5 horizontal bars in `color`, labeled
/// below with its bucket and mono amount. `tip` is the hover tooltip.
pub struct AgingGroup {
    filled: usize,
    color: Hsla,
    label: SharedString,
    amount: SharedString,
    tip: Option<SharedString>,
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
            tip: None,
        }
    }

    /// Hover tooltip for the column (bucket detail + share).
    pub fn tip(mut self, tip: impl Into<SharedString>) -> Self {
        self.tip = Some(tip.into());
        self
    }
}

/// The aging strip: columns of horizontal bars, fullest on the left.
/// Columns hover (wash pill + tooltip), click to filter, and the strip is
/// a focus stop whose arrows move the selection (Escape clears).
#[derive(IntoElement)]
pub struct AgingBars {
    id: ElementId,
    groups: Vec<AgingGroup>,
    selected: Option<usize>,
    on_select: Option<super::OptPickCallback>,
}

impl AgingBars {
    pub fn new(id: impl Into<ElementId>, groups: Vec<AgingGroup>) -> Self {
        Self {
            id: id.into(),
            groups,
            selected: None,
            on_select: None,
        }
    }

    /// The filtered bucket (soft wash on its column).
    pub fn selected(mut self, ix: Option<usize>) -> Self {
        self.selected = ix;
        self
    }

    /// Called when a column click toggles or arrow keys move the bucket;
    /// `None` clears the filter (Escape).
    pub fn on_select(mut self, f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for AgingBars {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let track = cx.theme().border;
        let n = self.groups.len().max(1);

        let hovered = window.use_keyed_state((id.clone(), "hovered"), cx, |_, _| None::<usize>);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus.is_focused(window);
        let track_w = window
            .use_keyed_state((id.clone(), "track-w"), cx, |_, _| {
                Rc::new(RefCell::new(px(0.)))
            })
            .read(cx)
            .clone();
        let hovered_ix = *hovered.read(cx);

        let sel = self.selected;
        let selectable = self.on_select.is_some();
        let on_select = self.on_select;

        let pill = hovered_ix.and_then(|ix| {
            let tw = *track_w.borrow();
            let text = self.groups.get(ix).and_then(|g| g.tip.clone())?;
            (tw > px(0.)).then(|| hover_pill((ix as f32 + 0.5) / n as f32, tw, text, cx))
        });

        v_flex()
            .id(id.clone())
            .gap_2()
            .child(div().relative().h(px(20.)).w_full().children(pill))
            .child(
                h_flex()
                    .id((id.clone(), "strip"))
                    .gap_3()
                    .items_start()
                    .on_prepaint({
                        let track_w = track_w.clone();
                        move |bounds, _, _| {
                            *track_w.borrow_mut() = bounds.size.width;
                        }
                    })
                    .when(selectable, |this| {
                        this.tab_index(0).track_focus(&focus).on_key_down({
                            let on_select = on_select.clone();
                            move |event, window, cx| {
                                let Some(on_select) = &on_select else { return };
                                let cur = sel.unwrap_or(0).min(n - 1);
                                let next = match event.keystroke.key.as_str() {
                                    "left" => Some(cur.saturating_sub(1)),
                                    "right" => Some((cur + 1).min(n - 1)),
                                    "home" => Some(0),
                                    "end" => Some(n - 1),
                                    "escape" => None,
                                    _ => return,
                                };
                                cx.stop_propagation();
                                on_select(next, window, cx);
                            }
                        })
                    })
                    .children(self.groups.into_iter().enumerate().map(|(ix, group)| {
                        let is_hover = hovered_ix == Some(ix);
                        let is_sel = sel == Some(ix);
                        v_flex()
                            .id((id.clone(), ix.to_string()))
                            .flex_1()
                            .gap_1p5()
                            .px_1()
                            .mx(px(-4.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .when(is_hover, |this| this.bg(super::Accent::wash(cx)))
                            .when(is_sel && !is_hover, |this| this.bg(super::Accent::soft(cx)))
                            .when(is_focused && sel.unwrap_or(0) == ix, |this| {
                                this.focus_ring_style(window, cx)
                            })
                            .on_hover({
                                let hovered = hovered.clone();
                                move |entered, _window, cx| {
                                    hovered.update(cx, |h, cx| {
                                        *h = entered.then_some(ix);
                                        cx.notify();
                                    });
                                }
                            })
                            .on_click({
                                let on_select = on_select.clone();
                                move |_, window, cx| {
                                    if let Some(on_select) = &on_select {
                                        on_select((sel != Some(ix)).then_some(ix), window, cx);
                                    }
                                }
                            })
                            .children((0..5).map(|bar| {
                                div().w_full().h(px(6.)).rounded_full().mb(px(2.)).bg(
                                    if bar < group.filled {
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
                    })),
            )
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
