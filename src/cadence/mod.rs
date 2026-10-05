//! Cadence — components cloned from the finance-UI reference
//! (`x.com/yurygok/status/2105191658337841531`; the four renders are the
//! tweet's images, mirrored under `~/quaqua/runs/ref-cadence/`).
//!
//! ## Style analysis (what the reference is made of)
//!
//! Surfaces — a light-gray canvas; white cards (`rounded` ~16px, hairline
//! border, faint shadow); recessed "wells" inside cards (chart areas, filter
//! sections, the empty-state illustration); a light-blue wash for selection
//! and highlighted ranges. Mapped onto the mono theme: canvas =
//! `theme.background`, card = `theme.popover`, well = `theme.muted`,
//! hairline = `theme.border`, wash = [`Accent::wash`].
//!
//! Ink — near-black text, mid-gray secondary text, and one vivid indigo
//! accent used for every active/selected/positive state. The mono theme's
//! primary (`#124b68`) is a different hue, so the accent family is local to
//! this module (same policy as `status_chip`): [`Accent`] with a WCAG test.
//!
//! Typography — the reference's signature trait: a grotesk for UI copy and a
//! **monospace for every numeral** — amounts, counts, badges, axis labels,
//! even "1 col = 1 week". Here that is the theme's `mono_font_family`
//! (Google Sans Code); [`blocks::mono`] is the one-liner every numeric text
//! goes through.
//!
//! Controls — black primary buttons with white labels (light mode; inverted
//! in dark mode via foreground/popover), white bordered secondary buttons,
//! blue-filled checkboxes and switch, underline tabs and a pill segmented
//! control, both carrying mono counts.
//!
//! Data marks — micro-charts drawn from plain divs: histograms, stacked
//! bars over dashed gridlines, a waffle grid, horizontal aging bars, a
//! dual-handle range slider, and gantt-style row bars. GPUI cannot dash a
//! border, so dashed strokes (gridlines, pending gantt segments, empty
//! waffle cells) render as hairline solids at reduced alpha.
//!
//! ## Component inventory
//!
//! [`blocks`]: [`blocks::card`], [`blocks::CardHeader`],
//!   [`blocks::icon_tile`], [`blocks::mono`], [`blocks::MonoPill`],
//!   [`blocks::FilterChip`], [`blocks::Stat`], [`blocks::Meter`],
//!   [`blocks::legend_dot`].
//! [`controls`]: [`controls::Tabs`], [`controls::Segmented`].
//! [`charts`]: [`charts::MiniBars`], [`charts::BarChartMini`],
//!   [`charts::Waffle`], [`charts::AgingBars`], [`charts::RangeSlider`],
//!   [`charts::Gantt`].
//! [`showcase`]: the eight reference cards assembled on one canvas, mounted
//!   as the "Finance UI" settings page; interactions write to [`DemoState`].
//!
//! Showcase copy is the reference's own English sample data, kept verbatim
//! for fidelity (same policy as the mock's numbers); page chrome is `tr()`d.

pub mod blocks;
pub mod charts;
pub mod controls;
pub mod showcase;

use std::collections::HashSet;
use std::rc::Rc;

use self::charts::GanttSeg;
use gpui_kit::component::{ActiveTheme, ThemeMode};
use gpui_kit::{App, Global, Hsla, SharedString, Window, rgb};

/// Callback shapes shared by the interactive cadence components; the
/// aliases keep the builder fields under clippy's type-complexity limit.
pub type WindowCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type BoolCallback = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
pub type PickCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The reference's indigo accent family, resolved per theme mode.
///
/// Light values are sampled from the reference; dark values are its pastel
/// counterparts (desaturated fills, lightened ink). `ink` clears WCAG AA on
/// both the card surface and the tint backgrounds it appears on —
/// [`tests::accent_ink_clears_wcag_aa`] holds every pair to 4.5:1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accent;

impl Accent {
    /// Solid fills: bars, underline, checkbox fill, meter progress.
    pub fn solid(cx: &App) -> Hsla {
        set(mode(cx)).solid
    }

    /// Mid step of the accent ramp (waffle cells).
    pub fn mid(cx: &App) -> Hsla {
        set(mode(cx)).mid
    }

    /// Faint step: unscheduled bars, light waffle cells, meter remainder.
    pub fn faint(cx: &App) -> Hsla {
        set(mode(cx)).faint
    }

    /// Tint background: pills, the chart tooltip.
    pub fn soft(cx: &App) -> Hsla {
        set(mode(cx)).soft
    }

    /// Lighter wash: selected rows, highlighted sparkline ranges.
    pub fn wash(cx: &App) -> Hsla {
        set(mode(cx)).wash
    }

    /// Accent text on card surfaces and tints.
    pub fn ink(cx: &App) -> Hsla {
        set(mode(cx)).ink
    }

    /// Ink drawn ON the solid accent (the "Today" pill, the checkbox
    /// check): white in light mode; in dark mode the solid is a light
    /// pastel, so the ink flips dark to hold WCAG AA.
    pub fn on_solid(cx: &App) -> Hsla {
        set(mode(cx)).on_solid
    }
}

fn mode(cx: &App) -> ThemeMode {
    cx.theme().mode
}

/// One mode's accent ramp as opaque `0xRRGGBB` hex — the single source of
/// truth the renderer resolves and the WCAG test re-checks.
struct AccentHex {
    solid: u32,
    mid: u32,
    faint: u32,
    soft: u32,
    wash: u32,
    ink: u32,
    on_solid: u32,
}

const LIGHT: AccentHex = AccentHex {
    solid: 0x2b4bf2,
    mid: 0x7c8cf8,
    faint: 0xc9d2fc,
    soft: 0xe4e8fe,
    wash: 0xeef1fe,
    ink: 0x2b4bf2,
    on_solid: 0xffffff,
};

const DARK: AccentHex = AccentHex {
    solid: 0x6e86ff,
    mid: 0x4c5e9e,
    faint: 0x2e3b66,
    soft: 0x26325c,
    wash: 0x1e2745,
    ink: 0xa9b8ff,
    on_solid: 0x1a2244,
};

/// One mode's resolved accent ramp.
#[derive(Clone, Copy)]
struct AccentSet {
    solid: Hsla,
    mid: Hsla,
    faint: Hsla,
    soft: Hsla,
    wash: Hsla,
    ink: Hsla,
    on_solid: Hsla,
}

fn set(mode: ThemeMode) -> AccentSet {
    let h = if mode.is_dark() { DARK } else { LIGHT };
    AccentSet {
        solid: rgb(h.solid).into(),
        mid: rgb(h.mid).into(),
        faint: rgb(h.faint).into(),
        soft: rgb(h.soft).into(),
        wash: rgb(h.wash).into(),
        ink: rgb(h.ink).into(),
        on_solid: rgb(h.on_solid).into(),
    }
}

/// The PO lifecycle buckets, matching Card 1's tabs in order
/// ("All" isn't a status — tab 0 shows every row).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PoStatus {
    Draft,
    Awaiting,
    Approved,
    Billed,
}

/// The sortable PO table columns (Card 1, Phase 6).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PoSortField {
    Supplier,
    Amount,
}

/// Asc/desc ordering for `po_sort`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// One purchase order row: everything the table cells, gantt column, and
/// later approval/billing flows read. `sub` is the reference's verbatim
/// status subline ("PO-0418 · 100% billed").
pub struct PurchaseOrderItem {
    /// Stable key for selection state ("PO-0418").
    pub code: SharedString,
    pub supplier: SharedString,
    pub sub: SharedString,
    pub status: PoStatus,
    pub amount: f64,
    pub gantt: Vec<GanttSeg>,
}

/// An expense line assigned to a client project in Card 8, shown as a
/// billable row in Card 7 once `rebill_assigned` is populated (Phase 9).
#[allow(dead_code)] // fields read once Phase 9 renders Card 7's table
pub struct RebillItem {
    pub name: SharedString,
    /// Pre-markup amount.
    pub amount: f64,
}

/// Interactive state of the showcase page, stored as a GPUI global so the
/// `SettingItem::render` closures (which only receive `&App` / `&mut App`)
/// can read and write it. The default mirrors the reference render's
/// initial state: date chip on, toggle on, both doc types checked, two
/// purchase orders picked, Atlas assigned, all expenses checked, the bills
/// tooltip pinned on bar 8, 10% rebill markup, and both panels open.
///
/// Schema is intentionally complete for phases 3–9 (see
/// `docs/plans/active/cadence-interactive.md`) — chart/claim/rebill fields
/// are added here once so later phases don't collide on this file.
#[derive(Default)]
pub struct DemoState {
    /// Purchase orders: active tab index (0 All … 4 Billed).
    pub po_tab: usize,
    /// Purchase orders: the sample rows rendered in the table.
    pub po_items: Vec<PurchaseOrderItem>,
    /// Purchase orders: selected PO codes (replaces the old `[bool; 3]`).
    pub po_selected: HashSet<SharedString>,
    /// Purchase orders: active column sort (None = reference order).
    pub po_sort: Option<(PoSortField, SortDirection)>,
    /// Filter panel: the date-range chip is present (its × removes it).
    pub date_filter: bool,
    /// Filter panel: "Overdue only" switch.
    pub overdue_only: bool,
    /// Filter panel: Document Types checkboxes.
    pub doc_bills: bool,
    pub doc_credit_notes: bool,
    /// Filter panel: dual-handle range slider, 0..1 track positions.
    pub slider: (f32, f32),
    /// Filter panel: supplier chip selection (None = "Any").
    #[allow(dead_code)] // read once Phase 5 wires the supplier chip
    pub supplier_filter: Option<usize>,
    /// Filter panel (Card 2) is visible; its close button collapses it.
    pub show_filter: bool,
    /// Upcoming bills: pinned bar index (the tooltip anchor).
    pub selected_bill_bar: Option<usize>,
    /// Owed to suppliers: selected aging-bucket column (None = all).
    #[allow(dead_code)] // read once Phase 7 wires AgingBars
    pub selected_aging_bucket: Option<usize>,
    /// Expense claims: active segment index.
    pub claims_tab: usize,
    /// Expense claims: the month chip text.
    pub claims_month: SharedString,
    /// Expense claims: rows already paid (Phase 8 Pay flow).
    pub claims_paid: [bool; 3],
    /// By category: selected waffle/legend category index (None = all).
    #[allow(dead_code)] // read once Phase 7 wires Waffle/legend clicks
    pub selected_waffle_cat: Option<usize>,
    /// Cross-card filter: category chosen in Card 6 drills into Card 5.
    #[allow(dead_code)] // read once Phase 9 wires the drill-down
    pub selected_category: Option<SharedString>,
    /// Billable expenses: markup applied to assigned rebill items, percent.
    pub rebill_markup_pct: u32,
    /// Billable expenses: items Card 8 has committed to the project.
    #[allow(dead_code)] // read once Phase 9 renders Card 7's table
    pub rebill_assigned: Vec<RebillItem>,
    /// Assign panel: search field text (the mock renders it focused, "Atl").
    pub project_search: SharedString,
    /// Assign panel: selected project row (None = nothing picked).
    pub assigned: Option<usize>,
    /// Assign panel: per-expense checkboxes.
    pub expenses: [bool; 3],
    /// Assign panel (Card 8) is visible; its close button collapses it.
    pub show_assign: bool,
}

impl Global for DemoState {}

impl DemoState {
    /// The reference render's initial state.
    pub fn demo() -> Self {
        let po_items = vec![
            PurchaseOrderItem {
                code: "PO-0418".into(),
                supplier: "Northwind Freight".into(),
                sub: "PO-0418 · 100% billed".into(),
                status: PoStatus::Billed,
                amount: 6120.00,
                gantt: vec![GanttSeg::Solid(0.42), GanttSeg::Delivery],
            },
            PurchaseOrderItem {
                code: "PO-0421".into(),
                supplier: "Haldon Supplies".into(),
                sub: "PO-0421 · 40% billed".into(),
                status: PoStatus::Awaiting,
                amount: 4480.00,
                gantt: vec![
                    GanttSeg::Solid(0.2),
                    GanttSeg::Faint(0.2),
                    GanttSeg::Delivery,
                ],
            },
            PurchaseOrderItem {
                code: "PO-0427".into(),
                supplier: "Lumen Print Co.".into(),
                sub: "PO-0427 · approval".into(),
                status: PoStatus::Awaiting,
                amount: 1960.00,
                gantt: vec![GanttSeg::Dashed(0.42), GanttSeg::Delivery],
            },
        ];
        Self {
            po_tab: 0,
            po_items,
            po_selected: HashSet::from(["PO-0418".into(), "PO-0421".into()]),
            po_sort: None,
            date_filter: true,
            overdue_only: true,
            doc_bills: true,
            doc_credit_notes: true,
            slider: (0.25, 0.78),
            supplier_filter: None,
            show_filter: true,
            selected_bill_bar: Some(8),
            selected_aging_bucket: None,
            claims_tab: 2,
            claims_month: "Sep 2026".into(),
            claims_paid: [false; 3],
            selected_waffle_cat: None,
            selected_category: None,
            rebill_markup_pct: 10,
            rebill_assigned: Vec::new(),
            project_search: "Atl".into(),
            assigned: Some(0),
            expenses: [true; 3],
            show_assign: true,
        }
    }
}

impl DemoState {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Mutate through `f` and repaint — the one-liner every showcase
    /// interaction handler reduces to.
    pub fn update(cx: &mut App, f: impl FnOnce(&mut Self)) {
        f(cx.global_mut::<Self>());
        cx.refresh_windows();
    }
}

/// Query helpers — the counts and resolved views later phases render.
#[allow(dead_code)] // several helpers only get callers in phases 3–9
impl DemoState {
    /// PO rows under the active tab, in `po_sort` order when set.
    pub fn po_visible(&self) -> Vec<&PurchaseOrderItem> {
        let mut rows: Vec<&PurchaseOrderItem> = self
            .po_items
            .iter()
            .filter(|item| self.po_tab_matches(item))
            .collect();
        if let Some((field, dir)) = self.po_sort {
            rows.sort_by(|a, b| {
                let ord = match field {
                    PoSortField::Supplier => a.supplier.cmp(&b.supplier),
                    PoSortField::Amount => a
                        .amount
                        .partial_cmp(&b.amount)
                        .unwrap_or(std::cmp::Ordering::Equal),
                };
                match dir {
                    SortDirection::Asc => ord,
                    SortDirection::Desc => ord.reverse(),
                }
            });
        }
        rows
    }

    /// Does `item` belong on the active tab? Tab 0 ("All") takes every row.
    pub fn po_tab_matches(&self, item: &PurchaseOrderItem) -> bool {
        let status = match self.po_tab {
            1 => PoStatus::Draft,
            2 => PoStatus::Awaiting,
            3 => PoStatus::Approved,
            4 => PoStatus::Billed,
            _ => return true,
        };
        item.status == status
    }

    /// Rows on the "Awaiting" tab — the "2 need you" header pill.
    pub fn po_awaiting_count(&self) -> usize {
        self.po_items
            .iter()
            .filter(|i| i.status == PoStatus::Awaiting)
            .count()
    }

    pub fn po_selected_count(&self) -> usize {
        self.po_selected.len()
    }

    /// Dollar total of the selected POs (footer "{n} selected · $x").
    pub fn po_selected_total(&self) -> f64 {
        self.po_items
            .iter()
            .filter(|i| self.po_selected.contains(&i.code))
            .map(|i| i.amount)
            .sum()
    }

    pub fn expenses_selected_count(&self) -> usize {
        self.expenses.iter().filter(|on| **on).count()
    }

    pub fn doc_types_count(&self) -> usize {
        self.doc_bills as usize + self.doc_credit_notes as usize
    }

    pub fn claims_paid_count(&self) -> usize {
        self.claims_paid.iter().filter(|on| **on).count()
    }

    /// Rows still awaiting reimbursement — the "To pay" segment count.
    pub fn claims_to_pay_count(&self) -> usize {
        self.claims_paid.len() - self.claims_paid_count()
    }

    /// Card 7 stays on `EmptyState` while nothing is assigned.
    pub fn is_rebill_empty(&self) -> bool {
        self.rebill_assigned.is_empty()
    }

    /// Invoice total: assigned items plus `rebill_markup_pct`.
    pub fn rebill_invoice_total(&self) -> f64 {
        let base: f64 = self.rebill_assigned.iter().map(|i| i.amount).sum();
        base * (1. + self.rebill_markup_pct as f64 / 100.)
    }
}

/// Mutators — call inside `DemoState::update(cx, |s| s.…)`.
#[allow(dead_code)] // call sites land in phases 3–9
impl DemoState {
    pub fn po_is_selected(&self, code: &str) -> bool {
        self.po_selected.contains(code)
    }

    pub fn po_set_selected(&mut self, code: &str, on: bool) {
        if on {
            self.po_selected.insert(code.into());
        } else {
            self.po_selected.remove(code);
        }
    }

    /// Set every visible row's selection (the header select-all).
    pub fn po_set_visible_selected(&mut self, on: bool) {
        let codes: Vec<SharedString> = self
            .po_visible()
            .into_iter()
            .map(|i| i.code.clone())
            .collect();
        for code in codes {
            self.po_set_selected(&code, on);
        }
    }

    /// Awaiting → Approved; clears those rows' selection. Returns rows moved.
    pub fn po_approve_selected(&mut self) -> usize {
        let mut moved = 0;
        for item in &mut self.po_items {
            if item.status == PoStatus::Awaiting && self.po_selected.contains(&item.code) {
                item.status = PoStatus::Approved;
                moved += 1;
            }
        }
        self.po_selected.retain(|code| {
            self.po_items
                .iter()
                .any(|i| &i.code == code && i.status != PoStatus::Approved)
        });
        moved
    }

    /// Any status → Billed; clears the selection. Returns rows moved.
    pub fn po_mark_billed_selected(&mut self) -> usize {
        let mut moved = 0;
        for item in &mut self.po_items {
            if self.po_selected.contains(&item.code) && item.status != PoStatus::Billed {
                item.status = PoStatus::Billed;
                moved += 1;
            }
        }
        if moved > 0 {
            self.po_selected.clear();
        }
        moved
    }

    /// Header-click sort: Neutral → Asc → Desc, switching field resets to Asc.
    pub fn po_toggle_sort(&mut self, field: PoSortField) {
        self.po_sort = match self.po_sort {
            Some((f, SortDirection::Asc)) if f == field => Some((f, SortDirection::Desc)),
            Some((f, SortDirection::Desc)) if f == field => None,
            _ => Some((field, SortDirection::Asc)),
        };
    }

    /// Click toggles the pinned bar; the same bar again clears it.
    pub fn toggle_bill_bar(&mut self, ix: usize) {
        self.selected_bill_bar = (self.selected_bill_bar != Some(ix)).then_some(ix);
    }

    /// Click filters to a category; the same category again resets to all.
    pub fn toggle_waffle_cat(&mut self, cat: usize) {
        self.selected_waffle_cat = (self.selected_waffle_cat != Some(cat)).then_some(cat);
    }

    pub fn toggle_aging_bucket(&mut self, ix: usize) {
        self.selected_aging_bucket = (self.selected_aging_bucket != Some(ix)).then_some(ix);
    }

    /// Card 5 Pay flow: mark one claim reimbursed.
    pub fn mark_claim_paid(&mut self, ix: usize) {
        if let Some(paid) = self.claims_paid.get_mut(ix) {
            *paid = true;
        }
    }

    /// "Reset" restores the filter card to the reference defaults.
    pub fn reset_filters(&mut self) {
        self.date_filter = true;
        self.overdue_only = true;
        self.doc_bills = true;
        self.doc_credit_notes = true;
        self.slider = (0.25, 0.78);
        self.supplier_filter = None;
    }

    pub fn set_rebill_markup(&mut self, pct: u32) {
        self.rebill_markup_pct = pct;
    }

    /// Card 8 commit: hand the checked expenses to Card 7, uncheck them.
    pub fn assign_expenses(&mut self, items: Vec<RebillItem>) {
        self.rebill_assigned = items;
        self.expenses = [false; 3];
    }

    /// "Clear Assignments" empties Card 7 back to `EmptyState`.
    pub fn rebill_clear(&mut self) {
        self.rebill_assigned.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{DARK, LIGHT};
    use gpui_kit::component::ThemeMode;

    /// WCAG 2.1 relative luminance of an opaque sRGB color.
    fn relative_luminance(hex: u32) -> f32 {
        let channel = |shift: u32| {
            let c = ((hex >> shift) & 0xff) as f32 / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }

    /// WCAG 2.1 contrast ratio between two opaque sRGB colors.
    fn contrast(a: u32, b: u32) -> f32 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Accent ink must clear WCAG AA (4.5:1) on every surface it is drawn
    /// over: the card (popover), the canvas (background), and both tint
    /// backgrounds (`soft`, `wash`), in both modes.
    ///
    /// The theme surfaces are pinned to the published mono.json values here:
    /// the test needs concrete hex, and a drift in those tokens must fail
    /// loudly rather than silently drop the accent below AA.
    #[test]
    fn accent_ink_clears_wcag_aa() {
        const POPOVER: [u32; 2] = [0xffffff, 0x212121];
        const BACKGROUND: [u32; 2] = [0xf7f7f7, 0x171717];
        for (mode, ix) in [(ThemeMode::Light, 0), (ThemeMode::Dark, 1)] {
            let set = if mode.is_dark() { DARK } else { LIGHT };
            for (name, surface) in [
                ("popover", POPOVER[ix]),
                ("background", BACKGROUND[ix]),
                ("soft", set.soft),
                ("wash", set.wash),
            ] {
                let ratio = contrast(set.ink, surface);
                assert!(
                    ratio >= 4.5,
                    "{mode:?}: ink on {name} is {ratio:.2}:1, need 4.5:1"
                );
            }
        }
    }

    /// The "Today" pill and the checkbox draw `on_solid` ink over the
    /// solid accent; that pair must clear AA in both modes.
    #[test]
    fn on_solid_ink_clears_aa_on_solid_accent() {
        for set in [LIGHT, DARK] {
            let ratio = contrast(set.on_solid, set.solid);
            assert!(
                ratio >= 4.5,
                "on_solid ink on solid accent is {ratio:.2}:1, need 4.5:1"
            );
        }
    }

    /// The demo schema pins the reference render: the two picked POs
    /// resolve to real items totalling $10,600, visibility defaults let
    /// every panel show, and the selection/toggle helpers behave the way
    /// phases 3–9 rely on.
    #[test]
    fn demo_state_schema_and_helpers() {
        use super::{DemoState, PoSortField, PoStatus};

        let mut s = DemoState::demo();
        assert_eq!(s.po_selected_count(), 2);
        assert_eq!(s.po_selected_total(), 10_600.00);
        assert_eq!(s.po_awaiting_count(), 2);
        assert!(s.po_is_selected("PO-0418"));
        assert!(!s.po_is_selected("PO-0427"));
        assert!(s.show_filter && s.show_assign && s.is_rebill_empty());

        // Tab filtering + sorting resolve against items, not indices.
        s.po_tab = 2; // Awaiting
        let visible = s.po_visible();
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|i| i.status == PoStatus::Awaiting));
        s.po_toggle_sort(PoSortField::Amount);
        let sorted = s.po_visible();
        assert_eq!(sorted[0].code.as_ref(), "PO-0427"); // $1,960 first
        s.po_toggle_sort(PoSortField::Amount);
        let sorted = s.po_visible();
        assert_eq!(sorted[0].code.as_ref(), "PO-0421"); // $4,480 first

        // Approval moves Awaiting → Approved and drops their selection.
        s.po_tab = 0;
        assert_eq!(s.po_approve_selected(), 1); // only PO-0421 is selected+awaiting
        assert_eq!(s.po_awaiting_count(), 1);
        assert_eq!(s.po_selected_count(), 1); // PO-0418 (billed) stays picked

        // Chart selections toggle and clear.
        s.toggle_bill_bar(3);
        assert_eq!(s.selected_bill_bar, Some(3));
        s.toggle_bill_bar(3);
        assert_eq!(s.selected_bill_bar, None);
        s.toggle_waffle_cat(2);
        assert_eq!(s.selected_waffle_cat, Some(2));

        // Claims pay + rebill markup feed the card formulas.
        s.mark_claim_paid(0);
        assert_eq!(s.claims_to_pay_count(), 2);
        s.set_rebill_markup(10);
        s.assign_expenses(vec![super::RebillItem {
            name: "Flights".into(),
            amount: 100.0,
        }]);
        assert!(!s.is_rebill_empty());
        assert!((s.rebill_invoice_total() - 110.0).abs() < 1e-9);
        assert_eq!(s.expenses_selected_count(), 0);
        s.rebill_clear();
        assert!(s.is_rebill_empty());
    }

    /// The select-all box toggles by *visible* rows and "Mark billed" moves
    /// every selected PO to Billed, dropping the selection.
    #[test]
    fn po_select_all_and_mark_billed() {
        use super::{DemoState, PoStatus};

        let mut s = DemoState::demo();

        // Tab Awaiting shows two rows; select-all picks only those two and
        // leaves out the Billed row, which isn't on this tab.
        s.po_tab = 2;
        s.po_selected.clear();
        s.po_set_visible_selected(true);
        assert_eq!(s.po_selected_count(), 2);
        assert!(s.po_is_selected("PO-0421") && s.po_is_selected("PO-0427"));
        assert!(!s.po_is_selected("PO-0418"));

        // All visible picked → the next click clears them all.
        s.po_set_visible_selected(false);
        assert_eq!(s.po_selected_count(), 0);

        // Mark billed moves selected non-billed rows and clears selection;
        // an already-billed row is a no-op.
        s.po_tab = 0;
        s.po_set_selected("PO-0421", true);
        s.po_set_selected("PO-0418", true); // already Billed
        assert_eq!(s.po_mark_billed_selected(), 1);
        assert!(
            s.po_items
                .iter()
                .all(|i| i.status == PoStatus::Billed || i.code == "PO-0427")
        );
        assert_eq!(s.po_selected_count(), 0);
        // Nothing selected → no rows moved.
        assert_eq!(s.po_mark_billed_selected(), 0);
    }
}
