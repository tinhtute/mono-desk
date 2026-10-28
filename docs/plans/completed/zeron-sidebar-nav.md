---
status: superseded
lane: normal
---
# Zeron sidebar structure: custom nav, sections, footer identity

> **Superseded 2026-10-28 (owner decision).** The gpui-kit/vendored-Settings
> approach this plan builds on was dropped: the owner directed a port to
> zeron's own UI code and GPUI stack. This file records the work and its
> evidence; the successor initiative is
> `docs/plans/active/zeron-stack-port.md`. The implementation is preserved on
> branch `wip/gpui-kit-shell` (commit 50823e3).

## Goal
- outcome: The settings sidebar matches zeron's structure instead of the flat
  vendored menu: a header filter row ("All settings", labelled like zeron's
  spaces dropdown), two collapsible sections (Preferences: General,
  Notifications, Profile; Workspace: Finance UI, Reset, About) with disclosure
  chevrons, and a footer identity row (avatar initial + display name + theme
  toggle icon). The window titlebar gains a centered title showing the active
  page (zeron's centered session title). The brand wordmark leaves the rail.
  Navigation selection is app-controlled so the custom rows and the rendered
  page cannot drift.
- success_signal: `cargo test && make check && make lint && make fmt-check`
  green; offscreen light/dark renders show the filter header, the two
  sections (collapsed/expanded), rows with an active chip, footer identity and
  the centered titlebar title; view tests prove section toggling, page
  selection through the custom nav, footer name, and that the vendored
  default menu is not rendered when the custom nav is supplied.
- actors: the owner (lab demo) and developers reading this repo for GPUI
  settings-shell patterns.
- authority: owner decisions in this session (2026-10-28): the zeron
  reference screenshot (light theme: "All projects" header, Sessions /
  Archived collapsible sections, footer identity, centered titlebar title)
  and the two explicit choices "Vendor patch thêm sidebar_nav + custom nav"
  and "Preferences / Workspace + footer identity". Reference sources:
  https://github.com/zeronsh/zeron (cloned at /tmp/zeron-src) —
  `crates/ui/src/shell.rs:8520` render_sidebar (transparent column, full
  height), `crates/ui/src/shell_spaces.rs` SIDEBAR_DISCLOSURE_HEADER_HEIGHT
  (28) / BODY_INSET (4), `crates/ui/src/shell.rs:8655` settings nav column
  with `pt(TITLEBAR_HEIGHT)`. Repo conventions: AGENTS.md, prior plans
  `docs/plans/completed/zeron-shell-layout.md` and `glass-background.md`.
- requirements:
  - R1: A new vendor patch `sidebar-nav-slot.patch` adds two `Settings`
    builder surfaces: `sidebar_nav(impl IntoElement)` (renders a custom nav
    tree instead of the default `SidebarMenu`, and hides the built-in search
    while active) and `selected_page(usize)` (controlled page selection: the
    render reconciles the internal index to it, so the app owns the selection
    state). The default path is unchanged when neither is set. | acceptance:
    vendored diff matches the patch file byte-for-byte; the existing view
    tests that do not use the new API stay green; `make check` green. |
    source: owner decision (vendor patch) + vendored `settings.rs` render
  - R2: The app sidebar renders the zeron header filter row ("All settings"
    plus chevron, in the `sidebar_header` slot), two collapsible sections in
    the custom nav (28px header rows with a chevron, 4px body inset, section
    titles "Preferences"/"Workspace"), and page rows (icon + label, active
    row chip styled like the current active item). Section open/closed state
    lives on the view; the filter row cycles All / Preferences / Workspace
    and hides rows outside the filter. | acceptance: view test toggles each
    section and asserts the row is gone/reappears; view test cycles the
    filter and asserts rows of the other group are hidden; a11y roles/labels
    on the rows and section headers. | source: owner decisions + zeron
    `SIDEBAR_DISCLOSURE_*` metrics
  - R3: Clicking a row (a) sets the app's `current_page`, (b) Settings receives
    `selected_page(current_page)` so the content pane follows, and (c) the
    active chip + titlebar title update. No state duplication: the app value
    is the single source. | acceptance: view test clicks a row from the custom
    nav and asserts the rendered page changed (a page-specific content
    selector appears) and the titlebar title text updated. | source: owner
    decision + vendored `SelectIndex`
  - R4: The footer identity row shows the configured display name (or
    username) with an initial avatar and keeps the theme toggle as an icon
    button; it replaces the old footer brand/theme row. | acceptance: view
    test asserts the footer shows the display name from settings and the
    theme toggle still flips appearance. | source: owner decision
  - R5: The brand wordmark is removed from the rail (zeron has no brand
    there); the rail keeps the single 38px chrome offset from
    `sidebar_header_padding()` and the full height tone/hairline. | acceptance:
    render evidence has no brand row; style assertions unchanged.
  - R6: The titlebar overlay gets a centered page-title label (zeron's
    centered session title) that follows `current_page`, without covering the
    sidebar toggle or the traffic lights. | acceptance: view test asserts the
    title text; render shows it centered. | source: owner screenshot
  - R7: New user-visible strings are translated (en/vi/zh-CN/ja): filter
    label, section titles, default when unnamed. Existing persistence keys are
    untouched. | acceptance: i18n completeness test green. | source: repo
    convention (src/i18n.rs)
- non-goals:
  - NG1: No zeron composer/bottom status bar, message canvas, project/session
    semantics or timestamps/badges; mono-desk remains a settings app.
  - NG2: No back/forward/plus/ellipsis titlebar cluster from the screenshot
    (they control surfaces mono-desk does not have); only the centered title
    is adopted.
  - NG3: No new persistence keys (section/filter state is in-memory).
  - NG4: No changes to the settings content pages themselves.

## Phases and Verification
- approach: one normal-lane phase. Order: vendor patch + default-path proof
  first (R1), then view state + custom nav + footer (R2/R4/R5), then
  selection wiring + titlebar title (R3/R6), then i18n/tests (R7), then
  renders + gate.
- constraints: the patch must be minimal and reproduce via
  `vendor/patches/`; the app owns selection; the vendored default menu must
  stay reachable for any caller that does not pass `sidebar_nav`; no new
  dependencies.
- recovery: revert `vendor/gpui-component/src/setting/settings.rs` and the new
  patch file, `git checkout -- src/`, delete this plan.
- risks: render-time reconciliation of `selected_page` must not loop (write
  only when different, and the app passes its own current value); hiding the
  vendored search while the custom nav is active removes the vendored filter
  affordance for those callers (documented in the patch); a cycling filter row
  is a V1 stand-in for a real dropdown menu (recorded as a decision, not
  claimed as the final control).
- phase_slug: `main`
  status: planned
  - goal: R1, R2, R3, R4, R5, R6, R7
  - surfaces: `vendor/gpui-component/src/setting/settings.rs`,
    `vendor/patches/sidebar-nav-slot.patch`, `Cargo.toml` (patch-list
    comment), `src/main.rs`, `src/shell.rs`, `src/i18n.rs`,
    `src/view_tests.rs` | avoided: `assets/`, `src/persist.rs`
  - escalate_when: the vendored render cannot hide the default menu without
    breaking group selection; the controlled selection loop oscillates; or
    the custom nav cannot reach `cx.notify` for section toggles.
  - wave 1:
    - T1 — patch `Settings` with `sidebar_nav` + `selected_page`, write the
      patch file, update the Cargo.toml comment; default path unchanged. —
      check: `make check` + `cargo test` (existing tests). — stop_if: render
      reconciliation loops.
  - wave 2:
    - T2 — `SettingsWindow` gains `current_page`, `sections` state and the
      filter; build the custom nav (sections + rows + header filter) and
      footer identity; drop the brand header. — check: `make check` +
      offscreen render.
  - wave 3:
    - T3 — wire `selected_page(current_page)` and the centered titlebar
      title; i18n keys. — check: `make check && make lint`.
  - wave 4:
    - T4 — view tests (sections, filter, selection/title, footer name, theme
      toggle) + renders; gate. — check: `cargo test && make check && make
      lint && make fmt-check`.
  - phase check: `cargo test && make check && make lint && make fmt-check`
    plus light/dark renders.
  - owner check: `make run` — sections toggle, filter cycles, row click
    switches the page, footer identity + centered title, glass/opaque both
    look structured.
  - rollback_point: working tree (no commit created).

## Log

- 2026-10-28T14:10Z — main — start — working the waves; vendor surfaces first, then the app
  nav/footer, then the titlebar title and tests.
- 2026-10-28T14:10Z — main/T1 — decision — the `sidebar_nav` slot renders the custom tree
  **inside the Sidebar header slot**, not as a `Sidebar::child`: the Sidebar
  virtualized `list` clones every content item, and `AnyElement` is not
  `Clone`, so a child-based nav vanished after the first clone (diagnosed by
  tracing render calls; a throwaway `Rc<RefCell<Option<..>>>` wrapper rendered
  once then empty). The patch therefore keeps the single `SidebarMenu` child
  type and paints an empty menu when `sidebar_nav` is set. Patch file
  `vendor/patches/sidebar-nav-slot.patch` was verified to reproduce the
  vendored file byte-for-byte (`patch -p1` on the HEAD copy + `diff -q`).
- 2026-10-28T14:10Z — main/T2 — decision — nav rows/section headers/filter use the
  vendored `Button` (ghost, `selected` for the active page), not hand-styled
  `div`/`h_flex` rows: plain divs with `.role()` do not get AccessKit nodes in
  this gpui build, so they were invisible to the registered tree and tests
  could not click them; `Button` gives labels, roles and hit targets for free.
  Existing app-window tests were moved from the old vendored item ids (`0-N`)
  to `nav-page-N`; the obsolete collapsed-icon-rail tooltip test and its
  helpers were deleted (the zeron rail collapses to a 1px sliver, so there is
  no icon rail to hover).
- 2026-10-28T14:10Z — main/T2 — decision — the header filter cycles All → Preferences →
  Workspace (V1) instead of opening a popover menu; the vendored menu module
  has no dropdown primitive that fits the row, so the real popup stays a
  follow-up. `appearance.cycle` i18n key added so the footer toggle has a
  translated tooltip/label.
- 2026-10-28T14:10Z — main/T4 — done — gate green after removing the now-dead brand row
  (`shell::sidebar_brand`) and its imports.
- 2026-10-28T14:42Z — main/T5 — defect (owner GUI check) — the collapsed rail rendered a
  "nồi thập cẩm": nav icons/labels painted over the page content. Root cause:
  the spring closure still passed `.sidebar_collapsed(sidebar_icon_mode(collapsed, w))`
  once collapse started, and the vendored `Sidebar` icon mode pins its own
  width to `COLLAPSED_WIDTH = 48` inside the panel that is animating to 1px —
  the rail overflows the 1px resizable panel and paints across the page (with
  the custom nav's labels wrapping to ~56px rows inside that 48px column).
  Fix: never hand `sidebar_collapsed` to `Settings`; unmount the header row,
  nav tree and footer identity while `ShellState.collapsed` (so they cannot
  paint outside the sliver or stay in the tab order); the animated panel
  width is the only collapse driver. The scroll/paint proof came from a
  throwaway snapshot probe: expanded every `nav-page-N` is registered at
  x≈10.5/width 234; after collapse only the page control (`appearance-light`)
  remains; after reopen the rows come back. Regression assertions added to
  `sidebar_collapses_to_sliver_and_resizes`; the dead `sidebar_icon_mode` /
  `SIDEBAR_LABEL_REVEAL_W` / `SIDEBAR_EXPANDED_W` helper and its unit test
  removed.
- 2026-10-28T14:42Z — main/T5 — decision (owner note on the traffic-light area) — aligned
  the titlebar cluster to zeron's source geometry instead of the vendored
  defaults: cluster start 88px (`titlebar_cluster_start`, the vendored
  `TitleBar` pads 80), a 24px `.small()` toggle (zeron's `window_control_button`),
  and a 4px cluster top pad (`TITLEBAR_TOP_PAD`) so the button center lands on
  the traffic-light axis (y≈21) instead of the bar's geometric center (y≈19).
  The back/forward/+/⋯ controls stay out: this settings app has no session or
  navigation actions to drive them (recorded as a known deviation).
- 2026-10-28T14:46Z — main/T5 — evidence (live app, rebuilt bundle, AX tree) — collapsed: every
  nav row (`All settings`, `Preferences`, `General`, `Notifications`, `Profile`,
  `Workspace`, `Finance UI`, `Reset`, `About`) and the footer theme toggle are
  gone from the accessibility tree while the page controls stay (swatches,
  Accent color, Font Size, Send, About links); re-expanding restores them.
  The titlebar toggle's frame is window-local (88, 10, 21×21) — zeron's 88px
  cluster start with its center on the traffic-light axis. Pixel capture of
  other apps' windows is unavailable here (macOS hides windows from
  `screencapture` without Screen Recording permission: only wallpaper + menu
  bar come back), so the owner's eye remains the visual gate.

## Validation

- 2026-10-28T14:10Z — phase `main` — verdict: APPROVED — mode: gate
  - `cargo test` — ok. 101 passed; 0 failed — pass
  - `make check` — rc 0 — pass
  - `make lint` — rc 0 (clippy `-D warnings`; only the pre-existing
    `block v0.1.6` future-incompat note) — pass
  - `make fmt-check` — rc 0 — pass
  - scope: on target — vendor `settings.rs` + new
    `vendor/patches/sidebar-nav-slot.patch`, `Cargo.toml` patch-list comment,
    `src/main.rs`, `src/shell.rs`, `src/i18n.rs`, `src/view_tests.rs`;
    `src/persist.rs`, `assets/` untouched.
  - requirements: R1 met (patch surfaces + byte-for-byte reproduction proof) |
    R2 met (filter header row, two collapsible sections, page rows, active
    chip, section/filter state on the view, i18n) | R3 met (row click drives
    `current_page` → `selected_page`, content follows; test asserts a
    page-specific control appears) | R4 met (footer identity + working theme
    toggle; display name is painted, tested through the toggle and reviewed
    by eye, not asserted textually) | R5 met (brand row deleted, chrome offset
    kept) | R6 met in code (centered title follows `current_page`) |
    R7 met (new strings in all four locales; no persistence changes).
  - rollback_point: working tree only (no commit); revert the vendor files from
    HEAD, delete the new patch file, `git checkout -- src/`, delete the plan.
  - not_independently_verified: owner macOS GUI check (section toggles, filter
    cycle, active chip, footer identity, centered title, glass + opaque) and
    offscreen light/dark renders — no render tool is in the tree after the
    earlier probe was removed; the centered title has no view assertion yet
    (plain text, not a component).
  - judge: same-session
  - judge_model: deepseek-v4.1-flash

- 2026-10-28T14:42Z — phase `main` (collapse + titlebar follow-up) — verdict: APPROVED — mode: gate
  - `cargo test` — ok. 100 passed; 0 failed (the removed icon-mode unit test
    accounts for the drop from 101) — pass
  - `make check` — rc 0 — pass
  - `make lint` — rc 0 (clippy `-D warnings`; same pre-existing `block v0.1.6`
    future-incompat note) — pass
  - `make fmt-check` — rc 0 — pass
  - regression proof: `sidebar_collapses_to_sliver_and_resizes` now asserts
    the collapsed rail unmounts `nav-page-0` and `nav-footer-theme`, keeps the
    page mounted (`appearance-light`), and restores the nav tree on reopen.
    A throwaway probe confirmed the before/after snapshot bounds (nav rows at
    x≈10.5/width 234 expanded → absent collapsed → restored).
  - not_independently_verified: owner GUI re-check of the fixed collapse and
    the zeron titlebar cluster on the running bundle; offscreen render evidence
    is still unavailable — macOS hides other apps' windows from `screencapture`
    (only wallpaper + menu bar are captured), so the screenshot must come from
    the owner or from a future in-app probe.
  - judge: same-session
  - judge_model: deepseek-v4.1-flash

## Current State and Next Action
- active_phase: main
- lifecycle_status: in-progress
- blockers: none
- open_items: owner GUI re-check on macOS (collapsed rail must be a bare 1px
  sliver; titlebar toggle now on zeron's 88px/24px geometry); light/dark render
  evidence (still no probe in tree); independent `check full` before handoff;
  real filter dropdown menu is a follow-up (V1 cycles); zeron's back/forward/
  plus/ellipsis titlebar controls are out of scope (no actions to drive).
- exact_next_action: owner visual check of the rebuilt bundle, then independent
  `check full` and `handoff`.
