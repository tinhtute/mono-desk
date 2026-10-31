# Cadence Interactive Charts Implementation Plan

**Scope:** `src/cadence/charts.rs`, `src/cadence/blocks.rs` (`Meter`), `src/cadence/mod.rs` (`DemoState`), and chart integration sites in `src/cadence/showcase.rs`.  
**Reference Design:** Cloned finance UI from `x.com/yurygok/status/2105191658337841531`.  
**Mode:** READ-ONLY planning task. No repository files modified.  
**Baseline Verification:** `make check` PASS (`cargo check` dev profile in 0.58s).

---

## 1. Executive Summary & Architecture

The charts in `src/cadence/charts.rs` currently render static markup (`RenderOnce` returning un-interactive `Div` hierarchies). While visually aligned with the finance reference, they lack user interaction: slider thumbs cannot be dragged or tabbed to, bar charts cannot be hovered or inspected, histograms and sparklines do not respond to pointer input or filter synchronization, waffle cells and legend rows are disconnected, and meter progress bars are hardcoded numbers.

This plan details how to make Cadence data marks genuinely interactive with premier desktop UX:
1. **Physical Dragging & Scrubbing:** Fluid pointer dragging on slider thumbs, track click-to-seek, and direct histogram brushing.
2. **Keyboard Accessibility & Focus:** Standard GPUI focus rings (`focus_visible`), tab stops (`tab_index(0)`), and arrow key navigation (`←`/`→`/`↑`/`↓`, `Home`/`End`, `Shift` acceleration).
3. **Rich Hover Inspection & Tooltips:** Dynamic floating tooltips, bar highlight halos, and coordinated dimming across sibling elements.
4. **Unified State Cohesion:** Seamless wiring between local element state (`window.use_keyed_state`) and global showcase state (`DemoState`), linking the range slider directly to the histogram and filter stats, and the waffle chart directly to the category breakdown.
5. **Audit Rectification:** Complete folding of all open chart defects identified in audits A and B (stack overflow H3, thumb center anchor H4, label alignment H7, gridline drift M2, waffle sizing M4, band slot geometry L2, byte-length tooltip bug L3).

---

## 2. Prior Audit Findings Integration Matrix

All chart-specific findings from `/tmp/mono-desk-ui-audit-b.md` (and related geometry notes in `audit-a.md`) are folded directly into this plan:

| Finding | Component & Location | Defect Description | Plan Resolution |
|---|---|---|---|
| **H3** [High] | `BarChartMini` (`charts.rs:209-228`) | Stacked bars overflow plot height (`BILL_BARS[8] = 0.86 + 0.62 = 1.48` → 222px in 150px plot). Overflows upward over tooltip slot. | Clamp/normalize stack: `faint_val = faint.clamp(0., (1. - solid).max(0.))` and add `.overflow_hidden()` to the plot container (`charts.rs:188-191`). |
| **H4** [High] | `RangeSlider` (`charts.rs:398-408`) | Thumbs anchored by left edge (`left(relative(at))`), overhanging track at bounds. | Preserve and verify negative center margin `.ml(px(-8.))` on 16px thumb, ensuring center coordinates align with drag calculation and track ends. |
| **H7** [High] | `BarChartMini` (`charts.rs:233-241`) | X-labels misaligned with bars (`.pl(px(36.))` guess; `justify_between` pins first/last to padded edges instead of bar centers). | Structure the label row with the identical 25-slot column geometry as the bars (`flex_1.items_center.justify_center`), placing labels over exact bar column indices. |
| **M2** [Med] | `BarChartMini` (`charts.rs:179-185`) | Y-axis labels drift inward by half a line-height from gridlines. | Wrap each Y-label in a 1px-high centered container (`div().h(px(1.)).flex().items_center().overflow_visible()`) anchored to gridline coordinates. |
| **M4** [Med] | `Waffle` (`charts.rs:271`) | Waffle grid does not fill card and lacks sizing builder (`cell: px(14.)` fixed → 176px in 298px card well). | Expose `.cell_size(px)` builder and auto-fit cell size to fill card well width evenly across 10 columns (`~26px` cells with 4px gap). |
| **L2** [Low] | `MiniBars` (`charts.rs:61-97`) | Band wash math assumes bars centered in `1/n` slots, but `justify_between` edge-aligns first/last bars, causing a `bar_w/2` offset. | Wrap each bar in a `flex_1.items_end.justify_center` slot, guaranteeing mathematical alignment with the wash band. |
| **L3** [Low] | `BarChartMini` (`charts.rs:153-154`) | Tooltip width estimation `text.len() * 7.5` counts UTF-8 bytes (`·` is 2 bytes), causing asymmetric centering. | Replace byte counting with char count or zero-width flex container centered via `left(relative(slot_w)).items_center.overflow_visible`. |

---

## 3. Component-by-Component Specifications

### 3.1 Range Slider (`RangeSlider`)

#### Current Behavior (`src/cadence/charts.rs:381-440`)
- Purely static visual `RenderOnce` element taking `lo: f32` and `hi: f32`.
- Draws a 3px faint track (`Accent::faint`), a solid accent active span, and two 16px circular thumbs.
- No `ElementId`, no pointer event handlers, no keyboard listeners, no focus handles.
- Rendered in Card 2 (`showcase.rs:458`) with static amount labels (`$500 - $12k`, `$0`, `$5k`, `$20k+`).

#### Target Interaction + UX Details
- **Pointer Dragging on Thumbs:**
  - Thumbs take `cursor_ew_resize()`.
  - Hovering a thumb applies `hover(|style| style.shadow_sm().border_color(Accent::solid(cx)))`.
  - Pressing a thumb initiates drag mode. Moving the pointer recalculates thumb track fraction based on relative horizontal mouse position within the track bounds.
  - Active drag clamps to `[0.0, 1.0]` and enforces a minimum gap `min_gap = 0.02` so thumbs cannot cross or invert.
- **Track Click & Range Scrubbing:**
  - Clicking on the track jumps the nearest thumb to the click position.
  - Clicking to the left of `lo` jumps `lo`. Clicking to the right of `hi` jumps `hi`.
  - Clicking and dragging inside the active span (`lo..hi`) shifts both thumbs simultaneously (range pan/scrub), keeping span width `hi - lo` constant.
- **Keyboard Navigation & Accessibility:**
  - Each thumb is an independent tab stop (`tab_index(0)`), labeled `"Minimum Amount"` and `"Maximum Amount"`.
  - Focused thumb displays a high-contrast focus ring (`focus_visible` with `ring_2().ring_color(Accent::solid(cx))`).
  - Arrow Left / Down decrements thumb by `step = 0.01` (or `$200`).
  - Arrow Right / Up increments thumb by `step`.
  - `Shift` + Arrow increments/decrements by `step * 5` (`$1,000`).
  - `Home` snaps `lo` to `0.0`. `End` snaps `hi` to `1.0`.
  - Screen reader semantics: `role(Role::Slider)`, `aria_numeric_value`, `aria_min_numeric_value`, `aria_max_numeric_value`, `aria_orientation(Orientation::Horizontal)`.
- **Card 2 Synergy:**
  - Range changes immediately recompute the Card 2 Amount header (`$500 - $12k` becomes live, e.g. `$2,400 - $15,600`).
  - Histogram (`MiniBars`) above the slider dynamically updates its `accent_range` to match `[lo_ix, hi_ix]`.

#### State Ownership
- **Global (`DemoState` in `mod.rs:196`):** `pub slider: (f32, f32)` (already initialized to `(0.25, 0.78)`).
- **Local Interaction State:**
  - `track_bounds: Bounds<Pixels>` captured during prepaint.
  - `active_drag: Option<DragTarget>` where `DragTarget = Lo | Hi | Span { initial_x, initial_span }`.

#### GPUI APIs Used & Verified
- `gpui::InteractiveElement::id(ElementId)` — turns `div()` into `Stateful<Div>`.
- `gpui_base::element_ext::ElementExt::on_prepaint` (or `gpui::canvas`) — records track `Bounds<Pixels>`.
- `gpui::InteractiveElement::on_mouse_down(MouseButton::Left, ...)` — track hit testing and span dragging.
- `gpui::StatefulInteractiveElement::on_drag(...)` with a lightweight `Render` struct (`DragThumb`) — initiates drag gesture.
- `gpui::InteractiveElement::on_drag_move(...)` with `DragMoveEvent<DragThumb>` — tracking mouse move across window.
- `gpui::InteractiveElement::on_mouse_up` and `on_mouse_up_out` — releases dragging state.
- `gpui::InteractiveElement::track_focus(&focus_handle)` and `tab_index(0)` — keyboard focus management.
- `gpui::InteractiveElement::on_key_down(...)` — inspects `event.keystroke.key` and `modifiers.shift`.
- `gpui::InteractiveElement::focus_visible(...)` — keyboard-only focus ring.

#### Edge Cases
- Track rendered with 0 width (pre-layout frame): guard division by zero; treat as no-op.
- Collision: `lo` moving right clamps at `hi - min_gap`; `hi` moving left clamps at `lo + min_gap`.
- Dragging cursor outside the application window: `on_mouse_up_out` resets dragging state cleanly.
- Keyboard navigation reaching boundaries: clamp at `0.0` and `1.0` respectively.

#### Effort: **M** (Medium)
#### Dependencies: Card 2 Filter Amount header and `MiniBars` histogram in `showcase.rs`.

---

### 3.2 Stacked Bar Chart Mini (`BarChartMini`)

#### Current Behavior (`src/cadence/charts.rs:102-245`)
- Takes `bars: Vec<(f32, f32)>` (25 stacked bars in `BILL_BARS`), `height: Pixels`, `y_labels`, `tooltip`, `x_labels`.
- Hardcoded static tooltip on bar 8 (`showcase.rs:617`: `.tooltip(8, "Oct 1 · $52k")`).
- Suffers from stack height overflow (Audit H3), x-label misalignments (Audit H7), y-label drift (Audit M2), and byte-length tooltip bug (Audit L3).

#### Target Interaction + UX Details
- **Hover on Bars:**
  - Each of the 25 bar columns receives a unique ID (`id(format!("bill-bar-{ix}"))`).
  - Hovering column `ix`:
    - Hovered bar solid segment brightens; faint segment gets accent wash; subtle rounded hover well/pill behind the column.
    - Sibling bars dim to `opacity(0.70)`, bringing clear visual focus to the inspected day.
    - Tooltip dynamically anchors above the hovered bar:
      `"Oct 1 · $52k (Sched: $30k · Unsched: $22k)"`.
    - Cursor switches to `cursor_pointer()`.
- **Bar Selection (Click & Lock):**
  - Clicking a bar selects it (`DemoState.selected_bill_bar = Some(ix)`).
  - Selected bar receives an active anchor dot or accent triangle on the baseline.
  - Clicking the currently selected bar clears the selection (`None`).
  - When selected, Card 3's stat row or subtitle updates to show figures for that specific day.
- **Keyboard Navigation:**
  - Chart container is a focusable element (`track_focus`, `tab_index(0)`).
  - When focused:
    - Arrow Left decrements selected bar index (`ix.saturating_sub(1)`).
    - Arrow Right increments selected bar index (`(ix + 1).min(bars.len() - 1)`).
    - `Home` selects first bar (`0`), `End` selects last bar (`24`).
    - `Escape` clears selection.
    - Focus visible ring outlines the chart plot well.
- **Layout and Alignment Fixes (Audits H3, H7, M2, L3):**
  - Normalize or clamp: `faint_val = faint.clamp(0., (1. - solid).max(0.))`; add `.overflow_hidden()` on plot.
  - Structure Y-labels in 1px centered wrappers on gridline fractions.
  - Structure X-labels in an identical 25-column slot grid (`flex_1.items_center.justify_center`).
  - Tooltip centering via zero-width wrapper with `left(relative(slot_w)).items_center.overflow_visible`.

#### State Ownership
- **Global (`DemoState` in `mod.rs`):** `pub selected_bill_bar: Option<usize>` (new field, default `Some(8)` to match reference initial state).
- **Local Interaction State:** `hovered_bill_bar: Option<usize>` via `window.use_state` or `on_hover`. Effective tooltip index: `hovered_bill_bar.or(selected_bill_bar).unwrap_or(8)`.

#### GPUI APIs Used & Verified
- `gpui::InteractiveElement::id(ElementId)` — per-bar element ID.
- `gpui::StatefulInteractiveElement::on_hover(...)` — per-bar pointer hover entry/exit.
- `gpui::StatefulInteractiveElement::on_click(...)` — click selection / toggle.
- `gpui::InteractiveElement::track_focus(&focus_handle)` & `tab_index(0)` — chart container focus.
- `gpui::InteractiveElement::on_key_down(...)` — arrow key navigation.
- `gpui::InteractiveElement::focus_visible(...)` — focus indicator.
- `gpui::StyledExt::cursor_pointer()` — pointer affordance.

#### Edge Cases
- Bars at plot edges (`ix = 0` and `ix = 24`): tooltip pill must not clip off the card edge; clamp tooltip horizontal offset.
- All bars deselected: tooltip hides or reverts to default pinned date.
- Font size changes: tooltip layout stays centered and text does not wrap awkwardly.

#### Effort: **M** (Medium)
#### Dependencies: `DemoState.selected_bill_bar`, Card 3 layout in `showcase.rs`.

---

### 3.3 Mini Bars (Histograms & Sparklines) (`MiniBars`)

#### Current Behavior (`src/cadence/charts.rs:13-100`)
- Renders histograms or sparkline strips from `values: Vec<f32>`, `accent_range`, and `band`.
- Purely static divs in `h_flex().justify_between()`.
- Used at two sites:
  1. Card 2 Filter (`showcase.rs:453`): 28-bar amount histogram above `RangeSlider`. Hardcoded `.accent_range((6, 21))`.
  2. Card 3 Sparkline (`showcase.rs:629`): 60-bar cash sparkline. Hardcoded `.accent_range((18, 38)).band((16, 40))`.
- Suffers from band alignment offset (Audit L2).

#### Target Interaction + UX Details
- **Card 2 Histogram: Dynamic Slider Sync & Brushing:**
  - Coordinated directly with `RangeSlider`: `accent_range` and `band` derive dynamically from `state.slider.0` and `state.slider.1`.
  - Direct Brush / Scrub interaction: clicking and dragging across histogram bars updates `state.slider` in real time, making the histogram both a data visualization and a filter input.
- **Card 3 Sparkline: Micro Hover Inspection:**
  - Individual day bars hoverable:
    - Hovering a day bar highlights it with an accent color and expands its height slightly.
    - Displays a micro-tooltip with date and value (e.g. `"Oct 14 · $18.4k"`).
    - Cursor switches to `cursor_pointer()`.
- **Band Geometry Alignment (Audit L2):**
  - Wrap each bar in a `flex_1.items_end.justify_center` slot so bar centers coincide with `ix as f32 / n` band coordinates.

#### State Ownership
- **Card 2 Histogram:** Purely derived from `DemoState.slider`.
- **Card 3 Sparkline:** Local hover state `hovered_spark: Option<usize>` via `window.use_state`.

#### GPUI APIs Used & Verified
- `gpui::InteractiveElement::id(ElementId)` — slot IDs.
- `gpui::InteractiveElement::on_mouse_down(MouseButton::Left, ...)` & `on_drag_move` — histogram brush.
- `gpui::StatefulInteractiveElement::on_hover(...)` — sparkline day hover.
- `gpui::StyledExt::cursor_pointer()` — interactive pointer.

#### Edge Cases
- Dragging backwards across the histogram (right to left): sort indices before setting `DemoState.slider`.
- Very narrow window widths: bar widths stay visible (min 1.5px) without collapsing.

#### Effort: **S-M** (Small to Medium)
#### Dependencies: `RangeSlider`, Card 2 & Card 3 in `showcase.rs`.

---

### 3.4 Waffle Chart (`Waffle`)

#### Current Behavior (`src/cadence/charts.rs:247-307`)
- Renders `cells: Vec<WaffleCell>` in chunks of `cols: usize`.
- Cell size hardcoded to `14px` (Audit M4: only 176px wide in 298px card well).
- Static divs with no click/hover listeners.
- Used in Card 6 ("By category", `showcase.rs:965-1035`) with 5 categories (Travel, Meals, Software, Mileage, Other) and a separate legend list below.

#### Target Interaction + UX Details
- **Card Sizing & Layout (Audit M4):**
  - Expose `.cell_size(Pixels)` builder and scale cells to `~26px` with 4px gap so the 10 columns span the full 298px card well width.
- **Waffle Cell Hover & Coordinated Dimming:**
  - Each cell is interactive (`id(format!("waffle-{ix}"))`).
  - Hovering a cell:
    - Cell scales up (`scale(1.15)`), gaining a subtle shadow and outline.
    - Cells of *other* categories dim to `opacity(0.35)`.
    - Cells of the *same* category stay fully bright, highlighting the category's distribution across the entire waffle.
    - Tooltip displays category details: `"Travel · 18 of 50 claims (36%) · $8,960"`.
    - The corresponding legend row below highlights with an active background wash.
- **Legend Row Hover & Click Filtering:**
  - Hovering a legend row in Card 6 highlights all corresponding waffle cells and dims the rest.
  - Clicking a legend row or waffle cell filters Card 6 to that category (`DemoState.selected_waffle_cat = Some(cat_ix)`):
    - Top stat (`$18,420`) updates to that category's total (e.g. `$8,960`).
    - Subtitle (`Q3 · 50 claims`) updates to `"Travel · 18 claims"`.
    - Legend row displays an active checkmark or solid accent pip.
    - Clicking again resets to "All categories".
- **Keyboard Navigation:**
  - Waffle grid container is focusable (`track_focus`, `tab_index(0)`).
  - Arrow keys (Up, Down, Left, Right) navigate the 5x10 grid.
  - Enter or Space toggles selection of the focused cell's category.
  - Focused cell displays `focus_visible` ring.

#### State Ownership
- **Global (`DemoState` in `mod.rs`):** `pub selected_waffle_cat: Option<usize>` (new field, default `None`).
- **Local Interaction State:** `hovered_waffle_cat: Option<usize>` and `focused_cell: usize`.

#### GPUI APIs Used & Verified
- `gpui::InteractiveElement::id(ElementId)` — unique cell and legend IDs.
- `gpui::StatefulInteractiveElement::on_hover(...)` — cross-highlighting on hover.
- `gpui::StatefulInteractiveElement::on_click(...)` — category filter toggle.
- `gpui::InteractiveElement::track_focus(&focus_handle)` & `tab_index(0)` — keyboard focus.
- `gpui::InteractiveElement::on_key_down(...)` — 2D grid arrow navigation.
- `gpui::InteractiveElement::focus_visible(...)` — active cell ring.
- `gpui::StyledExt::cursor_pointer()` — pointer affordance.

#### Edge Cases
- Navigating out of bounds on 2D grid: clamp row and column indices.
- Empty cells (`WaffleCell::Empty`): fully interactive, mapped to "Other" category.
- Rapid hover transitions across cells of the same category: use category ID check to prevent re-render flickering.

#### Effort: **M** (Medium)
#### Dependencies: Card 6 ("By category") layout and legend in `showcase.rs`.

---

### 3.5 Meters (`Meter`)

#### Current Behavior (`src/cadence/blocks.rs:348-382`)
- Takes `fraction: f32` clamped to `0.0..=1.0`.
- Renders a 6px two-tone rounded progress bar (`Accent::faint` track + `Accent::solid` fill).
- Completely static: hardcoded to `0.57` in Card 5 (Expense claims, `showcase.rs:885`).
- Sits below stats: Submitted ($2,316.40), To Be Paid ($1,284.00), Reimbursed ($1,032.40).

#### Target Interaction + UX Details
- **Dynamic Wiring to Card 5 State:**
  - Fraction is computed dynamically from `DemoState.claims_tab` and `DemoState.expenses`:
    - Switching segment tabs ("Your own", "To review", "To pay") or toggling claim checkboxes dynamically recomputes reimbursed / submitted totals and updates the meter fraction!
- **Hover Inspection & Tooltip:**
  - Meter is interactive: `id("claims-meter")`.
  - Hovering the meter:
    - Bar height smoothly expands from 6px to 8px (`h(px(8.))`), cursor switches to `cursor_pointer()`.
    - Floating tooltip displays exact breakdown:
      `"Reimbursed: $1,032.40 / $2,316.40 (44.6%) · Remaining: $1,284.00"`.
- **Keyboard & Accessibility:**
  - `role(Role::ProgressIndicator)`
  - `aria_numeric_value((fraction * 100.0) as f64)`
  - `aria_min_numeric_value(0.0)`, `aria_max_numeric_value(100.0)`
  - `aria_value(format!("{:.0}% reimbursed", fraction * 100.0))`

#### State Ownership
- **Global (`DemoState` in `mod.rs`):** Calculated from existing `claims_tab: usize` and `expenses: [bool; 3]`.
- **Local Interaction State:** Meter hover state.

#### GPUI APIs Used & Verified
- `gpui::InteractiveElement::id(ElementId)` — interactive ID.
- `gpui::StatefulInteractiveElement::on_hover(...)` — height expansion and tooltip.
- `gpui::StatefulInteractiveElement::role(Role::ProgressIndicator)` — accessible role.
- `gpui::StatefulInteractiveElement::aria_numeric_value(...)` — accessible progress.
- `gpui::StyledExt::cursor_pointer()` — interactive pointer.

#### Edge Cases
- Zero submitted amount: fraction defaults safely to `0.0` (avoid divide-by-zero).
- Fraction > 1.0: clamp visual fill at 1.0 while tooltip reports actual percentage.

#### Effort: **S** (Small)
#### Dependencies: Card 5 in `showcase.rs`, `DemoState.claims_tab`, `DemoState.expenses`.

---

### 3.6 Auxiliary Data Marks: Aging Bars & Gantt (`AgingBars`, `Gantt`)

#### Current Behavior (`src/cadence/charts.rs:309-379`, `442-497`)
- `AgingBars`: 5 vertical columns of horizontal bars (Now, 1-30, 31-60, 61-90, 90+). Static divs in Card 4.
- `Gantt`: Sequential pill segments representing PO statuses in Card 1. Static divs.
- Audit findings: H2 (Gantt header children order), H5 (Today line broken at row boundary), L5 (gantt track width comment).

#### Target Interaction + UX Details
- **AgingBars (Card 4):**
  - Hovering a column highlights the group with a wash background pill and displays a detailed tooltip:
    `"1–30 days overdue · 48 bills · $124,000 (30.0%)"`.
  - Clicking a column filters the "Top Suppliers" list below in Card 4 (`DemoState.selected_aging_bucket = Some(ix)`).
  - Keyboard navigation: Left/Right arrows navigate buckets.
- **Gantt Rows (Card 1):**
  - Hovering a Gantt segment shows milestone tooltip:
    - Solid: `"Billed: $18,400 (100%)"`
    - Faint: `"Ordered: $4,200 (in transit)"`
    - Dashed: `"Pending approval: $9,600"`
    - Delivery dot: `"Estimated delivery: Oct 12"`

#### State Ownership
- `DemoState.selected_aging_bucket: Option<usize>` (new field). Local hover state for Gantt tooltips.

#### Effort: **S-M** (Small to Medium)
#### Dependencies: Card 4 and Card 1 in `showcase.rs`.

---

## 4. State Schema & Wireup Plan

### Changes to `DemoState` (`src/cadence/mod.rs:182-223`)

```rust
pub struct DemoState {
    // Existing fields
    pub po_tab: usize,
    pub po_rows: [bool; 3],
    pub date_filter: bool,
    pub overdue_only: bool,
    pub doc_bills: bool,
    pub doc_credit_notes: bool,
    pub slider: (f32, f32),               // Filter dual-handle slider
    pub claims_tab: usize,
    pub assigned: Option<usize>,
    pub expenses: [bool; 3],

    // New chart interaction fields
    pub selected_bill_bar: Option<usize>,  // Card 3: selected Upcoming Bills bar (default: Some(8))
    pub selected_waffle_cat: Option<usize>,// Card 6: selected By Category group (default: None)
    pub selected_aging_bucket: Option<usize>, // Card 4: selected Aging bucket (default: None)
}

impl DemoState {
    pub fn demo() -> Self {
        Self {
            po_tab: 0,
            po_rows: [true, true, false],
            date_filter: true,
            overdue_only: true,
            doc_bills: true,
            doc_credit_notes: true,
            slider: (0.25, 0.78),
            claims_tab: 2,
            assigned: Some(0),
            expenses: [true, true, true],
            selected_bill_bar: Some(8),
            selected_waffle_cat: None,
            selected_aging_bucket: None,
        }
    }
}
```

### Call Sites Wiring in `src/cadence/showcase.rs`
1. **Card 2 (Filter Panel):**
   - `RangeSlider` emits `(lo, hi)` → `DemoState::update(cx, |s| s.slider = (lo, hi))`.
   - Card 2 Amount header maps `s.slider` to currency string: `$20k * lo .. $20k * hi`.
   - `MiniBars` histogram `accent_range` derives from `((lo * 27.0).round(), (hi * 27.0).round())`.
2. **Card 3 (Upcoming Bills):**
   - `BarChartMini` bar click → `DemoState::update(cx, |s| s.selected_bill_bar = if s.selected_bill_bar == Some(ix) { None } else { Some(ix) })`.
   - Card 3 stat row / subtitle displays metrics for `s.selected_bill_bar`.
3. **Card 5 (Expense Claims):**
   - `Meter::new(fraction)` calculates `fraction` from active tab and selected `s.expenses` checkboxes.
4. **Card 6 (By Category):**
   - `Waffle` and legend row click → `DemoState::update(cx, |s| s.selected_waffle_cat = if s.selected_waffle_cat == Some(cat) { None } else { Some(cat) })`.
   - Header amount updates to category amount when filtered.
5. **Card 4 (Owed to Suppliers):**
   - `AgingBars` click → `DemoState::update(cx, |s| s.selected_aging_bucket = Some(ix))`.

---

## 5. Ordered Implementation Phases & Verification Steps

### Phase 1: Range Slider Interactivity & Filter Card Synergy
**Goals:**
1. Make `RangeSlider` interactive with pointer dragging on thumbs, track click-to-seek, and keyboard arrow controls.
2. Ensure thumbs are center-anchored (`.ml(px(-8.))`, Audit H4) and respect `min_gap`.
3. Wire `RangeSlider` to `DemoState.slider` in Card 2, connecting live amount formatting and `MiniBars` histogram accent range.
4. Fix `MiniBars` slot geometry (`flex_1.items_end.justify_center`, Audit L2).

**Verification Steps:**
- Run `make check` → must compile cleanly with 0 warnings/errors.
- Run `make run` → navigate to "Finance UI" in the settings rail.
- In Card 2 (Filter):
  1. Click and drag the left slider thumb: thumb moves smoothly under pointer; amount label updates from `$500` upwards; histogram bars dynamically colorize/decolorize in sync.
  2. Click and drag the right slider thumb: thumb moves smoothly; cannot cross left thumb.
  3. Click on the track to the right of the active range: right thumb jumps to click position.
  4. Press `Tab` to focus a thumb: visible focus ring appears. Press `←`/`→` to step by 1%; hold `Shift` + `←`/`→` to step by 5%.

---

### Phase 2: Bar Chart Mini Hover, Selection & Layout Alignment
**Goals:**
1. Fix stacked bar overflow by clamping/normalizing stack sums (`faint <= 1.0 - solid`) and adding `.overflow_hidden()` on plot (Audit H3).
2. Fix Y-axis label vertical drift with centered 1px line wrappers (Audit M2).
3. Fix X-axis label slot alignment by matching 25-column slot geometry (Audit H7).
4. Fix tooltip byte-length calculation and center with flex wrapper (Audit L3).
5. Add interactive hover tooltips and selection clicking on bars, wiring to `DemoState.selected_bill_bar`.
6. Add keyboard arrow navigation when chart is focused.

**Verification Steps:**
- Run `make check` → passes.
- Run `make run` → navigate to "Finance UI".
- In Card 3 (Upcoming Bills):
  1. Inspect column 8: bars do not overflow into header/stat area (height clamped neatly to 150px).
  2. Y-axis labels align precisely with gridlines; X-axis labels align directly beneath their respective bar columns.
  3. Hover mouse across columns: hovered column highlights, sibling columns dim to 70%, and a floating tooltip dynamically tracks the hovered bar showing date and scheduled/unscheduled amounts.
  4. Click on bar 12: bar becomes selected with an active baseline marker.
  5. Press `Tab` to focus chart: press `←`/`→` to step through bars; tooltip and highlight move across the chart.

---

### Phase 3: Waffle Chart Resizing, Cell Hover & Category Legend Sync
**Goals:**
1. Sizable waffle cells (`.cell_size(px)`), expanding cells from 14px to ~26px to evenly fill the 298px card well (Audit M4).
2. Add cell hover interaction: hovered cell scales up; cells of different categories dim to 35%; cells of same category stay bright.
3. Show rich tooltip on hovered cell (`"Travel · 18 of 50 claims (36%) · $8,960"`).
4. Coordinated legend interaction: hovering legend row highlights corresponding waffle cells; clicking legend row filters Card 6 stat and subtitle (`DemoState.selected_waffle_cat`).
5. Add 2D keyboard navigation (Up/Down/Left/Right) across the 50-cell grid.

**Verification Steps:**
- Run `make check` → passes.
- Run `make run` → navigate to "Finance UI".
- In Card 6 (By Category):
  1. Waffle grid fills card well width symmetrically without large empty right margin.
  2. Hover mouse over a Travel cell: all 18 Travel cells remain bright while Meals/Software/Mileage/Other cells dim, clearly revealing category distribution. Tooltip shows category count and amount.
  3. Hover mouse over "Meals" in the legend: all Meals cells light up in the waffle.
  4. Click "Meals": Card 6 header amount switches from `$18,420` to `$4,210`, subtitle switches to `"Meals · 13 claims"`. Click "Meals" again to reset.
  5. Tab into waffle grid: press arrow keys to navigate cell by cell with visible focus ring.

---

### Phase 4: Dynamic Meters, Aging Bars & Gantt Enhancements
**Goals:**
1. Wire `Meter` in Card 5 to dynamic calculation from `DemoState.claims_tab` and `DemoState.expenses` instead of hardcoded `0.57`.
2. Add hover height expansion (6px → 8px) and tooltip breakdown to `Meter`.
3. Add accessible `Role::ProgressIndicator` and ARIA numeric values.
4. Make `AgingBars` in Card 4 interactive (column hover highlight, tooltip, and click-to-filter top suppliers).
5. Add hover milestone tooltips on `Gantt` row segments in Card 1.

**Verification Steps:**
- Run `make check` → passes.
- Run `make run` → navigate to "Finance UI".
- In Card 5 (Expense Claims):
  1. Hover over the progress meter: height smoothly expands to 8px and displays tooltip with reimbursed vs submitted amounts.
  2. Toggle a claim checkbox or switch segment tab: meter fill recalculates and visibly updates.
- In Card 4 (Owed to Suppliers):
  1. Hover over aging columns ("1–30", "31–60"): column highlights with tooltip. Click a column to filter the supplier list below.
- In Card 1 (Purchase Orders):
  1. Hover over Gantt bar segments: milestone tooltip pops up detailing billing and delivery status.
