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
  zeron's *Settings* screen structure (the app's only surface, owner decision
  2026-10-29): traffic lights at (14,14), a 38px overlay strip that drags the
  window (zeron's settings route paints no control cluster and no centered
  title there), the shell root painting `theme.glass()` over the blurred
  window background, a full-height sidebar column at 256px over zeron's
  `wash(0.05)` tone + 1px right hairline, the settings search row, the
  six-section nav and the footer identity row, and the page column rendered
  with zeron's widgets. All six pages render and round-trip through
  `AppSettings` + persistence; light/dark and glass/opaque modes work;
  `cargo test && make check && make lint && make fmt-check` are green on the
  new stack; no `gpui_kit`/`gpui-component` reference remains in
  `Cargo.toml`, `Cargo.lock` or `src/`.
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
  - R2: Window chrome is ported from zeron source, not approximated. The app
    has a single surface — zeron's Settings route — so the chrome is that
    route's chrome, literally (owner decision 2026-10-29, see Log): the shell
    root paints `theme.glass()` with the theme text/font and the 14px rem
    scale, a full-width 38px overlay strip drags the window
    (`WindowControlArea::Drag` + `start_window_move`, double-click zoom) and
    clears the native traffic lights, and the sidebar column spans the window
    height at 256px over zeron's `sidebar_tone` (`wash(0.05)` + 1px right
    hairline). zeron's chat-only chrome is *not* ported: there is no titlebar
    control cluster/toggle, no centered title, and no sidebar resize
    (242–400) or collapsed 1px sliver, because zeron's settings route paints
    none of them. | acceptance: `MONO_DESK_SELFTEST` reports glass alpha < 1
    with the `Blurred` window background plus the sidebar tone/hairline and
    the ported titlebar-clearance numbers; owner visual check. | source:
    zeron `shell.rs` (`Theme::glass` root surface, `sidebar_tone`,
    `titlebar_drag_region`, `titlebar_cluster_start`/`cluster_clearance`,
    `render_settings_page`)
  - R3: The left column on the settings screen is zeron's settings section
    list (`SettingsSection`-style nav: label + icon rows, selected state,
    hover/selection motion, keyboard-tabbable) over zeron's search row, with
    our six pages mapped to sections: General, Notifications, Profile,
    Finance UI, Reset, About. Selecting a section drives the page pane.
    zeron's "Back" row is dropped (this app has no chat surface to return to,
    owner decision 2026-10-29), so the sidebar starts with the search row.
    The search row's *input* primitive arrives with the `settings-surface`
    phase (zeron's field is the chat composer input, which is out of scope);
    until then the column starts with the section list. | acceptance:
    clicking each row switches the page; selected row state visible; keyboard
    Tab/Enter works; AX tree exposes rows with role/label. | source: zeron
    `shell.rs` `render_settings_nav` + `settings.rs` `SettingsSection`,
    `shell/settings_search.rs` field layout
  - R4: Page content is rendered with zeron's settings widgets (`page_column`,
    `page_header`, `section`, `section_card`, `card_row`, `toggle_switch`,
    `select_trigger`, `dropdown`, `block_fill`, `row_divider`,
    `field_label`), not with gpui-component controls. All existing settings
    behavior survives: appearance mode (light/dark/system), accent, font
    size, glass switches + opacity/tone sliders, language, time zone,
    notifications switch + test notification, profile display name,
    About, page reset, persistence keys unchanged. The Appearance page is
    rebuilt as zeron's *custom* Appearance page (`settings/appearance.rs`),
    not a generic form: System/Light/Dark option cards with previews, light
    and dark theme-variant selectors, the accent swatch grid, the "Glass"
    surface select (Opaque/Frosted), and the fonts/layout rows — wallpaper and
    cloud-only rows dropped as NG2. | acceptance: functional tests or
    AX-driven interaction prove each control writes `AppSettings` and
    persists; the persisted file round-trips; owner visual check against
    zeron's Appearance page. | source: zeron `settings/widgets.rs`,
    `settings/appearance.rs`, `src/settings.rs` pages
  - R5: Theme, typography, icons and frost come from zeron's code: zeron-theme
    token crate (vendored), `theme.rs` light/dark/accent surface, the app's
    Momo Trust Sans font stack (kept from the glass initiative), zeron's
    Solar Icons subset (CC BY 4.0 attribution kept) and the frost/glass
    window surfaces ported from `frost.rs`/`glass.rs`. | acceptance:
    appearance toggle repaints without restart; glass on/off changes the
    window background appearance (`Blurred` under Frosted, `Opaque` under
    Opaque); the shell's translucent surfaces (frost layer + backdrop blur on
    the floating chrome, sidebar tone/hairlines over it) match zeron's recipe
    rather than approximating it; icons load; third-party notices present.
    | source: zeron `crates/theme`, `crates/ui/src/{theme,typography,icons,
    frost,glass,shell}.rs`
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
  - R8 (owner directive 2026-10-29, "layout & background, translucent phải
    giống cái zeron… có custom UI Appearance y hệt đó"): visual fidelity is
    literal for (a) the shell layout, (b) the window background / translucent
    surfaces, and (c) the custom Appearance page. Where zeron's code paints a
    frost layer, a backdrop blur, a sidebar tone or a page header/card
    structure, the port uses that code and those constants instead of an
    equivalent hand-rolled look. | acceptance: the ported surface cites the
    zeron source file/function it came from in a comment, the constants match
    (no local approximations), and the owner's visual check at each phase
    checkpoint confirms parity. | source: owner message this session; zeron
    `shell.rs`, `frost.rs`, `glass.rs`, `settings/appearance.rs`
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
  status: checked
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
  status: in-progress
  - goal: R2, R3, R5 (translucent parity) | depends_on: `theme-foundation`
  - surfaces: `vendor/zeron-ui/chrome.rs` (ported titlebar geometry +
    drag-strip/sidebar-tone helpers), `vendor/zeron-ui/settings/` (nav row +
    footer identity), `src/shell.rs` (app wiring + section state),
    `src/main.rs` | avoided: settings page content
  - escalate_when: ported chrome needs state/geometry data only the zeron
    app has (sessions/routes/spaces); or zeron's settings route turns out to
    include chrome we cannot stand up without the chat surface.
  - wave 1:
    - T1 — port the shell surface and window chrome: root
      (`id("shell-root")`, `.relative().flex().flex_row().size_full()`,
      `.bg(theme.glass())`, theme text/font, 14px rem scale), the 38px
      full-width drag strip (`WindowControlArea::Drag` + `start_window_move`,
      double-click zoom, `Theme::TITLEBAR_HEIGHT`), zeron's `sidebar_tone`
      layer (`wash(0.05)` + 1px right hairline, 256px), the sidebar column and
      the page pane with the titlebar clearance — with the ported geometry
      helpers/tests in `vendor/zeron-ui/chrome.rs`. — output: window chrome
      matches zeron's settings route; no opaque fill over the `Blurred`
      window background. — check: `cargo test` (ported geometry tests) +
      `MONO_DESK_SELFTEST` (glass alpha, tone alpha, drag strip height) +
      run/CGWindow probe. — stop_if: the drag strip or glass root needs
      window APIs the fork lacks.
    - T2 — port the settings section list: six rows from our `Section` enum
      using zeron's `widgets::section_tab` + `tab_selection_t` (icon + label,
      selected wash, hover/selection motion, `Role::Tab`/`aria_selected`,
      keyboard up/down/home/end) over the footer identity row
      (`render_sidebar_footer_row`: avatar initial + display name), and drive
      the page pane from the selection. zeron's search row (sidebar top) lands
      in `settings-surface` with the text-input primitive; the Back row is
      dropped (owner decision). — output: nav works and switches the pane;
      sidebar matches zeron's settings column. — check: run + keyboard
      selection + AX role/probe. — stop_if: the nav cannot be driven from app
      state.
  - phase check: `cargo build` + run + `MONO_DESK_SELFTEST` translucent
    parity probe (window background `Blurred` + shell root painting
    `theme.glass()`, sidebar tone from `wash(0.05)`), then owner visual
    checkpoint.
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
    - T3 — rebuild Appearance as zeron's custom page: port
      `settings/appearance.rs`'s structure (`option_card` mode cards with
      previews, theme-variant rows, `accent_swatch` grid, "Glass" surface
      select, fonts/layout rows) plus its `widgets::option_card`/
      `accent_swatch` primitives, wiring mode/accent/surface/font-size to
      `AppSettings` and persistence; drop wallpaper/cloud rows (NG2). —
      output: Appearance matches zeron's page structure and behaviour
      (glass select switches the native window background live). — check:
      run + settings round-trip + `MONO_DESK_SELFTEST` on both surface
      preferences + owner visual. — stop_if: the page needs zeron
      wallpaper/theme-library infrastructure beyond the vendored theme
      registry.
    - T4 — port zeron's settings search row into the sidebar
      (`shell/settings_search.rs` layout: 32px field, rounded 8, magnifier,
      "Search settings" placeholder, shortcut hint) on a text-input
      primitive adapted from the zui fork's `examples/input.rs`
      (Apache-2.0), filtering the six section rows; it replaces the Back row
      the owner dropped, so the sidebar starts with it. — output: search
      filters the section list; typing/escape/clear work. — check: run +
      keyboard round-trip + AX probe. — stop_if: the fork's input handler
      cannot be adapted without porting the chat composer.
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

- 2026-10-29T12:05Z — theme-foundation/T1 — done — vendored zeron's visual
  foundation into a new workspace member `vendor/zeron-ui/` (adapted from
  `crates/ui` @ 8b7afb3, MIT): `theme.rs` (tokens/appearance global; local
  `highlight.rs` replaces the `zeron_syntax::HighlightKind` import, wallpaper
  tinting and the `gpui_base` scrollbar sync dropped), `typography.rs`
  (rewritten around the bundled families — Momo Trust Sans, Hanken Grotesk,
  Google Sans Code, Xanh Mono — with zeron's font-inspection and settings
  writes dropped), and the bootstrap wiring in `src/main.rs`
  (`with_assets(icons::Assets)`, `register_fonts` + `typography::init`,
  `Theme::install_preferences` before `open_window`, window background from the
  theme, appearance observer re-syncing the native window background). This
  lands gate finding 1 from `stack-bringup`: the window background is no
  longer hardcoded (`Blurred` under zeron's default frosted surface) and the
  base colour comes from `Theme::bg`. Check: `cargo check` rc 0;
  `MONO_DESK_SELFTEST=1 MONO_DESK_APPEARANCE=light|dark` prints Light
  (`bg` white, `surface` 0.958, `text` 0.131, indigo accent) / Dark (`bg`
  0.024, `surface` 0.051, `text` 0.898) with all four families registered and
  `window_background=Blurred frost=true`.
- 2026-10-29T12:10Z — theme-foundation/T2 — done — icon registry + the full
  128-SVG Solar set (`assets/icons/`, zeron's `Assets` source minus the
  file-icon theme), `motion.rs` adapted (zeron_proto's pure phase math vendored
  as `loader_math.rs`; OS reduce-motion detection and settings writes dropped),
  `frost.rs` and `glass.rs` copied verbatim. Check: `icons::Assets.load(SUN)`
  returns SVG bytes in the selftest; `cargo test` runs 26 unit tests
  (24 `vendor/zeron-theme`, 2 `vendor/zeron-ui`); `make check`/`lint`/
  `fmt-check` rc 0; app window still 1320×880 at (120,120) (CGWindow probe) and
  stays up.
- 2026-10-29T12:12Z — theme-foundation — decision — the vendored crates are
  workspace `default-members` so plain `cargo test` exercises them (this
  required restoring `tempfile` as a `zeron-theme` dev-dependency), while
  `make lint` and `make fmt`/`fmt-check` are scoped to `-p mono-desk`: clippy's
  newer style lints fire on upstream files and `cargo fmt` would rewrite them
  away from the pinned rev, so each vendor crate carries
  `#![allow(clippy::all)]` and keeps upstream formatting. Recorded in
  `AGENTS.md` and `docs/memory/zeron-stack-port.md`.
- 2026-10-29T12:13Z — theme-foundation — decision — added two dev-only review
  hooks in `src/main.rs` (`MONO_DESK_APPEARANCE`, `MONO_DESK_SELFTEST`): the
  phase check needs machine-checkable evidence of the painted theme, and the
  alternatives are unavailable — macOS Screen Recording is not granted to
  agent tooling and `Window::render_to_image()` is behind gpui's
  `test-support` feature with no macOS platform implementation.

- 2026-10-29T12:30Z — theme-foundation — decision (owner directive) — owner
  asked that the port remember "layout & background, translucent phải giống
  cái zeron… có custom UI Appearance y hệt đó". Recorded as R8 and as an
  explicit scope amendment: R4's Appearance page is zeron's *custom* page
  (`settings/appearance.rs`: mode option cards with previews, light/dark
  theme-variant rows, accent swatch grid, "Glass" surface select, fonts/layout
  rows), `settings-surface` gains task T3 to port it, and R5/the `chrome`
  phase check now require the translucent recipe itself (frost layer +
  backdrop blur, `Blurred` window background under Frosted) rather than a
  visually similar approximation. Recon: `settings/appearance.rs` is ~5k lines
  and depends on zeron's wallpaper/theme-library rows; the port keeps the
  structure and drops wallpaper/cloud rows per NG2.

- 2026-10-29T12:45Z — theme-foundation — gate — verdict REJECTED (independent
  judge; 1 blocking, 4 minor, 2 deferred — everything else re-ran green).
  Blocking: the probe's swatch caption used a hardcoded `hsla(0,0,0.45,1)`
  instead of a theme token, contradicting the phase's "tokens, not hardcoded
  colours" claim; fixed to `theme.text_faint`. Minors fixed: stale
  `MONO_DESK_CAPTURE` doc comment (no such hook exists), `icons.rs` doc still
  claiming the file-identity icon theme, `motion.rs` doc claiming OS
  reduce-motion detection (now a documented no-op), and the `Cargo.toml`
  comment claiming `make lint` covers the vendored crates. Deferred items were
  fixed now rather than carried: the unvendored Geist defaults (including
  `font_sans_fixed`, which no builder set) now map to the bundled families via
  `BUNDLED_FONT_*` constants plus a `with_font_sans_fixed` builder, and
  upstream's "Bundled theme palette adaptations" attributions were carried
  into `THIRD_PARTY_NOTICES.md`. Scope-drift note accepted: the Makefile gate
  scoping (lint/fmt `-p mono-desk`) changed in this phase although it is not in
  the phase's declared surface list — it is a consequence of the vendored
  crates becoming workspace members in the same phase and is recorded in the
  decision entry above.

- 2026-10-29T12:26Z — theme-foundation — correction — the two Log timestamps
  above are ahead of the clock and are corrected here because the Log is
  append-only (nothing was rewritten): the gate entry recorded `12:45Z` and the
  owner-directive entry `12:30Z`, whereas the gated commit `105a984` is
  authored `12:20:46Z` and the wall clock at judgment was `12:24Z` (`bdd6680`
  is authored `12:13:22Z`). The content of both entries is accurate; only the
  timestamps were wrong, because they were written from an estimate instead of
  the clock. Timestamps below are taken from `date -u` / `git log`.
- 2026-10-29T12:26Z — theme-foundation — gate (re-gate) — verdict APPROVED by
  a fresh independent judge (the first judge could not be re-addressed): the
  blocking hardcoded-colour fix is real, all seven claimed fixes verify in the
  tree, every gate re-ran green, no regression. The judge independently
  re-confirmed `frost.rs`/`glass.rs` diff-identical to upstream and 128/128
  icon SVGs identical, and that the Log stayed append-only across
  `8c3ad7a`→HEAD. Two non-blocking minors, both fixed before this entry: the
  future-dated Log timestamps (corrected above rather than rewritten) and a
  stale `vendor/zeron-theme/Cargo.toml` comment claiming `tempfile` was
  dropped, while it is a dev-dependency for the ported tests. Pixels for this
  phase are unverifiable by agent tooling and were not claimed by the judge;
  the phase check rests on the selftest tokens, asset/background probes and
  the owner's eye.

- 2026-10-29T12:36Z — chrome — decision (owner choice) — the app has exactly
  one surface, and zeron's *Settings* route paints none of the chrome R2/R3
  described (the 88px control cluster, the sidebar collapse, the 242–400
  resize, the collapsed sliver and the centered title are all chat-route
  chrome). Asked the owner; they chose "copy y màn Settings của zeron" and,
  for zeron's `Back` row (which returns to the chat this app does not have),
  "bỏ nút Back, sidebar bắt đầu bằng ô search". Recorded as R2/R3 amendments
  and as a rewrite of the phase tasks: T1 = shell surface + drag strip +
  sidebar tone/column + page clearance; T2 = section list + footer identity
  row; the search row moves to `settings-surface` T4, because zeron's field is
  the chat composer input and the port needs a text-input primitive first.
  Dropped from the phase: titlebar cluster/toggle, collapse, resize, centered
  title (they never appear on the settings screen the owner asked to match).
- 2026-10-29T12:36Z — chrome/T1 — done — vendored `chrome.rs` (ported titlebar
  geometry, `window_control_button`/`_with`, and five upstream tests) and
  added `src/shell.rs`, the first-party shell: root `id("shell-root")` row
  layout with `bg(theme.glass())`, theme text/font and `ui_rems(14)`;
  `sidebar_tone` layer at 256px (`wash(0.05)` + 1px `theme.border` right
  hairline); the settings sidebar column and page pane with the in-scroll
  `TITLEBAR_HEIGHT + SPACE_LG` clearance; zeron's `titlebar_drag_region`
  (`WindowControlArea::Drag`, `start_window_move` behind the `pressed_button`
  guard, double-click `titlebar_double_click`). `src/main.rs` now opens `Shell`
  instead of the phase-2 probe view and the selftest reports the shell
  surface. Check: `cargo test` 31 tests pass (24 `zeron-theme`, 7
  `zeron-ui`, including the ported geometry tests); `make check` /
  `make lint` / `make fmt-check` rc 0; selftest light
  `glass a=0.80 is_glass=true tone l=0.10 a=0.05 border a=0.10 sidebar=256`,
  dark `glass a=0.80 tone l=0.92 a=0.05`, both with
  `window_background=Blurred frost=true` and
  `chrome titlebar=38 top_pad=4 cluster_start=88 spacer=78 clearance=162`;
  runtime probes (light and dark) show the window at 1320×880 at (120,120),
  no panic, process exits cleanly. Pixels are the owner's check.

- 2026-10-29T12:43Z — chrome/T2 — done — ported the section list and the
  footer identity row. New `vendor/zeron-ui/src/settings/`: `mod.rs` with the
  six-section `SettingsSection` (labels/icons/`starts_nav_group` following
  zeron's enum conventions) and `widgets.rs` with `section_tab`,
  `tab_selection_t` and `TabSelectionTravel` ported from zeron's
  `settings/widgets.rs`; the shell side adds `render_nav` (zeron's
  `render_settings_nav`: `Role::TabList` + row `Role::Tab`/`aria_selected`,
  roving `tab_stop` handles, accent focus ring, up/left/down/right/home/end,
  spacing-only group break at Finance UI) and `render_footer` (zeron's
  `render_sidebar_footer_row` metrics and user-menu trigger pill, identity
  only — no account popover; the footer's square action is the appearance
  toggle, which flips the theme and re-points the native window background).
  Added `vendor/zeron-ui/assets/icons/user.svg`, a mono-desk hand-drawn Solar
  Linear glyph for the Profile row (neither the embedded set nor zeron ships
  one), and recorded it in `THIRD_PARTY_NOTICES.md`. Check: `cargo test` 33
  tests (24 zeron-theme, 9 zeron-ui: the 5 ported chrome-geometry tests,
  section-order and selection-travel tests); `make check`/`lint`/`fmt-check`
  rc 0; selftest `nav sections=6
  labels=General|Notifications|Profile|Finance UI|Reset|About
  group_break=Finance UI selected=General`; light and dark runtime probes keep
  the window at 1320×880 at (120,120) with an empty log and a clean exit.
  Click/keyboard/selection motion are the owner's visual check — the AX
  content tree exposes no GPUI nodes and agent tooling has no synthetic-input
  grant, so those are not claimed here.

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

- 2026-10-29T12:26Z — phase `theme-foundation` — verdict: APPROVED — mode: gate, independent judge (high-risk lane)
  - judge: fresh independent subagent (separate context, read-only, no history
    reuse); judge_model: deepseek-v4.1-flash; graded commit `105a984`.
  - `cargo test` rc 0 — 26 tests passed, 0 failed (24 `vendor/zeron-theme`,
    2 `vendor/zeron-ui`), doc-tests 0 — pass
  - `make check` / `make lint` / `make fmt-check` / `make build` /
    `cargo build --locked` — rc 0 — pass
  - selftest light: `appearance=Light`; bg l=1.0, surface l=0.958, text
    l=0.131, indigo accent; four bundled families registered
    (`effective=Momo Trust Sans fixed=Google Sans Code`); `icon_sun_svg=Some(true)`;
    `window_background=Blurred frost=true`; rem=16 — pass
  - selftest dark: `appearance=Dark`; bg l=0.0235, surface l=0.051, text
    l=0.898; same fonts/icon/background lines — pass
  - runtime probe: window layer 0 alpha 1 onscreen 1, bounds 1320×880 at
    (120,120), empty log (no panic), process exits cleanly; a light-pinned run
    reports the same frame — pass
  - `diff -u` `frost.rs` / `glass.rs` against `/tmp/zeron-src` — identical
    (rc 0 both); `diff -rq` icon directory — identical, 128 SVGs each side —
    pass
  - no `zeron_syntax`/`zeron_proto`/`gpui_base` use paths in
    `vendor/zeron-ui/src` (only explanatory doc mentions); no
    `gpui_kit`/`gpui-kit`/`gpui-component`/`vendor/patches` in first-party or
    vendored manifests — pass
  - surfaces: `vendor/zeron-theme/**`, `vendor/zeron-ui/{theme,typography,
    icons,motion,frost,glass}.rs` + `assets/icons/`, `src/main.rs`,
    `Cargo.toml`, `Cargo.lock`, `THIRD_PARTY_NOTICES.md`, `AGENTS.md`,
    `docs/memory/` — on target — pass
  - scope: `Makefile` gate scoping changed although outside the declared
    surface list, recorded as accepted drift with its reason — noted, not
    blocking
  - findings: no blocking. Two minors fixed after the verdict (future-dated
    Log timestamps; stale `zeron-theme/Cargo.toml` comment). Deferred/named:
    rendered pixels and the owner visual for this phase are not recorded —
    agent tooling cannot capture this window, so the check rests on
    tokens/assets/background probes; old-stack text in `README.md`,
    `docs/PROJECT.md`, `docs/design/*` and completed plans is Phase 5 (R6);
    `ReduceMotion::System` no-op and the unported file-icon theme are
    documented vendor adaptations; R5's "appearance toggle repaints without
    restart" is not exercisable until chrome/settings.
  - rollback_point: `feat/zeron-stack` commit `bdd6680` (last green commit
    before the gate fixes); previous approach on `wip/gpui-kit-shell`
    (`50823e3`).


## Current State and Next Action
- active_phase: chrome
- lifecycle_status: in-progress
- blockers: none
- open_items: `chrome` T1+T2 are implemented and machine-checked; the phase
  gate (independent judge) and the owner's visual checkpoint are still owed,
  and pixels/synthetic input remain unverifiable by agent tooling (no Screen
  Recording, no AX nodes for GPUI controls, no in-process capture on macOS).
  Deliberately not ported from zeron's chat chrome: titlebar cluster/toggle,
  centered title, sidebar resize and collapsed sliver (R2, owner decision —
  zeron's settings route has none). Still to come in `chrome`'s successor
  phases: zeron's settings search row (needs the text-input primitive, T4 of
  `settings-surface`) and an Appearance section row for the custom Appearance
  page (T3 adds it). `vendor/zeron-ui/` chrome/settings code is compiled and
  tested but not clippy-linted (`make lint` is `-p mono-desk`), so new
  non-verbatim code is reviewed by hand; `ReduceMotion::System` stays a no-op
  with chrome animations; `font_sans_fixed` resolves to the bundled Google
  Sans Code.
- exact_next_action: independent phase gate for `chrome` (T1+T2) against the
  current tree, then mark the phase `checked` and start `settings-surface`:
  T1 port `settings/widgets.rs` page scaffolding (`page_column`,
  `page_header`, `section`, `section_card`, `card_row`, labels/dividers) and
  rebuild General + Notifications on `AppSettings`/persistence; T2 the
  remaining pages; T3 the custom Appearance page + its nav row; T4 the search
  row on a text-input primitive. Ask the owner for the visual checkpoint after
  the phase gate.
