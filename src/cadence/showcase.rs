//! The showcase — the reference's eight cards re-assembled on one canvas,
//! mounted as the "Finance UI" settings page. Every interaction writes to
//! [`DemoState`](super::DemoState); copy is the reference's own sample data.

use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable, Size, StyledExt as _, ThemeStyled as _,
    WindowExt as _, h_flex,
    notification::Notification,
    setting::{SettingGroup, SettingItem, SettingPage},
    v_flex,
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, Stateful, StatefulInteractiveElement as _,
    Styled as _, Window, div, prelude::FluentBuilder as _, px, relative,
};

use super::blocks::{
    CadenceCheckbox, CadenceToggle, CardHeader, CloseButton, EmptyState, FilterChip, Meter,
    MonoPill, PillTone, Stat, btn_primary, btn_secondary, card, icon_tile_sized, legend_dot,
    legend_swatch, mono, pie_avatar, stat_row,
};
use super::charts::{
    AgingBars, AgingGroup, BarChartMini, Gantt, GanttSeg, MiniBars, RangeSlider, Waffle, WaffleCell,
};
use super::controls::{Segment, Segmented, Tab, Tabs};
use super::{Accent, DemoState, PoSortField, PoStatus, SortDirection, WindowCallback};
use crate::i18n::tr;

/// The "Finance UI" settings page: one untitled group whose single element
/// item is the whole canvas.
pub fn page(lang: &str) -> SettingPage {
    SettingPage::new(tr("page.finance", lang))
        .icon(Icon::new(IconName::LayoutDashboard))
        .description(tr("page.finance.desc", lang))
        .group(
            SettingGroup::new().item(SettingItem::render(|_options, window, cx| {
                canvas(window, cx)
            })),
        )
}

/// The gray canvas with the eight cloned cards in two columns, mirroring
/// the reference's art board. Fixed card widths; `flex_wrap` stacks the
/// side card below its main card in narrow windows.
fn canvas(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = DemoState::global(cx);
    let show_filter = state.show_filter;
    let show_assign = state.show_assign;
    v_flex()
        .w_full()
        .gap_4()
        .rounded(px(12.))
        .bg(cx.theme().background)
        .p(px(20.))
        .child(row(
            purchase_orders(window, cx),
            if show_filter {
                Some(filter_panel(cx))
            } else {
                Some(
                    RestoreChip::new("filter-restore", "Open Filter", |_window, cx| {
                        DemoState::update(cx, |s| s.show_filter = true);
                    })
                    .into_any_element(),
                )
            },
        ))
        .child(row(upcoming_bills(cx), Some(owed_to_suppliers(cx))))
        .child(row(expense_claims(cx), Some(by_category(cx))))
        .child(row(
            billable_expenses(cx),
            if show_assign {
                Some(assign_to_project(cx))
            } else {
                Some(
                    RestoreChip::new("assign-restore", "Open Assign", |_window, cx| {
                        DemoState::update(cx, |s| s.show_assign = true);
                    })
                    .into_any_element(),
                )
            },
        ))
        .into_any_element()
}

fn row(main: AnyElement, side: Option<AnyElement>) -> AnyElement {
    h_flex()
        .items_start()
        .flex_wrap()
        .gap_4()
        .child(main)
        .children(side)
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Card 1 — Purchase orders (tabs, filter chips, gantt table, footer)
// ---------------------------------------------------------------------------

fn purchase_orders(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = DemoState::global(cx);
    let picked = state.po_selected_count();
    let total = state.po_selected_total();
    let sort = state.po_sort;
    let po_tab = state.po_tab;
    let date_filter = state.date_filter;
    let awaiting = state.po_awaiting_count();
    // Live per-status counts for the tab pills (tab 0 "All" takes every row).
    let tab_counts: Vec<u64> = [
        None,
        Some(PoStatus::Draft),
        Some(PoStatus::Awaiting),
        Some(PoStatus::Approved),
        Some(PoStatus::Billed),
    ]
    .into_iter()
    .map(|status| {
        state
            .po_items
            .iter()
            .filter(|i| status.is_none_or(|s| i.status == s))
            .count() as u64
    })
    .collect();
    // Owned row view: `window` is borrowed mutably below (keyed focus state),
    // which ends the `state` borrow — collect first.
    let rows: Vec<PoRow> = state
        .po_visible()
        .into_iter()
        .map(|item| PoRow {
            on: state.po_is_selected(&item.code),
            code: item.code.clone(),
            supplier: item.supplier.clone(),
            sub: item.sub.clone(),
            amount: item.amount,
            gantt: item.gantt.clone(),
        })
        .collect();

    let any_on = rows.iter().any(|r| r.on);
    let all_on = !rows.is_empty() && rows.iter().all(|r| r.on);
    // Per-row keyboard focus; keyed state keeps each row's handle stable
    // across re-renders while the row is mounted.
    let handles: Vec<FocusHandle> = rows
        .iter()
        .map(|row| {
            window
                .use_keyed_state(
                    ElementId::Name(format!("po-focus-{}", row.code).into()),
                    cx,
                    |_w, cx| cx.focus_handle(),
                )
                .read(cx)
                .clone()
        })
        .collect();

    let table = v_flex()
        .px_4()
        .child(
            // Header row; insets match the data rows (H6) and the gantt cell
            // carries the "Today" marker line.
            h_flex()
                .gap_3()
                .px_2()
                .py_2()
                .child(select_all_checkbox(
                    if all_on {
                        SelectAll::All
                    } else if any_on {
                        SelectAll::Some
                    } else {
                        SelectAll::None
                    },
                    cx,
                ))
                .child(
                    sort_header(
                        "po-sort-supplier",
                        "Supplier",
                        PoSortField::Supplier,
                        sort,
                        cx,
                    )
                    .flex_1(),
                )
                .child(gantt_head(cx))
                .child(
                    sort_header("po-sort-amount", "Amount", PoSortField::Amount, sort, cx)
                        .w(px(80.))
                        .justify_end(),
                ),
        )
        .when(rows.is_empty(), |this| {
            this.child(
                v_flex()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("No purchase orders match this filter"),
                    )
                    .child(btn_secondary("po-show-all", "Show all").on_click(
                        |_event, _window, cx| {
                            DemoState::update(cx, |s| s.po_tab = 0);
                        },
                    )),
            )
        })
        .children(
            rows.iter()
                .cloned()
                .zip(&handles)
                .enumerate()
                .map(|(ix, (row, focus))| {
                    let PoRow {
                        on,
                        ref code,
                        ref supplier,
                        ref sub,
                        amount,
                        ..
                    } = row;
                    let code_click = code.clone();
                    let code_check = code.clone();
                    let code_key = code.clone();
                    // Arrow keys move focus across the *visible* rows.
                    let prev = ix.checked_sub(1).map(|i| handles[i].clone());
                    let next = handles.get(ix + 1).cloned();
                    h_flex()
                        .id(SharedString::from(format!("po-row-{code_key}")))
                        .gap_3()
                        .px_2()
                        .py_2()
                        .rounded(px(8.))
                        .border_1()
                        .border_color(Accent::solid(cx).alpha(0.))
                        .track_focus(focus)
                        .tab_index(0)
                        .cursor_pointer()
                        .when(on, |this| this.bg(Accent::wash(cx)))
                        .hover(|this| {
                            this.bg(if on {
                                Accent::wash(cx)
                            } else {
                                cx.theme().accent.alpha(0.5)
                            })
                        })
                        .focus(|this| this.border_color(Accent::solid(cx)))
                        .on_click(move |_, _window, cx| {
                            DemoState::update(cx, |s| s.po_set_selected(&code_click, !on));
                        })
                        .on_key_down(
                            move |event, window, cx| match event.keystroke.key.as_str() {
                                "up" | "down" => {
                                    let target = if event.keystroke.key == "up" {
                                        prev.as_ref()
                                    } else {
                                        next.as_ref()
                                    };
                                    if let Some(handle) = target {
                                        window.focus(handle, cx);
                                    }
                                    window.prevent_default();
                                }
                                "space" => {
                                    window.prevent_default();
                                    DemoState::update(cx, |s| s.po_set_selected(&code_key, !on));
                                }
                                "enter" => {
                                    window.prevent_default();
                                    window.push_notification(
                                        Notification::info(format!(
                                            "Open {code_key} — PO details are not part of this demo"
                                        )),
                                        cx,
                                    );
                                }
                                _ => {}
                            },
                        )
                        .child(
                            CadenceCheckbox::new(
                                SharedString::from(format!("po-check-{code_check}")),
                                on,
                            )
                            .on_click(move |checked, _window, cx| {
                                // Keep the row's own toggle from double-firing.
                                cx.stop_propagation();
                                DemoState::update(cx, |s| s.po_set_selected(&code_check, *checked));
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
                                        .child(supplier.clone()),
                                )
                                .child(
                                    mono(sub.clone(), cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .child(gantt_cell(cx, &row))
                        .child(
                            mono(usd(amount), cx)
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
            .trailing(MonoPill::new(format!("{awaiting} need you"))),
        )
        .child(
            Tabs::new(
                "po-tabs",
                vec![
                    Tab::new("All").count(tab_counts[0]),
                    Tab::new("Draft").count(tab_counts[1]),
                    Tab::new("Awaiting").count(tab_counts[2]),
                    Tab::new("Approved").count(tab_counts[3]),
                    Tab::new("Billed").count(tab_counts[4]),
                ],
            )
            .active(po_tab)
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
                        .when(date_filter, |this| {
                            this.child(
                                FilterChip::new("po-date-chip", "Sep 1 – Sep 30")
                                    .icon(IconName::Calendar)
                                    .on_remove(|_window, cx| {
                                        DemoState::update(cx, |s| s.date_filter = false);
                                    }),
                            )
                        })
                        .when(!date_filter, |this| {
                            this.child(
                                FilterChip::new("po-date-restore", "Add date filter")
                                    .icon(IconName::Plus)
                                    .ghost()
                                    .on_open(|_window, cx| {
                                        DemoState::update(cx, |s| s.date_filter = true);
                                    }),
                            )
                        })
                        .child(
                            FilterChip::new("po-supplier-chip", "Any")
                                .prefix("Supplier")
                                .caret()
                                .on_open(|window, cx| {
                                    window.push_notification(
                                        Notification::info("Supplier picker (demo)"),
                                        cx,
                                    );
                                }),
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
                        .child(
                            btn_secondary("po-mark-billed", "Mark billed")
                                .disabled(picked == 0)
                                .on_click(|_event, window, cx| {
                                    let mut moved = 0;
                                    DemoState::update(cx, |s| {
                                        moved = s.po_mark_billed_selected();
                                    });
                                    window.push_notification(
                                        Notification::info(if moved > 0 {
                                            format!("Marked {moved} purchase orders billed")
                                        } else {
                                            "Selected purchase orders are already billed"
                                                .to_string()
                                        }),
                                        cx,
                                    );
                                }),
                        )
                        .child(
                            btn_primary("po-approve", format!("Approve {picked}"), cx)
                                .disabled(picked == 0)
                                .on_click(|_event, window, cx| {
                                    let mut moved = 0;
                                    DemoState::update(cx, |s| {
                                        moved = s.po_approve_selected();
                                    });
                                    window.push_notification(
                                        if moved > 0 {
                                            Notification::success(format!(
                                                "Approved {moved} purchase orders"
                                            ))
                                        } else {
                                            Notification::info(
                                                "No selected purchase orders are awaiting approval",
                                            )
                                        },
                                        cx,
                                    );
                                }),
                        ),
                ),
        )
        .into_any_element()
}

/// One PO table row's owned cell data — collected up front so the row
/// closures and the keyed focus state don't hold the `DemoState` borrow.
#[derive(Clone)]
struct PoRow {
    on: bool,
    code: SharedString,
    supplier: SharedString,
    sub: SharedString,
    amount: f64,
    gantt: Vec<GanttSeg>,
}

/// Tri-state of the table header's select-all box.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectAll {
    None,
    Some,
    All,
}

/// The header's select-all box — a `CadenceCheckbox` lookalike that adds the
/// indeterminate (dash) state the header needs. Clicking selects every
/// visible row, or clears them all when every one is already picked.
fn select_all_checkbox(state: SelectAll, cx: &App) -> Stateful<Div> {
    let checked = state != SelectAll::None;
    let (bg, border, fg) = if checked {
        (Accent::solid(cx), Accent::solid(cx), Accent::on_solid(cx))
    } else {
        (
            cx.theme().popover,
            cx.theme().border,
            cx.theme().muted_foreground,
        )
    };
    div()
        .id("po-select-all")
        .cursor_pointer()
        .size(px(18.))
        .flex_shrink_0()
        .rounded(px(5.))
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_, _window, cx| {
            DemoState::update(cx, |s| s.po_set_visible_selected(!all_selected(s)));
        })
        .when(state == SelectAll::All, |this| {
            this.child(
                Icon::new(IconName::Check)
                    .with_size(Size::Size(px(12.)))
                    .text_color(fg),
            )
        })
        .when(state == SelectAll::Some, |this| {
            this.child(
                Icon::new(IconName::Minus)
                    .with_size(Size::Size(px(12.)))
                    .text_color(fg),
            )
        })
}

/// Whether every currently visible PO row is selected — the select-all box's
/// click direction: all picked → clear, otherwise pick all.
fn all_selected(s: &DemoState) -> bool {
    let rows = s.po_visible();
    !rows.is_empty() && rows.iter().all(|i| s.po_is_selected(&i.code))
}

/// A clickable column header that cycles Neutral → Asc → Desc on click,
/// with a sort indicator glyph (neutral: arrow-up-down from the full
/// Lucide catalog; active: arrow-up/-down).
fn sort_header(
    id: impl Into<ElementId>,
    label: &'static str,
    field: PoSortField,
    sort: Option<(PoSortField, SortDirection)>,
    cx: &App,
) -> Stateful<Div> {
    let icon_path = match sort {
        Some((f, SortDirection::Asc)) if f == field => "icons/arrow-up.svg",
        Some((f, SortDirection::Desc)) if f == field => "icons/arrow-down.svg",
        _ => "icons/arrow-up-down.svg",
    };
    h_flex()
        .id(id)
        .cursor_pointer()
        .gap_1()
        .items_center()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .hover(|this| this.text_color(cx.theme().foreground))
        .on_click(move |_, _window, cx| {
            DemoState::update(cx, |s| s.po_toggle_sort(field));
        })
        .child(label)
        .child(
            Icon::empty()
                .path(icon_path)
                .with_size(Size::Size(px(12.)))
                .text_color(cx.theme().muted_foreground),
        )
}

/// The gantt column of one table row: fixed 200px track with the "Today"
/// marker line running through it. `.self_stretch()` fills the row's
/// content box (rows center their children) and the line bleeds through
/// the 8px row padding, so per-row segments join into one continuous
/// line down the table (H5).
fn gantt_cell(cx: &App, row: &PoRow) -> AnyElement {
    let tips = row.gantt.iter().map(|seg| gantt_tip(seg, row)).collect();
    div()
        .w(px(200.))
        .relative()
        .self_stretch()
        .justify_center()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-8.))
                .bottom(px(-8.))
                .w(px(1.))
                .bg(Accent::solid(cx).alpha(0.35)),
        )
        .child(
            Gantt::new(
                SharedString::from(format!("gantt-{}", row.code)),
                row.gantt.clone(),
            )
            .tips(tips),
        )
        .into_any_element()
}

/// Hover text for one gantt segment — the showcase's contextual labels
/// (percentage + dollars for bars, estimate for the delivery marker).
fn gantt_tip(seg: &GanttSeg, row: &PoRow) -> SharedString {
    match *seg {
        GanttSeg::Solid(f) => format!(
            "Billed: {:.0}% ({}) · Paid Sep 14",
            f * 100.,
            usd(row.amount * f as f64)
        )
        .into(),
        GanttSeg::Faint(f) => format!(
            "Ordered: {:.0}% ({}) · Dispatched",
            f * 100.,
            usd(row.amount * f as f64)
        )
        .into(),
        GanttSeg::Dashed(_) => "Pending Approval · Approval required".into(),
        GanttSeg::Delivery => format!("Estimated Delivery: Oct 12 · {}", row.supplier).into(),
    }
}

/// The gantt column header: week ticks either side of the "Today" pill,
/// with the marker line bleeding through the row padding like the cells.
fn gantt_head(cx: &App) -> AnyElement {
    div()
        .w(px(200.))
        .relative()
        .self_stretch()
        .justify_center()
        .child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-8.))
                .bottom(px(-8.))
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
    let doc_selected = state.doc_types_count();

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
            CardHeader::new(
                IconName::SlidersHorizontal,
                "Filter",
                "Bills and credit notes",
            )
            .trailing(CloseButton::new("filter-close").on_close(|_window, cx| {
                DemoState::update(cx, |s| s.show_filter = false);
            })),
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
                                    mono(
                                        format!(
                                            "{} - {}",
                                            usd0(state.slider.0 as f64 * 20_000.),
                                            usd0(state.slider.1 as f64 * 20_000.)
                                        ),
                                        cx,
                                    )
                                    .text_sm()
                                    .font_medium()
                                    .text_color(cx.theme().foreground),
                                ),
                        )
                        .child(
                            MiniBars::new(FILTER_HISTOGRAM.to_vec())
                                .height(px(44.))
                                .bar_w(px(5.))
                                .accent_range((
                                    (state.slider.0 * 27.).round() as usize,
                                    (state.slider.1 * 27.).round() as usize,
                                )),
                        )
                        .child(
                            RangeSlider::new("filter-slider", state.slider.0, state.slider.1)
                                .on_change(|&(lo, hi), _window, cx| {
                                    DemoState::update(cx, |s| s.slider = (lo, hi));
                                }),
                        )
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
                                .id("filter-overdue-label")
                                .cursor_pointer()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Overdue only")
                                .on_click(|_, _window, cx| {
                                    DemoState::update(cx, |s| s.overdue_only = !s.overdue_only);
                                }),
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
                        .child(btn_secondary("filter-reset", "Reset").flex_1().on_click(
                            |_, _window, cx| {
                                DemoState::update(cx, |s| s.reset_filters());
                            },
                        ))
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
    let toggle = Rc::new(toggle);
    let box_toggle = toggle.clone();
    h_flex()
        .id(SharedString::from(format!("{id}-row")))
        .cursor_pointer()
        .justify_between()
        .items_center()
        .py_1()
        .rounded(px(6.))
        .hover(|this| this.bg(cx.theme().accent))
        .on_click(move |_, _window, cx| {
            toggle(cx);
        })
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    CadenceCheckbox::new(id, checked).on_click(move |_, _window, cx| {
                        box_toggle(cx);
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

/// The ghost chip that stands in for a closed panel — click/Enter/Space
/// reopens it ("Open Filter", "Open Assign").
#[derive(IntoElement)]
struct RestoreChip {
    id: ElementId,
    label: SharedString,
    on_open: WindowCallback,
}

impl RestoreChip {
    fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        on_open: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_open: Rc::new(on_open),
        }
    }
}

impl RenderOnce for RestoreChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        let on_open = self.on_open.clone();
        let on_key = self.on_open.clone();
        h_flex()
            .id(self.id)
            .cursor_pointer()
            .tab_index(0)
            .track_focus(&focus_handle)
            .w(px(330.))
            .h(px(36.))
            .rounded(px(10.))
            .border_1()
            .border_color(cx.theme().border)
            .gap_1p5()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .hover(|this| this.bg(cx.theme().accent).text_color(cx.theme().foreground))
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .on_click(move |_, window, cx| {
                window.prevent_default();
                on_open(window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if matches!(event.keystroke.key.as_ref(), "space" | "enter") {
                    window.prevent_default();
                    on_key(window, cx);
                    cx.stop_propagation();
                }
            })
            .child(
                Icon::new(IconName::Plus)
                    .with_size(Size::Size(px(14.)))
                    .text_color(cx.theme().muted_foreground),
            )
            .child(div().text_sm().child(self.label))
    }
}

// ---------------------------------------------------------------------------
// Card 3 — Upcoming bills (stat strip, stacked chart, sparkline strip)
// ---------------------------------------------------------------------------

fn upcoming_bills(cx: &App) -> AnyElement {
    let state = DemoState::global(cx);
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
                    .tooltip(state.selected_bill_bar.unwrap_or(8), "Oct 1 · $52k")
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
            IconName::HandCoins,
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
    let totals = claim_totals(&state.claims_paid);
    let queue = match state.claims_tab {
        0 => ClaimQueue::Own,
        1 => ClaimQueue::Review,
        _ => ClaimQueue::Pay,
    };
    let rows: Vec<&ClaimRow> = CLAIMS.iter().filter(|c| c.queue == queue).collect();
    let review_count = CLAIMS
        .iter()
        .filter(|c| c.queue == ClaimQueue::Review)
        .count() as u64;
    let to_pay = state.claims_to_pay_count();

    card(cx)
        .w(px(500.))
        .child(
            CardHeader::new(
                IconName::Receipt,
                "Expense claims",
                "Reimbursements · September",
            )
            .trailing(MonoPill::new(format!("{to_pay} to pay"))),
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
                            Segment::new("To review").count(review_count),
                            Segment::new("To pay").count(to_pay as u64),
                        ],
                    )
                    .active(state.claims_tab)
                    .on_pick(|ix, _window, cx| DemoState::update(cx, |s| s.claims_tab = ix)),
                )
                .child(
                    FilterChip::new("claims-month", state.claims_month.clone())
                        .caret()
                        .on_open(|window, cx| {
                            window.push_notification(Notification::info("Month picker (demo)"), cx);
                        }),
                ),
        )
        .child(
            div().px_4().py_3().child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_8()
                            .child(claim_stat("Submitted", totals.submitted, false, cx))
                            .child(claim_stat("To Be Paid", totals.to_be_paid, true, cx))
                            .child(claim_stat("Reimbursed", totals.reimbursed, false, cx)),
                    )
                    .child(Meter::new((totals.reimbursed / totals.submitted) as f32)),
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
                    mono(format!("{to_pay} claims"), cx)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .child(
            v_flex()
                .px_4()
                .pb_4()
                .when(
                    rows.is_empty() || (queue == ClaimQueue::Pay && to_pay == 0),
                    |this| {
                        this.child(
                            div()
                                .py_4()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(if queue == ClaimQueue::Pay {
                                    "All claims settled"
                                } else {
                                    "Nothing in this queue"
                                }),
                        )
                    },
                )
                .when(
                    !(rows.is_empty() || (queue == ClaimQueue::Pay && to_pay == 0)),
                    |this| {
                        this.children(rows.into_iter().map(|claim| {
                            let action: AnyElement = match claim.queue {
                                ClaimQueue::Pay => {
                                    let Some(pay_ix) = claim.pay_ix else {
                                        unreachable!("Pay queue claims carry a claims_paid index");
                                    };
                                    if state.claims_paid[pay_ix] {
                                        MonoPill::new("Paid")
                                            .tone(PillTone::Neutral)
                                            .into_any_element()
                                    } else {
                                        btn_secondary(
                                            SharedString::from(format!("claim-pay-{pay_ix}")),
                                            "Pay",
                                        )
                                        .on_click(move |_event, window, cx| {
                                            DemoState::update(cx, |s| s.mark_claim_paid(pay_ix));
                                            window.push_notification(
                                                Notification::success(format!(
                                                    "Processed {} reimbursement to {}",
                                                    usd(claim.amount),
                                                    claim.recipient
                                                )),
                                                cx,
                                            );
                                        })
                                        .into_any_element()
                                    }
                                }
                                ClaimQueue::Review => MonoPill::new("Needs review")
                                    .tone(PillTone::Neutral)
                                    .into_any_element(),
                                ClaimQueue::Own => MonoPill::new("Submitted")
                                    .tone(PillTone::Neutral)
                                    .into_any_element(),
                            };
                            let paid = claim.pay_ix.is_some_and(|ix| state.claims_paid[ix]);
                            h_flex()
                                .gap_3()
                                .items_center()
                                .py_1p5()
                                .when(paid, |this| this.opacity(0.55))
                                .child(icon_tile_sized(claim.icon, px(32.), px(16.), px(8.), cx))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().foreground)
                                                .child(claim.title),
                                        )
                                        .child(
                                            // Numerals are the reference's mono
                                            // subline (M6).
                                            mono(claim.sub, cx)
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    mono(usd(claim.amount), cx)
                                        .text_sm()
                                        .font_medium()
                                        .text_color(cx.theme().foreground),
                                )
                                .child(action)
                        }))
                    },
                ),
        )
        .into_any_element()
}

/// The claims card queues, in `Segmented` order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClaimQueue {
    /// "Your own" — the viewer's personal submitted claims.
    Own,
    /// "To review" — awaiting manager sign-off.
    Review,
    /// "To pay" — awaiting reimbursement; `pay_ix` keys `claims_paid`.
    Pay,
}

/// One claim row's static demo data.
struct ClaimRow {
    queue: ClaimQueue,
    /// `claims_paid` slot — only `Pay` rows carry one.
    pay_ix: Option<usize>,
    icon: IconName,
    title: &'static str,
    /// "Mara K. · Sep 18 · 4200 Travel" — rendered via `mono` (M6).
    sub: &'static str,
    /// First name for the Pay toast ("… reimbursement to Mara").
    recipient: &'static str,
    amount: f64,
}

/// The reference's claim data plus the "Your own"/"To review" queues the
/// segmented control filters to. The three `Pay` rows keep their original
/// `claims_paid` indices.
const CLAIMS: &[ClaimRow] = &[
    ClaimRow {
        queue: ClaimQueue::Own,
        pay_ix: None,
        icon: IconName::TrainFront,
        title: "Rail pass · Cascais line",
        sub: "Mara K. · Sep 12 · 4200 Travel",
        recipient: "Mara",
        amount: 96.00,
    },
    ClaimRow {
        queue: ClaimQueue::Own,
        pay_ix: None,
        icon: IconName::Hotel,
        title: "Hotel · Porto offsite",
        sub: "Mara K. · Sep 15 · 4210 Lodging",
        recipient: "Mara",
        amount: 348.00,
    },
    ClaimRow {
        queue: ClaimQueue::Review,
        pay_ix: None,
        icon: IconName::Plane,
        title: "Flights BER → LHR",
        sub: "Jonas P. · Sep 19 · 4200 Travel",
        recipient: "Jonas",
        amount: 524.00,
    },
    ClaimRow {
        queue: ClaimQueue::Review,
        pay_ix: None,
        icon: IconName::Utensils,
        title: "Team lunch · Berlin",
        sub: "Priya S. · Sep 20 · 6230 Meals",
        recipient: "Priya",
        amount: 212.00,
    },
    ClaimRow {
        queue: ClaimQueue::Pay,
        pay_ix: Some(0),
        icon: IconName::Plane,
        title: "Flights LIS → BER",
        sub: "Mara K. · Sep 18 · 4200 Travel",
        recipient: "Mara",
        amount: 412.00,
    },
    ClaimRow {
        queue: ClaimQueue::Pay,
        pay_ix: Some(1),
        icon: IconName::Utensils,
        title: "Client dinner · Lisbon",
        sub: "Mara K. · Sep 18 · 6230 Meals",
        recipient: "Mara",
        amount: 184.00,
    },
    ClaimRow {
        queue: ClaimQueue::Pay,
        pay_ix: Some(2),
        icon: IconName::CarFront,
        title: "Mileage · 214 km",
        sub: "Yuri G. · Sep 21 · 4250 Mileage",
        recipient: "Yuri",
        amount: 688.00,
    },
];

/// Reimbursements already processed before this demo opened — the
/// reference's "Reimbursed $1,032.40".
const CLAIMS_REIMBURSED_BASE: f64 = 1_032.40;

/// The stat trio behind Card 5's meter: every figure the Pay flow moves.
struct ClaimTotals {
    submitted: f64,
    to_be_paid: f64,
    reimbursed: f64,
}

/// Submitted / To Be Paid / Reimbursed under the current `claims_paid`
/// bits: paying a claim moves its amount from To Be Paid to Reimbursed
/// (and onto Submitted, which starts from the reference's $2,316.40).
fn claim_totals(paid: &[bool; 3]) -> ClaimTotals {
    let mut payable = 0.;
    let mut paid_sum = 0.;
    for claim in CLAIMS {
        if let Some(ix) = claim.pay_ix {
            payable += claim.amount;
            if paid[ix] {
                paid_sum += claim.amount;
            }
        }
    }
    ClaimTotals {
        submitted: CLAIMS_REIMBURSED_BASE + payable + paid_sum,
        to_be_paid: payable - paid_sum,
        reimbursed: CLAIMS_REIMBURSED_BASE + paid_sum,
    }
}

/// One column of the claims stat trio: muted label over a mono amount.
/// `accent` inks the value with the accent (the "To Be Paid" figure).
fn claim_stat(label: &'static str, amount: f64, accent: bool, cx: &App) -> Div {
    v_flex()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(
            mono(usd(amount), cx)
                .text_xl()
                .font_semibold()
                .text_color(if accent {
                    Accent::ink(cx)
                } else {
                    cx.theme().foreground
                }),
        )
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
    let state = DemoState::global(cx);
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
                mono(format!("Default markup {}%", state.rebill_markup_pct), cx)
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
        ("Flights LIS → BER", "Atlas"),
        ("Hotel · 2 nights", "Atlas"),
        ("Client dinner", "Pending"),
    ];
    let expense_total: f64 = EXPENSE_AMOUNTS
        .iter()
        .zip(state.expenses.iter())
        .filter(|(_, on)| **on)
        .map(|(v, _)| *v)
        .sum();

    card(cx)
        .w(px(330.))
        .child(
            CardHeader::new(
                IconName::FolderClosed,
                "Assign to project",
                format!("{} expenses selected", state.expenses_selected_count()),
            )
            .trailing(CloseButton::new("assign-close").on_close(|_window, cx| {
                DemoState::update(cx, |s| s.show_assign = false);
            })),
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
                                .child(state.project_search.clone()),
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
                            mono(usd(expense_total), cx)
                                .text_sm()
                                .font_medium()
                                .text_color(cx.theme().foreground),
                        ),
                )
                .children(expenses.into_iter().enumerate().map(|(ix, (title, tag))| {
                    h_flex()
                        .gap_2p5()
                        .items_center()
                        .py_1()
                        .child(
                            CadenceCheckbox::new(
                                SharedString::from(format!("assign-expense-{ix}")),
                                state.expenses[ix],
                            )
                            .on_click(move |checked, _window, cx| {
                                DemoState::update(cx, |s| s.expenses[ix] = *checked);
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
                                        .child(title),
                                )
                                .child(
                                    mono(usd(EXPENSE_AMOUNTS[ix]), cx)
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .child(MonoPill::new(tag).tone(if tag == "Atlas" {
                            PillTone::Accent
                        } else {
                            PillTone::Neutral
                        }))
                }))
                .child(
                    h_flex()
                        .gap_2()
                        .pt_1()
                        .child(btn_secondary("assign-cancel", "Cancel"))
                        .child(
                            btn_primary(
                                "assign-confirm",
                                format!("Assign {}", state.expenses_selected_count()),
                                cx,
                            )
                            .flex_1(),
                        ),
                ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Demo data + formatting
// ---------------------------------------------------------------------------

/// Assign-card expense amounts (parallel to the title/tag rows above —
/// the reference's three fixed lines).
const EXPENSE_AMOUNTS: [f64; 3] = [412.00, 388.00, 184.00];

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

/// Whole-dollar amounts for the live filter label: `usd` without cents
/// (`$5,000`).
fn usd0(v: f64) -> String {
    usd(v.round()).trim_end_matches(".00").to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        CLAIMS, CLAIMS_REIMBURSED_BASE, ClaimQueue, GanttSeg, PoRow, claim_totals, gantt_tip, usd,
    };

    #[test]
    fn usd_formats_with_thousands_separator() {
        assert_eq!(usd(0.0), "$0.00");
        assert_eq!(usd(412.0), "$412.00");
        assert_eq!(usd(6120.0), "$6,120.00");
        assert_eq!(usd(16480.0), "$16,480.00");
        assert_eq!(usd(412860.0), "$412,860.00");
        assert_eq!(usd(2316.4), "$2,316.40");
    }

    /// Card 5's stat trio must move exactly one claim's amount per Pay:
    /// the reference render (nothing paid) and the fully-settled end
    /// state pin the math.
    #[test]
    fn claim_totals_follow_paid_bits() {
        let payable: f64 = CLAIMS
            .iter()
            .filter(|c| c.queue == ClaimQueue::Pay)
            .map(|c| c.amount)
            .sum();
        assert_eq!(payable, 1284.00);

        let t = claim_totals(&[false; 3]);
        assert_eq!(t.submitted, 2316.40);
        assert_eq!(t.to_be_paid, 1284.00);
        assert_eq!(t.reimbursed, CLAIMS_REIMBURSED_BASE);

        // Paying the $412 claim shifts it To Be Paid → Reimbursed.
        let t = claim_totals(&[true, false, false]);
        assert_eq!(t.submitted, 2316.40 + 412.00);
        assert_eq!(t.to_be_paid, 1284.00 - 412.00);
        assert_eq!(t.reimbursed, CLAIMS_REIMBURSED_BASE + 412.00);

        let t = claim_totals(&[true; 3]);
        assert_eq!(t.to_be_paid, 0.0);
        assert_eq!(t.reimbursed, CLAIMS_REIMBURSED_BASE + payable);
    }

    /// Every gantt segment renders a contextual tooltip; percentage and
    /// dollars come from the row's own data.
    #[test]
    fn gantt_tips_cover_every_segment() {
        let row = PoRow {
            on: false,
            code: "PO-0418".into(),
            supplier: "Northwind Freight".into(),
            sub: "PO-0418 · 100% billed".into(),
            amount: 6120.00,
            gantt: vec![
                GanttSeg::Solid(0.42),
                GanttSeg::Faint(0.2),
                GanttSeg::Dashed(0.1),
                GanttSeg::Delivery,
            ],
        };
        let tips: Vec<String> = row
            .gantt
            .iter()
            .map(|seg| gantt_tip(seg, &row).to_string())
            .collect();
        assert_eq!(
            tips,
            [
                "Billed: 42% ($2,570.40) · Paid Sep 14",
                "Ordered: 20% ($1,224.00) · Dispatched",
                "Pending Approval · Approval required",
                "Estimated Delivery: Oct 12 · Northwind Freight",
            ]
        );
    }
}
