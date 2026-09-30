---
id: plan-zdesk-ui-port-20260928
intake_id: intake-20260928-zdesk-ui-port
status: active
lane: normal
created: 2026-09-28
updated: 2026-09-28
approach: planned
planning_status: planned
exact_next_action: "work phase `zdesk-correction` (wave: zc-1 ∥ zc-2 ∥ zc-3)"
---

# zdesk → mono-desk UI port (corrected)

Port the zdesk chrome onto mono-desk's gpui-kit stack. **Revised target
(owner visual review, 2026-09-28):** exactly two regions — ONE sidebar +
raised main card — not a multi-destination shell.

## Corrected target layout

- **Window chrome:** transparent titlebar, macOS traffic lights at
  (16,18), vendored `TitleBar` drag region. (Kept from the shell pass.)
- **Region 1 — sidebar (the Settings navigation):** `Settings`' internal
  sidebar restyled to zdesk values — `bg-background` (window base),
  icons on EVERY item (pages AND their group children), brand row
  (20px mark + Xanh Mono italic wordmark) at top, search input below it.
  Fixed 240px, no collapse.
- **Region 2 — main area:** raised card (`rounded-lg` 10px, 1px
  `border`, `mr`/`mb` 6px, `bg` = raised/group_box) hosting the Settings
  page content.
- **Removed:** icon rail, Search/Workspace destinations + EmptyState
  placeholders, ⌘B collapse + collapsed-rail tooltips, StatusBar.
- **Kept:** Phase-0 tokens/fonts (graphite palette, Inclusive Sans 13px,
  GSC 12px), ⌘Q/menu, Dark Mode switch (Settings-only theme control),
  inactive-item 70% opacity via `sidebar_style`.

## Outcome

`make run` shows: transparent titlebar → one 240px settings sidebar
(brand → search → icon'd item tree) on the window background → rounded
lighter card holding the active settings page. Success when the two-
region structure holds in both modes and the full gate passes.

## Requirements

1. One sidebar: `Settings` internal sidebar only; no outer nav column.
2. Sidebar order top→bottom: brand row, search input, item list.
3. Every sidebar item has an icon: pages (already) + group children
   (needs `SettingGroup` icon support — vendored patch).
4. Main area card: `rounded-lg` + border + raised bg + 6px right/bottom
   margins, content clipped inside (needs a `Settings` content-panel
   style hook — vendored patch).
5. No statusbar, no icon rail, no view-switching, no collapse.
6. Deleted bindings: ToggleNav/OpenSearch/OpenSettings; `⌘W → quit`
   default kept (close-window semantics, no tabs) unless owner objects.

## Non-goals

- Search/Workspace views, EmptyState, tabs, command palette, statusbar,
  resizable-panel UX changes (drag handle remains, recorded).
- Settings-internal item density / 10px-uppercase group labels
  (component-internal, still deviations).
- Any theme/token/font rework — Phase 0 stands.

## Approach and Risks

### Approach

`Settings` is `h_resizable`: `resizable_panel(sidebar)` +
`resizable_panel(content)` (vendored `settings.rs:386-397`); panels take
`refine_style`, but the builder exposes no content-panel slot — so two
small additive vendored deltas are needed. Brand row: the deleted
`sidebar-slots.patch` is restored (`git show HEAD:vendor/patches/
sidebar-slots.patch`) — it renders `sidebar_header` inside
`Sidebar::header` ABOVE the search input, exactly the target order.
Alternative considered and rejected: absolutely-positioned brand overlay
(fragile z-order/hit-test, rejected in the prior initiative too).

### Vendored deltas after correction

| Patch | Purpose | Status |
|---|---|---|
| `vendor/patches/sidebar-slots.patch` | `sidebar_header` slot (brand row). `sidebar_footer` arrives unused — flag to owner or leave dormant. | restore |
| `vendor/patches/settings-content-style.patch` (new) | `content_panel_style(StyleRefinement)` on `Settings`, refined onto the second `resizable_panel` — card styling hook | new |
| `vendor/patches/settings-group-icon.patch` (new) | `SettingGroup::icon()` → propagated to child `SidebarMenuItem` in `render_sidebar` | new |
| `vendor/patches/sidebar-cursor.patch` | pointer cursor on rail items | unchanged |

### Risks

- **Resizable seam:** `h_resizable` still renders a drag handle between
  sidebar and card; `size_range` may be narrowed (240..240) rather than
  removed — recorded deviation, not a defect.
- **Card clipping:** panel `refine_style` rounds the container; content
  may need `overflow_hidden` on the panel — verify visually.
- **Sidebar bg:** `Sidebar` uses `sidebar.background` (= base `#171717`/
  `#f7f7f7` after Phase-0 remap); ensure sidebar reads darker than the
  raised card per target.
- **Footer slot dormancy:** restored patch adds `sidebar_footer` unused;
  harmless but note in spec.

## Phases and Verification

Completed earlier: `zdesk-tokens` ✓, `zdesk-shell` ✓ (superseded in
part), `zdesk-interactions` ✓ (partially reverted by this correction),
`zdesk-polish` ✓ (docs to be updated). Remaining: `zdesk-correction`.
Gate after the phase: `make check` && `make lint` && `make fmt` &&
`cargo test`, plus owner `make run` visual.

### Phase `zdesk-correction` — story_id: story-zdesk-correction-20260928

- **Status:** `done` (superseded by `zdesk-layout-fix` — owner visual 2)
- **Goal:** two-region layout — `Settings` sidebar (brand + search +
  icon'd tree) beside a raised card; rail/statusbar/destinations gone.
- **Depends on:** `zdesk-tokens`, `zdesk-shell` (state to modify).
- **Allowed surfaces:** `src/shell.rs`, `src/main.rs`,
  `vendor/gpui-component/src/setting/settings.rs`, `vendor/patches/`,
  `Cargo.toml` comment, `docs/design/zdesk-port-spec.md`,
  `docs/design/mono-port-spec.md`.

**Wave 1** (three independent tasks, parallel — disjoint files; `zc-3`
codes against the vendored API `zc-1` restores, and both know it):

1. `zc-1` — vendor/ + `Cargo.toml` comment: restore
   `vendor/patches/sidebar-slots.patch` from git
   (`git show HEAD:vendor/patches/sidebar-slots.patch > vendor/patches/
   sidebar-slots.patch`; `cd vendor/gpui-component && git apply
   ../patches/sidebar-slots.patch` — vendor/ is committed so `git apply`
   works on tracked files); create `settings-content-style.patch`
   (additive: `content_panel_style: StyleRefinement` field +
   `.content_panel_style(&style)` builder + `.refine_style(&self
   .content_panel_style)` on the second `resizable_panel()` in `render`)
   and `settings-group-icon.patch` (additive: `icon: Option<Icon>` field
   + `SettingGroup::icon()` builder + `.when_some(group.icon, |i,icon|
   i.icon(icon))` on child `SidebarMenuItem`s in `render_sidebar`);
   update the `Cargo.toml` patch comment to list all three patches.
   - Output: vendored crate compiles with `sidebar_header`,
     `content_panel_style`, `SettingGroup::icon` available.
   - Check: `cargo check`.
2. `zc-2` — `src/shell.rs`: strip to `TitleBar` + body. DELETE:
   `ShellDestination`, `ShellState`, `nav_item`, `nav_column`,
   `empty_state`, `content_card` match arm, `StatusBar` usage +
   imports, collapse toggle button. KEEP/EXPORT: `brand_row` →
   `pub(crate) fn sidebar_brand() -> impl IntoElement` (20px mark +
   Xanh Mono italic wordmark, full-width row). `AppShell` becomes
   `TitleBar::new()` + `Settings` directly (no card wrapper — the card
   is applied via `content_panel_style` on the `Settings` element
   itself); `settings_view()` stays the entry point. If what's left is
   ~60 lines, folding `shell.rs` back into `main.rs` is acceptable —
   note it in the report.
   - Output: compiles against contract (`AppShell::new()
     .settings_view(Settings)`).
   - Check: `cargo check`.
3. `zc-3` — `src/main.rs`: on the `Settings` builder add
   `.sidebar_header(shell::sidebar_brand())` (restored slot),
   `.content_panel_style(&StyleRefinement::default().rounded_lg()
   .border_1().border_color(stroke).bg(raised).mr(px(6.)).mb(px(6.))
   .overflow_hidden())`, keep `.sidebar_style` 70%-opacity line and
   `sidebar_width(px(240.))` + narrow `sidebar_size_range(240..240)`;
   add `.icon(IconName::…)` to every `SettingGroup` (Appearance→e.g.
   Palette/SunMoon, Startup→Power, Delivery→Bell, Actions→Zap,
   Account→User, Links→Link, brand group→Info — pick existing
   `IconName` variants, grep the subset first); DELETE
   `ToggleNav`/`OpenSearch`/`OpenSettings` actions + bindings +
   handlers + `cx.set_global(ShellState::default())` + `use
   shell::ShellState`; keep `CloseWindow`(⌘W→quit, comment adjusted) and
   ⌘Q/menu.
   - Output: compiles once `zc-1` lands.
   - Check: `cargo check`.

**Wave 2** (sequential, single owner — spec docs + integration fix):
4. `zc-4` — update `docs/design/zdesk-port-spec.md` (replace shell/
   nav/statusbar sections with the two-region layout; record the two new
   patches + restored slots patch; correct "pending zp-2" verdicts) and
   the rail-chrome section of `docs/design/mono-port-spec.md` (footer
   toggle gone; brand via `sidebar_header` again). Fix any wave-1 seam
   mismatches.
   - Check: docs render; `make check` still green.

**Phase checks:** full gate + `make run` owner visual — two regions only,
sidebar darker than card, icons on all items, brand above search, card
rounded/lighter/margined.

**Stop:** a patch fails to apply/compile twice, or the content card
cannot be expressed via `content_panel_style` → halt, report.

### Phase `zdesk-layout-fix` — story_id: story-zdesk-layout-fix-20260928

- **Status:** `done`
- **Goal:** match `docs/design/shell-layout-target.md` exactly — card is
  the only bordered surface (all-4-side 6px inset), borderless titlebar,
  no resizable-handle line, 240⇄48 icon-collapse with titlebar toggle +
  ⌘B, raised-pill active items.
- **Depends on:** `zdesk-correction`.
- **Allowed surfaces:** `src/shell.rs`, `src/main.rs`,
  `vendor/gpui-component/src/setting/settings.rs`,
  `vendor/gpui-component/src/sidebar/menu.rs`, `vendor/patches/`,
  `Cargo.toml` comment, `docs/design/zdesk-port-spec.md`.
- **Avoided:** `assets/themes/`, `src/theme.rs` (token remap already
  encodes raised = `group_box`; re-map keys only if the pill bg needs a
  different token — prefer `sidebar.accent.background` = raised).

**Wave 1** (parallel — disjoint files; `zl-3` codes against `zl-1` API):

1. `zl-1` — vendor/: two new additive patches, same mechanics as before.
   a) `settings-collapse.patch` on `settings.rs`: `sidebar_collapsed:
   bool` field + `.sidebar_collapsed(bool)` builder; in `render_sidebar`
   use `.collapsible(SidebarCollapsible::Icon)` + `.collapsed(self
   .sidebar_collapsed)` (replacing `collapsible(false).collapsed(false)`)
   and gate the whole `.header(...)` block on `!self.sidebar_collapsed`
   (hides brand + search when collapsed); clicking a page already selects
   its first content (settings.rs on_click sets `page_ix` — verify
   first-group behavior; if it renders empty, also set `group_ix:
   Some(0)` when the page has groups).
   b) `settings-item-style.patch` on `settings.rs` + `sidebar/menu.rs`:
   `active_item_style: Option<StyleRefinement>` on `Settings` +
   `.active_item_style(&style)` builder; in `render_sidebar`, apply the
   refinement on each `SidebarMenuItem` **only when the item is active**
   (page and group-child sites) — `SidebarMenuItem` impls `Styled`, so
   `refine_style` or field assignment both viable; keep additive.
   Update `Cargo.toml` patch comment (5 patches total).
   - Check: `cargo check` on vendored crate; each patch `git apply -R
     --check` reversible.
2. `zl-2` — `src/shell.rs`: restore a minimal `ShellState` —
   `pub struct ShellState { pub collapsed: bool }` + `Default` +
   `Global`. `TitleBar::new().child(<ghost Button PanelLeft toggling
   collapsed + window.refresh()>).refine_style(&StyleRefinement::
   default().border_0())` (kill the titlebar bottom border — check the
   vendored render applies `border_b_1` before refine). `sidebar_brand`
   gains `collapsed: bool` param → collapsed = centered 20px mark only;
   expanded = zdesk metrics (row h-32, mark 20px, wordmark ~15px/600,
   gap ~8px, px-2 — mirror zdesk NavView values).
   - Check: `cargo check`.
3. `zl-3` — `src/main.rs`: `cx.set_global(ShellState::default())`; new
   `ToggleSidebar` action + `cmd-b` + `on_action` (flip + refresh);
   `Settings` builder: `.sidebar_header(shell::sidebar_brand(
   cx.global::<ShellState>().collapsed))`, `.sidebar_collapsed(
   collapsed)`, `.sidebar_width(px(if collapsed { 48. } else { 240. }))`
   (drop the 240..240 pin; verify the panel honors the dynamic size);
   `content_panel_style` gains `ml(px(6.)).mt(px(6.))` (4-side inset);
   `sidebar_style` adds `.border_0()` (kill sidebar right border); keep
   70%-opacity refinement; `.active_item_style(&StyleRefinement::
   default().bg(cx.theme().group_box).border_1().border_color(
   cx.theme().border))` (white raised pill).
   - Check: `cargo check`.

**Wave 2** (sequential): `zl-4` — update
`docs/design/zdesk-port-spec.md` (two new patches, collapse mechanics,
pill styling, border removals, brand-row metrics); fix wave-1 seams.
- Check: `make check` green; doc claims match code.

**Phase checks:** full gate + `make run` owner visual against
`shell-layout-target.md` in both widths and modes.
**Stop:** collapse or handle-hiding can't be expressed cleanly after two
attempts → halt, report.

## Progress

- 2026-09-28 — plan authored from read-only zdesk study (scout inventory)
  + mono-desk source; four owner decisions locked; `to-plan` normalized
  the file into the repo plan format with story IDs and waves.
- 2026-09-28 — phase `zdesk-tokens` **done**. `zt-1` rewrote
  `mono.json` to zdesk-resolved hex (graphite ink ladder, teal accent,
  per-mode status + tints; `font.size` 13, `mono_font.size` 12,
  `font.family` "Inclusive Sans"); `zt-2` vendored Inclusive Sans
  400/500/600/700 from `~/Library/Fonts` statics into `assets/fonts/`
  and registered them in `theme.rs`. Gate: `make check` ✓, `cargo test`
  2/2 ✓, `make lint` ✓, `make fmt` clean. Issues: none.
- 2026-09-28 — phase `zdesk-shell` **done**. `sh-1` wrote `src/shell.rs`
  (~340 lines): `AppShell` stateless element = `TitleBar` (PanelLeft
  ghost-button collapse toggle) + body (240px⇄48px nav column, brand row,
  Search/Workspace/Settings items with selected=raised+stroke and accent
  pill on collapsed rail, `empty_state` placeholders) + `StatusBar`
  (left=view, right=mode+version). `sh-2` rewired `main.rs`: zdesk
  traffic lights (16,18) over `TitleBar::window_options()`, `Settings`
  mounted via `settings_view`, `rail_brand`/`rail_footer` deleted.
  Contract amendment: `impl Default for ShellState` added (contract
  omitted it); `IconButton` does not exist — `Button.ghost().icon()` used.
  Icons resolved from component `IconName`: PanelLeft/Search/Folder/
  Settings all present. Gate: `make check` ✓, `cargo test` 2/2 ✓,
  `make lint` ✓, `make fmt` clean. Visual (`make run`): pending owner
  check — traffic lights, drag, collapse, card margins.
- 2026-09-28 — phase `zdesk-interactions` **done**. `zi-1` shell.rs:
  tooltips on collapsed rail items via core `StatefulInteractiveElement::
  tooltip` + `Tooltip::new(label).build` (managed `Placement::Right` path
  is `pub(crate)` — deviation recorded); nav label → `text_size(px(13.))`.
  `zi-2` main.rs: actions ToggleNav/OpenSearch/OpenSettings/CloseWindow;
  `cmd-b`, `cmd-k`, `cmd-,`, `cmd-w` bound; handlers mutate `ShellState`
  + `cx.refresh_windows()`; CloseWindow → `cx.quit()` (no tabs —
  semantics recorded). `zi-3`: `sidebar-slots.patch` reverse-applied via
  `git apply -R`, patch file deleted, `settings.rs` back to upstream,
  `Cargo.toml` comment updated; `sidebar-cursor.patch` kept. Gate:
  `make check` ✓, `cargo test` 2/2 ✓, `make lint` ✓, `make fmt` clean.
  Phase `zdesk-polish` wave 1 (`zp-1` ∥ `zp-2` ∥ `zp-3`) is next.
- 2026-09-28 — phase `zdesk-polish` **done**. `zp-1` aligned
  `empty_state()` to zdesk (13px medium title / muted desc / max-w-256 /
  p-8) and nav body padding (expanded `p-1`, collapsed `py-1`). `zp-2`
  verdict: partially expressible — inactive-item 70% opacity **closed**
  via `Settings::sidebar_style` (main.rs); 10px-uppercase labels and
  per-item density remain component-internal deviations (recorded in
  `mono-port-spec.md`, updated). `zp-3` wrote `docs/design/
  zdesk-port-spec.md` (token derivations, shell metrics, deviations,
  patch state). Gate: `make check` ✓, `cargo test` 2/2 ✓, `make lint` ✓,
  `make fmt` clean. All phases done; implementation complete.
- 2026-09-28 — **owner visual review: FAILED.** Shell showed two
  sidebars (48px icon rail + `Settings` sidebar) plus a statusbar —
  diverges from target. Owner corrected the target layout: exactly two
  regions (one settings sidebar + raised main card), icons on every
  sidebar item, brand above search, theme toggle stays Settings-only.
  Re-planned as phase `zdesk-correction`: deletes nav column/StatusBar/
  EmptyState/⌘B+⌘K+⌘, machinery, restores `sidebar-slots.patch`
  (brand via `sidebar_header`), adds two new vendored patches
  (`content_panel_style`, `SettingGroup::icon`). Tokens, fonts,
  titlebar, ⌘Q, ⌘W retained.
- 2026-09-28 — **owner visual review 2: FAILED.** The correction rendered
  the sidebar right border, a visible resizable-handle line, a titlebar
  bottom border, and an inset on only 2 card sides; no collapse control;
  active item lacked the raised pill. Owner supplied the approved target:
  `docs/design/shell-layout-target.md`. Re-planned as phase
  `zdesk-layout-fix` — card gets equal 6px inset on all four sides; only
  the card keeps a border; sidebar collapse (titlebar PanelLeft + ⌘B)
  restored at 240⇄48 via `SidebarCollapsible::Icon` (group icons only,
  search hidden, click page = first child); active item = raised pill.
- 2026-09-28 — phase `zdesk-correction` **done**. `zc-1` restored
  `sidebar-slots.patch` from HEAD + authored `settings-content-style.patch`
  (`content_panel_style` → `resizable_panel().refine_style` via StyledExt)
  and `settings-group-icon.patch` (`SettingGroup::icon` → child items);
  `Cargo.toml` comment lists all four patches; all independently
  reversible. `zc-2` stripped `src/shell.rs` to 60 lines (empty `TitleBar`
  + `Settings`; exported `sidebar_brand()`); deleted ShellState/
  ShellDestination/nav column/StatusBar/empty_state/tooltips. `zc-3`
  wired `main.rs`: `sidebar_header`, `content_panel_style` card
  (rounded-lg/border/raised/6px margins/clip), `sidebar_size_range
  240..240`, icons on all six titled groups, removed three nav actions +
  ⌘B/⌘K/⌘,; kept ⌘W→quit + ⌘Q. `zc-4` synced both spec docs. Gate (post
  wave 1 AND post wave 2): `make check` ✓, `cargo test` 2/2 ✓,
  `make lint` ✓, `make fmt` clean. Visual: pending owner `make run`.
- 2026-09-28 — phase `zdesk-layout-fix` **done**. `zl-1` vendored
  `settings-collapse.patch` (`sidebar_collapsed` + `SidebarCollapsible::
  Icon` + header gated on `!collapsed`) and `settings-item-style.patch`
  (`active_item_style` → `SidebarMenuItem::active_style` applied after
  built-in active styling so it wins) + a `show_handle(bool)` hook
  (no-op `ResizeHandleRenderer` — 1px line suppressed, hit area kept,
  requested mid-flight by zl-3); caught+repaired a dropped
  `default_open` regression. `zl-2` shell.rs: `ShellState{collapsed}`
  (derived Default after clippy), `TitleBar` + ghost PanelLeft button
  (right of traffic lights) + `border_0` refinement, `sidebar_brand
  (collapsed)` tuned to zdesk NavView metrics. `zl-3` main.rs: ⌘B +
  toggle handler, dynamic `sidebar_width`/`size_range` pin 48⇄240,
  `.border_0` sidebar, card `m(6)` 4-side inset, `active_item_style`
  raised pill. `zl-4` synced `zdesk-port-spec.md` (6 patches, reversal
  order). Clippy fail once (`derivable_impls` on ShellState) — fixed
  first retry. Gate: `make check` ✓, `cargo test` 2/2 ✓, `make lint` ✓,
  `make fmt` clean. Visual: pending owner `make run` against
  `docs/design/shell-layout-target.md`.

## Decisions

- 2026-09-28 — owner locks: shell+Settings-only views; keep `mono-*`
  theme names; keep Xanh Mono wordmark; theme toggle Settings-only
  (rail-footer toggle removed in `zdesk-shell`/`zdesk-interactions`).
- 2026-09-28 — owner visual review overrides the multi-view shell:
  single-sidebar layout; icon rail, destinations, EmptyState, collapse,
  tooltips, StatusBar all out of scope; group-child icons and raised
  content card require two new additive vendor patches plus restoring
  `sidebar-slots.patch` (header slot only; footer slot dormant).

## Validation

Per phase gate: `make check` && `make lint` && `make fmt` && `cargo
test`. GUI-verifiable items recorded as `make run` owner checks in
`## Progress` — for the correction: two-region layout, traffic lights,
window drag, sidebar darker than card, icons on all items, brand above
search, card radius/margins, both modes.

## Current State

- branch: `main` (075992d); PRs #3–#9 merged, PR #1 closed as superseded
- lifecycle_status: in-progress (no phase closed via `check` yet; no `full` independent review recorded in `## Validation`)
- completed: tokens/fonts, shell (one borderless sidebar, raised card, active pill), main merge (i18n.rs, persist.rs, font 8-32); titlebar fix #4 (`TITLEBAR_H=36`, light y = 36/2 − 14/2; GPUI y = button top edge); spring sidebar collapse #5; InputField + token retune #6 (font 14, radius 8/16); Finance UI `src/cadence` #9 with input reset revalidation + wrapping PO tabs; gates green, 42 tests
- review: codex reviewer on #6–#9 combined → SHIP-WITH-FIXES; the two priority fixes landed (01f12a9)
- blockers: none
- open_items: owner `make run` visual pass on main (both modes, collapsed + expanded, font 8–32, Finance page); follow-ups: cadence checkbox/toggle/tabs lack keyboard focus (`src/cadence/blocks.rs:454`), Reset filter button has no handler (`showcase.rs:517`), bar chart col 8 overflows plot (`charts.rs:209`), Segmented pills overflow at large fonts, `docs/design/zdesk-port-spec.md` stale rows (`rounded_lg` 16px vs real 7px; titlebar 34px/(16,18)); `full` independent check before completing this plan
- next action: owner runs `make run` on main; then `check full` on main for this plan
