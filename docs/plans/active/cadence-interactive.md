# Cadence interactive components

Goal: turn `src/cadence` (finance UI cloned from x.com/yurygok/status/2105191658337841531) from a static showcase into operable UI: real state in `DemoState`, click/hover/drag/keyboard, focus rings, cheap transitions.

Detail specs (per component, file:line, APIs, edge cases): `cadence-interactive/{controls,charts,showcase}.md`. Source audits: `cadence-interactive/audit-{a,b}.md`.

Already shipped in PR #20 (`fix/cadence-alignment`), skip in every phase: `.flex()` centering (icon_tile, checkbox, close_button, claim tile), gantt header order, bar stack clamp, slider thumb `ml(-8)`, CardHeader `items_start`, `ChartPie`, EmptyState `mx_4`.

APIs verified to exist: `window.use_keyed_state`, gpui `spring`, `on_drag_move`, `with_animation` (gpui-pre 0.3.6); `focus_ring_style`, `push_notification`, `open_sheet_at` (vendor/gpui-component); `gpui_kit::assets::AllAssets` (gpui-kit-assets 0.6.1). Note: sheet API is `open_sheet_at`, not `open_sheet` as showcase.md says.

## Phases

Each phase: `make check` green + the `make run` click-through listed in its detail spec. One phase per PR.

| # | Phase | Scope | Source |
|---|---|---|---|
| 1 | Foundation | `AllAssets` + `impl IconNamed` props, real glyphs (Plane/Utensils/CarFront/Receipt/HandCoins); `btn_primary` hover/active (F8); dark toggle thumb (F9); MonoPill size (F13); `close_button` → blocks.rs; `icon_tile_sized` (F14) | controls P1, showcase P1 |
| 2 | DemoState schema | Add all new fields once (PO items + selection + sort, selected_bill_bar, selected_waffle_cat, selected_aging_bucket, rebill_markup_pct, card visibility) so later phases don't fight over `mod.rs` | charts §4, showcase P2, controls §3 |
| 3 | Binary inputs | Checkbox/Toggle focus + Space/Enter + spring thumb; row hit areas; close buttons dismiss + restore chips | controls P2, showcase P6.1 |
| 4 | Navigators | Segmented pill + sliding indicator; Tabs per-tab baseline (F3/F11); arrow/Home/End keys; 36px control height (M5) | controls P3 |
| 5 | Filter card | RangeSlider drag/seek/keys wired to `slider`; MiniBars slot geometry (L2); FilterChip mono values + split hit areas + restore chip; Reset | charts P1, controls P4, showcase P6.3 |
| 6 | PO table | Header insets (H6), tri-state select-all, sort, tab filtering, bulk Approve {n} + toast | showcase P2, controls P5 |
| 7 | Charts | BarChartMini y/x label alignment (M2/H7), tooltip (L3), hover/select/keys; Waffle cell size (M4) + legend sync; Meter dynamic; AgingBars hover/click | charts P2–P4 |
| 8 | Gantt + claims | Continuous Today line (H5), segment tooltips; claims Pay flow updates stats/meter/pill; segmented filters claims; mono subline (M6) | showcase P3–P4 |
| 9 | Cross-card flows | Rebill empty→populated via Card 8 assign, "0 to invoice" pill (L4), Card 8 search + Assign {n}, Card 4 → Card 1 drill-down | showcase P5–P6 |

Parallelism: 1 and 2 first (shared files). After that 3/4, 5, 6, 7, 8 touch mostly disjoint areas but all write `showcase.rs`; run them sequentially or in worktrees with rebase. 9 last.

## Open decisions
- D1 Scope cut: phases 1–6 deliver "operable controls"; 7–9 are polish/demo flows. Ship all, or stop after 6?
- D2 Spring/animations: include (controls P2–P3) or snap-only for v1?
- D3 Binary size: `AllAssets` adds ~1.2 MB of SVG; OK, or register only the needed icons?
