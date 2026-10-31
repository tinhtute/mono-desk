# UI audit — `src/cadence` (finance-UI clone)

**Scope:** `src/cadence/charts.rs`, `src/cadence/showcase.rs`, `src/cadence/mod.rs` (read-only; nothing edited).
Out-of-scope files (`blocks.rs`, `controls.rs`) are cited only where they are the root cause of a defect rendered by the scoped files.

**Reference caveat:** `mod.rs:3` says the tweet's four renders live under `~/quaqua/runs/ref-cadence/` — that directory does not exist on this machine, so fidelity claims are judged against the style notes in `mod.rs:5–34` plus layout mechanics, not pixel-diff.

**Gate:** `make check` → `cargo check` **passes** (`Finished dev profile in 0.60s`; only a pre-existing future-incompat note for dep `block v0.1.6`).

**Key layout fact:** `gpui`'s `Style::default()` is `display: block` (`gpui-pre-0.3.6/src/style.rs:774`). `items_center()`/`justify_center()` on a plain `div()` are no-ops unless `.flex()` (or `h_flex`/`v_flex`) is set; children stack at the top-left. This is the root cause of most of the visible icon misalignment.

---

## High severity

### H1 — Icons pinned top-left in every "centered" tile/box: `items_center().justify_center()` without `.flex()`

`Icon` renders a plain `svg()` with a fixed size and no self-centering (`vendor/gpui-component/src/icon.rs:169–195`). Inside a `Display::Block` div the flex-alignment calls do nothing, so the icon hugs the tile's top-left corner with dead space to the right/below. Sites:

| Site | Container | Icon | Visual result |
|---|---|---|---|
| `showcase.rs:909–917` | 32px claim-row icon tile | 16px | icon at top-left corner of every claim row (Card 5) |
| `showcase.rs:564–565, 571` | 28px `close_button` | 14px | × off-center top-left (Cards 2 & 8 headers) |
| `blocks.rs:39–42` *(root cause)* | 36px `icon_tile` | 18px | title icon off-center in **all 8 card headers** |
| `blocks.rs:463–471` *(root cause)* | 18px `CadenceCheckbox` | 12px | checkmark in the corner, clipped-feeling, on every checked box |

Fix: insert `.flex()` before `.items_center().justify_center()` in each container (e.g. `div().size(px(32.)).flex().items_center().justify_center()`). Same pattern the correct `CadenceToggle` already uses at `blocks.rs:513`.

### H2 — Gantt header children in wrong order: "Today" pill lands on the right edge, "Oct 1" sits on the Today line
`showcase.rs:315–325`. `h_flex().justify_between()` gets children `[Sep 1, Oct 1, Today]` — so "Sep 1" is flush-left, **"Oct 1" is centered** (directly over the `left(relative(0.5))` marker at `:307–313`), and the "Today" pill is flush-right. The comment says "week ticks either side of the Today pill". Fix: reorder so the pill is the middle child — emit `"Sep 1"`, then `MonoPill::new("Today").solid()`, then `"Oct 1"`.

### H3 — Stacked bars overflow the plot and paint over the header area
`charts.rs:209–228` draws each column as `faint` + `solid` stacked in a `v_flex().justify_end()` inside a fixed `h(height)` row (`items_end`). The data at `showcase.rs:385–411` has columns whose sum exceeds 1.0 — worst: `BILL_BARS[8] = (0.86, 0.62) = 1.48` → 222px of bar in a 150px plot. Because the column is bottom-aligned, the ~72px excess overflows **upward** — over the tooltip slot (`charts.rs:153–169`, `.top(px(4.))`) and into the stat strip above. Fix: clamp the stack — e.g. `let faint_h = height * faint.clamp(0., (1. - solid).max(0.))` — or normalize `BILL_BARS` so `solid + faint ≤ 1`; also consider `.overflow_hidden()` on the plot (`charts.rs:188–191`) as a guard.

### H4 — Range-slider thumbs anchored by left edge, not center
`charts.rs:398–408`. Thumb `left(relative(at))` + `size(px(16.))` puts the thumb's left edge at the position, so both thumbs sit 8px right of where the value reads; at `lo=0`/`hi=1` they overhang the track by 8px instead of straddling the end. Vertically it's centered (`top(2)` on a 20px box). Fix: `.left(relative(at)).ml(px(-8.))` on the thumb (the tooltip at `charts.rs:159` already uses this negative-margin trick).

### H5 — "Today" marker line is broken at every row boundary
`showcase.rs:288–295` (cell) and `:307–313` (head). The marker bleeds only `.top(px(-4.)).bottom(px(-4.))` past a content-height cell, while rows have `.py_2()` (8px) at `:134` and the head has `py_2` + header `pb_2`. Result: ~4px gaps in what the comment describes as "one continuous line down the table" — the reference's signature dashed vertical. Fix: `.self_stretch()` the `w(px(200.))` cell so the marker spans the full row height, or draw one continuous line over the whole table column.

### H6 — PO table header doesn't share the rows' insets
`showcase.rs:106–124` (header row, no `px_2`, no checkbox column) vs `:131–136` (data rows, `px_2` + 18px checkbox + 12px gap). Consequences: the "Amount" header's right edge sits ~8px right of the row amounts (`:162–168`), and "Supplier" starts ~38px left of the supplier names. Fix: add `.px_2()` to the header row and `.pl(px(30.))` (or a 18px + gap_3 spacer) to the Supplier cell.

### H7 — BarChartMini x-labels misaligned with bars
`charts.rs:233–241`. Two stacked errors: (a) `.pl(px(36.))` is a hardcoded guess at the y-label gutter — actual gutter is y-column width + `gap_2` (~42px at default font, drifts further with the Font Size setting); (b) `justify_between` pins the first/last labels to the padded row's edges instead of centering each label on its bar slot. Net: "Sep 21" sits ~25px left of its bar, "Oct 19" hangs off the last bar. Fix: give the label row the same slot geometry as the bars — e.g. one `flex_1` + `text_center` label per bar group (the labels here correspond to every 5th bar), or compute the gutter from a measured/fixed y-label width.

## Medium severity

### M1 — Icon substitutions forced by the ~118-icon default bundle
`gpui_kit::component::IconName` is generated from `default-icons.txt` only (~118 of 1830 Lucide SVGs; `gpui-kit-assets-0.6.1/build.rs:54–110`), and `gpui_kit::assets::Assets` embeds only those paths — so e.g. `plane.svg`, `car.svg`, `utensils.svg`, `receipt.svg`, `wallet.svg`, `hand-coins.svg`, `sliders-horizontal.svg`, `list-filter.svg` all **exist** in the crate but are neither selectable via `component::IconName` nor renderable via `Assets`. Visible substitutions in `showcase.rs`:
- `:774, :786` — `IconName::Map` for "Flights LIS → BER" and "Mileage · 214 km" (should be `Plane` / `Car`).
- `:783` — `IconName::FileText` for "Client dinner · Lisbon" (should be `Utensils`).
- `:973` — `IconName::LayoutDashboard` for "By category" — `ChartPie` **is** in the default set; use it today.
- `:670` — `IconName::LayoutDashboard` for "Owed to suppliers" (`Wallet`/`HandCoins` need the full catalog).
- `:797` — `IconName::FileText` for the "Expense claims" header (`Receipt` reads better).
Fix: register `gpui_kit::assets::AllAssets` (or compose `icon_assets!` with `Assets` in `main.rs`) and take claim-row/header icons from `gpui_kit::assets::IconName`; swap `ChartPie` in immediately.

### M2 — Y-axis labels drift inward from their gridlines
`charts.rs:179–185` vs `:195–198`. `v_flex().justify_between()` over `h(height)` puts the first label's *top* on the top gridline and the last label's *bottom* on the baseline, so every label's center sits ~half a line-height (~8px) inward of its line. Fix: wrap each label in a 1px-high centered container (e.g. `div().h(px(1.)).flex().items_center().overflow_visible().child(mono…)`), or apply a negative half-line-height top margin on the first label / bottom on the last.

### M3 — Empty-state illustration is double-inset
`showcase.rs:1070–1072` sets `.mx_4()` on the well, but `EmptyState` already applies `.px_4()` (`blocks.rs:615`). The illustration well lands at 32px while title/body sit at 16px — left edge visibly off-grid from the heading. Fix: drop `.mx_4()`.

### M4 — Waffle grid doesn't fill its card and can't be sized
`charts.rs:271` hardcodes `cell: px(14.)` with no public builder (only `.cols()`), so `showcase.rs:994`'s 10-col waffle renders ~176px wide inside a 298px content box (`card .w(px(330.))` at `:971` minus `px_4`) — left-aligned dead space, unlike the reference's full-width waffle. Fix: expose `.cell(px)` / `.cell_size()` (field already exists) or make cells `flex_1`, and call it from `showcase.rs:994` to fill the row.

### M5 — Control heights drift inside shared rows
`FilterChip` is `h(px(34.))` (`blocks.rs:223`) but sits beside a 36px-high `Segmented` container (`controls.rs:190–208`, `p(3)`+`h(30)`) in `showcase.rs:804–821`, and beside nothing matching the 36px date inputs (`showcase.rs:389`) or search field (`:1230`) it shares visual language with. 2px height mismatches in a single row are noticeable in this design. Fix: standardize chips/segments on `h(px(36.))`.

### M6 — Numerals rendered in the UI font instead of mono
`mod.rs:19–23` makes "monospace for every numeral" the signature trait, but several numerals bypass `blocks::mono`: CardHeader subtitles are plain divs (`blocks.rs:103–108` — "September · 24 orders", "Q3 · 50 claims", "Aging · 214 open bills", "3 expenses selected"), `FilterChip` text is a plain div (`blocks.rs:242` — "Sep 1 – Sep 30"), and the claim sub-line "Mara K. · Sep 18 · 6230 Travel" is sans (`showcase.rs:931–935`). Fix: route those strings through `mono()` (CardHeader/FilterChip would need to take mono children or a flag).

## Low severity

- **L1 `controls.rs:92 + :123`** *(adjacent)* — `gap_1` on the tab `v_flex` **and** `mt_1` on the indicator stack to an 8px label↔underline gap; the reference sits tighter (~2–4px). Drop one.
- **L2 `charts.rs:61–97`** — `MiniBars` band math assumes bars centered in `1/n` slots (`slot(range.0)` at `:71`), but `justify_between` edge-aligns the first/last bars, shifting the band up to ~`bar_w/2` off its highlight at both ends. Wrap each bar in a `flex_1,items_center` slot to match the band geometry.
- **L3 `charts.rs:153–154`** — tooltip width `text.len()*7.5` counts **bytes**: '·' is 2 bytes, so the estimate is ~7.5px too wide and the pill sits slightly left of bar center; it also ignores the font-size setting. Prefer measuring or sizing the pill to content (`w` auto + centered via a wrapper with `left(slot) + ml(-50%)`-style anchor… concretely: put the pill in a 0-width flex container at `left(relative(slot_w))` with `justify_center`).
- **L4 `showcase.rs:1058–1062`** — "0 to invoice" renders as bare muted text; `blocks.rs:122–123` documents it as one of the reference's mono pills. Use `MonoPill::new("0 to invoice").tone(PillTone::Neutral)`.
- **L5 `showcase.rs:280 vs :285`** — stale comment says the gantt track is 220px; code is `w(px(200.))`. Either fix the comment or confirm 200px matches the reference.
- **L6 `blocks.rs:339`** *(adjacent)* — `stat_row` dividers are `h(px(36.))` `self_center`ed against ~72px stat columns; the reference's hairline dividers run the strip's height. Use `self_stretch` + `py` inset.
- **L7 `blocks.rs:87`** *(adjacent)* — `CardHeader` is `items_start`, so the trailing pill/legend/close button aligns with the tile's top edge rather than the title's optical center; tolerable but contributes to the "header row looks off" feel when combined with H1.
- **L8 `showcase.rs:524–551`** — `doc_row` wraps a single child in `h_flex().justify_between()` (dead layout, no defect — just noise).
- **L9 `showcase.rs:1075–1077`** — the rebill well calls `.items_center().justify_center()` **and** `.flex()` (correct order), unlike the H1 sites — listed only to show the intended pattern exists.

## Files with no findings
- `src/cadence/mod.rs` — accent ramp, `DemoState`, and state plumbing are clean; `DemoState::demo()` is correctly the seed (`main.rs:174`).
