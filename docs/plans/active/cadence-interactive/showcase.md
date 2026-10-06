# Cadence Showcase Interactivity & UX Architecture Plan

This plan details how to transform the finance UI showcase components in `src/cadence` from static displays into fully interactive, high-fidelity components with persistent state, keyboard navigation, hover/active/focus affordances, and cross-card reactive flows wired into `DemoState`.

All open audit findings from `/tmp/mono-desk-ui-audit-a.md` and `/tmp/mono-desk-ui-audit-b.md` pertaining to showcase flows and icon assets are folded directly into this plan.

---

## 1. Audit Findings Folded into Showcase Scope

| ID | Origin | Target Component | Defect & Scope | Planned Resolution |
|---|---|---|---|---|
| **H1** | Audit B | Card 5 claim tiles, close buttons, Card 8 | Icons pinned to top-left due to `items_center().justify_center()` without `.flex()`. | Insert `.flex()` on icon tile wrappers, `close_button` containers, and claim-row tiles. |
| **H2** | Audit B | Card 1 Gantt header | Gantt header children in wrong order (`["Sep 1", "Oct 1", "Today"]`), placing "Today" on right edge and "Oct 1" over the marker. | Reorder children: `"Sep 1"`, `MonoPill::new("Today").solid()`, `"Oct 1"`. |
| **H4** | Audit B | Card 2 RangeSlider | Range slider thumbs anchored by left edge (`left(relative(at))`), causing an 8px offset. | Apply `.left(relative(at)).ml(px(-8.))` to center thumbs on values. |
| **H5** | Audit B | Card 1 Gantt column | "Today" vertical marker line broken by 4px row gaps between cells and header. | Use `.self_stretch()` on the 200px cell or negative bleed margins matching row padding (`py_2`). |
| **H6** | Audit B | Card 1 PO table header | PO table header lacks `px_2` and checkbox column spacer, misaligning "Supplier" and "Amount" with data rows. | Add `.px_2()` to header row and align with checkbox spacer (`w(px(18.))` + `gap_3`). |
| **F8** | Audit A | Card 1, 2, 7, 8 Buttons | `btn_primary` hover/active background is set to `cx.theme().button_foreground` (#2e2e2e), rendering hover invisible. | Set `.hover(cx.theme().foreground.alpha(0.82))` and `.active(cx.theme().foreground.alpha(0.75))`. |
| **F10 / M1** | Audits A & B | Global / Showcase | Icon props restricted to 101-icon `gpui_kit::component::IconName`, forcing substitutions (`Map` for Flights/Mileage, `FileText` for Meals/Claims, `LayoutDashboard` for Suppliers). | Switch `AppAssets` to `AllAssets`, genericize component icon props to `impl IconNamed`, and use authentic Lucide icons (`Plane`, `CarFront`, `Utensils`, `Receipt`, `HandCoins`, `SlidersHorizontal`). |
| **F12 / M3** | Audits A & B | Card 7 EmptyState | Empty-state illustration well double-insets (`mx_4` on illustration plus `px_4` on container). | Remove `.mx_4()` from `rebill_illustration` call site in `showcase.rs:1071`. |
| **F14** | Audit A | Card 5 Claim rows | Claim row duplicates `icon_tile` with arbitrary 32px box, 8px radius, and no `flex_shrink_0`. | Standardize on parameterized `icon_tile_sized(icon, px(32.), px(16.), px(8.), cx)`. |
| **F15** | Audit A | Card 1 & 5 FilterChip | Element `id` assigned to the tiny × button instead of the outer chip container. | Place `.id(self.id)` on the outer chip and derive `{id}-remove` for the × target. |
| **M4** | Audit B | Card 6 Waffle Grid | Waffle grid hardcodes 14px cell size with no width-fill builder, leaving dead space. | Expose `.cell_size(px)` or flex column sizing so waffle grid spans the card content box. |
| **M6** | Audit B | Card 1, 5, 8 Numerals | Numerals in `FilterChip`, claim subtitles ("Mara K. · Sep 18 · 6230 Travel"), and subtitles rendered in UI sans font. | Route numeric substrings through `blocks::mono`. |
| **L4** | Audit B | Card 7 Header | "0 to invoice" rendered as raw muted text instead of `MonoPill`. | Use `MonoPill::new("0 to invoice").tone(PillTone::Neutral)`. |
| **L5** | Audit B | Card 1 Gantt | Stale comment states track is 220px while implementation is 200px. | Update comment and normalize track width constant. |

---

## 2. Component-by-Component Specifications

### 2.1 PO Table Selection, Sorting, & Bulk Actions
- **Current Behavior (`src/cadence/showcase.rs:67-278`, `src/cadence/mod.rs:183-223`):**
  - Data rows and amounts are statically defined (`PO_AMOUNTS` at :67, `rows` at :80-100).
  - Selection is stored in a fixed array `DemoState.po_rows: [bool; 3]` and toggled solely via `CadenceCheckbox` (:138-142).
  - Row container `h_flex` (:131) has no hover state, no pointer cursor, no keyboard event listeners, and no row-click selection toggle.
  - Header row (:106-125) has no select-all checkbox, no `px_2` padding, and no column sort indicators (Finding H6).
  - Tabs (:183-195) write to `DemoState.po_tab` but do not filter rows, adjust tab counts, or recompute totals.
  - Bulk action buttons (:271-275) `btn_secondary("po-mark-billed")` and `btn_primary("po-approve", "Approve 2")` are completely inert (no `.on_click`), hardcode the label `"Approve 2"`, and hover states are invisible (Finding F8).
- **Target Interaction + UX Details:**
  - **Tri-State Header Checkbox:** Renders checked when all visible rows are selected, unchecked when none are, and indeterminate (dash glyph) when partially selected. Clicking toggles selection across all currently filtered rows.
  - **Row Interaction:** Entire row is clickable (`cursor_pointer()`) to toggle selection. Hover state applies subtle accent tint (`cx.theme().accent.alpha(0.5)` when unselected; `Accent::wash(cx)` when selected).
  - **Keyboard Navigation & Focus:** Rows receive `track_focus(&focus_handle)` and `.focus(|s| s.border_1().border_color(Accent::solid(cx)))`. `ArrowUp`/`ArrowDown` moves active focus; `Space` toggles selection; `Enter` opens purchase order details.
  - **Column Sorting:** Headers for "Supplier" and "Amount" are clickable with interactive sort indicators (Lucide `ArrowUpDown`, `ArrowUp`, `ArrowDown`). Clicking toggles Neutral → Ascending → Descending.
  - **Tab Filtering:** Selecting tabs ("All", "Draft", "Awaiting", "Approved", "Billed") filters rows to matching statuses. Tab counts reflect live datasets.
  - **Reactive Bulk Actions:** Button labels dynamically update (`format!("Approve {picked}")`). Buttons disable via `.disabled(picked == 0)` when no items are selected. Clicking "Approve":
    - Transitions selected POs from `Awaiting` to `Approved`.
    - Decrements the header pill `"2 need you"` dynamically (turning into `"0 need you"` or a green checkmark).
    - Dispatches a toast notification: `window.push_notification(Notification::success(format!("Approved {picked} purchase orders")), cx)`.
    - "Mark billed" advances status to `Billed` (updating Gantt progress to 100%) and dispatches `Notification::info`.
- **State Ownership:**
  - Shared in `DemoState`:
    - `po_tab: usize`
    - `po_selected: std::collections::HashSet<String>` (or bitmask by PO ID, replacing rigid `[bool; 3]`)
    - `po_sort: Option<(PoSortField, SortDirection)>`
    - `po_items: Vec<PurchaseOrderItem>` (carrying supplier name, PO code, status, amount, and Gantt segments)
  - Component-local: Hover states, `FocusHandle` per row.
- **GPUI APIs to Use (Verified):**
  - `div().id(...)`, `track_focus(&FocusHandle)`, `focus(...)`, `hover(...)`, `active(...)`
  - `on_click(...)`, `on_key_down(...)`
  - `gpui_kit::component::button::Button::disabled(bool)`, `Button::on_click(...)`
  - `window.push_notification(Notification::success(...), cx)`
  - `DemoState::update(cx, |s| ...)`
- **Edge Cases:**
  - Zero rows selected: Bulk buttons disabled, amount sum displays `$0.00`, select-all is unchecked.
  - Switching tabs with active selections: Selection is preserved across tab transitions, but bulk action counts reflect visible items.
  - Empty search or filter results: Render a clean inline empty state ("No purchase orders match this filter") with an "Add filter" or "Reset" action.
- **Effort:** **L**
- **Dependencies:** Icon catalog (sort arrows), Gantt component, `blocks::btn_primary`, `blocks::btn_secondary`.

---

### 2.2 Gantt Micro-Chart & Timeline Interactions
- **Current Behavior (`src/cadence/charts.rs:430-485`, `src/cadence/showcase.rs:280-332`):**
  - Rendered as static pill segments (`pill(frac)`) with fixed 8px height and 2px gap in an `h_flex`.
  - Delivery marker is an inert 10px circular div.
  - Header children order is incorrect: `["Sep 1", "Oct 1", "Today"]`, misplacing the "Today" pill on the far right (Finding H2).
  - "Today" marker line has visible 4px gaps at row boundaries (Finding H5).
  - Segments and delivery dots have no hover states, no cursor styling, no tooltips, and no drag interactions.
- Target Interaction + UX Details:
  - **Layout Fixes (H2 & H5):**
    - Correct header child sequence to `"Sep 1"`, `MonoPill::new("Today").solid()`, `"Oct 1"`.
    - Extend the vertical "Today" indicator seamlessly through the table column using `.self_stretch()` on the 200px cell container.
  - **Hover Affordances & Rich Floating Tooltips:**
    - Each segment (Solid, Faint, Dashed) and the Delivery marker receives `.hover(|this| this.brightness(1.15))` with a pointer cursor.
    - Hovering renders a floating tooltip positioned above the segment slot:
      - Solid: `"Billed: 42% ($2,570.40) · Paid Sep 14"`
      - Faint: `"Ordered: 20% ($896.00) · Dispatched"`
      - Dashed: `"Pending Approval · Approval required"`
      - Delivery marker: `"Estimated Delivery: Oct 12 · Northwind Freight"`
  - **Milestone Inspection & Drag Rescheduling:**
    - Clicking the Delivery marker opens a milestone delivery sheet/dialog (`window.open_sheet`) showing transit checkpoints.
    - Dragging the Delivery marker (`on_mouse_down`, `on_mouse_move`, `on_mouse_up`) allows scrub-rescheduling the delivery timeline, interactively updating the fraction and date tooltip.
  - **Timeline Scrubber:**
    - The "Today" line is interactive: scrubbing left/right shifts the relative milestone reference line.
- **State Ownership:**
  - Gantt segment data owned by `PurchaseOrderItem` in `DemoState`.
  - Hovered segment index `(row_ix, seg_ix)` stored locally or in `DemoState.gantt_hover`.
  - Dragging state (active drag origin, current fraction) managed via `window.use_state` or component interaction handlers.
- **GPUI APIs to Use (Verified):**
  - `div().id(...)`, `on_hover(...)`, `on_mouse_down(...)`, `on_mouse_move(...)`, `on_mouse_up(...)`
  - `cursor_pointer()`, `cursor_col_resize()`, `cursor_ew_resize()`
  - `relative()`, `absolute()`, `left(relative(frac))`
  - `div().tooltip(|cx| ...)` / custom popover tooltip
- **Edge Cases:**
  - Total fraction sum > 1.0 clamped via `.clamp(0.0, 1.0)`.
  - Delivery marker dragging clamped to `0.0..=1.0` so it cannot escape the track boundary.
  - Tooltips near canvas edges repositioned to prevent viewport clipping.
- **Effort:** **M**
- **Dependencies:** PO Table row integration, `Accent` color tokens.

---

### 2.3 Expense Claims List
- **Current Behavior (`src/cadence/showcase.rs:772-959`):**
  - Header carries static `IconName::FileText` and trailing `MonoPill::new("3 to pay")`.
  - Segmented control (:816-826) updates `state.claims_tab`, but neither filters rows nor updates financial stats.
  - Month chip (:827) is inert.
  - Stat trio (:834-884) displays hardcoded `$2,316.40`, `$1,284.00`, `$1,032.40`, and static `Meter::new(0.57)`.
  - Claim rows (:909-956) render hardcoded entries with forced icon substitutions (`Map` for Flights/Mileage, `FileText` for Meals; Findings F10, M1).
  - Icon tile is missing `.flex()`, pinning icons top-left (Finding H1), and duplicates `icon_tile` dimensions (Finding F14).
  - Numerals in claim subtitles bypass `mono()` (Finding M6).
  - "Pay" button (:951-954) on each row is completely inert.
- **Target Interaction + UX Details:**
  - **Segmented Control Reactive Filtering:**
    - Tab 0 ("Your own"): Filters to user's personal submitted claims; updates meter and stats.
    - Tab 1 ("To review" - count 2): Displays claims requiring manager sign-off.
    - Tab 2 ("To pay" - count 3): Displays claims awaiting reimbursement payment.
  - **Interactive "Pay" Workflow:**
    - Clicking `btn_secondary("claim-pay-{ix}", "Pay")`:
      - Row updates visual status: displays green `Paid` checkmark or dims with `Accent::wash(cx)`.
      - Financial recalculation: Claim amount is deducted from "To Be Paid" and added to "Reimbursed".
      - Progress bar update: `Meter::new(ratio)` dynamically updates fill width.
      - Segmented count ("To pay 3" → "To pay 2") and CardHeader pill ("3 to pay" → "2 to pay") decrement.
      - Dispatches toast: `window.push_notification(Notification::success(format!("Processed {amount} reimbursement to {recipient}")), cx)`.
  - **Row Expansion & Receipt Inspection:**
    - Clicking a claim row opens an expense detail drawer/sheet via `WindowExt::open_sheet`:
      - Shows receipt image/breakdown, category code ("4200 Travel"), tax amount, and audit history.
    - Hover state: `.hover(|this| this.bg(cx.theme().accent.alpha(0.6)))` with pointer cursor.
    - Keyboard: Up/Down arrow selection, `Enter` to trigger Pay, `Space` to inspect.
  - **Icon & Typography Fixes:**
    - Authentic Lucide icons: `Plane` (Flights), `Utensils` (Client dinner), `CarFront` (Mileage), `Receipt` (CardHeader).
    - Center icons using `.flex().items_center().justify_center()` (Finding H1).
    - Format all numeral sequences in claim subtitles using `blocks::mono` (Finding M6).
- **State Ownership:**
  - Shared in `DemoState`:
    - `claims_tab: usize`
    - `claims: Vec<ClaimItem>` with fields: id, title, subtitle, amount, category, status (`Pending`, `Paid`), submitter, icon.
    - Computed financial properties (`submitted_total()`, `to_be_paid_total()`, `reimbursed_total()`, `reimbursement_ratio()`).
  - Local state: Active sheet entity, row hover/focus handles.
- **GPUI APIs to Use (Verified):**
  - `Segmented::new(...).on_pick(...)`
  - `Button::new(...).on_click(...)`
  - `WindowExt::open_sheet(...)`
  - `window.push_notification(Notification::success(...), cx)`
  - `DemoState::update(cx, |s| ...)`
- **Edge Cases:**
  - All claims in queue paid: Displays celebratory empty state ("All claims settled") and disables bulk pay.
  - Meter fraction 0.0 or 1.0 handled safely without division-by-zero or track overflow.
- **Effort:** **M**
- **Dependencies:** Icon catalog (Lucide `Plane`, `Utensils`, `CarFront`, `Receipt`), `controls::Segmented`, `blocks::Meter`.

---

### 2.4 Empty State & Rebill Expenses Flow
- **Current Behavior (`src/cadence/blocks.rs:596-645`, `src/cadence/showcase.rs:1054-1096`):**
  - Card 7 (`billable_expenses`) renders static `EmptyState`.
  - Trailing header slot uses raw muted text "0 to invoice" instead of `MonoPill` (Finding L4).
  - Illustration container has `.mx_4()` causing double-inset with `EmptyState.px_4()` (Findings F12, M3).
  - Illustration (:1100-1199) consists of static non-interactive divs.
  - Action buttons "Assign expenses" and "Set markup" are completely inert.
  - Card 7 remains permanently empty even when expenses are assigned in Card 8.
- **Target Interaction + UX Details:**
  - **Cross-Card Reactive Flow (Card 7 ↔ Card 8):**
    - Initial state (0 assigned expenses): Card 7 displays `EmptyState` with corrected single 16px inset.
    - Upon assigning expenses in Card 8 (e.g. Atlas Studio + 3 expenses): Card 7 dynamically transitions from `EmptyState` to the **Populated Billable Expenses Table**:
      - Header trailing pill updates to `MonoPill::new("3 to invoice · $1,082.40").tone(PillTone::Accent)` (includes 10% markup).
      - Table renders itemized billable rows: Flights ($412 + 10%), Hotel ($388 + 10%), Client dinner ($184 + 10%) linked to "Atlas Studio".
      - Action row switches to:
        - Primary: `btn_primary("rebill-generate", "Generate Invoice ($1,082.40)", cx)` (opens invoice preview modal).
        - Secondary: `btn_secondary("rebill-clear", "Clear Assignments")` (resets Card 7 back to `EmptyState`).
  - **"Assign expenses" Action:**
    - In `EmptyState`, clicking "Assign expenses" scrolls/focuses Card 8 and applies a transient pulse ring (`Accent::solid(cx)`), or opens an assign sheet.
  - **"Set markup" Popover:**
    - Clicking "Set markup" opens a popover with percentage presets (5%, 10%, 15%, 20%) or an input slider.
    - Selecting a markup updates `DemoState.rebill_markup_pct`, updating the aside "Default markup {pct}%" and recalculating invoice totals immediately.
  - **Interactive Illustration Depth:**
    - Hovering the illustration well applies subtle parallax elevation to the floating card (`shadow_md`, negative Y translation) and highlights card stack borders.
- **State Ownership:**
  - Shared in `DemoState`:
    - `rebill_markup_pct: u32` (default 10)
    - `rebill_assigned_items: Vec<RebillItem>`
    - `is_rebill_empty(&self) -> bool`
- **GPUI APIs to Use (Verified):**
  - `Button::new(...).on_click(...)`
  - `window.open_sheet(...)`, `window.open_dialog(...)` via `WindowExt`
  - `window.push_notification(...)`
  - `div().hover(|s| ...)`
- **Edge Cases:**
  - 0% markup selected: Markup line displays "+$0.00 (0%)".
  - Clearing assignments restores `EmptyState` cleanly with default copy.
- **Effort:** **M**
- **Dependencies:** Card 8 (Assign to project), `blocks::EmptyState`, `blocks::MonoPill`.

---

### 2.5 Card-Level Actions Across Showcase Cards
- **Current Behavior (`src/cadence/showcase.rs` across Cards 2, 3, 4, 6, 8):**
  - Close buttons in Cards 2 and 8 (:559-582) only dispatch `Notification::info("Panel closed (demo)")`. Panels never collapse or hide.
  - Close button container is missing `.flex()`, pinning the × icon top-left (Finding H1).
  - Card 2 (`filter_panel`): RangeSlider is static with misaligned thumbs (Finding H4); date inputs are inert divs; "Reset" and "Show bills" buttons are inert.
  - Card 8 (`assign_to_project`): Search field (:1234-1264) is a hardcoded mock div showing `"Atl"` and `"3 results"`. Expense total `$984.00` and button `"Assign 3"` are hardcoded strings. "Cancel" and "Assign 3" buttons are inert.
  - Card 3 (`upcoming_bills`): Stat items and bar chart tooltips are static.
  - Card 4 (`owed_to_suppliers`): Aging bars and supplier rows are non-clickable.
  - Card 6 (`by_category`): Waffle grid cells are inert and hardcode 14px size leaving dead space (Finding M4); legend items have no click handlers.
- **Target Interaction + UX Details:**
  - **Panel Dismiss & Collapse (Cards 2 & 8):**
    - Clicking `close_button` toggles `DemoState.filter_panel_open` / `assign_panel_open`.
    - Collapsing transitions the card into a compact chip on the canvas ("+ Filter bills" / "+ Assign expenses"); clicking the chip re-expands the card.
    - Fix close button icon centering with `.flex().items_center().justify_center()` (Finding H1).
  - **Card 2 Interactive Controls:**
    - RangeSlider: Add thumb drag handlers (`on_mouse_down`, `on_mouse_move`, `on_mouse_up`) updating `DemoState.slider`. Fix thumb centering using `.left(relative(at)).ml(px(-8.))` (Finding H4).
    - Reset button: Resets all filters in `DemoState` to defaults and emits `Notification::info`.
    - Show bills button: Applies filters to Card 3 and scrolls/focuses the bills queue.
  - **Card 8 Interactive Search & Dynamic Assignment:**
    - Real Search Input: Replace mock div with live filtering input. Typing filters `projects` in real time, updating the count label `"{count} results"`.
    - Dynamic Totals: Checkbox toggles dynamically sum checked expenses into the header total (e.g. checking Flights + Hotel displays `$800.00`).
    - Dynamic Button Label: Button displays `format!("Assign {count}")`, disabled when count is 0 or no project is picked.
    - Assign Action: Confirms assignment, updates tags from "Pending" to the project name, triggers Card 7 population, and emits `Notification::success`.
    - Cancel Action: Clears selections and collapses panel.
  - **Drill-Downs in Cards 3, 4, 6:**
    - Card 3: Hovering any bar column shows a tooltip at that column's exact slot; clicking a stat bucket filters the chart.
    - Card 4: Clicking a supplier row filters Card 1 PO table to that supplier. Clicking aging bars filters bills by age bucket.
    - Card 6: Hovering a waffle legend item highlights corresponding cells; clicking a category filters Card 5 expense claims. Expose `.cell_size()` to fill card width (Finding M4).
- **State Ownership:**
  - `DemoState`:
    - `filter_panel_open: bool`, `assign_panel_open: bool`
    - `project_search: SharedString`
    - `selected_aging_bracket: Option<usize>`
    - `selected_category: Option<SharedString>`
    - `hovered_bar_column: Option<usize>`
- **GPUI APIs to Use (Verified):**
  - `div().on_mouse_down(...)`, `on_mouse_move(...)`, `on_mouse_up(...)`
  - `Button::new(...).on_click(...)`
  - `window.push_notification(...)`
- **Effort:** **L**
- **Dependencies:** All cards across showcase, `charts` module, `blocks` module.

---

### 2.6 Icon Catalog Architecture (101-Icon Subset vs Full Lucide Catalog)
- **Current Behavior (`src/assets.rs:10-36`, `src/cadence/blocks.rs:33, 58, 179`):**
  - `AppAssets` delegates to `gpui_kit::assets::Assets`, which embeds only ~118 default icons from `default-icons.txt`.
  - Component props in `blocks.rs` (`CardHeader::new`, `icon_tile`, `FilterChip::icon`) take `gpui_kit::component::IconName` (the 101-icon enum).
  - Attempting to load unbundled Lucide icons (e.g. `Plane`, `CarFront`, `Utensils`, `Receipt`, `HandCoins`, `SlidersHorizontal`) fails at runtime, forcing inaccurate visual substitutions (Findings F10, M1).
- **Target Interaction + UX Details:**
  - **Runtime Asset Pipeline Resolution:**
    - Update `src/assets.rs` to delegate non-brand asset loading to `gpui_kit::assets::AllAssets` instead of `Assets`.
    - `AllAssets` embeds the complete catalog of ~2800 Lucide SVGs via `rust-embed`.
  - **Type Genericization in `blocks.rs`:**
    - Update signatures to accept `impl IconNamed`:
      - `pub fn icon_tile(icon: impl IconNamed, cx: &App) -> Div`
      - `pub fn new(icon: impl IconNamed, title: impl Into<SharedString>, subtitle: impl Into<SharedString>) -> Self` on `CardHeader`
      - `pub fn icon(mut self, icon: impl IconNamed) -> Self` on `FilterChip`
    - Because both `gpui_kit::component::IconName` and `gpui_kit::assets::IconName` implement `IconNamed`, existing call sites remain source-compatible while full Lucide variants become accessible.
  - **Authentic Showcase Glyphs:**
    - Expense claims rows: `Plane` for flights, `Utensils` for meals, `CarFront` for mileage.
    - Card headers: `Receipt` for Expense claims, `HandCoins` for Owed to suppliers, `ChartPie` for By category, `SlidersHorizontal` for Filter bills.
- **State Ownership:** Stateless asset loading registered in `main.rs:752`.
- **GPUI APIs to Use (Verified):**
  - `gpui_kit::assets::AllAssets` (implements `gpui::AssetSource`)
  - `gpui_kit::assets::IconName` (~2800 Lucide variants)
  - `gpui_kit::assets::IconNamed`
- **Edge Cases:**
  - Binary size impact: `AllAssets` adds ~1.2 MB compressed SVG data, standard for desktop targets.
  - Unit test in `src/assets.rs:39-70` (`brand_mark_loads_and_icons_still_delegate`) must continue passing.
- **Effort:** **S**
- **Dependencies:** Prerequisite for authentic glyphs in Cards 1, 2, 4, 5, 6.

---

## 3. Ordered Implementation Phases & Verification Steps

### Phase 1: Foundation, Asset Pipeline, & Control Centering
1. In `src/assets.rs`, delegate `AppAssets::load` and `AppAssets::list` to `gpui_kit::assets::AllAssets`.
2. In `src/cadence/blocks.rs`, genericize `icon_tile`, `CardHeader::new`, and `FilterChip::icon` to accept `impl IconNamed`.
3. In `src/cadence/blocks.rs` and `src/cadence/showcase.rs`, insert `.flex()` before `.items_center().justify_center()` in `icon_tile`, `close_button`, `CadenceCheckbox`, and claim row tiles (Fixes H1).
4. In `src/cadence/blocks.rs:563-564`, update `btn_primary` custom variant hover/active colors to use foreground with alpha (Fixes F8).
5. Standardize claim-row icon tiles using `icon_tile_sized` (Fixes F14).
- **Verification:**
  - Fast check: `make check` passes with 0 warnings.
  - Visual check (`make run`): Navigate to Finance UI page. Confirm primary buttons lighten on hover, checkboxes have centered checkmarks, close buttons have centered × icons, and test loading a full Lucide icon.

### Phase 2: PO Table Selection, Sorting, & Bulk Actions
1. Expand `DemoState` in `src/cadence/mod.rs` with `PurchaseOrderItem`, dynamic selection set, and sort state.
2. Fix PO table header insets in `src/cadence/showcase.rs:106-125` by adding `px_2` and a 18px checkbox column spacer (Fixes H6).
3. Implement tri-state Select-All checkbox in the header.
4. Add column sorting on "Supplier" and "Amount" headers with interactive sort indicators.
5. Make row `h_flex` clickable, hoverable, and keyboard-focusable (`track_focus`, `focus`, Space/Enter handlers).
6. Connect tabs ("All", "Draft", "Awaiting", "Approved", "Billed") to filter row collections and update counters.
7. Wire bulk buttons ("Approve {n}", "Mark billed") with dynamic labels, disabled states when 0 selected, state mutations, and `Notification` dispatches.
- **Verification:**
  - Fast check: `make check` passes.
  - Interactive check (`make run`):
    1. Click Select-All: all 3 rows highlight with `Accent::wash`; header displays checked.
    2. Click single row: selection toggles; bulk button label changes to "Approve 1".
    3. Click "Amount" header: table sorts descending, then ascending.
    4. Click "Billed" tab: displays only Northwind Freight.
    5. Select 2 rows and click "Approve 2": success toast appears and "2 need you" pill clears.

### Phase 3: Gantt Micro-Chart & Timeline Interactivity
1. Fix Gantt header child ordering in `showcase.rs:315-325` to `["Sep 1", "Today", "Oct 1"]` (Fixes H2).
2. Fix "Today" marker line continuity across row boundaries using full-height continuous track (Fixes H5).
3. Add hover brightness and rich contextual floating tooltips to each Gantt segment and delivery marker.
4. Add click handler on Delivery marker to open delivery milestone sheet.
5. Add drag/scrub affordance on delivery marker and "Today" timeline line.
- **Verification:**
  - Fast check: `make check` passes.
  - Interactive check (`make run`):
    1. Observe Gantt header: "Today" pill is centered directly over the continuous vertical marker line.
    2. Hover over solid bar: tooltip displays "Billed: 42% ($2,570.40)".
    3. Hover over delivery dot: tooltip displays delivery estimate.
    4. Click delivery dot: opens milestone inspection sheet.

### Phase 4: Expense Claims Flow & Financial State Engine
1. Swap substituted icons in Card 5 with `gpui_kit::assets::IconName::Plane`, `Utensils`, `CarFront`, and `Receipt` (Fixes F10, M1).
2. Wrap claim subtitle numerals in `blocks::mono` (Fixes M6).
3. Connect "Pay" button on each row: marks claim as paid, deducts amount from "To Be Paid", adds to "Reimbursed", updates `Meter` fill, decrements "3 to pay" pill, and shows toast notification.
4. Connect `Segmented` control tabs ("Your own", "To review", "To pay") to filter claim rows and update stat trio figures.
5. Make claim rows clickable to open receipt detail drawer via `WindowExt::open_sheet`.
- **Verification:**
  - Fast check: `make check` passes.
  - Interactive check (`make run`):
    1. Confirm Plane, Utensils, and Car icons render centered in Card 5.
    2. Click "Pay" on Flights row: "To Be Paid" drops by $412, "Reimbursed" increases by $412, meter fill advances, header pill updates to "2 to pay", and toast appears.
    3. Switch Segmented control to "Your own": claims queue clears or filters to personal items.

### Phase 5: Empty State & Rebill Expenses Flow (Cross-Card Reactive Flow)
1. Remove `.mx_4()` from `rebill_illustration` call site in `showcase.rs:1071` (Fixes F12, M3).
2. Replace raw muted text with `MonoPill::new("0 to invoice").tone(PillTone::Neutral)` in Card 7 header (Fixes L4).
3. Wire "Assign expenses" in Card 7 `EmptyState` to focus/scroll Card 8 with an accent pulse.
4. Wire "Set markup" to open markup selection popover, updating `DemoState.rebill_markup_pct`.
5. Implement populated Billable Expenses view in Card 7: displays itemized rows with markup breakdown and "Generate Invoice" action once Card 8 assigns expenses.
6. Add "Clear / Unassign" action to return Card 7 back to `EmptyState`.
- **Verification:**
  - Fast check: `make check` passes.
  - Interactive check (`make run`):
    1. Observe Card 7 illustration well aligns with card padding (no double-inset).
    2. Click "Assign expenses" in Card 7: focus transitions to Card 8.
    3. In Card 8, pick "Atlas Studio" and click "Assign 3": Card 7 instantly flips from `EmptyState` to populated table showing "$1,082.40 to invoice"!
    4. Click "Clear / Unassign" in Card 7: returns cleanly to `EmptyState`.

### Phase 6: Card-Level Actions & Cross-Card Drill-Downs
1. Wire `close_button` in Cards 2 and 8 to toggle collapse/dismiss state in `DemoState`, rendering restore chips on canvas.
2. In Card 8, replace mock search div with real filtering input; dynamically compute header expense total and button label `"Assign {count}"`.
3. In Card 2, add drag handlers to RangeSlider, fix thumb centering via `.ml(px(-8.))` (Fixes H4), and wire "Reset" button to restore filter defaults.
4. Add drill-downs: hovering Card 3 bar columns displays dynamic tooltips; clicking Card 4 supplier rows filters Card 1 PO table; hovering Card 6 waffle legend highlights grid cells and cell sizing fills card width (Fixes M4).
- **Verification:**
  - Fast check: `make check` passes.
  - Interactive check (`make run`):
    1. Click close button on Card 2: card collapses with restore chip available.
    2. Drag RangeSlider thumbs: amount label updates smoothly.
    3. Type "Atl" in Card 8 search input: project list filters in real time.
    4. Toggle expense checkboxes in Card 8: header sum and "Assign {n}" label update dynamically.
    5. Click "Northwind Freight" in Card 4: Card 1 filters to Northwind purchase orders.

---

## 4. Summary & Verification Gate
- Gate status: Read-only planning task. No repository files modified. `make check` passes cleanly.
- Plan artifact stored at: `/tmp/mono-desk-plan-showcase.md`.
