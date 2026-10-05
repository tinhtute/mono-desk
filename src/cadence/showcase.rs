//! The showcase — the reference's eight cards re-assembled on one canvas,
//! mounted as the "Finance UI" settings page. Every interaction writes to
//! [`DemoState`](super::DemoState); copy is the reference's own sample data.

use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Sizable, Size, StyledExt as _, WindowExt, h_flex,
    notification::Notification,
    setting::{SettingGroup, SettingItem, SettingPage},
    v_flex,
};
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px, relative,
};

use super::blocks::{
    CadenceCheckbox, CadenceToggle, CardHeader, EmptyState, FilterChip, Meter, MonoPill, PillTone,
    Stat, btn_primary, btn_secondary, card, legend_dot, legend_swatch, mono, pie_avatar, stat_row,
};
use super::charts::{
    AgingBars, AgingGroup, BarChartMini, Gantt, GanttSeg, MiniBars, RangeSlider, Waffle, WaffleCell,
};
use super::controls::{Segment, Segmented, Tab, Tabs};
use super::{Accent, DemoState};
use crate::i18n::tr;

/// The "Finance UI" settings page: one untitled group whose single element
/// item is the whole canvas.
pub fn page(lang: &str) -> SettingPage {
    SettingPage::new(tr("page.finance", lang))
        .icon(Icon::new(IconName::LayoutDashboard))
        .description(tr("page.finance.desc", lang))
        .group(SettingGroup::new().item(SettingItem::render(|_options, _window, cx| canvas(cx))))
}

/// The gray canvas with the eight cloned cards in two columns, mirroring
/// the reference's art board. Fixed card widths; `flex_wrap` stacks the
/// side card below its main card in narrow windows.
fn canvas(cx: &App) -> AnyElement {
    v_flex()
        .w_full()
        .gap_4()
        .rounded(px(12.))
        .bg(cx.theme().background)
        .p(px(20.))
        .child(row(purchase_orders(cx), filter_panel(cx)))
        .child(row(upcoming_bills(cx), owed_to_suppliers(cx)))
        .child(row(expense_claims(cx), by_category(cx)))
        .child(row(billable_expenses(cx), assign_to_project(cx)))
        .into_any_element()
}

fn row(main: AnyElement, side: AnyElement) -> AnyElement {
    h_flex()
        .items_start()
        .flex_wrap()
        .gap_4()
        .child(main)
        .child(side)
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 1 — Purchase orders (tabs, filter chips, gantt table, footer)
// ---------------------------------------------------------------------------

const PO_AMOUNTS: [f64; 3] = [6120.00, 4480.00, 1960.00];

fn purchase_orders(cx: &App) -> AnyElement {
    let state = DemoState::global(cx);
    let selected = state.po_rows;
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

    let table = v_flex()
        .px_4()
        .child(
            // Header row; the gantt cell carries the "Today" marker line.
            h_flex()
                .gap_3()
                .py_2()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Supplier"),
                )
                .child(gantt_head(cx))
                .child(
                    div()
                        .w(px(80.))
                        .text_right()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Amount"),
                ),
        )
        .children(
            rows.into_iter()
                .enumerate()
                .map(|(ix, (name, sub, segments))| {
                    let on = selected[ix];
                    h_flex()
                        .gap_3()
                        .px_2()
                        .py_2()
                        .rounded(px(8.))
                        .when(on, |this| this.bg(Accent::wash(cx)))
                        .child(
                            CadenceCheckbox::new(SharedString::from(format!("po-row-{ix}")), on)
                                .on_click(move |checked, _window, cx| {
                                    DemoState::update(cx, |s| s.po_rows[ix] = *checked);
                                }),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .text_color(cx.theme().foreground)
                                        .child(name),
                                )
                                .child(
                                    mono(sub, cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .child(gantt_cell(cx, segments))
                        .child(
                            mono(usd(PO_AMOUNTS[ix]), cx)
                                .w(px(80.))
                                .text_right()
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground),
                        )
                }),
        );

    card(cx)
        .w(px(500.))
        .child(
            CardHeader::new(
                IconName::FileText,
                "Purchase orders",
                "September · 24 orders",
            )
            .trailing(MonoPill::new("2 need you")),
        )
        .child(
            Tabs::new(
                "po-tabs",
                vec![
                    Tab::new("All").count(24),
                    Tab::new("Draft").count(3),
                    Tab::new("Awaiting").count(2),
                    Tab::new("Approved").count(11),
                    Tab::new("Billed").count(8),
                ],
            )
            .active(state.po_tab)
            .on_pick(|ix, _window, cx| DemoState::update(cx, |s| s.po_tab = ix)),
        )
        .child(
            h_flex()
                .px_4()
                .py_3()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .when(state.date_filter, |this| {
                            this.child(
                                FilterChip::new("po-date-chip", "Sep 1 – Sep 30")
                                    .icon(IconName::Calendar)
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
                    Icon::new(IconName::Search)
                        .with_size(Size::Size(px(16.)))
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .child(table)
        .child(
            h_flex()
                .px_4()
                .py_2()
                .justify_between()
                .child(
                    h_flex()
                        .gap_4()
                        .child(legend_dot(Accent::solid(cx), "Billed", cx))
                        .child(legend_dot(Accent::faint(cx), "Ordered", cx))
                        .child(outline_legend("Pending", cx))
                        .child(ring_legend("Delivery", cx)),
                )
                .child(
                    mono("1 col = 1 week", cx)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .child(
            h_flex()
                .px_4()
                .py_3()
                .justify_between()
                .items_center()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child(SharedString::from(format!("{picked} selected"))),
                        )
                        .child(
                            mono(usd(total), cx)
                                .text_sm()
                                .text_color(cx.theme().muted_foreground),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(btn_secondary("po-mark-billed", "Mark billed"))
                        .child(btn_primary("po-approve", "Approve 2", cx)),
                ),
        )
        .into_any_element()
}

/// The gantt column of one table row: fixed 220px track with the "Today"
/// marker line running through it (segmented per row, so it reads as one
/// continuous line down the table).
fn gantt_cell(cx: &App, segments: Vec<GanttSeg>) -> AnyElement {
    div()
        .w(px(200.))
        .relative()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-4.))
                .bottom(px(-4.))
                .w(px(1.))
                .bg(Accent::solid(cx).alpha(0.35)),
        )
        .child(Gantt::new(segments))
        .into_any_element()
}

/// The gantt column header: week ticks either side of the "Today" pill,
/// with the marker line running through the column.
fn gantt_head(cx: &App) -> AnyElement {
    div()
        .w(px(200.))
        .relative()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-4.))
                .bottom(px(-4.))
                .w(px(1.))
                .bg(Accent::solid(cx).alpha(0.35)),
        )
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    mono("Sep 1", cx)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                )
                .child(MonoPill::new("Today").solid())
                .child(
                    mono("Oct 1", cx)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .into_any_element()
}

fn outline_legend(label: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .border_1()
                .border_color(cx.theme().muted_foreground),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .into_any_element()
}

fn ring_legend(label: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .border_2()
                .border_color(Accent::solid(cx))
                .bg(cx.theme().popover),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 2 — Filter panel (date inputs, histogram + slider, toggle, types)
// ---------------------------------------------------------------------------

fn filter_panel(cx: &App) -> AnyElement {
    let state = DemoState::global(cx);
    let doc_selected = (state.doc_bills as u64) + (state.doc_credit_notes as u64);

    let section_label = |text: &'static str, cx: &App| {
        div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(text)
    };

    let date_input = |id: &'static str, text: &'static str, focused: bool, cx: &App| {
        h_flex()
            .id(id)
            .flex_1()
            .h(px(36.))
            .rounded(px(10.))
            .border_1()
            .border_color(if focused {
                Accent::solid(cx)
            } else {
                cx.theme().border
            })
            .bg(cx.theme().popover)
            .px_2()
            .gap_1p5()
            .items_center()
            .child(
                Icon::new(IconName::Calendar)
                    .with_size(Size::Size(px(14.)))
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(text),
            )
    };

    card(cx)
        .w(px(330.))
        .child(
            CardHeader::new(IconName::Settings2, "Filter", "Bills and credit notes")
                .trailing(close_button("filter-close", cx)),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .gap_4()
                .child(
                    v_flex().gap_2().child(section_label("Due Date", cx)).child(
                        h_flex()
                            .gap_2()
                            .child(date_input("filter-date-from", "Sep 1", false, cx))
                            .child(date_input("filter-date-to", "Sep 30", true, cx)),
                    ),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .justify_between()
                                .child(section_label("Amount", cx))
                                .child(
                                    mono("$500 - $12k", cx)
                                        .text_sm()
                                        .font_medium()
                                        .text_color(cx.theme().foreground),
                                ),
                        )
                        .child(
                            MiniBars::new(FILTER_HISTOGRAM.to_vec())
                                .height(px(44.))
                                .bar_w(px(5.))
                                .accent_range((6, 21)),
                        )
                        .child(RangeSlider::new(state.slider.0, state.slider.1))
                        .child(
                            h_flex()
                                .justify_between()
                                .children(["$0", "$5k", "$20k+"].map(|t| {
                                    mono(t, cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                })),
                        ),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Overdue only"),
                        )
                        .child(
                            CadenceToggle::new("filter-overdue", state.overdue_only).on_click(
                                |checked, _window, cx| {
                                    DemoState::update(cx, |s| s.overdue_only = *checked);
                                },
                            ),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(section_label("Document Types", cx))
                                .when(doc_selected > 0, |this| {
                                    this.child(MonoPill::new(format!("{doc_selected} selected")))
                                }),
                        )
                        .child(doc_row(
                            "filter-doc-bills",
                            "Bills",
                            state.doc_bills,
                            |cx| {
                                DemoState::update(cx, |s| s.doc_bills = !s.doc_bills);
                            },
                            cx,
                        ))
                        .child(doc_row(
                            "filter-doc-credit",
                            "Credit notes",
                            state.doc_credit_notes,
                            |cx| {
                                DemoState::update(cx, |s| s.doc_credit_notes = !s.doc_credit_notes);
                            },
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .pt_1()
                        .child(btn_secondary("filter-reset", "Reset").flex_1())
                        .child(btn_primary("filter-show", "Show 142 bills", cx).flex_1()),
                ),
        )
        .into_any_element()
}

fn doc_row(
    id: &'static str,
    label: &'static str,
    checked: bool,
    toggle: impl Fn(&mut App) + 'static,
    cx: &App,
) -> AnyElement {
    h_flex()
        .justify_between()
        .items_center()
        .py_1()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    CadenceCheckbox::new(id, checked).on_click(move |_, _window, cx| {
                        toggle(cx);
                    }),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(label),
                ),
        )
        .into_any_element()
}

fn close_button(id: &'static str, cx: &App) -> AnyElement {
    div()
        .id(id)
        .cursor_pointer()
        .size(px(28.))
        .rounded(px(8.))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .shadow_xs()
        .flex()
        .items_center()
        .justify_center()
        .hover(|this| this.bg(cx.theme().accent))
        .on_click(|_, window, cx| {
            window.push_notification(Notification::info("Panel closed (demo)"), cx);
        })
        .child(
            Icon::new(IconName::Close)
                .with_size(Size::Size(px(14.)))
                .text_color(cx.theme().muted_foreground),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 3 — Upcoming bills (stat strip, stacked chart, sparkline strip)
// ---------------------------------------------------------------------------

fn upcoming_bills(cx: &App) -> AnyElement {
    card(cx)
        .w(px(500.))
        .child(
            CardHeader::new(
                IconName::Calendar,
                "Upcoming bills",
                "Due by day · next five weeks",
            )
            .trailing(
                h_flex()
                    .gap_3()
                    .child(legend_swatch(Accent::solid(cx), "Scheduled", cx))
                    .child(legend_swatch(Accent::faint(cx), "Unscheduled", cx)),
            ),
        )
        .child(div().px_4().py_2().child(stat_row(
            vec![
                Stat::new("Draft", "$24,185").sub("13 bills"),
                Stat::new("Awaiting Approval", "$8,940").sub("4 bills"),
                Stat::new("Awaiting Payment", "$184,920").sub("38 bills"),
                Stat::new("Overdue", "$12,360").sub("5 bills").accent(),
            ],
            cx,
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
                    .bg(cx.theme().muted)
                    .p(px(10.))
                    .child(
                        MiniBars::new(SPARK_VALUES.to_vec())
                            .height(px(24.))
                            .bar_w(px(2.))
                            .accent_range((18, 38))
                            .band((16, 40)),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .px_1()
                            .children(["Sep", "Oct", "Nov"].map(|t| {
                                mono(t, cx)
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                            })),
                    ),
            ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 4 — Owed to suppliers (big stat, aging bars, supplier rows)
// ---------------------------------------------------------------------------

fn owed_to_suppliers(cx: &App) -> AnyElement {
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

    card(cx)
        .w(px(330.))
        .child(CardHeader::new(
            IconName::LayoutDashboard,
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
                        .child(
                            mono("$412,860", cx)
                                .text_2xl()
                                .font_semibold()
                                .text_color(cx.theme().foreground),
                        )
                        .child(
                            mono("+$18.4k this week", cx)
                                .text_sm()
                                .text_color(Accent::ink(cx)),
                        ),
                )
                .child(AgingBars::new(vec![
                    AgingGroup::new(5, Accent::solid(cx), "Now", "$218k"),
                    AgingGroup::new(3, Accent::solid(cx), "1–30", "$124k"),
                    AgingGroup::new(2, Accent::solid(cx), "31–60", "$42k"),
                    AgingGroup::new(1, Accent::faint(cx), "61–90", "$16k"),
                    AgingGroup::new(1, Accent::faint(cx), "90+", "$12k"),
                ]))
                .child(div().h(px(1.)).w_full().bg(cx.theme().border))
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child("Top Suppliers"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Overdue"),
                        ),
                )
                .children(
                    suppliers
                        .into_iter()
                        .map(|(name, sub, amount, overdue, frac)| {
                            h_flex()
                                .gap_3()
                                .items_center()
                                .py_1()
                                .child(pie_avatar(frac, px(28.), cx))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().foreground)
                                                .child(name),
                                        )
                                        .child(
                                            mono(sub, cx)
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .items_end()
                                        .gap_0p5()
                                        .child(
                                            mono(amount, cx)
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().foreground),
                                        )
                                        .children(overdue.map(|o| {
                                            mono(o, cx).text_sm().text_color(Accent::ink(cx))
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

fn expense_claims(cx: &App) -> AnyElement {
    let state = DemoState::global(cx);
    let claims = [
        (
            IconName::Map,
            "Flights LIS → BER",
            "Mara K. · Sep 18 · 4200 Travel",
            "$412.00",
        ),
        (
            IconName::FileText,
            "Client dinner · Lisbon",
            "Mara K. · Sep 18 · 6230 Meals",
            "$184.00",
        ),
        (
            IconName::Map,
            "Mileage · 214 km",
            "Yuri G. · Sep 21 · 4250 Mileage",
            "$688.00",
        ),
    ];

    card(cx)
        .w(px(500.))
        .child(
            CardHeader::new(
                IconName::FileText,
                "Expense claims",
                "Reimbursements · September",
            )
            .trailing(MonoPill::new("3 to pay")),
        )
        .child(
            h_flex()
                .px_4()
                .py_2()
                .justify_between()
                .items_center()
                .child(
                    Segmented::new(
                        "claims-seg",
                        vec![
                            Segment::new("Your own"),
                            Segment::new("To review").count(2),
                            Segment::new("To pay").count(3),
                        ],
                    )
                    .active(state.claims_tab)
                    .on_pick(|ix, _window, cx| DemoState::update(cx, |s| s.claims_tab = ix)),
                )
                .child(FilterChip::new("claims-month", "Sep 2026").caret()),
        )
        .child(
            div().px_4().py_3().child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_8()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Submitted"),
                                    )
                                    .child(
                                        mono("$2,316.40", cx)
                                            .text_xl()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("To Be Paid"),
                                    )
                                    .child(
                                        mono("$1,284.00", cx)
                                            .text_xl()
                                            .font_semibold()
                                            .text_color(Accent::ink(cx)),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Reimbursed"),
                                    )
                                    .child(
                                        mono("$1,032.40", cx)
                                            .text_xl()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground),
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
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Awaiting Payment"),
                )
                .child(
                    mono("3 claims", cx)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .children(claims.into_iter().enumerate().map(
                    |(ix, (icon, title, sub, amount))| {
                        h_flex()
                            .gap_3()
                            .items_center()
                            .py_1p5()
                            .child(
                                div()
                                    .size(px(32.))
                                    .flex_shrink_0()
                                    .rounded(px(8.))
                                    .bg(cx.theme().accent)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(cx.theme().foreground)
                                    .child(Icon::new(icon).with_size(Size::Size(px(16.)))),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(cx.theme().foreground)
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(sub),
                                    ),
                            )
                            .child(
                                mono(amount, cx)
                                    .text_sm()
                                    .font_medium()
                                    .text_color(cx.theme().foreground),
                            )
                            .child(btn_secondary(
                                SharedString::from(format!("claim-pay-{ix}")),
                                "Pay",
                            ))
                    },
                )),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 6 — By category (big stat, waffle, legend)
// ---------------------------------------------------------------------------

fn by_category(cx: &App) -> AnyElement {
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

    card(cx)
        .w(px(330.))
        .child(CardHeader::new(
            IconName::ChartPie,
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
                        .child(
                            mono("$18,420", cx)
                                .text_2xl()
                                .font_semibold()
                                .text_color(cx.theme().foreground),
                        )
                        .child(mono("+12% QoQ", cx).text_sm().text_color(Accent::ink(cx))),
                )
                .child(Waffle::new(cells).cols(10))
                .child(div().h(px(1.)).w_full().bg(cx.theme().border))
                .children(legend.into_iter().map(|(cell, name, count, amount)| {
                    h_flex()
                        .justify_between()
                        .items_center()
                        .py_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(waffle_dot(cell, cx))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().foreground)
                                        .child(name),
                                )
                                .child(
                                    mono(count.to_string(), cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .child(
                            mono(amount, cx)
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground),
                        )
                })),
        )
        .into_any_element()
}

fn waffle_dot(cell: WaffleCell, cx: &App) -> AnyElement {
    let bg = match cell {
        WaffleCell::Full => Some(Accent::solid(cx)),
        WaffleCell::Mid => Some(Accent::mid(cx)),
        WaffleCell::Light => Some(Accent::faint(cx)),
        WaffleCell::Faint => Some(Accent::soft(cx)),
        WaffleCell::Empty => None,
    };
    let mut dot = div().size(px(10.)).rounded(px(3.));
    dot = match bg {
        Some(color) => dot.bg(color),
        None => dot.border_1().border_color(cx.theme().border),
    };
    dot.into_any_element()
}

// ---------------------------------------------------------------------------
// Card 7 — Billable expenses (empty state with illustration)
// ---------------------------------------------------------------------------

fn billable_expenses(cx: &App) -> AnyElement {
    card(cx)
        .w(px(500.))
        .child(
            CardHeader::new(
                IconName::GalleryVerticalEnd,
                "Billable expenses",
                "Rebill costs to client projects",
            )
            .trailing(
                mono("0 to invoice", cx)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
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
                    .bg(cx.theme().muted)
                    .items_center()
                    .justify_center()
                    .flex()
                    .child(rebill_illustration(cx)),
            )
            .action(btn_primary("rebill-assign", "Assign expenses", cx))
            .action(btn_secondary("rebill-markup", "Set markup"))
            .aside(
                mono("Default markup 10%", cx)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            ),
        )
        .into_any_element()
}

/// Abstract stand-in for the reference's isometric illustration: a device
/// slab, a stack of cards, a floating card and a soft square, all divs.
fn rebill_illustration(cx: &App) -> AnyElement {
    div()
        .relative()
        .size(px(220.))
        .child(
            // Device slab.
            div()
                .absolute()
                .left_0()
                .bottom_0()
                .size(px(150.))
                .rounded(px(16.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_xs(),
        )
        .child(
            // Soft square on the slab.
            div()
                .absolute()
                .left(px(84.))
                .bottom(px(24.))
                .size(px(52.))
                .rounded(px(8.))
                .bg(Accent::soft(cx)),
        )
        .child(
            // Card stack.
            v_flex()
                .absolute()
                .left(px(20.))
                .top(px(28.))
                .gap(px(3.))
                .children((0..3).map(|ix| {
                    div()
                        .w(px(88.))
                        .h(px(22.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(if ix == 0 {
                            Accent::soft(cx)
                        } else {
                            cx.theme().popover
                        })
                })),
        )
        .child(
            // Floating card.
            div()
                .absolute()
                .right_0()
                .top_0()
                .w(px(96.))
                .h(px(64.))
                .rounded(px(10.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
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
                                .bg(Accent::solid(cx)),
                        )
                        .child(
                            div()
                                .w(px(64.))
                                .h(px(5.))
                                .rounded_full()
                                .bg(Accent::faint(cx)),
                        )
                        .child(
                            div()
                                .w(px(52.))
                                .h(px(5.))
                                .rounded_full()
                                .bg(Accent::faint(cx)),
                        ),
                ),
        )
        .child(
            // Dashed connector (hairline stand-in).
            div()
                .absolute()
                .left(relative(0.62))
                .top(px(64.))
                .w(px(1.))
                .h(px(48.))
                .bg(Accent::solid(cx).alpha(0.4)),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 8 — Assign to project (search, results, expenses, footer)
// ---------------------------------------------------------------------------

fn assign_to_project(cx: &App) -> AnyElement {
    let state = DemoState::global(cx);
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

    card(cx)
        .w(px(330.))
        .child(
            CardHeader::new(
                IconName::FolderClosed,
                "Assign to project",
                "3 expenses selected",
            )
            .trailing(close_button("assign-close", cx)),
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
                        .border_color(Accent::solid(cx))
                        .bg(cx.theme().popover)
                        .px_2()
                        .gap_2()
                        .items_center()
                        .child(
                            Icon::new(IconName::Search)
                                .with_size(Size::Size(px(14.)))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Atl"),
                        )
                        .child(
                            div().flex_1().child(
                                mono("3 results", cx)
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_right(),
                            ),
                        ),
                )
                .children(projects.into_iter().enumerate().map(|(ix, (name, sub))| {
                    let picked = state.assigned == Some(ix);
                    h_flex()
                        .id(SharedString::from(format!("assign-project-{ix}")))
                        .cursor_pointer()
                        .gap_2p5()
                        .items_center()
                        .px_2()
                        .py_1p5()
                        .rounded(px(8.))
                        .when(picked, |this| this.bg(Accent::wash(cx)))
                        .on_click(move |_, _window, cx| {
                            DemoState::update(cx, |s| {
                                s.assigned = if picked { None } else { Some(ix) };
                            });
                        })
                        .child(pie_avatar(0.5 - ix as f32 * 0.12, px(24.), cx))
                        .child(
                            h_flex()
                                .flex_1()
                                .gap_1p5()
                                .items_center()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .text_color(cx.theme().foreground)
                                        .child(name),
                                )
                                .child(
                                    mono(sub, cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .when(picked, |this| {
                            this.child(
                                Icon::new(IconName::Check)
                                    .with_size(Size::Size(px(16.)))
                                    .text_color(Accent::ink(cx)),
                            )
                        })
                }))
                .child(div().h(px(1.)).w_full().bg(cx.theme().border))
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Expenses"),
                        )
                        .child(
                            mono("$984.00", cx)
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground),
                        ),
                )
                .children(
                    expenses
                        .into_iter()
                        .enumerate()
                        .map(|(ix, (title, amount, tag))| {
                            h_flex()
                                .gap_2p5()
                                .items_center()
                                .py_1()
                                .child(
                                    CadenceCheckbox::new(
                                        SharedString::from(format!("assign-expense-{ix}")),
                                        state.expenses[ix],
                                    )
                                    .on_click(
                                        move |checked, _window, cx| {
                                            DemoState::update(cx, |s| s.expenses[ix] = *checked);
                                        },
                                    ),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().foreground)
                                                .child(title),
                                        )
                                        .child(
                                            mono(amount, cx)
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground),
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
                        .child(btn_secondary("assign-cancel", "Cancel"))
                        .child(btn_primary("assign-confirm", "Assign 3", cx).flex_1()),
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
