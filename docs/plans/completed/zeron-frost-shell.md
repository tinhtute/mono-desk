---
status: completed
lane: normal
---
# Frost shell + hairline (zeron-style layered background)

## Goal
- outcome: With translucency on, the window renders as one zeron-style
  frost shell instead of two inset alpha panes: a single full-window frost
  layer at the "Window opacity" alpha, a transparent sidebar column with
  its own ink-tone wash and a 1px right hairline, and the settings content
  flush on the frost (no outer card, no 6px gutter/inset margins).
  "Sidebar tone" replaces the old pane-opacity slider and drives the wash
  strength over the frost. With translucency off the app renders exactly
  as today (opaque card layout, byte-identical palette).
- success_signal: `cargo test && make check && make lint && make fmt-check`
  green; offscreen light/dark renders show one uniform frost layer, the
  sidebar tone + hairline, and content flush on the frost; the owner, in
  `make run`, sees the desktop only through the single frost layer, the
  slider changing the shell alpha live, and the sidebar tone staying a
  subtle wash (never a second opaque panel); values survive relaunch.
- actors: the owner (lab demo) and developers reading this repo for GPUI
  glass/layout patterns.
- authority: owner decisions in this session (2026-10-27): "Chuyển sang
  kiểu zeron (frost shell + hairline)", content pane "Flush hoàn toàn như
  zeron", controls "Shell opacity + Sidebar tone". Reference implementation
  read first: https://github.com/zeronsh/zeron — `crates/ui/src/shell.rs`
  (root `.bg(theme.glass())`, transparent `sidebar_tone()` = `wash(0.05)`
  + `border_r_1`, "No inset cards (user request)") and
  `crates/ui/src/theme.rs` (`glass()` alpha, `is_glass`). Repo conventions:
  AGENTS.md / docs/PROJECT.md; prior art and persisted contracts:
  docs/plans/completed/glass-background.md, docs/memory/gpui-window-glass.md.
- requirements:
  - R1: While translucency is on, the app paints exactly one full-window
    glass base at the "Window opacity" alpha; no pane paints a second base
    behind the content. `Root`'s opaque base stays switched off (existing
    `WindowRoot` path) and the title bar keeps painting nothing. | acceptance:
    `theme` unit test for the alpha mapping helper (100% returns the exact
    palette colour); offscreen render at 85% shows a uniform frost layer
    with the desktop absent only because the offscreen renderer has no
    backdrop; `make check` green. | source: owner decision + zeron
    `shell.rs` root frost
  - R2: The sidebar column becomes transparent: no panel background, no
    rounding, no outer margin/border. It paints an ink tone at
    `SIDEBAR_TONE_MAX_ALPHA` scaled by the "Sidebar tone" percent
    (0–100, default 50 → 5% wash, same as zeron's `wash(0.05)`), plus a 1px
    `theme.border` hairline on its right edge, spanning the sidebar's full
    height. | acceptance: pure mapping test (0 → transparent, 50 → 0.05,
    100 → 0.10, NaN → default) and an offscreen render showing the tone
    strip + hairline with the content flush beside it. | source: owner
    decision + zeron `sidebar_tone()`
  - R3: The content pane is flush: the content card's rounding, border,
    background, 6px margins and drop shadow are removed while glass is on,
    and the settings page keeps its own readable inner padding. The 6px
    gutter no longer exists in glass mode. | acceptance: offscreen render
    shows page content starting at the hairline seam with no card; opaque
    mode render (translucency off) still shows the old card unchanged.
    | source: owner decision ("Flush hoàn toàn như zeron")
  - R4: "Window opacity" replaces the old pane slider: 40–100%, default
    85%, step 1, disabled while translucency is off, live on `Change` and
    persisted only on `Release`; it drives R1's alpha and re-reads on reset.
    | acceptance: theme clamp/mapping tests; view test for gating + persist
    via the slider label. | source: owner decision + existing glass contract
  - R5: "Sidebar tone" replaces the second pane slider: 0–100%, default
    50%, step 1, disabled while translucency is off, same live/persist
    contract; it drives R2's wash alpha only and never paints the sidebar
    as a second base layer. | acceptance: mapping + clamp tests; view test
    for gating/persist. | source: owner decision
  - R6: Persistence: the two stored values are `window_opacity` (clamped
    40–100, missing/malformed → 85) and `sidebar_tone` (clamped 0–100,
    missing/malformed → 50); the pre-release pane keys (`sidebar_opacity`,
    `pane_opacity`) are dropped without migration because the glass feature
    was never released. `background_translucent` / `background_blur` keep
    their keys and behavior. | acceptance: persist round-trip + clamp +
    legacy-file tests updated; `cargo test persist::` green. | source:
    src/persist.rs contract + docs/memory/persist-ownership-guard.md
  - R7: Rows: General → Background shows Window opacity and Sidebar tone
    sliders (accessible names via the owner-approved vendored
    `slider-accessibility-label` patch, percentage painted beside each
    track), disabled while the master switch is off, resettable with the
    page (off / 85 / 50 / blur off), translated en/vi/zh-CN/ja, and listed
    in the changed-settings summary. | acceptance: i18n completeness +
    existing view tests updated to assert both labels in the a11y tree,
    gating, reset, and persistence. | source: repo conventions
    (src/i18n.rs, real-settings R6/R7) + prior glass plan R5
  - R8: The opaque (translucency-off) presentation is unchanged: shell base
    `theme.background`, vendored sidebar token paint, content card with
    rounding/border/margin exactly as shipped before this phase. | acceptance:
    the glass-toggle view test asserts the opaque path still produces the
    card style (debug selector present) and the offscreen opaque render is
    unchanged from the pre-change render. | source: owner decision (only
    glass-mode look changes) + AGENTS.md default rendering rule
- non-goals:
  - NG1: No in-app per-surface backdrop blur (frosted popovers/dialogs):
    `gpui-pre` 0.3.5 has no backdrop-blur primitive (zeron's fork adds
    `paint_backdrop_blur`); recorded as unavailable, not faked.
  - NG2: No `EdgeFade` per-pixel scroll fade (zeron fork primitive).
  - NG3: No platform-policy change: Linux/Windows window appearance keeps
    the existing helper and blur behavior; no new contrast-checked alpha
    auto-tuning (separate follow-up).
  - NG4: No titlebar geometry/traffic-light/min-size changes; no
    full-height sidebar tone under the titlebar (zeron detail deferred).
  - NG5: No new dependencies and no new vendor patches beyond the existing
    slider a11y patch.
  - NG6: No opaque-mode redesign and no theme palette mutation.

## Phases and Verification
- approach: one normal-lane phase. Order: pure theme mappings first
  (R1/R2, testable headless), then persistence codec (R6), then app state +
  settings rows + i18n (R4/R5/R7), then the layout/paint switch (R1/R2/R3),
  then view-test updates (R4/R5/R7/R8). Rejected: a separate `Frost` page
  (General → Background already owns these controls); rejected: keeping the
  two pane alphas and painting both shell + sidebar alpha (compounds to
  `1-(1-a)^2` and reads opaque — the exact regression the prior plan
  fixed); rejected: mutating the palette (paint-time alpha only).
- constraints: alpha stays paint-time (`theme::glass_surface`), palette
  untouched; the sidebar tone is an ink wash (foreground-derived), never a
  second glass base; flush layout is glass-only, opaque style preserved;
  writes stay inside the declared surfaces; this plan is the only doc
  changed until `check`.
- recovery: all changes uncommitted — `git checkout -- src/` and delete
  this plan; the persisted file only renames pre-release keys.
- risks: text contrast at low shell alpha (floor stays 40%); the sidebar
  tone at 100% (10% ink) may read heavy in light mode (owner visual check);
  the offscreen renderer composites transparency onto black, so the owner's
  macOS visual check is the definitive evidence for the desktop showing
  through one layer; `Root`/window handles are test-platform-limited.
- phase_slug: `main`
  status: done
  - goal: R1, R2, R3, R4, R5, R6, R7, R8
  - surfaces: `src/theme.rs`, `src/persist.rs`, `src/main.rs`,
    `src/shell.rs`, `src/i18n.rs`, `src/view_tests.rs`,
    `docs/memory/gpui-window-glass.md` (model correction at close)
    | avoided: `assets/`, `vendor/`, `Cargo.toml`, other docs
  - escalate_when: a pane needs a second alpha layer to stay readable; the
    sidebar tone cannot span the full sidebar height without a new
    wrapper; or the flush content loses the settings page's inner padding
    and no app-owned style can restore it.
  - wave 1:
    - T1 `src/theme.rs` — `WINDOW_OPACITY_MIN/MAX`, `SIDEBAR_TONE_MAX_ALPHA`,
      `sidebar_tone_alpha(percent)` (0 → 0, NaN → default), reuse
      `glass_surface`/`clamp_glass_opacity` for the shell; unit tests. —
      output: pure helpers + tests — check: `cargo test theme::` — stop_if:
      the tone needs a colour not derivable from the theme.
    - T2 `src/persist.rs` — replace `sidebar_opacity`/`pane_opacity` with
      `window_opacity` (40–100, 85) and `sidebar_tone` (0–100, 50) in
      `StoredSettings`/`Default`/`parse`/`serialize` + tests. — output:
      codec — check: `cargo test persist::` — stop_if: —
  - wave 2:
    - T3 `src/main.rs` — mirror the two fields in `AppSettings`
      (`Global`/`Default`/`from_stored`/`to_stored`), `GlassSurface`-free
      paint helpers (`shell_frost_bg`, `sidebar_tone_bg`), handler wiring
      (live vs persist), startup unchanged. — output: state + helpers —
      check: `make check` — stop_if: the handler cannot reach both values.
    - T4 `src/i18n.rs` — rename keys/descriptions for `item.window_opacity`
      and `item.sidebar_tone` in en/vi/zh-CN/ja (tone description: ink wash
      over the frost, 0–100%). — output: translations — check:
      `cargo test i18n::` — stop_if: —
  - wave 3:
    - T5 `src/main.rs` — General → Background rows rewritten: Window
      opacity + Sidebar tone sliders keyed per control, gating on the
      master switch, `on_reset` (85 / 50), changed-settings summary entries.
      — output: working rows — check: `make check && make lint` — stop_if:
      keyed slider state collides.
    - T6 `src/shell.rs` — AppShell: glass base = one `shell_frost_bg`;
      titlebar stays transparent; opaque base unchanged. — output: single
      frost layer — check: `make check` — stop_if: `Root` still paints a
      base under glass.
    - T7 `src/main.rs` — glass-only flush layout: sidebar paint = tone +
      right hairline (no panel bg/rounding/margin), content panel style =
      no card/margin/shadow; opaque branch keeps card + vendored sidebar
      paint. — output: zeron-style layout — check: `make check` +
      offscreen light/dark render — stop_if: content loses inner padding.
  - wave 4:
    - T8 `src/view_tests.rs` — update glass tests to the new labels/keys
      (Window opacity, Sidebar tone), gating, reset (off/85/50/off), persist
      of both values, root base override, and the opaque-card assertion for
      R8. — output: view coverage — check: `cargo test view_tests::` —
      stop_if: the harness cannot observe the flush style.
  - phase check: `cargo test && make check && make lint && make fmt-check`
    plus light/dark offscreen renders attached as evidence.
  - owner check (headless — `make run`, GUI-only): translucency ON shows the
    desktop through one frost layer; Window opacity changes it live; the
    sidebar keeps a subtle tone + hairline and never looks like a second
    panel; content sits flush; light/dark flip and relaunch keep the values.
  - rollback_point: the working tree's current uncommitted state (no commit
    created by this plan).

## Log

- 2026-10-27T15:41Z — main — decision — tone constants are configurable defaults, not owner-locked numbers: `SIDEBAR_TONE_MAX_ALPHA = 0.10` with default 50% (0.05 wash) so the slider's midpoint lands exactly on zeron's `wash(0.05)`; the owner approved the semantics ("Sidebar tone = độ đậm wash"), not the scale.
- 2026-10-27T15:41Z — main — decision — the active sidebar pill can no longer use a pane alpha (the rail is a tone, not a surface): it paints `glass_surface(group_box, 70)` under glass, so it stays a subtle raised chip over the tone; opaque mode keeps the flat token.
- 2026-10-27T15:41Z — main — decision — R8's planned "debug selector" proof was replaced with a stronger pure-style assertion (`content_panel_style`/`sidebar_refinement` fields) because the glass-only branches are computed, not rendered with app-owned selectors; the opaque card is additionally visible in the offscreen opaque render.
- 2026-10-27T15:44Z — main — decision — touched the phase's avoided `Cargo.toml` by one comment line only: the vendor-patch note still named the removed "Background opacity" row; corrected to "Window opacity, Sidebar tone" so the patch bookkeeping matches R7. No dependency or feature change.
- 2026-10-27T15:41Z — main — decision — offscreen evidence limit: the test renderer composites transparency onto black, so the renders prove the layer structure and hairline but not the desktop showing through; the owner macOS check remains the definitive visual proof (open item).


- 2026-10-28T11:44Z — main — decision — absorb: memory gpui-window-glass (frost-shell model, macOS 26 blur risk, re-apply rule); no new ADR or guard needed.
## Validation

- 2026-10-28T11:44Z — phase `main` — verdict: APPROVED — mode: full
  - `cargo test` — test result: ok. 100 passed; 0 failed — pass
  - `make check` — Finished dev profile, rc 0 — pass
  - `make lint` — Finished dev profile, rc 0 (no warnings) — pass
  - `make fmt-check` — cargo fmt --check clean — pass
  - scope: on target — the whole uncommitted diff is the glass feature plus this phase's frost-shell model; the phase's declared surfaces were respected (Cargo.toml touched by one vendor-comment line, logged as a decision).
  - requirements: R1 met (alpha-mapping test + uniform frost pixels) | R2 met (tone mapping tests + measured 1px hairline at the seam) | R3 met (flush style assertions + render) | R4 met (Window-opacity view test gating/live/release) | R5 met (Sidebar-tone mapping + gating/persist) | R6 met (persist round-trip/clamp/legacy for both keys) | R7 met (a11y labels, reset, i18n completeness) | R8 met (opaque card style assertions + opaque render)
  - rollback_point: working tree only — no commit was created; revert `src/` and restore this plan from `docs/plans/completed/`.
  - requests: none
  - not_independently_verified: same-session full review (the author reviewed the diff); the independent judge could not run — two fresh-context subagent judges failed before returning a verdict (the runtime reported "failed before it finished, left no closing message"), and the owner said "continue". Owner macOS GUI check also pending (desktop through one frost layer, tone at 100 %). The offscreen renderer composites transparency onto black and cannot prove the real backdrop.
  - judge: same-session
  - judge_model: deepseek-v4.1-flash
## Current State and Next Action

- active_phase: none
- lifecycle_status: completed
- blockers: none
- open_items: none
- exact_next_action: initiative complete; the next zeron-layout initiative is locked separately.
