# Interactive Cadence Controls & Inputs Implementation Plan

**Scope:** `src/cadence/controls.rs`, `src/cadence/blocks.rs`, with state integration into `src/cadence/mod.rs` (`DemoState`) and consumption in `src/cadence/showcase.rs`.  
**Audits Incorporated:** Folded all open findings from `/tmp/mono-desk-ui-audit-a.md` (F2, F3, F4, F5, F7, F8, F9, F11, F13, F15) and `/tmp/mono-desk-ui-audit-b.md` (H1, M5, M6, L1, L7).  
**Status:** Read-only planning artifact for branch `fix/cadence-alignment`.

---

## 1. Executive Summary & Architecture

The Cadence UI clone (`src/cadence`) is currently a high-fidelity visual showcase with mostly inert controls. To transform it into a responsive, production-grade desktop experience, the control layer must implement:
1. **Unified State Architecture:** Distinguish between **transient element state** (focus handles, spring position, hover tracking, indicator bounds via `window.use_keyed_state`) and **domain application state** (active tabs, row selections, toggle values, panel visibility in `DemoState`).
2. **Keyboard Navigation & Accessibility (WAI-ARIA patterns):** Full keyboard operability via `tab_index(0)`, `track_focus`, arrow key cycling for segmented controls and tabs, Space/Enter activation, and standardized focus rings via `focus_ring_style(window, cx)`.
3. **Physical Micro-Transitions:** Spring-driven sliding indicators for tabs and segmented controls (`gpui_base::spring`), thumb travel physics for toggles, and hover/active ink transitions.
4. **Cohesive Feedback Loops:** Action buttons must dynamically compute counts from selected state, reflect disabled states when selection is empty, execute real state mutations, and fire user notifications (`window.push_notification`).

---

## 2. Component Specifications

### 2.1. Tabs (Underline Tabs)
- **Current Behavior (`src/cadence/controls.rs:35-130`):**
  - Static underline tab row (`Tabs::new`) rendering `v_flex` children inside `h_flex` with `flex_wrap` and `border_b_1`.
  - Underline is a static 2px bar (`controls.rs:119-125`) tinted only when active.
  - *Audit F3:* Underline sits outside the container's border box, creating a disjointed double line; wrapping rows lose baseline.
  - *Audit F11 & L1:* Stacking `gap_1` and `.mt_1()` creates an 8px label-to-indicator gap (mock is 2–4px); mono count center is misaligned with label baseline.
  - Zero hover feedback on inactive tabs; no keyboard focus or arrow navigation; instant snapping between tabs.
- **Target Interaction & UX:**
  - **Click:** Instant tab activation calling `on_pick(ix, window, cx)`.
  - **Hover:** Inactive tabs lighten label text to `cx.theme().foreground.alpha(0.85)` with a faint baseline hint.
  - **Keyboard Navigation:** W3C TabList pattern: Tab list receives focus (`tab_index(0)`), ArrowLeft / ArrowRight cycles active tab (with wraparound or clamp), Home / End jumps to start/end, Space / Enter selects.
  - **Focus Ring:** Accessible focus outline on the active tab header via `focus_ring_style(window, cx)` or subtle offset ring.
  - **Indicator Transition:**
    - Unwrapped single row: Smooth sliding accent bar (`gpui_base::spring` interpolating `left` and `width` derived from tab bounds stored via `window.use_keyed_state`), mirroring `vendor/gpui-component/src/tab/tab_bar.rs`.
    - Wrapped multi-line rows: Graceful fallback to per-tab baseline highlight (`mb(px(-1.))` overlapping baseline) to prevent cross-line spring distortion.
- **State Ownership:**
  - Active index: Owned by `DemoState::po_tab` (Card 1).
  - Transient focus & indicator bounds: Owned by `window.use_keyed_state` (`"{id}-focus"`, `"{id}-bounds"`).
- **GPUI APIs (Verified):**
  - `window.use_keyed_state` (persists `FocusHandle` and `RefCell<TabIndicatorBounds>`).
  - `div().track_focus(&focus_handle).tab_index(0)`.
  - `div().on_key_down(move |event, window, cx| ...)` (handling ArrowLeft, ArrowRight, Home, End).
  - `gpui_base::spring` (spring-driven `left` and `width`).
  - `div().on_prepaint(...)` (capturing rendered tab bounds).
- **Edge Cases:** Dynamic font-size scaling causing multi-line wrapping; rapid tab switching mid-spring (spring preserves velocity); tab row with 0 tabs.
- **Effort:** **M**
- **Dependencies:** None.

---

### 2.2. Segmented Control (Pill Segmented)
- **Current Behavior (`src/cadence/controls.rs:153-235`):**
  - Track has squarish corners `rounded(px(10.))` and active segment has `rounded(px(8.))` (`controls.rs:191, 209`).
  - *Audit F4:* Reference is a true pill capsule (`rounded_full()`).
  - *Audit F5 & M5:* Height is hardcoded to 30px (`h(px(30.))`), drifting against 36px filter chips and inputs in the same row.
  - Instant background snapping on click; no hover state on inactive segments; no keyboard navigation.
- **Target Interaction & UX:**
  - **Geometry Alignment:** Track and segment pills changed to `rounded_full()`. Track height standardized to 36px (`p(px(3.))` enclosing a 30px pill).
  - **Sliding Pill Transition:** Active segment rendered as an absolute floating pill (`bg(popover).shadow_xs().rounded_full()`) sliding behind labels via `gpui_base::spring` interpolating `left` and `width`.
  - **Hover:** Inactive segments display a soft background wash (`hover(|this| this.bg(cx.theme().background.alpha(0.5)))`).
  - **Keyboard Navigation:** Container tracks focus (`track_focus`); ArrowLeft / ArrowRight shifts active selection immediately; Space / Enter confirms. Focus ring hugs pill track.
- **State Ownership:**
  - Active segment: Owned by `DemoState::claims_tab` (Card 5).
  - Spring & Focus State: Owned by `window.use_keyed_state`.
- **GPUI APIs (Verified):**
  - `window.use_keyed_state`
  - `gpui_base::spring`
  - `div().track_focus(&focus_handle).tab_index(0)`
  - `StyledExt::focus_ring_style(window, cx)`
- **Edge Cases:** Single segment (no spring movement needed); rapid alternating clicks; dark mode contrast of active white pill (uses `cx.theme().popover` with subtle top highlight).
- **Effort:** **M**
- **Dependencies:** Standardizing height with `FilterChip`.

---

### 2.3. Checkbox (`CadenceCheckbox`)
- **Current Behavior (`src/cadence/blocks.rs:430-477`):**
  - 18px square box (`blocks.rs:457`).
  - *Audit H1:* Centering bug fixed in current tree, but box lacks hover feedback, keyboard focus, and Space-key toggle.
  - *UX Gap in `showcase.rs`:* In `doc_row` (`showcase.rs:529-556`) and PO table, only the tiny 18px box is clickable; clicking the adjacent text label does nothing.
- **Target Interaction & UX:**
  - **Click:** Toggles boolean via `on_click(&!checked, window, cx)`.
  - **Hover Feedback:**
    - Unchecked: `hover(|this| this.border_color(Accent::solid(cx)).bg(Accent::wash(cx)))`.
    - Checked: `hover(|this| this.bg(Accent::solid(cx).alpha(0.85)))`.
  - **Keyboard Navigation:** `tab_index(0)` with `track_focus(&focus_handle)`. Displays `focus_ring_style` on focus. Space key triggers toggle (`window.prevent_default()` to stop page scroll).
  - **Checkmark Transition:** Subtle pop-in / scale animation (`0.7 -> 1.0`) on the checkmark glyph when checked.
  - **Row / Label Hit Area:** Support `.label(...)` or integrate with parent row container so clicking either the label or the box toggles the checkbox.
- **State Ownership:**
  - `DemoState::po_rows` (`[bool; 3]`), `DemoState::doc_bills`, `DemoState::doc_credit_notes`, `DemoState::expenses` (`[bool; 3]`).
- **GPUI APIs (Verified):**
  - `window.use_keyed_state(self.id, cx, |_, cx| cx.focus_handle())`
  - `div().track_focus(&focus_handle).tab_index(0)`
  - `div().on_key_down(...)`
  - `StyledExt::focus_ring_style`
- **Edge Cases:** Table multi-selection (Shift+Click range selection in future); preventing Space key scroll.
- **Effort:** **S**
- **Dependencies:** None.

---

### 2.4. Toggle (`CadenceToggle`)
- **Current Behavior (`src/cadence/blocks.rs:479-534`):**
  - 40×22 pill switch with an 18px thumb (`blocks.rs:509-532`).
  - *Audit F9:* Off-state thumb uses `bg(cx.theme().popover)` (#212121) over `cx.theme().border` (#262626) in dark mode — virtually invisible (<4% lightness contrast).
  - Instant thumb jump (`justify_start` vs `justify_end`); no sliding physics.
  - No hover feedback, no keyboard focus, no Space/Enter support. Label "Overdue only" is inert.
- **Target Interaction & UX:**
  - **Contrast Fix (F9):** In dark mode, off-state thumb uses high-contrast ink: `if cx.theme().mode.is_dark() { rgb(0xffffff) } else { cx.theme().popover }` with `shadow_xs()`.
  - **Spring Travel Physics:** Thumb position animated via `gpui_base::spring` (`left: px(2.)` when off -> `left: px(20.)` when on; travel delta = 18px).
  - **Hover:** Track border/fill lightens slightly on hover.
  - **Keyboard Navigation:** `tab_index(0)`, `track_focus`, `focus_ring_style`. Space and Enter toggle state.
  - **Row Integration:** Clicking the "Overdue only" label row triggers toggle.
- **State Ownership:**
  - `DemoState::overdue_only`.
- **GPUI APIs (Verified):**
  - `gpui_base::spring((format!("{id}-thumb"), "x"), target_x, ...)`
  - `window.use_keyed_state`
  - `div().track_focus(&focus_handle).tab_index(0)`
- **Edge Cases:** Rapid double-clicking mid-travel reverses spring velocity naturally without jumping.
- **Effort:** **S**
- **Dependencies:** None.

---

### 2.5. Filter Chips (`FilterChip`)
- **Current Behavior (`src/cadence/blocks.rs:172-265`):**
  - Chip container (`h(px(34.))`, `blocks.rs:223`).
  - *Audit F15:* `self.id` is attached only to the tiny trailing × button; outer chip has no ID.
  - *Audit F7 & M6:* Value text is rendered in proportional font, breaking the "every numeral is monospace" rule.
  - *Audit M5:* Height is 34px, misaligned with adjacent 36px controls.
  - Chip body is non-interactive: caret dropdowns ("Supplier Any ▾", "Sep 2026 ▾") do nothing.
  - When × is clicked in Card 1, `date_filter` becomes `false`, permanently removing the chip with no way to restore it.
- **Target Interaction & UX:**
  - **Structural Fixes:** Outer chip receives `self.id`; height standardized to 36px; numerals formatted with `mono(self.text, cx)`.
  - **Two Distinct Click Zones:**
    1. *Body / Open Action:* When `caret` is true or `on_open` is passed, clicking the chip body triggers a dropdown menu / popover (or demo notification). Cursor is `cursor_pointer`; hover provides subtle tint `hover(|this| this.bg(cx.theme().accent))`.
    2. *Remove Action (×):* Separate button target (`p(px(4.))`, 14px icon). Clicking dismisses chip (`DemoState::date_filter = false`) with stop-propagation.
  - **Restore Affordance:** When `date_filter` is false, render an "Add date filter +" ghost chip in Card 1 so the user can re-open it.
  - **Keyboard Navigation:** Chip body is focusable (`tab_index(0)`); Enter/Space opens dropdown; Backspace/Delete removes chip.
- **State Ownership:**
  - `DemoState::date_filter: bool`.
  - New state fields: `DemoState::supplier_filter: Option<usize>`, `DemoState::claims_month: SharedString`.
- **GPUI APIs (Verified):**
  - `window.use_keyed_state`
  - `div().track_focus(&focus_handle).tab_index(0)`
  - `div().on_key_down(...)`
  - Event stop propagation on × click.
- **Edge Cases:** Removing chip while focused passes focus cleanly to next sibling; keyboard focus indicator shouldn't clip.
- **Effort:** **M**
- **Dependencies:** Popover / selection dialog or menu integration.

---

### 2.6. Buttons (`btn_primary`, `btn_secondary`, Actions)
- **Current Behavior (`src/cadence/blocks.rs:557-573`, `showcase.rs`):**
  - `btn_primary`: Custom button variant with `color(foreground)` and `foreground(popover)`.
  - *Audit F8:* Hover and active colors are set to `cx.theme().button_foreground` (#2e2e2e) — identical to base ink! Hover and pressed states are completely invisible.
  - *Dead Buttons in Showcase:* All major action buttons are inert with no `on_click`:
    - Card 1: `po-approve` ("Approve 2" hardcoded), `po-mark-billed`.
    - Card 3: `filter-reset`, `filter-show` ("Show 142 bills").
    - Card 5: `claim-pay-{ix}` ("Pay" buttons).
    - Card 7: `rebill-assign`, `rebill-markup`.
    - Card 8: `assign-confirm` ("Assign 3" hardcoded), `assign-cancel`.
- **Target Interaction & UX:**
  - **Visual Contrast Fix (F8):**
    - Light Mode: Hover lightens primary button (`foreground.alpha(0.85)` / #444444); active presses to `foreground.alpha(0.95)` / #1a1a1a.
    - Dark Mode: Hover slightly darkens; active deepens.
    - Secondary: Clear border and background highlight on hover (`border_color(foreground.alpha(0.4)).bg(accent)`).
  - **Dynamic Labels & Selection-Driven State:**
    - `po-approve`: Label dynamically displays `"Approve {n}"` where `n = state.po_rows.iter().filter(|&&x| x).count()`. When `n == 0`, button is `.disabled(true)`.
    - `assign-confirm`: Label dynamically displays `"Assign {n}"` where `n = state.expenses.iter().filter(|&&x| x).count()`. Disabled if `n == 0`.
    - `filter-show`: Dynamically updates bill count based on `overdue_only` and doc filters.
  - **Real State Mutations:**
    - `po-approve`: Clears selected POs, pushes success notification: `window.push_notification(Notification::success(format!("Approved {n} purchase orders")), cx)`.
    - `filter-reset`: Reverts `DemoState` filters to defaults.
    - `claim-pay-{ix}`: Marks claim as paid in state, pushes notification.
    - `assign-confirm`: Commits assignment to `state.assigned`, clears selected expenses, notifies user.
- **State Ownership:**
  - Mutations written via `DemoState::update(cx, ...)`.
- **GPUI APIs (Verified):**
  - `Button::new(id).on_click(...)`
  - `Button::disabled(bool)`
  - `window.push_notification(Notification::...)`
- **Edge Cases:** Rapid repeated clicks; 0 items selected triggering disabled styling and blocking clicks.
- **Effort:** **S**
- **Dependencies:** `DemoState` helper methods.

---

### 2.7. MonoPill (`MonoPill`)
- **Current Behavior (`src/cadence/blocks.rs:122-170`):**
  - Static pill badge with `mono` font text and `rounded_full`.
  - *Audit F13:* Slightly large vs reference (`text_sm` + `px_2` + `py(2)` -> 20–22px tall; reference is ~18px with `text_xs`).
  - *Audit F2:* Trailing slot in `CardHeader` is top-aligned, drifting against centered icon tiles.
  - Currently 100% static; cannot be clicked or focused.
- **Target Interaction & UX:**
  - **Geometry Sizing (F13):** Scaled down to `text_xs()`, padding `px_2().py(px(1.5))` for standard tags; `px_2p5().py(px(2.))` for solid "Today".
  - **CardHeader Alignment (F2):** Vertically center-aligned in `CardHeader`.
  - **Interactive Capability:** Support optional `.on_click(...)`. When present:
    - Sets `cursor_pointer()`, hover background tint, active pressed state.
    - Focusable with keyboard support (`tab_index(0)`).
  - **Dynamic State Wiring in Showcase:**
    - Card 1 "2 need you": Dynamically reflects pending count (`"{n} need you"`). Clicking it selects only the pending PO rows in `DemoState::po_rows`.
    - Card 8 "3 expenses selected": Dynamically reflects count from `state.expenses`.
- **State Ownership:**
  - Reads and optionally mutates `DemoState`.
- **GPUI APIs (Verified):**
  - `div().cursor_pointer().hover(...).on_click(...)`
  - `div().tab_index(0)`
- **Edge Cases:** 0 count state switches tone to `PillTone::Neutral` ("0 need you").
- **Effort:** **S**
- **Dependencies:** `CardHeader` centering fix (F2).

---

### 2.8. Close Buttons (`close_button`)
- **Current Behavior (`src/cadence/showcase.rs:559-582`):**
  - Hand-rolled private function `close_button` in `showcase.rs` rather than a modular component in `blocks.rs`.
  - Click handler pushes a mock notification `"Panel closed (demo)"`, but does not close or collapse anything.
  - No keyboard navigation; no focus ring.
- **Target Interaction & UX:**
  - **Promote to First-Class Component in `blocks.rs`:** `pub fn close_button(id: impl Into<ElementId>, on_close: impl Fn(&mut Window, &mut App) + 'static, cx: &App) -> Div`.
  - **Real Panel Dismissal:**
    - Card 3 (Filter) and Card 8 (Assign to project) can be closed.
    - Wire into `DemoState::show_filter: bool` and `DemoState::show_assign: bool`.
    - When clicked, collapses panel and renders a compact "Open Filter" / "Open Assign" affordance or status pill in the canvas header.
  - **Keyboard Navigation:** `tab_index(0)`, `track_focus`, `focus_ring_style`, Space/Enter/Escape triggers close.
- **State Ownership:**
  - `DemoState::show_filter`, `DemoState::show_assign`.
- **GPUI APIs (Verified):**
  - `window.use_keyed_state`
  - `div().track_focus(&focus_handle).tab_index(0)`
  - `StyledExt::focus_ring_style`
- **Edge Cases:** Both panels closed simultaneously; keyboard focus restoration after panel closes.
- **Effort:** **S**
- **Dependencies:** Adding visibility flags to `DemoState`.

---

## 3. DemoState Enhancements (`src/cadence/mod.rs`)

To support genuine reactivity across controls, expand `DemoState`:

```rust
pub struct DemoState {
    // Existing fields
    pub po_tab: usize,
    pub po_rows: [bool; 3],
    pub date_filter: bool,
    pub overdue_only: bool,
    pub doc_bills: bool,
    pub doc_credit_notes: bool,
    pub slider: (f32, f32),
    pub claims_tab: usize,
    pub assigned: Option<usize>,
    pub expenses: [bool; 3],

    // New reactive fields
    pub show_filter: bool,
    pub show_assign: bool,
    pub supplier_filter: Option<usize>,
    pub claims_month: SharedString,
    pub claims_paid: [bool; 3],
}
```

Add helper query methods:
- `pub fn po_selected_count(&self) -> usize`
- `pub fn expenses_selected_count(&self) -> usize`
- `pub fn doc_types_count(&self) -> usize`

---

## 4. Implementation Phases & Verification Gates

### Phase 1: Core Primitives & Visual Alignment
- **Goals:** Fold open audit findings:
  - Fix `CardHeader` optical centering (`blocks.rs:87`, Audit A F2).
  - Fix button hover/active colors in `btn_primary` (`blocks.rs:563-565`, Audit A F8).
  - Fix toggle thumb contrast in dark mode (`blocks.rs:526-530`, Audit A F9).
  - Standardize MonoPill font and padding (`blocks.rs:163-168`, Audit A F13).
  - Promote `close_button` from `showcase.rs` to `blocks.rs`.
- **Verification:**
  - `make check` passes.
  - `make run`: Open "Finance UI" settings page. Hover over black "Approve 2" and "Show 142 bills" buttons — confirm visible lightness change on hover and pressed states. Toggle app to Dark Mode — confirm toggle thumb is crisp white with clear contrast. Confirm CardHeader pill and icon tile share an optical center line.

### Phase 2: Binary & Toggle Inputs (`Checkbox`, `Toggle`, `CloseButton`)
- **Goals:**
  - Implement focus tracking (`track_focus`, `tab_index(0)`), keyboard toggle (Space/Enter), and hover styles in `CadenceCheckbox` and `CadenceToggle`.
  - Implement spring-driven sliding thumb animation on `CadenceToggle` via `gpui_base::spring`.
  - Wire row-click hit areas in `doc_row` and PO table.
  - Wire `close_button` to `DemoState::show_filter` and `DemoState::show_assign`.
- **Verification:**
  - `make check` passes.
  - `make run`:
    - Click "Overdue only" label text — confirm toggle smoothly glides right/left with spring physics instead of snapping.
    - Tab to a checkbox with the keyboard — confirm focus ring appears; press Space to toggle on/off.
    - Click the close button on Card 3 (Filter) — confirm the Filter card dismisses cleanly and an "Open Filter" chip appears.

### Phase 3: Selection Navigators (`Tabs`, `Segmented`)
- **Goals:**
  - Implement pill capsule geometry (`rounded_full`) and 36px height standard on `Segmented` (Audit A F4, M5).
  - Implement spring-animated sliding indicator for `Segmented` and unwrapped `Tabs`.
  - Implement per-tab baseline highlight fix for wrapped tabs (Audit A F3, F11, L1).
  - Add keyboard arrow navigation (ArrowLeft / ArrowRight / Home / End) and focus ring to both controls.
- **Verification:**
  - `make check` passes.
  - `make run`:
    - Click between "Your own", "To review", "To pay" — observe the white rounded pill slide smoothly under the text.
    - Tab to the Segmented control and press ArrowRight/ArrowLeft — confirm active segment cycles instantly.
    - Tab to Purchase Orders tabs ("All", "Draft", "Pending") — press ArrowRight to switch tabs; confirm active indicator slides and baseline hairline does not show double-line artifacts.

### Phase 4: Complex Controls (`FilterChip`, `MonoPill`)
- **Goals:**
  - Standardize `FilterChip` height to 36px and format numeral values with `mono()` (Audit A F7, M5, M6).
  - Split `FilterChip` hit areas into body (`on_open`) and trailing × (`on_remove`).
  - Add "Add date filter +" restore chip when `date_filter` is false.
  - Add interactive filter capability to `MonoPill` ("2 need you").
- **Verification:**
  - `make check` passes.
  - `make run`:
    - Click "Sep 1 – Sep 30" × button — chip dismisses without triggering chip body.
    - Click "+ Add date filter" — chip re-appears.
    - Click "2 need you" MonoPill in Card 1 header — observe unapproved PO rows toggle to selected.
    - Verify all numerals inside chips are rendered in monospace font.

### Phase 5: Action Controls & Full Showcase Wiring
- **Goals:**
  - Bind dynamic counts to `btn_primary` ("Approve {n}", "Assign {n}").
  - Add `.disabled(n == 0)` states with muted styling.
  - Wire all button `on_click` handlers to execute mutations on `DemoState` and push feedback notifications.
  - Wire "Pay" buttons on expense claim rows to mark items paid.
- **Verification:**
  - `make check` passes.
  - `make run`:
    - Uncheck all PO rows — confirm "Approve" button disables and reads "Approve 0".
    - Check 2 PO rows — confirm button updates to "Approve 2". Click it — observe confirmation notification and selection clearing.
    - Click "Pay" on an expense claim — confirm item updates to paid state and notification fires.
    - Click "Reset" in Filter card — confirm slider and toggles reset to default values.
