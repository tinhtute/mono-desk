---
status: completed
lane: normal
---
# Glass background: opacity + blur

## Goal
- outcome: General → Background exposes a translucency switch, a
  window-background opacity slider (40–100%), and a blur switch that make
  the app's own base surfaces see-through and, when the platform supports
  it, frosted — mirroring the glass treatment in
  https://github.com/hardbeat920/monocode (transparent Tauri window +
  NSVisualEffectView vibrancy + per-surface alpha). With translucency off
  the app renders exactly as today, byte-for-byte in the theme palette.
- success_signal: `cargo test && make check && make lint && make fmt-check`
  green; with translucency on, the owner sees the desktop through the
  title bar, sidebar and content card at the chosen opacity (owner-approved
  single-alpha-layer model; the 6px window gutter stays clear as the inset
  frame), and macOS shows a frosted (vibrancy) backdrop while blur is on;
  values survive relaunch.
- actors: the owner (lab demo) and developers evaluating gpui-kit window
  chrome patterns.
- authority: owner request in this session (2026-10-27) naming the
  monocode reference and asking for "giảm opacity + blur cho background";
  GPUI engine actually in use: gpui-pre 0.3.5 `WindowBackgroundAppearance`
  (`Opaque`/`Transparent`/`Blurred`) + `Window::set_background_appearance`;
  app conventions in AGENTS.md / docs/PROJECT.md (single gpui-kit dep, no
  vendor patch unless required); persistence + i18n patterns established
  by docs/plans/completed/real-settings.md.
- requirements:
  - R1: A "Translucent window" switch in a new General → Background
    group. ON: the window background appearance becomes `Blurred` when
    blur is on, else `Transparent`; OFF: `Opaque`. The decision is one
    pure function so it is testable without a GPU window. | acceptance:
    unit tests for `window_background_for(translucent, blur)` covering all
    four combinations; `make check` green. | source: owner request +
    gpui-pre `WindowBackgroundAppearance`
  - R2: A "Background opacity" slider, 40–100% step 1, default 85%,
    disabled while translucency is off. While translucency is on, the
    knob sets the alpha of the three panes the app paints — title bar,
    sidebar, content card — and the full-window shell base is left clear
    so each pixel carries at most one alpha layer over the platform
    backdrop (stacked alpha would composite to 1-(1-a)² and read opaque
    at 85%). Live on `Change`, persisted only on `Release` (same contract
    as the Font Size slider). At 100% or with translucency off the palette
    is left untouched. | acceptance: `theme::glass_surface` unit tests
    (alpha mapping; 100%/NaN → exact palette colour) + clamp tests;
    view-test coverage of the slider handler (live vs persist, clamp).
    | source: monocode "Sidebar opacity" (0.85 default) + owner request
  - R3: A "Background blur" switch, default off, disabled while
    translucency is off, that requests platform window blur through the
    single helper from R1 (macOS NSVisualEffectView vibrancy; Windows DWM
    blur; Linux where the compositor exposes blur, plain transparency
    otherwise). No custom material/radius — GPUI exposes none. |
    acceptance: `window_background_for` tests; disabled/inert behaviour
    shown by a view test; owner visual check on macOS. | source:
    gpui-pre-macos 0.3.5 `BlurredView`; monocode native glass
  - R4: The three settings persist (`background_translucent`,
    `background_opacity`, `background_blur`) through the existing guarded
    `key=value` file: defaults on absence, clamp 40–100 on malformed
    opacity, old files without the keys keep working, save on every
    committed change, applied at startup before/at window creation so
    there is no opaque flash. | acceptance: persist round-trip + clamp +
    legacy-file tests; wiring visible in the startup diff. | source:
    src/persist.rs contract (docs/memory/persist-ownership-guard.md)
  - R5: The new rows are themed, translated (en/vi/zh-CN/ja), resettable
    with the General page (back to off/85%/off, window returned to
    opaque), and accessible: both switches and both sliders (Font Size and
    Background opacity) carry accessible names via the owner-approved
    `slider-accessibility-label` vendor patch; the opacity slider exposes
    its numeric value to assistive tech and paints the percentage beside
    the track; disabled controls do not react, proven behaviorally with a
    real track click (the disabled slider attaches no listeners). (gpui-pre
    0.3.5 never sets AccessKit's disabled flag, so `snapshot.disabled()` is
    always `None`; and the on-disk live-vs-Release split is not observed
    because the shared test config dir races — both recorded as proof gaps.) | acceptance: i18n completeness test; view
    test asserting both slider names in the a11y tree + gating + reset;
    `cargo test && make check && make lint && make fmt-check`. | source:
    repo conventions (src/i18n.rs, real-settings R6/R7) + owner decision
    2026-10-27 (vendor patch over exception)
  - R6: No regression in the default path: with translucency off, the
    rendered palette is exactly the shipped `mono.json` one; a theme-mode
    flip (light↔dark, OS-follow) leaves the glass state intact — the
    alpha is read per paint and the window surface/Root override live
    outside `Theme::change`. | acceptance: theme test proving
    `glass_surface` at the opaque end returns the exact palette colour
    (both modes); view test that a mode flip with translucency on keeps
    the setting and the Root override. | source:
    docs/memory/gpui-component-setting-field-pitfalls.md item 3
- non-goals:
  - NG1: No wallpaper/image background picker (monocode's chat-background
    feature is out of scope here).
  - NG2: No separate sidebar / content-card / main-pane opacity knobs —
    one opacity for the window base surfaces.
  - NG3: No custom blur material, radius, or saturation tuning: gpui-pre
    0.3.5 exposes no API and a vendored engine fork is out of scope.
  - NG4: No in-app backdrop-filter for dialogs, popovers or notifications
    (GPUI has no per-element blur); those stay opaque for readability.
  - NG5: No Windows Mica/Acrylic variants in the UI (engine supports
    `MicaBackdrop`, but the products asked for blur+opacity only).
  - NG6: No new direct dependencies. One owner-approved scope expansion:
    the `slider-accessibility-label` delta in `vendor/gpui-component` plus
    its `Cargo.toml` patch-list comment (no dependency or feature change
    beyond that).
  - NG7: No per-window/per-display persistence.

## Phases and Verification
- approach: one normal-lane phase. Order: pure theme helper first (R2/R6,
  testable without a window), then persistence codec (R4), then app state
  + startup + window-appearance helper (R1/R3), then i18n (R5), then the
  settings rows (R1/R2/R3/R5), then view tests. Rejected: a separate
  Appearance page — the General page owns the existing Appearance group
  and its reset flow; rejected: mutating `mono.json` — the alpha must be
  runtime-only and default-off; rejected: a vendored patch for a blur
  radius — no API and NG5.
- constraints: theme tokens may only be mutated inside `apply_overrides`
  (re-applied after every `Theme::change`); window appearance writes go
  through one helper; opacity is clamped at both the codec and the theme
  boundary; writes stay inside the declared surfaces (`src/`, the one
  vendored slider file and its patch, the `Cargo.toml` comment, this plan);
  the memory artifact belongs to the close, not the work phase.
- recovery: all changes are uncommitted — `git checkout -- src/ Cargo.toml
  vendor/gpui-component/src/slider.rs`, delete
  `vendor/patches/slider-accessibility-label.patch`, and delete this plan
  file; the persisted file gains only ignorable keys. Revert the vendor
  file even if only `src/` is rolled back, or the patch file and the
  vendored source disagree.
- risks: transparency cost on GPU and text contrast at 40% (mitigated by
  the 40% floor and by keeping the feature off by default); Linux needs a
  compositing WM and may show plain transparency instead of blur (owner
  check is macOS-first, matching the reference); the test platform's
  `set_background_appearance` is a no-op, so the visual result is
  owner-verified while the decision function is unit-tested.
- phase_slug: `main`
  status: done
  - goal: R1, R2, R3, R4, R5, R6
  - surfaces: `src/theme.rs`, `src/persist.rs`, `src/main.rs`,
    `src/shell.rs`, `src/i18n.rs`, `src/view_tests.rs`,
    `vendor/gpui-component/src/slider.rs`,
    `vendor/patches/slider-accessibility-label.patch`, `Cargo.toml`
    (patch-list comment only) | avoided: `assets/`, `docs/` (only this
    plan) — surfaces corrected with owner approval when the vendor-slider
    patch was chosen (see Log 2026-10-27T14:06Z)
  - escalate_when: `WindowBackgroundAppearance`/`set_background_appearance`
    are not reachable through `gpui_kit`; a theme surface token proves
    immutable through `Theme::global_mut`; or a view test cannot observe
    the disabled gating.
  - wave 1:
    - T1 `src/theme.rs` — `GLASS_OPACITY_MIN/MAX`, `clamp_glass_opacity`,
      `write_glass(theme, percent)` setting `.a` on `background`,
      `sidebar`, `title_bar`, `group_box` (no-op at >= 100), and threading
      the percent through `install` + `apply_overrides`; unit tests. —
      output: pure helper + tests — check: `cargo test theme::` — stop_if:
      `Theme` deref makes a token unreachable.
    - T2 `src/persist.rs` — `background_translucent: bool`,
      `background_opacity: f64` (40–100), `background_blur: bool` in
      `StoredSettings`/`Default`/`parse`/`serialize`; tests for round-trip,
      clamp, unknown/missing keys. — output: codec — check:
      `cargo test persist::` — stop_if: —
  - wave 2:
    - T3 `src/main.rs` — mirror the three fields in `AppSettings`
      (`Global`, `Default`, `from_stored`, `to_stored`),
      `window_background_for`, `apply_window_glass`, startup
      `WindowOptions.window_background` + `theme::install` arguments, and
      re-apply glass in `apply_font_size`. — output: state + startup wiring
      — check: `make check` — stop_if: `open_window` closure cannot reach
      the settings global.
    - T4 `src/i18n.rs` — keys for group title, three item labels +
      descriptions, percent format, in en/vi/zh-CN/ja. — output: translated
      rows — check: `cargo test i18n::` — stop_if: —
  - wave 3:
    - T5 `src/main.rs` — General → Background group: translucent switch
      (`background-translucent-row`), opacity slider with keyed
      `SliderState` (`Change` → live theme, `Release` → persist,
      `background-opacity-row`), blur switch (`background-blur-row`),
      gating via `options.is_disabled() || !translucent`, `on_reset` on
      all three, window-appearance helper invoked on every change. —
      output: working rows — check: `make check && make lint` — stop_if:
      `SliderState` cannot be keyed twice in one page.
    - T6 `src/view_tests.rs` — gating test (blur + opacity inert while
      translucent is off), reset test (page reset returns the three values
      + opaque window), persistence test (toggle writes
      `background_translucent=true`). — output: view coverage — check:
      `cargo test view_tests::` — stop_if: the harness cannot observe the
      window background appearance.
  - phase check: `cargo test && make check && make lint && make fmt-check`
  - owner check (headless — `make run`, GUI-only): General → Background:
    translucency ON reveals the desktop through base/sidebar/titlebar and
    the content card; slider changes alpha live and shows the percentage;
    macOS blur gives a frosted backdrop; light/dark flip keeps the glass;
    relaunch restores all three; page reset returns to opaque.
  - rollback_point: the commit before this phase (working tree only).

## Log

- 2026-10-27T13:31Z — main — decision — requirement text says the alpha is written onto four theme tokens; execution found the vendored `Root`/`Dialog` share the `theme.tokens.background` snapshot, so mutating the palette would also make dialogs translucent (unreadable, no per-element blur in GPUI). Switched to paint-time alpha (`theme::glass_surface`) on exactly the four app-owned window-base surfaces plus one `Root` style override published via a `WindowRoot` global. Outcome (R2/R6) unchanged; the token palette stays byte-identical.
- 2026-10-27T13:31Z — main — decision — GPUI 0.3.5 exposes only `Opaque|Transparent|Blurred` (plus Windows Mica) for the window surface and no blur radius/material API, so the feature ships as switch + opacity + blur switch, not monocode's 1–64px blur-radius slider (NG3). Linux blurs only on compositors that advertise it; Windows acrylic is experimental — both degrade to transparency, documented in the row description.
- 2026-10-27T13:40Z — main — decision — the Background-opacity slider inherits the vendored `Slider`'s a11y gap (no name field; same TODO as Font Size in `font_size_field`): the value is announced as a bare number and the chosen percentage is painted beside the track. Exposing it properly needs a vendored patch, which NG6 rules out; recorded rather than silently claimed.
- 2026-10-27T13:50Z — main — decision — independent check returned REQUEST_CHANGES (M1–M4). M1 resolved in the plan text, not the code: shipped default stays 85% (monocae's 0.85 sidebar opacity; at 100% the first "translucent on" would look inert until the slider moves), so R2/R5/success_signal were reconciled and the deviation is no longer silent. R5's slider-a11y wording was narrowed to the truth (inherited vendored `Slider` gap, NG6).
- 2026-10-27T13:50Z — main — decision — M2 (alpha stacking) fixed in `src/shell.rs`: while glass is on, the full-window shell base paints nothing and only the three panes paint the chosen alpha, so every pixel has at most one alpha layer over the platform backdrop. Trade-off accepted: the 6px gutter around the content card stays fully clear (an inset-glass look), and at 100% the panes are opaque while that gutter still shows the backdrop. R2 now states this model; `surface_bg(background)` is no longer used for a full-window base.
- 2026-10-27T13:50Z — main — decision — M3 (scope drift) fixed by removing the untracked `docs/memory/gpui-window-glass.md` from the work phase: the note belongs to `handoff`'s absorb step, which will recreate it at close. Phase surfaces stay exactly as locked. (m5: the T3 line's "re-apply glass in apply_font_size" claim is wrong — glass state lives outside `Theme::change`; recorded here because task definitions are immutable.)
- 2026-10-27T13:56Z — main — decision — absorb: memory gpui-window-glass (recreated at close from the work-phase note; the reusable rules are the tokens-vs-colors trap, the Root-handle requirement and the one-alpha-layer-per-pixel rule).
- 2026-10-27T13:57Z — main — decision — REOPENED after a second independent review returned REQUEST_CHANGES after the close. Bookkeeping defects it found are real: the `main` phase's locked `surfaces` list omits `src/shell.rs` (which had to change), the completed plan's `success_signal` still says the desktop shows through the "window base" although the M2 fix made the full-window base clear, and several comments still describe the superseded four-token model. The earlier APPROVED entry is kept (it was a real, independent verdict from a different reviewer) but is superseded by the REQUEST_CHANGES entry below. Product-level questions (default opacity, which surfaces carry the alpha, slider a11y exception) are being put to the owner before any code or spec is finalised.
- 2026-10-27T14:06Z — main — decision — owner decisions (session answers): (1) the opacity paints the three panes and the full-window base stays clear, accepting the 6px clear gutter at 100% as the inset-glass look (single alpha layer per pixel); (2) default opacity 85%; (3) the unnamed-slider problem is fixed with a vendor patch rather than accepted as an NG6 exception. This is the explicit owner approval the review asked for: Goal `success_signal`, R1 (now "Translucent window", matching the shipped label), R2 (85% default), R5 (named sliders) and the phase `surfaces` were updated to state the approved model and the widened scope (shell.rs + vendored slider + patch file + Cargo.toml comment).
- 2026-10-27T14:06Z — main — decision — a11y patch target correction: the `Role::Slider` node is built inside `gpui-base` (registry crate, no label API, private node), so patching the vendored wrapper alone cannot name it. The implemented delta mirrors the ~40-line behaviour root (role, aria value/min/max/step/orientation, increment/decrement, release listeners) as `SliderRoot` inside `vendor/gpui-component/src/slider.rs`, adds `Slider::accessibility_label`, and applies the name to Font Size and Background opacity. `vendor/patches/slider-accessibility-label.patch` reproduces the vendored file byte-for-byte from the pre-patch copy, and `Cargo.toml` lists the delta. Also: `unit.percent` i18n key replaces the hardcoded `%`, and `theme::GLASS_OPACITY_DEFAULT` is the single source for the 85 default (pinned to `StoredSettings::default()` in its tests).
- 2026-10-27T14:06Z — main — decision — proof-gap honesty: the "opacity row is inert while translucency is off" and "slider Change/Release widget mapping" cannot be asserted in this harness — gpui-pre 0.3.5 never sets AccessKit's disabled flag (`snapshot.disabled()` is `None` for every element) and the test platform has no drag synthesis (no precedent in the repo or vendored tests). They stay recorded as proof gaps; the handler-level contract is covered directly (Change=live, Release=persist via the persist flag) and the gating is enforced by `Slider::disabled` (listeners not attached). Closed instead: R6 painted-alpha-after-mode-flip is now asserted (`surface_bg(...).a`), and both slider names are asserted in the a11y tree.
- 2026-10-27T14:06Z — main — decision — pre-existing sibling left alone on purpose: `changed_settings` uses `item.channels`/`item.digest` but the i18n table only has their `.desc` keys, so the Reset page can show raw keys. It predates this phase and discovery grants no authority to fix adjacent findings; recorded here as a follow-up, not fixed in this diff.
- 2026-10-27T14:26Z — main — decision — correction to the 14:06Z proof-gap note (append-only, earlier text kept for lineage): the test harness DOES synthesize pointer events (`VisualTestContext::simulate_mouse_down/move/up`, precedent in `vendor/gpui-component/src/text/window_selection.rs`), so "the harness has no drag synthesis" was wrong. The opacity slider's inert-while-off and live-`Change`/persist-`Release` behaviour is now asserted with a real track click (disabled click leaves 85; enabled click at 50% of the 40–100 track lands ~70 and the released value is persisted). What remains genuinely unobservable: AccessKit's disabled flag (gpui-pre 0.3.5 never sets it) and the on-disk live-vs-Release *file* split (the shared scratch config dir races across parallel view tests).
- 2026-10-27T14:26Z — main — decision — adopted review cleanups without expanding product scope: opacity description copy now names exactly the three alpha panes (all 4 locales); stale comments about the pre-patch Slider and the old four-token model fixed; the startup Root override and `apply_glass` share one `root_glass_bg` helper; the vendored slider delta is rustfmt-clean and the patch was regenerated + replayed byte-for-byte. Accepted limitation recorded instead of fixed: the Finance/cadence showcase page paints its own opaque canvas (`src/cadence/showcase.rs`), so glass shows only around that page's content — it is app content, outside the owner-approved window-chrome surfaces.
- 2026-10-27T14:28Z — main — decision — absorb: memory docs/memory/gpui-window-glass.md (already exists: the tokens-vs-colors trap, the Root-handle requirement, one-alpha-layer-per-pixel, and the window-appearance limits). No ADR: the one cross-cutting decision (vendored slider a11y delta) is encoded in the patch file + `Cargo.toml` list + that memory note.

## Validation

- 2026-10-27T14:26Z — phase `main` — verdict: APPROVE_WITH_REQUESTS — mode: full
  - `make check` — pass (cargo check, no errors)
  - `make lint` — pass (clippy --all-targets -D warnings, no warnings)
  - `cargo test` — pass (99 passed, 0 failed, 0 ignored)
  - `make fmt-check` — pass (cargo fmt --check, no diff)
  - `git diff --check` — clean
  - vendor patch replay (`patch -p1` on the pre-patch vendored slider) — byte-identical to the working file
  - scope: on target — 8 modified files match the corrected surfaces; the only extra artifact is `docs/memory/gpui-window-glass.md`, adopted by the close
  - requirements: R1 met (window_background_for + 4-combination test) | R2 met (three-pane alpha, shell/Root clear; owner-approved model; glass_surface/clamp tests; real-click Change/Release proof) | R3 met (appearance helper; blur gating/inert proof; macOS vibrancy = owner check) | R4 met (codec defaults/clamp/legacy tests; startup applies before first frame; single-sourced default) | R5 met (4-locale i18n completeness; both sliders named via the vendored patch and asserted in the a11y tree; disabled slider proven inert behaviorally; reset off/85/off + Root restore) | R6 met (opaque end returns the exact palette colour both modes; mode flip keeps setting, Root override and painted alpha)
  - requests: 1 major + 8 minor from the fresh independent review; all addressed before this entry — M1 (harness DOES synthesize drags) fixed with real click proofs; m1 stale comments fixed; m2 stale plan prose/NG6/constraints/recovery/dangling ref corrected; m3 Validation history reordered and superseded note added; m4 copied text reworded to the three panes; m5 memory artifact named for the close; m6 Finance-page opaque canvas recorded as an accepted limitation (app content, outside the approved chrome surfaces); m7 vendored delta rustfmt-clean + patch regenerated and replayed; m8 startup/app share one `root_glass_bg` helper; m9 pre-existing i18n sibling left untouched (no authority) and recorded
  - rollback_point: c6ded8f — revert `src/`, `Cargo.toml`, `vendor/gpui-component/src/slider.rs`, delete `vendor/patches/slider-accessibility-label.patch` and this plan
  - judge: independent
  - judge_model: deepseek-v4.1-flash
  - proof_gaps: macOS pixels/vibrancy + the clear 6px gutter (headless test platform no-ops `set_background_appearance`); AccessKit disabled flag never set by gpui-pre 0.3.5 (disabled state proven behaviorally, not via the a11y flag); on-disk live-vs-Release split (shared test config dir races); vendored SliderRoot equivalence review-verified, not executed by repo gates; Linux/Windows transparency behaviour unverified on this macOS host

## Current State and Next Action
- active_phase: none
- lifecycle_status: completed
- blockers: none
- open_items: owner-visible GUI check (macOS pixels/vibrancy; the 100% clear 6px gutter should read as the inset frame); proof gaps listed in the Validation entry (headless/engine limits); accepted limitation: Finance/cadence page paints its own opaque content canvas; pre-existing `item.channels`/`item.digest` i18n keys recorded as an out-of-scope follow-up.
- exact_next_action: none (initiative closed; owner GUI check pending, not blocking). If a later session revisits the area, cite the memory artifact instead of this run log.
