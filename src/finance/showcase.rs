//! The showcase — the reference's eight cards re-assembled on one canvas,
//! mounted as the "Finance UI" settings page. Every interaction writes to
//! [`DemoState`](super::DemoState); copy is the reference's own sample data.
//!
//! The old stack mounted this as a `SettingPage` whose single element item was
//! the canvas; the zeron stack mounts it inside the host page's `page_column`,
//! so the canvas is now the page body (title/subtitle come from
//! `src/settings/finance.rs`). Widths are scaled from the reference art board
//! (500/330px cards) to fit the settings page's 680px content measure while
//! keeping two columns: 400/252px.

use gpui::prelude::*;
use gpui::{AnyElement, App, IntoElement, RenderOnce, SharedString, Window, div, px, relative};

use zeron_ui::icons;
use zeron_ui::settings::widgets::{self, ActionTone};
use zeron_ui::theme::Theme;

use super::Accent;
use super::blocks::{
    self, CardHeader, EmptyState, FilterChip, Meter, MonoPill, PillTone, Stat, legend_dot,
    legend_swatch, mono, pie_avatar, stat_row,
};
use super::charts::{
    AgingBars, AgingGroup, BarChartMini, Gantt, GanttSeg, MiniBars, RangeSlider, Waffle, WaffleCell,
};
use super::controls::{self, Segment, Tab};
use super::{DemoState, h_flex, v_flex};

/// Art-board card widths, scaled from the reference's 500/330 to the settings
/// page's 680px page measure (two columns + 12px gutters + 8px canvas inset).
const MAIN_CARD: f32 = 400.;
const SIDE_CARD: f32 = 252.;
/// Canvas gutter between the two card columns and between rows.
const CANVAS_GAP: f32 = 12.;
/// Purchase-orders gantt column width.
const GANTT_W: f32 = 104.;
/// Purchase-orders amount column width.
const AMOUNT_W: f32 = 68.;

/// The showcase canvas element. A `RenderOnce` because the old page hosted it
/// from a settings item closure; the host passes only `lang`, so the element
/// pulls the theme and window from its own render.
pub(crate) fn page(_lang: &str) -> AnyElement {
    Showcase.into_any_element()
}

#[derive(IntoElement)]
struct Showcase;

impl RenderOnce for Showcase {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        canvas(&theme, window, cx)
    }
}

/// The gray canvas with the eight cloned cards in two columns, mirroring the
/// reference's art board. The stacks wrap into one column on narrow windows.
fn canvas(theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .w_full()
        .gap(px(CANVAS_GAP))
        .rounded(px(12.))
        .bg(theme.surface)
        .p(px(8.))
        .child(row(
            purchase_orders(theme, window, cx),
            filter_panel(theme, window, cx),
        ))
        .child(row(upcoming_bills(theme), owed_to_suppliers(theme)))
        .child(row(expense_claims(theme, window, cx), by_category(theme)))
        .child(row(
            billable_expenses(theme),
            assign_to_project(theme, window, cx),
        ))
        .into_any_element()
}

fn row(main: AnyElement, side: AnyElement) -> AnyElement {
    h_flex()
        .items_start()
        .flex_wrap()
        .gap(px(CANVAS_GAP))
        .child(main)
        .child(side)
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 1 — Purchase orders (tabs, filter chips, gantt table, footer)
// ---------------------------------------------------------------------------

const PO_AMOUNTS: [f64; 3] = [6120.00, 4480.00, 1960.00];

fn purchase_orders(theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let (po_tab, selected, date_filter) = {
        let state = DemoState::global(cx);
        (state.po_tab, state.po_rows, state.date_filter)
    };
    let picked: u64 = selected.iter().filter(|on| **on).count() as u64;
    let total: f64 = PO_AMOUNTS
        .iter()
        .zip(selected.iter())
        .filter(|(_, on)| **on)
        .map(|(v, _)| *v)
        .sum();

    let rows = [
        (
            "Northwind Freight",
            "PO-0418 · 100% billed",
            vec![GanttSeg::Solid(0.42), GanttSeg::Delivery],
        ),
        (
            "Haldon Supplies",
            "PO-0421 · 40% billed",
            vec![
                GanttSeg::Solid(0.2),
                GanttSeg::Faint(0.2),
                GanttSeg::Delivery,
            ],
        ),
        (
            "Lumen Print Co.",
            "PO-0427 · approval",
            vec![GanttSeg::Dashed(0.42), GanttSeg::Delivery],
        ),
    ];

    let table = v_flex().px_2().child(
        // Header row; the gantt cell carries the "Today" marker line.
        h_flex()
            .gap_2()
            .px_1()
            .py_2()
            .child(div().w(px(widgets::SWITCH_WIDTH)).flex_shrink_0())
            .child(
                div()
                    .flex_1()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Supplier"),
            )
            .child(gantt_head(theme))
            .child(
                div()
                    .w(px(AMOUNT_W))
                    .text_right()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Amount"),
            ),
    );
    let table = rows
        .into_iter()
        .enumerate()
        .fold(table, |table, (ix, row)| {
            let (name, sub, segments) = row;
            let on = selected[ix];
            table.child(
                h_flex()
                    .gap_2()
                    .px_1()
                    .py_1()
                    .rounded(px(8.))
                    .when(on, |this| this.bg(Accent::wash(theme)))
                    .child(blocks::switch(
                        theme,
                        on,
                        SharedString::from(format!("po-row-{ix}")),
                        name,
                        move |checked, _window, cx| {
                            DemoState::update(cx, |s| s.po_rows[ix] = checked)
                        },
                    ))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.text)
                                    .child(name),
                            )
                            .child(mono(sub, theme).text_xs().text_color(theme.text_muted)),
                    )
                    .child(gantt_cell(theme, segments))
                    .child(
                        mono(usd(PO_AMOUNTS[ix]), theme)
                            .w(px(AMOUNT_W))
                            .text_right()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.text),
                    ),
            )
        });

    blocks::card(theme)
        .w(px(MAIN_CARD))
        .child(
            CardHeader::new(icons::DOCUMENT, "Purchase orders", "September · 24 orders")
                .trailing(MonoPill::new("2 need you")),
        )
        .child(controls::tabs(
            theme,
            window,
            cx,
            "po-tabs",
            vec![
                Tab::new("All").count(24),
                Tab::new("Draft").count(3),
                Tab::new("Awaiting").count(2),
                Tab::new("Approved").count(11),
                Tab::new("Billed").count(8),
            ],
            po_tab,
            |ix, _window, cx| DemoState::update(cx, |s| s.po_tab = ix),
        ))
        .child(
            h_flex()
                .px_4()
                .py_3()
                .justify_between()
                .items_center()
                .flex_wrap()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .when(date_filter, |this| {
                            this.child(
                                FilterChip::new("po-date-chip", "Sep 1 – Sep 30")
                                    .icon(icons::CALENDAR)
                                    .on_remove(|_window, cx| {
                                        DemoState::update(cx, |s| s.date_filter = false);
                                    }),
                            )
                        })
                        .child(
                            FilterChip::new("po-supplier-chip", "Any")
                                .prefix("Supplier")
                                .caret(),
                        ),
                )
                .child(
                    icons::icon(icons::MAGNIFER)
                        .size(px(16.))
                        .text_color(theme.text_muted),
                ),
        )
        .child(table)
        .child(
            h_flex()
                .px_4()
                .py_2()
                .justify_between()
                .items_center()
                .flex_wrap()
                .gap_2()
                .child(
                    h_flex()
                        .gap_3()
                        .flex_wrap()
                        .child(legend_dot(Accent::solid(theme), "Billed", theme))
                        .child(legend_dot(Accent::faint(theme), "Ordered", theme))
                        .child(outline_legend("Pending", theme))
                        .child(ring_legend("Delivery", theme)),
                )
                .child(
                    mono("1 col = 1 week", theme)
                        .text_xs()
                        .text_color(theme.text_muted),
                ),
        )
        .child(
            h_flex()
                .px_4()
                .py_3()
                .justify_between()
                .items_center()
                .flex_wrap()
                .gap_2()
                .border_t_1()
                .border_color(theme.border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child(SharedString::from(format!("{picked} selected"))),
                        )
                        .child(
                            mono(usd(total), theme)
                                .text_xs()
                                .text_color(theme.text_muted),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(blocks::button(
                            theme,
                            "po-mark-billed",
                            "Mark billed",
                            ActionTone::Filled,
                        ))
                        .child(blocks::button(
                            theme,
                            "po-approve",
                            "Approve 2",
                            ActionTone::Solid,
                        )),
                ),
        )
        .into_any_element()
}

/// The gantt column of one table row: fixed-width track with the "Today"
/// marker line running through it (segmented per row, so it reads as one
/// continuous line down the table).
fn gantt_cell(theme: &Theme, segments: Vec<GanttSeg>) -> AnyElement {
    div()
        .w(px(GANTT_W))
        .relative()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-4.))
                .bottom(px(-4.))
                .w(px(1.))
                .bg(Accent::solid(theme).alpha(0.35)),
        )
        .child(Gantt::new(segments))
        .into_any_element()
}

/// The gantt column header: the "Today" pill with the marker line running
/// through the column.
fn gantt_head(theme: &Theme) -> AnyElement {
    div()
        .w(px(GANTT_W))
        .relative()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-4.))
                .bottom(px(-4.))
                .w(px(1.))
                .bg(Accent::solid(theme).alpha(0.35)),
        )
        .child(
            h_flex()
                .justify_center()
                .items_center()
                .child(MonoPill::new("Today").solid()),
        )
        .into_any_element()
}

fn outline_legend(label: &'static str, theme: &Theme) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .border_1()
                .border_color(theme.text_muted),
        )
        .child(div().text_xs().text_color(theme.text_muted).child(label))
        .into_any_element()
}

fn ring_legend(label: &'static str, theme: &Theme) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .border_2()
                .border_color(Accent::solid(theme))
                .bg(theme.surface_overlay),
        )
        .child(div().text_xs().text_color(theme.text_muted).child(label))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 2 — Filter panel (date inputs, histogram + slider, switch, types)
// ---------------------------------------------------------------------------

fn filter_panel(theme: &Theme, _window: &mut Window, cx: &mut App) -> AnyElement {
    let (overdue_only, doc_bills, doc_credit_notes, slider) = {
        let state = DemoState::global(cx);
        (
            state.overdue_only,
            state.doc_bills,
            state.doc_credit_notes,
            state.slider,
        )
    };
    let doc_selected = (doc_bills as u64) + (doc_credit_notes as u64);

    let section_label =
        |text: &'static str| div().text_xs().text_color(theme.text_muted).child(text);

    let date_input = |id: &'static str, text: &'static str, focused: bool| {
        h_flex()
            .id(id)
            .flex_1()
            .h(px(36.))
            .rounded(px(10.))
            .border_1()
            .border_color(if focused {
                Accent::solid(theme)
            } else {
                theme.border
            })
            .bg(theme.surface_overlay)
            .px_2()
            .gap_1p5()
            .items_center()
            .tab_index(0)
            .child(
                icons::icon(icons::CALENDAR)
                    .size(px(14.))
                    .text_color(theme.text_muted),
            )
            .child(div().text_xs().text_color(theme.text).child(text))
    };

    blocks::card(theme)
        .w(px(SIDE_CARD))
        .child(
            CardHeader::new(icons::TUNING, "Filter", "Bills and credit notes").trailing(
                blocks::icon_button(theme, "filter-close", icons::CLOSE, "Close filter panel"),
            ),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .gap_4()
                .child(
                    v_flex().gap_2().child(section_label("Due Date")).child(
                        h_flex()
                            .gap_2()
                            .child(date_input("filter-date-from", "Sep 1", false))
                            .child(date_input("filter-date-to", "Sep 30", true)),
                    ),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(section_label("Amount"))
                                .child(
                                    mono("$500 - $12k", theme)
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.text),
                                ),
                        )
                        .child(
                            MiniBars::new(FILTER_HISTOGRAM.to_vec())
                                .height(px(44.))
                                .bar_w(px(5.))
                                .accent_range((6, 21)),
                        )
                        .child(RangeSlider::new(slider.0, slider.1))
                        .child(
                            h_flex()
                                .justify_between()
                                .children(["$0", "$5k", "$20k+"].map(|t| {
                                    mono(t, theme).text_xs().text_color(theme.text_muted)
                                })),
                        ),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(div().text_xs().text_color(theme.text).child("Overdue only"))
                        .child(blocks::switch(
                            theme,
                            overdue_only,
                            "filter-overdue",
                            "Overdue only",
                            |checked, _window, cx| {
                                DemoState::update(cx, |s| s.overdue_only = checked);
                            },
                        )),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(section_label("Document Types"))
                                .when(doc_selected > 0, |this| {
                                    this.child(MonoPill::new(format!("{doc_selected} selected")))
                                }),
                        )
                        .child(doc_row(
                            theme,
                            "filter-doc-bills",
                            "Bills",
                            doc_bills,
                            |cx| {
                                DemoState::update(cx, |s| s.doc_bills = !s.doc_bills);
                            },
                        ))
                        .child(doc_row(
                            theme,
                            "filter-doc-credit",
                            "Credit notes",
                            doc_credit_notes,
                            |cx| {
                                DemoState::update(cx, |s| {
                                    s.doc_credit_notes = !s.doc_credit_notes;
                                });
                            },
                        )),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .pt_1()
                        .child(
                            blocks::button(theme, "filter-reset", "Reset", ActionTone::Filled)
                                .flex_1(),
                        )
                        .child(
                            blocks::button(
                                theme,
                                "filter-show",
                                "Show 142 bills",
                                ActionTone::Solid,
                            )
                            .flex_1(),
                        ),
                ),
        )
        .into_any_element()
}

fn doc_row(
    theme: &Theme,
    id: &'static str,
    label: &'static str,
    checked: bool,
    toggle: impl Fn(&mut App) + 'static,
) -> AnyElement {
    h_flex()
        .justify_between()
        .items_center()
        .py_1()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(blocks::switch(
                    theme,
                    checked,
                    id,
                    label,
                    move |_, _window, cx| {
                        toggle(cx);
                    },
                ))
                .child(div().text_xs().text_color(theme.text).child(label)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 3 — Upcoming bills (stat strip, stacked chart, sparkline strip)
// ---------------------------------------------------------------------------

fn upcoming_bills(theme: &Theme) -> AnyElement {
    blocks::card(theme)
        .w(px(MAIN_CARD))
        .child(
            CardHeader::new(
                icons::CALENDAR,
                "Upcoming bills",
                "Due by day · next five weeks",
            )
            .trailing(
                h_flex()
                    .gap_3()
                    .flex_wrap()
                    .child(legend_swatch(Accent::solid(theme), "Scheduled", theme))
                    .child(legend_swatch(Accent::faint(theme), "Unscheduled", theme)),
            ),
        )
        .child(div().px_4().py_2().child(stat_row(
            vec![
                Stat::new("Draft", "$24,185").sub("13 bills"),
                Stat::new("Awaiting Approval", "$8,940").sub("4 bills"),
                Stat::new("Awaiting Payment", "$184,920").sub("38 bills"),
                Stat::new("Overdue", "$12,360").sub("5 bills").accent(),
            ],
            theme,
        )))
        .child(
            div().px_4().pb_3().child(
                BarChartMini::new(BILL_BARS.to_vec())
                    .y_labels(vec!["$60k", "$40k", "$20k", "0"])
                    .tooltip(8, "Oct 1 · $52k")
                    .x_labels(vec!["Sep 21", "Sep 28", "Oct 5", "Oct 12", "Oct 19"]),
            ),
        )
        .child(
            div().px_4().pb_4().child(
                v_flex()
                    .gap_2()
                    .rounded(px(12.))
                    .bg(theme.band)
                    .p(px(10.))
                    .child(
                        MiniBars::new(SPARK_VALUES.to_vec())
                            .height(px(24.))
                            .bar_w(px(2.))
                            .accent_range((18, 38))
                            .band((16, 40)),
                    )
                    .child(
                        h_flex().justify_between().px_1().children(
                            ["Sep", "Oct", "Nov"]
                                .map(|t| mono(t, theme).text_xs().text_color(theme.text_muted)),
                        ),
                    ),
            ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 4 — Owed to suppliers (big stat, aging bars, supplier rows)
// ---------------------------------------------------------------------------

fn owed_to_suppliers(theme: &Theme) -> AnyElement {
    let suppliers = [
        (
            "Northwind Freight",
            "6 bills · NET 30",
            "$48,210",
            Some("$6,120"),
            0.55,
        ),
        ("Orbit Cloud", "2 bills · NET 15", "$36,000", None, 0.35),
        (
            "Haldon Supplies",
            "9 bills · NET 45",
            "$22,470",
            Some("$4,120"),
            0.25,
        ),
    ];

    blocks::card(theme)
        .w(px(SIDE_CARD))
        .child(CardHeader::new(
            icons::WALLET,
            "Owed to suppliers",
            "Aging · 214 open bills",
        ))
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .gap_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_baseline()
                        .flex_wrap()
                        .child(
                            mono("$412,860", theme)
                                .text_xl()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.text),
                        )
                        .child(
                            mono("+$18.4k this week", theme)
                                .text_xs()
                                .text_color(Accent::ink(theme)),
                        ),
                )
                .child(AgingBars::new(vec![
                    AgingGroup::new(5, Accent::solid(theme), "Now", "$218k"),
                    AgingGroup::new(3, Accent::solid(theme), "1–30", "$124k"),
                    AgingGroup::new(2, Accent::solid(theme), "31–60", "$42k"),
                    AgingGroup::new(1, Accent::faint(theme), "61–90", "$16k"),
                    AgingGroup::new(1, Accent::faint(theme), "90+", "$12k"),
                ]))
                .child(div().h(px(1.)).w_full().bg(theme.border))
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child("Top Suppliers"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Overdue"),
                        ),
                )
                .children(
                    suppliers
                        .into_iter()
                        .map(|(name, sub, amount, overdue, frac)| {
                            h_flex()
                                .gap_2()
                                .items_center()
                                .py_1()
                                .child(pie_avatar(frac, px(28.), theme))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text)
                                                .child(name),
                                        )
                                        .child(
                                            mono(sub, theme).text_xs().text_color(theme.text_muted),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .items_end()
                                        .gap_0p5()
                                        .child(
                                            mono(amount, theme)
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text),
                                        )
                                        .children(overdue.map(|o| {
                                            mono(o, theme).text_xs().text_color(Accent::ink(theme))
                                        })),
                                )
                        }),
                ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 5 — Expense claims (segmented, stat trio + meter, claim rows)
// ---------------------------------------------------------------------------

fn expense_claims(theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let claims_tab = DemoState::global(cx).claims_tab;
    let claims = [
        (
            icons::GLOBE,
            "Flights LIS → BER",
            "Mara K. · Sep 18 · 4200 Travel",
            "$412.00",
        ),
        (
            icons::DOCUMENT,
            "Client dinner · Lisbon",
            "Mara K. · Sep 18 · 6230 Meals",
            "$184.00",
        ),
        (
            icons::GLOBE,
            "Mileage · 214 km",
            "Yuri G. · Sep 21 · 4250 Mileage",
            "$688.00",
        ),
    ];

    blocks::card(theme)
        .w(px(MAIN_CARD))
        .child(
            CardHeader::new(
                icons::DOCUMENT,
                "Expense claims",
                "Reimbursements · September",
            )
            .trailing(MonoPill::new("3 to pay")),
        )
        .child(
            h_flex()
                .px_4()
                .py_2()
                .gap_2()
                .justify_between()
                .items_center()
                .flex_wrap()
                .child(controls::segmented(
                    theme,
                    window,
                    cx,
                    "claims-seg",
                    vec![
                        Segment::new("Your own"),
                        Segment::new("To review").count(2),
                        Segment::new("To pay").count(3),
                    ],
                    claims_tab,
                    |ix, _window, cx| DemoState::update(cx, |s| s.claims_tab = ix),
                ))
                .child(FilterChip::new("claims-month", "Sep 2026").caret()),
        )
        .child(
            div().px_4().py_3().child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_4()
                            .flex_wrap()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child("Submitted"),
                                    )
                                    .child(
                                        mono("$2,316.40", theme)
                                            .text_xl()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child("To Be Paid"),
                                    )
                                    .child(
                                        mono("$1,284.00", theme)
                                            .text_xl()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(Accent::ink(theme)),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child("Reimbursed"),
                                    )
                                    .child(
                                        mono("$1,032.40", theme)
                                            .text_xl()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text),
                                    ),
                            ),
                    )
                    .child(Meter::new(0.57)),
            ),
        )
        .child(
            h_flex()
                .px_4()
                .py_2()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Awaiting Payment"),
                )
                .child(
                    mono("3 claims", theme)
                        .text_xs()
                        .text_color(theme.text_muted),
                ),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .children(claims.into_iter().enumerate().map(
                    |(ix, (icon_path, title, sub, amount))| {
                        h_flex()
                            .gap_2()
                            .items_center()
                            .py_1p5()
                            .child(
                                div()
                                    .size(px(32.))
                                    .flex_shrink_0()
                                    .rounded(px(8.))
                                    .bg(theme.accent_wash)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        icons::icon(icon_path)
                                            .size(px(16.))
                                            .text_color(theme.accent),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text)
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .min_w_0()
                                            .truncate()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child(sub),
                                    ),
                            )
                            .child(
                                mono(amount, theme)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.text),
                            )
                            .child(blocks::button(
                                theme,
                                SharedString::from(format!("claim-pay-{ix}")),
                                "Pay",
                                ActionTone::Filled,
                            ))
                    },
                )),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 6 — By category (big stat, waffle, legend)
// ---------------------------------------------------------------------------

fn by_category(theme: &Theme) -> AnyElement {
    let legend = [
        (WaffleCell::Full, "Travel", 18, "$8,960"),
        (WaffleCell::Mid, "Meals", 13, "$4,210"),
        (WaffleCell::Light, "Software", 10, "$3,480"),
        (WaffleCell::Faint, "Mileage", 5, "$1,190"),
        (WaffleCell::Empty, "Other", 4, "$580"),
    ];
    let mut cells = Vec::new();
    for (cell, _, count, _) in legend {
        cells.extend(std::iter::repeat_n(cell, count));
    }

    blocks::card(theme)
        .w(px(SIDE_CARD))
        .child(CardHeader::new(
            icons::WIDGET,
            "By category",
            "Q3 · 50 claims",
        ))
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .gap_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_baseline()
                        .flex_wrap()
                        .child(
                            mono("$18,420", theme)
                                .text_xl()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.text),
                        )
                        .child(
                            mono("+12% QoQ", theme)
                                .text_xs()
                                .text_color(Accent::ink(theme)),
                        ),
                )
                .child(Waffle::new(cells).cols(10))
                .child(div().h(px(1.)).w_full().bg(theme.border))
                .children(legend.into_iter().map(|(cell, name, count, amount)| {
                    h_flex()
                        .justify_between()
                        .items_center()
                        .py_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(waffle_dot(cell, theme))
                                .child(div().text_xs().text_color(theme.text).child(name))
                                .child(
                                    mono(count.to_string(), theme)
                                        .text_xs()
                                        .text_color(theme.text_muted),
                                ),
                        )
                        .child(
                            mono(amount, theme)
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text),
                        )
                })),
        )
        .into_any_element()
}

fn waffle_dot(cell: WaffleCell, theme: &Theme) -> AnyElement {
    let bg = match cell {
        WaffleCell::Full => Some(Accent::solid(theme)),
        WaffleCell::Mid => Some(Accent::mid(theme)),
        WaffleCell::Light => Some(Accent::faint(theme)),
        WaffleCell::Faint => Some(Accent::soft(theme)),
        WaffleCell::Empty => None,
    };
    let mut dot = div().size(px(10.)).rounded(px(3.));
    dot = match bg {
        Some(color) => dot.bg(color),
        None => dot.border_1().border_color(theme.border),
    };
    dot.into_any_element()
}

// ---------------------------------------------------------------------------
// Card 7 — Billable expenses (empty state with illustration)
// ---------------------------------------------------------------------------

fn billable_expenses(theme: &Theme) -> AnyElement {
    blocks::card(theme)
        .w(px(MAIN_CARD))
        .child(
            CardHeader::new(
                icons::GALLERY,
                "Billable expenses",
                "Rebill costs to client projects",
            )
            .trailing(
                mono("0 to invoice", theme)
                    .text_xs()
                    .text_color(theme.text_muted),
            ),
        )
        .child(
            EmptyState::new(
                "Nothing to rebill yet",
                "Tag receipts and bill lines to a client project — Cadence adds them to the next invoice with your markup.",
            )
            .illustration(
                div()
                    .h(px(200.))
                    .rounded(px(12.))
                    .bg(theme.band)
                    .items_center()
                    .justify_center()
                    .flex()
                    .overflow_hidden()
                    .child(rebill_illustration(theme)),
            )
            .action(blocks::button(
                theme,
                "rebill-assign",
                "Assign expenses",
                ActionTone::Solid,
            ))
            .action(blocks::button(
                theme,
                "rebill-markup",
                "Set markup",
                ActionTone::Filled,
            ))
            .aside(
                mono("Default markup 10%", theme)
                    .text_xs()
                    .text_color(theme.text_muted),
            ),
        )
        .into_any_element()
}

/// Abstract stand-in for the reference's isometric illustration: a device
/// slab, a stack of cards, a floating card and a soft square, all divs.
fn rebill_illustration(theme: &Theme) -> AnyElement {
    div()
        .relative()
        .size(px(190.))
        .child(
            // Device slab.
            div()
                .absolute()
                .left_0()
                .bottom_0()
                .size(px(140.))
                .rounded(px(16.))
                .border_1()
                .border_color(theme.border)
                .bg(theme.surface_overlay)
                .shadow_xs(),
        )
        .child(
            // Soft square on the slab.
            div()
                .absolute()
                .left(px(80.))
                .bottom(px(20.))
                .size(px(48.))
                .rounded(px(8.))
                .bg(Accent::soft(theme)),
        )
        .child(
            // Card stack.
            v_flex()
                .absolute()
                .left(px(18.))
                .top(px(24.))
                .gap(px(3.))
                .children((0..3).map(|ix| {
                    div()
                        .w(px(82.))
                        .h(px(22.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(theme.border)
                        .bg(if ix == 0 {
                            Accent::soft(theme)
                        } else {
                            theme.surface_overlay
                        })
                })),
        )
        .child(
            // Floating card.
            div()
                .absolute()
                .right_0()
                .top_0()
                .w(px(90.))
                .h(px(62.))
                .rounded(px(10.))
                .border_1()
                .border_color(theme.border)
                .bg(theme.surface_overlay)
                .shadow_xs()
                .p(px(8.))
                .child(
                    v_flex()
                        .gap(px(5.))
                        .child(
                            div()
                                .w(px(40.))
                                .h(px(6.))
                                .rounded_full()
                                .bg(Accent::solid(theme)),
                        )
                        .child(
                            div()
                                .w(px(60.))
                                .h(px(5.))
                                .rounded_full()
                                .bg(Accent::faint(theme)),
                        )
                        .child(
                            div()
                                .w(px(48.))
                                .h(px(5.))
                                .rounded_full()
                                .bg(Accent::faint(theme)),
                        ),
                ),
        )
        .child(
            // Dashed connector (hairline stand-in).
            div()
                .absolute()
                .left(relative(0.62))
                .top(px(58.))
                .w(px(1.))
                .h(px(44.))
                .bg(Accent::solid(theme).alpha(0.4)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 8 — Assign to project (search, results, expenses, footer)
// ---------------------------------------------------------------------------

fn assign_to_project(theme: &Theme, _window: &mut Window, cx: &mut App) -> AnyElement {
    let (assigned, expenses_on) = {
        let state = DemoState::global(cx);
        (state.assigned, state.expenses)
    };
    let projects = [
        ("Atlas Studio", "Rebrand"),
        ("Atlas Studio", "Q4 retainer"),
        ("Atlantic Foods", "Launch"),
    ];
    let expenses = [
        ("Flights LIS → BER", "$412.00", "Atlas"),
        ("Hotel · 2 nights", "$388.00", "Atlas"),
        ("Client dinner", "$184.00", "Pending"),
    ];

    blocks::card(theme)
        .w(px(SIDE_CARD))
        .child(
            CardHeader::new(icons::FOLDER, "Assign to project", "3 expenses selected").trailing(
                blocks::icon_button(theme, "assign-close", icons::CLOSE, "Close assign panel"),
            ),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .gap_3()
                .child(
                    // Search field, rendered in its focused state like the
                    // reference ("Atl" typed, 3 results).
                    h_flex()
                        .h(px(36.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(Accent::solid(theme))
                        .bg(theme.surface_overlay)
                        .px_2()
                        .gap_2()
                        .items_center()
                        .child(
                            icons::icon(icons::MAGNIFER)
                                .size(px(14.))
                                .text_color(theme.text_muted),
                        )
                        .child(div().text_xs().text_color(theme.text).child("Atl"))
                        .child(
                            div().flex_1().child(
                                mono("3 results", theme)
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .text_right(),
                            ),
                        ),
                )
                .children(projects.into_iter().enumerate().map(|(ix, (name, sub))| {
                    let picked = assigned == Some(ix);
                    h_flex()
                        .id(SharedString::from(format!("assign-project-{ix}")))
                        .tab_index(0)
                        .role(gpui::Role::RadioButton)
                        .aria_label(format!("{name} · {sub}"))
                        .aria_selected(picked)
                        .cursor_pointer()
                        .gap_2()
                        .items_center()
                        .px_2()
                        .py_1p5()
                        .rounded(px(8.))
                        .when(picked, |this| this.bg(Accent::wash(theme)))
                        .on_click(move |_, _window, cx| {
                            DemoState::update(cx, |s| {
                                s.assigned = if picked { None } else { Some(ix) };
                            });
                        })
                        .child(pie_avatar(0.5 - ix as f32 * 0.12, px(24.), theme))
                        .child(
                            h_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1p5()
                                .flex_wrap()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.text)
                                        .child(name),
                                )
                                .child(mono(sub, theme).text_xs().text_color(theme.text_muted)),
                        )
                        .when(picked, |this| {
                            this.child(
                                icons::icon(icons::CHECK)
                                    .size(px(16.))
                                    .text_color(Accent::ink(theme)),
                            )
                        })
                }))
                .child(div().h(px(1.)).w_full().bg(theme.border))
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Expenses"),
                        )
                        .child(
                            mono("$984.00", theme)
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text),
                        ),
                )
                .children(
                    expenses
                        .into_iter()
                        .enumerate()
                        .map(|(ix, (title, amount, tag))| {
                            h_flex()
                                .gap_2()
                                .items_center()
                                .py_1()
                                .child(blocks::switch(
                                    theme,
                                    expenses_on[ix],
                                    SharedString::from(format!("assign-expense-{ix}")),
                                    title,
                                    move |checked, _window, cx| {
                                        DemoState::update(cx, |s| s.expenses[ix] = checked);
                                    },
                                ))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text)
                                                .child(title),
                                        )
                                        .child(
                                            mono(amount, theme)
                                                .text_xs()
                                                .text_color(theme.text_muted),
                                        ),
                                )
                                .child(MonoPill::new(tag).tone(if tag == "Atlas" {
                                    PillTone::Accent
                                } else {
                                    PillTone::Neutral
                                }))
                        }),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .pt_1()
                        .child(blocks::button(
                            theme,
                            "assign-cancel",
                            "Cancel",
                            ActionTone::Filled,
                        ))
                        .child(
                            blocks::button(theme, "assign-confirm", "Assign 3", ActionTone::Solid)
                                .flex_1(),
                        ),
                ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Demo data + formatting
// ---------------------------------------------------------------------------

/// Filter histogram: ghost bars at the edges, accent bars in the selected
/// $500–$12k window.
const FILTER_HISTOGRAM: [f32; 28] = [
    0.06, 0.10, 0.16, 0.30, 0.46, 0.62, 0.78, 0.92, 1.00, 0.88, 0.72, 0.58, 0.44, 0.34, 0.26, 0.20,
    0.16, 0.22, 0.30, 0.38, 0.30, 0.22, 0.16, 0.12, 0.08, 0.06, 0.04, 0.03,
];

/// Upcoming-bills bars: (scheduled, unscheduled) fractions over 5 weeks.
const BILL_BARS: [(f32, f32); 25] = [
    (0.10, 0.06),
    (0.42, 0.10),
    (0.68, 0.08),
    (0.22, 0.05),
    (0.34, 0.16),
    (0.26, 0.10),
    (0.30, 0.06),
    (0.50, 0.12),
    (0.86, 0.62),
    (0.38, 0.22),
    (0.16, 0.06),
    (0.10, 0.04),
    (0.38, 0.10),
    (0.28, 0.34),
    (0.48, 0.14),
    (0.12, 0.05),
    (0.30, 0.08),
    (0.16, 0.30),
    (0.10, 0.05),
    (0.22, 0.12),
    (0.14, 0.06),
    (0.10, 0.04),
    (0.30, 0.10),
    (0.44, 0.08),
    (0.18, 0.06),
];

/// Sparkline strip: 60 day-bars, the highlighted Oct range in accent.
const SPARK_VALUES: [f32; 60] = [
    0.30, 0.55, 0.35, 0.70, 0.45, 0.25, 0.60, 0.40, 0.75, 0.50, 0.30, 0.65, 0.45, 0.85, 0.55, 0.35,
    0.70, 0.50, 0.30, 0.60, 0.40, 0.80, 0.55, 0.35, 0.65, 0.45, 0.75, 0.50, 0.30, 0.60, 0.40, 0.70,
    0.50, 0.85, 0.60, 0.40, 0.30, 0.55, 0.75, 0.50, 0.65, 0.45, 0.80, 0.55, 0.35, 0.60, 0.40, 0.70,
    0.50, 0.30, 0.55, 0.75, 0.45, 0.65, 0.40, 0.80, 0.55, 0.35, 0.60, 0.45,
];

/// `f64` dollars → the reference's mono format: `$16,480.00`.
fn usd(v: f64) -> String {
    let s = format!("{v:.2}");
    let (int, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let digits = int.as_bytes();
    let mut grouped = String::with_capacity(int.len() + 3);
    for (i, b) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*b as char);
    }
    format!("${grouped}.{frac}")
}

#[cfg(test)]
mod tests {
    use super::usd;

    #[test]
    fn usd_formats_with_thousands_separator() {
        assert_eq!(usd(0.0), "$0.00");
        assert_eq!(usd(412.0), "$412.00");
        assert_eq!(usd(6120.0), "$6,120.00");
        assert_eq!(usd(16480.0), "$16,480.00");
        assert_eq!(usd(412860.0), "$412,860.00");
        assert_eq!(usd(2316.4), "$2,316.40");
    }
}
