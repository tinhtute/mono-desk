---
status: active
lane: high-risk
---
# Zeron stack port: build mono-desk on zeron's own UI code

## Goal
- outcome: mono-desk's window, chrome, sidebar/settings surface and widget
  styling are built from zeron's own MIT-licensed UI source (vendored into
  `vendor/zeron-ui/` and adapted to a settings-only app) running on zeron's
  GPUI stack (`gpui`/`gpui_platform` from `github.com/zeronsh/zui`, pinned
  rev). The gpui-kit/gpui-component stack, its vendor patch set and the
  app-owned custom shell are dropped. The existing settings behavior keeps
  working, re-rendered with zeron's settings widgets: appearance mode,
  accent, font size, glass (translucent window / window opacity / sidebar
  tone / background blur), language, time zone, notifications, profile,
  About, page reset, and persistence.
- success_signal: `make run` opens a client-side-decorated window matching
  zeron's settings screen structure — traffic lights at (14,14), 38px
  titlebar with the control cluster at 88px and a centered title, a
  full-height sidebar column carrying the 5% ink tone + 1px hairline with
  resize (242–400) and a 1px collapsed sliver, a section list nav, a footer
  identity row, and the page column rendered with zeron's widgets. All six
  pages render and round-trip through `AppSettings` + persistence; light/dark
  and glass/opaque modes work; `cargo test && make check && make lint &&
  make fmt-check` are green on the new stack; no `gpui_kit`/`gpui-component`
  reference remains in `Cargo.toml`, `Cargo.lock` or `src/`.
- actors: the owner (visual authority for this demo), developers reading the
  repo for GPUI shell/settings patterns.
- authority: owner decisions in this session (2026-10-28): "Port nguyên code
  zeron" and "drop custom của mình làm theo luôn zeron của họ đi, dùng của
  họ". Reference sources: zeron `github.com/zeronsh/zeron` @ 8b7afb3
  (MIT, Copyright 2026 Wing), cloned at `/tmp/zeron-src`; the pinned UI
  stack is `gpui`/`gpui_platform` from `github.com/zeronsh/zui` @
  b564a344 and zeron's `gpui-base` fork (`github.com/zeronsh/gpui-component`
  @ 59f8924, only if a ported module needs it). Prior requirements lineage:
  `docs/plans/completed/glass-background.md`, `zeron-shell-layout.md`,
  `zeron-sidebar-nav.md`. Repo rules: `AGENTS.md`, `docs/WORKFLOW.md`.
- requirements:
  - R1: The app builds and runs on zeron's GPUI stack only: dependencies
    `gpui` + `gpui_platform` from `zeronsh/zui` @ b564a344 (features
    `font-kit`, `runtime_shaders` on macOS), edition 2024, stable toolchain;
    900×600 minimum / 1320×880 default window, `appears_transparent`
    titlebar, traffic lights at (14,14), `app_owns_titlebar_drag`,
    `window_background` driven by the app's glass switches. No gpui-kit,
    gpui-component or `[patch.crates-io]` entry remains. | acceptance:
    `cargo build` rc 0; `make run` opens a window whose AX frame is
    1320×880 (min enforced); `Cargo.lock` pins the git rev; `grep -r
    gpui_kit src Cargo.toml` empty. | source: owner decision + zeron
    `crates/ui/src/lib.rs` `open_main_window`
  - R2: Window chrome is ported from zeron source, not approximated: 38px
    titlebar strip as an overlay (`Theme::TITLEBAR_HEIGHT`), left control
    cluster starting at 88px with 24px buttons on the traffic-light axis
    (`TITLEBAR_CLUSTER_PAD`/`TITLEBAR_TOP_PAD`), drag region + double-click
    zoom, a centered page title; sidebar column spans the full window height
    with zeron's `wash(0.05)` tone + 1px right hairline, resize clamp
    242–400, collapsed 1px grab sliver (corner-trimmed), ~200ms spring, and
    a footer identity row (avatar initial + display name + theme toggle).
    | acceptance: probe bounds (sidebar column/tone width, toggle at 88px,
    titlebar height 38) match; resize and collapse work; owner visual check.
    | source: zeron `shell.rs` (`render_sidebar`, `sidebar_tone`,
    `render_settings_page`, `render_sidebar_footer_row`, titlebar cluster
    constants)
  - R3: The left column on the settings screen is zeron's settings section
    list (`SettingsSection`-style nav: label + icon rows, selected state,
    hover/selection motion, keyboard-tabbable, "All settings"-equivalent
    header where zeron has one), with our six pages mapped to sections:
    General, Notifications, Profile, Finance UI, Reset, About. Selecting a
    section drives the page pane and the centered title. | acceptance:
    clicking each row switches the page; selected row state visible; keyboard
    Tab/Enter works; AX tree exposes buttons with labels. | source: zeron
    `shell.rs` `render_settings_nav` + `settings.rs` `SettingsSection`
  - R4: Page content is rendered with zeron's settings widgets (`page_column`,
    `page_header`, `section`, `section_card`, `card_row`, `toggle_switch`,
    `select_trigger`, `dropdown`, `block_fill`, `row_divider`,
    `field_label`), not with gpui-component controls. All existing settings
    behavior survives: appearance mode (light/dark/system), accent, font
    size, glass switches + opacity/tone sliders, language, time zone,
    notifications switch + test notification, profile display name,
    About, page reset, persistence keys unchanged. | acceptance: functional
    tests or AX-driven interaction prove each control writes `AppSettings`
    and persists; the persisted file round-trips; owner visual check.
    | source: zeron `settings/widgets.rs` + `src/settings.rs` pages
  - R5: Theme, typography, icons and frost come from zeron's code: zeron-theme
    token crate (vendored), `theme.rs` light/dark/accent surface, the app's
    Momo Trust Sans font stack (kept from the glass initiative), zeron's
    Solar Icons subset (CC BY 4.0 attribution kept) and the frost/glass
    window surfaces ported from `frost.rs`/`glass.rs`. | acceptance:
    appearance toggle repaints without restart; glass on/off changes the
    window background appearance; icons load; third-party notices present.
    | source: zeron `crates/theme`, `crates/ui/src/{theme,typography,icons,
    frost,glass}.rs`
  - R6: Repo hygiene and licensing: zeron-derived files live under
    `vendor/zeron-ui/` with the MIT notice and a `THIRD_PARTY_NOTICES.md`
    entry (zeron MIT; Solar Icons CC BY 4.0; zui/zed gpui license);
    `vendor/gpui-component/`, `vendor/patches/`, the gpui-kit dependency and
    their tests are deleted; docs that describe the old stack (AGENTS.md
    commands/deps, README, docs/memory) are updated. | acceptance:
    `grep -r "gpui-component\|vendor/patches" Cargo.toml src vendor/zeron-ui`
    shows only notices; `make check/lint/fmt-check` green; old test suite
    ported or replaced with equivalent coverage.
  - R7: Phase discipline: each phase gets a `checked` gate on the new stack,
    the owner gets a visual checkpoint after chrome and after the settings
    surface, and the final phase runs an independent `check full` before
    `handoff`.
- non-goals:
  - NG1: No zeron chat/session/engine/network features: no sessions,
    composer, transcript, terminal, browser, pickers, voice, pull requests,
    appshots or cloud/auth state. mono-desk stays a local settings app.
  - NG2: No zeron settings pages that do not map to mono-desk (Accounts,
    Billing, Devices, Cloud, Updates, Harnesses, Worktrees, Wallpaper,
    Shortcuts, Archived, Cloud); only the settings *scaffolding/widgets* and
    our six pages.
  - NG3: No Linux/Windows caption-button chrome in this port (macOS first);
    `window_decorations`/caption code is out unless trivially required to
    compile.
  - NG4: No new product features or persistence keys; behavior parity only.
  - NG5: No zeron app identity (brand, onboarding, update, menus) and no
    network dependencies at runtime.
- lane: high-risk — the work replaces the dependency stack, vendors external
  source, spans multiple phases/sessions and has visual acceptance.

## Phases and Verification
- approach: port zeron's code in dependency order, keeping the app runnable
  after every phase. First bring the window up on zeron's GPUI stack
  (Phase 1, validates the biggest unknown). Then port the visual foundation
  (theme/typography/icons/frost/motion) so ported chrome code compiles
  against it (Phase 2). Then port the chrome and settings section list
  (Phase 3), then the settings page surface and our page content (Phase 4),
  then cleanup/tests/docs and the final gates (Phase 5). Rejected
  alternative: depend on `zeron-ui`/the whole zeron workspace directly — the
  crate pulls 19 zeron domain crates and the engine/network stack, and its
  `Shell` reads `AppState`/sessions, so it cannot render a settings-only app
  without porting the engine too.
- constraints: zeron source is MIT — every copied file keeps its copyright/
  notice in `THIRD_PARTY_NOTICES.md`; copied modules may be adapted (imports
  renamed, domain state replaced by `AppSettings`) but their layout/animation
  constants and widget call shape stay; the pinned git rev is authoritative
  and must be recorded in `Cargo.lock`; no new runtime network calls.
- risks: (a) zeron's `zui` rev may need platform features or patches that
  only exist in zeron's workspace — mitigated by Phase 1 bring-up before any
  port; stop and report if the pinned rev cannot build on stable. (b)
  Copied modules carry domain imports (`crate::state`, sessions) — mitigated
  by porting bottom-up and replacing state with a small app state; each phase
  must compile. (c) Visual fidelity can only be checked by the owner (no
  screenshot pipeline for other apps' windows) — mitigated by bounding
  probes + owner checkpoints. (d) Test coverage on the new stack is unknown
  (zui fork's test-support API may differ) — mitigated by probing test
  support in Phase 1/2 and porting behavior tests by Phase 5. (e) Build
  time/disk (new dep tree) — 246 GiB free was measured; keep the old
  `wip/gpui-kit-shell` branch for rollback.
- recovery: work happens on `feat/zeron-stack` (baseline commit 50823e3,
  parent `c6ded8f`); the previous approach is preserved on
  `wip/gpui-kit-shell`. To abandon this initiative: `git switch main`,
  `git branch -D feat/zeron-stack`; to roll back a phase: `git checkout
  <last-good-commit> -- .` on the branch.
- phase_slug: `stack-bringup`
  status: checked
  - goal: R1 | depends_on: none
  - surfaces: `Cargo.toml`, `Cargo.lock`, `src/` (replaced with the new
    bootstrap), `Makefile` (only if the build needs it) | avoided:
    `vendor/`, `docs/`, `assets/`
  - escalate_when: the pinned `zeronsh/zui` rev fails to build on the
    installed stable toolchain, or `gpui_platform` cannot create a window.
  - wave 1:
    - T1 — replace the dependency graph with `gpui` + `gpui_platform`
      (pinned git rev/features), delete the gpui-kit dependency/patch
      entries, and record the resolution in `Cargo.lock`. — output:
      `cargo build` compiles the new stack. — check: `cargo build`. —
      stop_if: the pinned rev cannot resolve/build.
    - T2 — replace `src/main.rs` (and drop the old `src/*` modules) with a
      zeron-style bootstrap: `gpui_platform::application()`, `open_window`
      with zeron's options from `crates/ui/src/lib.rs` (900×600 min,
      1320×880 default, transparent titlebar, lights 14/14,
      `app_owns_titlebar_drag`, glass-driven `window_background`), a root
      view painting the theme base. — output: `make run` opens the window;
      AX probe reports the expected frame. — check: `make run` + AX frame
      probe. — stop_if: window cannot open or options are unsupported.
    - T3 — delete `vendor/gpui-component/`, `vendor/patches/`, the old view
      tests/helpers and any gpui-kit references. — output: repo contains no
      gpui-kit artifacts. — check: `grep -r "gpui_kit\|gpui-component" src
      Cargo.toml Cargo.lock`. — stop_if: a still-needed helper blocks
      deletion (record and keep only that file).
  - phase check: `cargo build` + `make run` window/AX probe (recording the
    AX frame and the window id).
- phase_slug: `theme-foundation`
  status: in-progress
  - goal: R5 | depends_on: `stack-bringup`
  - surfaces: `vendor/zeron-theme/`, `vendor/zeron-ui/theme.rs`,
    `typography.rs`, `icons.rs`, `motion.rs`, `frost.rs`, `glass.rs`,
    `vendor/zeron-ui/assets/icons/`, `src/` bootstrap, `Cargo.toml`,
    `THIRD_PARTY_NOTICES.md` | avoided: `vendor/zeron-ui/shell*`,
    `settings*`, `assets/brand/`
  - escalate_when: a copied module's dependency closure grows into zeron
    domain crates that cannot be stubbed; or fonts/icons cannot register
    without zeron's asset server.
  - wave 1:
    - T1 — vendor `zeron-theme` and port `theme.rs` + `typography.rs` so
      light/dark/accent surfaces resolve and the Momo Trust Sans stack
      registers. — output: theme globals initialize; app paints theme base.
      — check: `cargo build` + run. — stop_if: theme depends on app
      settings types that are not present.
    - T2 — vendor the icon SVG subset + `icons.rs` asset source and port
      `motion.rs`/`frost.rs`/`glass.rs`. — output: icons render; frost
      surface available. — check: `cargo build` + run; notice file updated.
      — stop_if: asset source cannot be registered with the platform app.
  - phase check: `cargo build`, `make run`, and an AX/probe check that the
    theme base is painted in both appearances.
- phase_slug: `chrome`
  status: planned
  - goal: R2, R3 | depends_on: `theme-foundation`
  - surfaces: `vendor/zeron-ui/` (titlebar cluster, sidebar column/tone/
    resize/collapse, settings nav, footer identity), `src/shell.rs` (app
    wiring + minimal settings state), `src/main.rs` | avoided: settings
    page content
  - escalate_when: ported chrome needs state/geometry data only the zeron
    app has (sessions/routes/spaces); or the resize/collapse animation cannot
    be reproduced with the fork's animation APIs.
  - wave 1:
    - T1 — port the titlebar cluster (88px start, 24px toggle, drag region,
      centered title) and wire the sidebar collapse toggle. — output:
      titlebar matches zeron geometry; toggle collapses the rail. — check:
      AX probe (toggle x=88, titlebar 38) + run. — stop_if: drag region or
      title layout needs window APIs the fork lacks.
    - T2 — port the sidebar column/tone/hairline, 242–400 resize, 1px
      collapsed sliver and footer identity row; add the section-list nav
      (our six pages) with selection/hover motion. — output: sidebar chrome
      and nav work; selecting a row switches the page title. — check: AX
      probe (column width, tone edge, section buttons) + run. — stop_if: the
      nav cannot be driven from app state.
  - phase check: `cargo build` + run + AX probe for titlebar/sidebar/nav;
  owner visual checkpoint.
- phase_slug: `settings-surface`
  status: planned
  - goal: R4 | depends_on: `chrome`
  - surfaces: `vendor/zeron-ui/settings/widgets.rs` (ported helpers),
    `src/settings/` (our six pages), `src/app_settings.rs`/`persist.rs`
    (ported), `src/main.rs` | avoided: zeron settings pages that are NG2
  - escalate_when: a ported widget requires popover/search infrastructure
    whose port would exceed the phase, or a control has no zeron widget
    equivalent.
  - wave 1:
    - T1 — port the settings page scaffolding (`page_column`, `page_header`,
      `section`, `section_card`, `card_row`, labels/dividers) and rebuild
      General + Notifications pages on it with our settings state and
      persistence. — output: two pages render and controls write settings.
      — check: `cargo build` + run + settings file round-trip. — stop_if:
      the page layout needs zeron's search/route infrastructure.
    - T2 — port toggle/select/dropdown/slider widgets and rebuild the
      remaining pages (Profile, Finance UI, Reset, About), including the
      glass controls and test notification. — output: all six pages
      functional. — check: run + settings round-trip + i18n strings.
      — stop_if: a needed control has no ported primitive.
  - phase check: `cargo build` + run + settings file round-trip; owner
  visual checkpoint.
- phase_slug: `fidelity-close`
  status: planned
  - goal: R6, R7 | depends_on: `settings-surface`
  - surfaces: `vendor/zeron-ui/` (cleanup), `Cargo.toml`, `Cargo.lock`,
    `Makefile`, `AGENTS.md`, `README.md`, `docs/memory/`, `THIRD_PARTY_NOTICES.md`,
    test modules | avoided: `assets/brand/`
  - escalate_when: ported test harness lacks the capabilities the old view
    tests used; or the independent gate finds layout/behavior gaps.
  - wave 1:
    - T1 — port/replace the test suite on the new stack (unit tests for
      settings/persist/theme + behavior tests where the fork's test support
      allows), and delete the old test modules. — output: `cargo test`
      covers settings state and persists. — check: `cargo test`. —
      stop_if: the fork exposes no test harness for windowed behavior.
    - T2 — hygiene: MIT/third-party notices, AGENTS/README/docs updates,
      remove dead vendor files, update memory entries. — output: no
      gpui-kit/gpui-component leftovers; docs describe the new stack. —
      check: `grep` sweep + `make check`. — stop_if: a doc claim cannot be
      verified.
  - phase check: `cargo test && make check && make lint && make fmt-check`
  then independent `check full` and `handoff`.

## Log

- 2026-10-28T14:52Z — stack-bringup — start — owner approved porting zeron's
  own UI code after the gpui-kit-based attempts were dropped; recon
  established the pinned stack (`zeronsh/zui` @ b564a344) builds and opens a
  window on this machine, and that zeron's settings/widgets chrome is
  separable from its engine/state. Previous approach preserved on
  `wip/gpui-kit-shell` (commit 50823e3); work continues on
  `feat/zeron-stack`.
- 2026-10-28T14:56Z — stack-bringup/T1 — done — dependency graph swapped to
  `gpui`/`gpui_platform` from `zeronsh/zui` @ b564a344 (features font-kit,
  runtime_shaders); gpui-kit, its dev-dep and `[patch.crates-io]` removed. A
  stale `Cargo.lock` pinned `cocoa-foundation 0.2.1` against gpui's `=0.2.0`
  pin and cargo failed to select a version; deleting the lock let cargo
  resolve like zeron's workspace. Check: `cargo build` rc 0; `Cargo.lock`
  records the git source/rev for `gpui 0.2.2` and `gpui_platform 0.1.0`.
  Changed: `Cargo.toml`, `Cargo.lock`.
- 2026-10-28T14:56Z — stack-bringup/T2 — done — zeron-style bootstrap in
  `src/main.rs`: `gpui_platform::application()`, `WindowOptions` copied from
  zeron `open_main_window` (1320×880 default, 900×600 min, transparent
  titlebar, lights 14/14, `app_owns_titlebar_drag`, app id `mono-desk`),
  placeholder root view. Check: `cargo run`; CGWindow layer 0 alpha 1
  bounds 1320×880 at (120,120), window id 174702. Changed: `src/main.rs`.
- 2026-10-28T14:56Z — stack-bringup/T3 — done — old stack deleted:
  `src/*` (12 modules + `cadence/`), `vendor/gpui-component/` and
  `vendor/patches/` removed. Check: `grep -r "gpui_kit\|gpui-kit\|
  gpui-component\|vendor/patches" src Cargo.toml Cargo.lock` → empty.
- 2026-10-28T14:56Z — stack-bringup — decision — the zui-fork window exposes
  no AX window tree to our probe (`AXUIElementCopyAttributeValue` windows =
  0) although `CGWindowListCopyWindowInfo` sees it; later phases must not
  depend on AX bounds probes (use CGWindow bounds + zeron/gpui test support +
  owner visual checks). Recorded under risks.
- 2026-10-28T15:04Z — stack-bringup — decision — correction to the earlier entry: the
  zui-fork window DOES expose one AX window (frame 1320×880, and an AX size
  write of 400×300 reads back 900×600, so the minimum is enforced) — the
  earlier "windows = 0" observation was a probe-state error. The remaining
  caveat is narrower and real: the AX *content* tree exposes only
  AXWindow/AXGroup plus the three native traffic-light buttons, not
  GPUI-painted nodes, so Phase 3 acceptance needs explicit role/aria labels
  and an early probe. Also recorded here: `src/main.rs` still hardcodes
  `WindowBackgroundAppearance::Opaque` and a placeholder base colour (T2/R1
  ask for the theme/glass-driven background); deferred to `theme-foundation`
  where the theme global exists, and it must land before the glass
  acceptance.
- 2026-10-28T15:04Z — theme-foundation — start — beginning the visual foundation port. The
  vendored zeron-theme domain crate is already in the tree and compiles as a
  path dependency; next: port `theme.rs` (tokens/appearance global),
  `typography.rs` (Momo Trust Sans registration), icons + Solar SVG subset,
  and motion/frost/glass, then wire the theme-driven `window_background`.

## Validation

- 2026-10-28T15:04Z — phase `stack-bringup` — verdict: APPROVED — mode: gate, independent judge (high-risk lane)
  - judge: independent subagent (separate context); judge_model: deepseek-v4.1-flash
  - `cargo build` / `cargo build --locked` — rc 0 — pass
  - `cargo run` (~7s): CGWindow layer 0 alpha 1 onscreen 1, bounds
    1320×880 at (120,120); direct AX probe: 1 window, frame 1320×880; AX
    resize 400×300 reads back 900×600 (minimum enforced); 3 native
    traffic-light buttons at relative x 13/33/53, y 14 — pass
  - `grep -rn "gpui_kit\|gpui-kit\|gpui-component\|vendor/patches" src Cargo.toml
    Cargo.lock` — no matches — pass
  - `make check` rc 0; `make lint` rc 0; `make fmt-check` rc 0; `cargo test`
    rc 0 (0 tests; Phase 5 owns coverage) — pass
  - surfaces: `src/` only `main.rs`; `vendor/` absent — pass
  - findings: (1) minor/deferred — hardcoded `WindowBackgroundAppearance::Opaque`
    + placeholder base; deferred to `theme-foundation`, must land before the
    glass acceptance. (2) minor — the earlier Log "AX windows=0" claim was
    wrong (direct AX probe sees the window); corrected above, content-tree
    caveat carried into Phase 3.
  - scope: on target (`Cargo.toml`, `Cargo.lock`, `src/main.rs`, plan/memory/
    notices; deletions of `vendor/` + old `src/` modules).
  - rollback_point: `feat/zeron-stack` commit `94ff732`; previous approach on
    `wip/gpui-kit-shell` (`50823e3`).


## Current State and Next Action
- active_phase: theme-foundation
- lifecycle_status: in-progress
- blockers: none
- open_items: port theme/typography/icons/motion/frost/glass and wire the
  theme-driven `window_background` (deferred gate finding 1, must land before
  the glass acceptance); Phase 3 AX content-tree caveat (explicit role/aria
  labels + early probe); AGENTS.md updated to the new stack this session;
  README/docs/design/memory cleanup is Phase 5 (R6).
- exact_next_action: work full phase `theme-foundation` — T1 port `theme.rs` +
  `typography.rs` on top of the vendored `zeron-theme` crate (window background
  from the theme), T2 vendor the icon SVG subset + `motion.rs`/`frost.rs`/
  `glass.rs`.
