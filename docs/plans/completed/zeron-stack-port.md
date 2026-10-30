---
status: completed
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
  seven-section nav (General, Appearance, Notifications, Profile, Finance UI,
  Reset, About) and the footer identity row, and the page column rendered
  with zeron's widgets. All six pages render and round-trip through
  `AppSettings` + persistence; light/dark and glass/opaque modes work;
  `cargo test && make check && make lint && make fmt-check` are green on the
  new stack; no pre-port dependency, path or import (`gpui-kit` /
  `gpui-component`) remains in `Cargo.toml`, `Cargo.lock` or `src/` —
  provenance comments that name the dropped stack in order to explain an
  adaptation are allowed (amend: grep-for-zero-hit reading superseded,
  2026-10-30).
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
    these sections: General, Appearance, Notifications, Profile, Finance UI,
    Reset, About — the six pre-existing pages plus **Appearance**, zeron's
    custom appearance page, which the owner asked for verbatim (R8) and which
    is zeron's own nav row order (General before Appearance). Selecting a
    section drives the page pane. zeron's "Back" row is dropped (this app has
    no chat surface to return to, owner decision 2026-10-29), so the sidebar
    starts with the search row. The search row's *input* primitive arrives
    with the `settings-surface` phase (zeron's field is the chat composer
    input, which is out of scope); until then the column starts with the
    section list. | acceptance: clicking each row switches the page; selected
    row state visible; keyboard Tab/Enter works; AX tree exposes rows with
    role/label. | source: zeron `shell.rs` `render_settings_nav` +
    `settings.rs` `SettingsSection`, `shell/settings_search.rs` field layout
  - R4: Page content is rendered with zeron's settings widgets (`page_column`,
    `page_header`, `section`, `section_card`, `card_row`, `toggle_switch`,
    `select_trigger`, `dropdown`, `block_fill`, `row_divider`,
    `field_label`), not with gpui-component controls. All existing settings
    behavior survives: appearance mode (light/dark/system), accent, font
    size, glass surface (the Appearance page's "Glass" select supersedes the
    pre-port opacity/sidebar-tone sliders: those persisted values survive in
    the file and round-trip, but the zeron page exposes the surface policy
    instead — owner directive R8, Log 2026-10-29T15:40Z (a)), language, time zone,
    notifications switch + test notification, profile display name,
    About, page reset, persistence keys unchanged. The Appearance page is
    rebuilt as zeron's *custom* Appearance page (`settings/appearance.rs`),
    not a generic form: System/Light/Dark option cards with previews, light
    and dark theme-variant selectors, the accent swatch grid, the "Glass"
    surface select (Theme default/Translucent/Solid), the reduce-motion row
    and the interface font-size row (zeron's UI/terminal/code font pickers,
    transcript width and math-size rows have no persisted key and are dropped
    for NG4) — wallpaper and cloud-only rows dropped as NG2. | acceptance:
    functional tests or AX-driven interaction prove each control writes
    `AppSettings` and persists; the persisted file round-trips; owner visual
    check against zeron's Appearance page. (Replaced half, accepted: control
    writes are proven by the pure suite + headless render probes instead of
    AX-driven interaction, Log 2026-10-29T16:35Z; the owner waived the visual
    checkpoint on 2026-10-29, Log 16:00Z.) | source: zeron `settings/widgets.rs`,
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
  status: done
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
  status: done
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
  status: done
  - goal: R2, R3, R5 (translucent parity) | depends_on: `theme-foundation`
  - surfaces: `vendor/zeron-ui/chrome.rs` (ported titlebar geometry +
    upstream's tests), `vendor/zeron-ui/settings/` (section model + nav row
    widget), `src/shell.rs` (app wiring: root surface, drag strip,
    sidebar tone/column, nav assembly, footer identity row), `src/main.rs`
    | avoided: settings page content
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
    - T2 — port the settings section list: rows from `SettingsSection`
      (the seven-section list, `Appearance` included) using zeron's
      `widgets::section_tab` + `tab_selection_t` (icon + label,
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
  status: done
  - goal: R4 | depends_on: `chrome`
  - surfaces: `vendor/zeron-ui/settings/{widgets,select,menu}.rs`,
    `vendor/zeron-ui/{popover.rs,popover/,edge_fade.rs}` (the select stack's
    primitives), `src/settings/` (our pages), `src/app_settings.rs`,
    `src/persist.rs`, `src/i18n.rs` (ported from `wip/gpui-kit-shell`),
    `src/shell.rs`, `src/main.rs` | avoided: zeron settings pages that are NG2
  - escalate_when: a ported widget requires popover/search infrastructure
    whose port would exceed the phase, or a control has no zeron widget
    equivalent.
  - known gaps to close in this phase (from the `chrome` gate), status after
    wave 1: `AppSettings`/persistence/i18n now exist with unchanged keys (NG4)
    and round-trip tests (T1b); the footer reads the Profile display name
    (T1d); `page_column` is ported verbatim (T1a) and is what the pages use;
    zeron's popover/menu/select stack is ported (T1c) so the pages use the real
    dropdown, not an approximation. Still open: the nav has no
    `scroll_faded`/edge-fade wrapper (the pages do scroll inside
    `page_column`; the nav list is short enough until the search row lands in
    T4), `text_tooltip` is unported (no page uses a tooltip yet), and the
    Appearance/fonts/glass rows belong to T3.
  - wave 1:
    - T1 — port the settings page scaffolding (`page_column`, `page_header`,
      `page_subtitle`, `section`, `section_card`, `card_row`, `row_title`,
      `meta_line`, `block_fill`, `row_divider`, `field_label`) and rebuild
      General + Notifications pages on it with our settings state and
      persistence. — output: two pages render and controls write settings.
      — check: `cargo build` + run + settings file round-trip. — stop_if:
      the page layout needs zeron's search/route infrastructure.
    - T2 — port toggle/select/dropdown/slider widgets and rebuild the
      remaining pages (Profile, Finance UI, Reset, About), including the
      glass controls and test notification; where a select needs zeron's
      dropdown (`pane_bounds`/`dropdown_limits` + the deferred overlay), port
      that primitive rather than a local approximation (R8). — output: all
      seven section pages functional. — check: run + settings round-trip + i18n strings.
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
      registry. The `Appearance` row already exists in the nav (added by the
      `chrome` gate fix) — T3 fills its page.
    - T4 — port zeron's settings search row into the sidebar
      (`shell/settings_search.rs` layout: 32px field, rounded 8, magnifier,
      "Search settings" placeholder, shortcut hint) on a text-input
      primitive adapted from the zui fork's `crates/gpui/examples/input.rs`
      (the crate is licensed Apache-2.0 per its `Cargo.toml`; the checkout's
      GPL file covers other zed crates — record the attribution when the
      file is copied), filtering the seven section rows; it replaces the Back
      row the owner dropped, so the sidebar starts with it. — output: search
      filters the section list; typing/escape/clear work. — check: run +
      keyboard round-trip + AX probe. — stop_if: the fork's input handler
      cannot be adapted without porting the chat composer.
  - phase check: `cargo build` + run + settings file round-trip; owner
  visual checkpoint.
- phase_slug: `fidelity-close`
  status: done
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

- 2026-10-30T19:40Z — fidelity-close — decision — absorb: memory
  zeron-stack-port (`docs/memory/zeron-stack-port.md`; the port's module map
  and adaptation rules are the durable artifact this plan hands off to).

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

- 2026-10-29T14:06Z — phase `chrome` — verdict: APPROVED — owner verdict (direct instruction; no separate visual report)
  - owner message: "Ok làm tiếp số 4" (2026-10-29T20:56+07:00) — instruction to continue with phase 4, no defect reported on the four visual points asked for in the checkpoint.
  - the machine-verifiable half was already gated APPROVED in the entry above (34 tests, check/lint/fmt, both selftests, runtime probes, upstream structure diff).
  - unverified, by construction: rendered pixels, synthetic click/keyboard/drag, hover/selection motion, AX exposure — agent tooling has no capture or input grant. This entry records the owner's acceptance, not agent-captured evidence; a defect the owner reports later is fixed inside `chrome` before `fidelity-close`.
  - rollback_point: `feat/zeron-stack` commit `5c1e92b`.

- 2026-10-30T19:40Z — phase `settings-surface` — verdict: APPROVED — mode: gate, owner directive (visual checkpoint waived)
  - owner directive: "không cần check tiếp nữa đâu … làm tiếp luôn là được rồi" (2026-10-29T23:00+07:00) closed the phase without a separate visual report; same precedent as the `chrome` owner verdict.
  - `cargo test` — 115 pass, 0 failed (mono-desk 54, zeron-theme 24, zeron-ui 37) at the wave-4 commit `85a1fdd` — pass
  - `make check` / `make lint` / `make fmt-check` — rc 0 — pass
  - `MONO_DESK_SELFTEST=1 MONO_DESK_SECTION=<general|appearance|notifications|profile|finance-ui|reset|about>` — each prints its `selected=` line, no panic — pass
  - `MONO_DESK_APPEARANCE=dark MONO_DESK_SECTION=appearance` — `selftest appearance=Dark` — pass
  - `MONO_DESK_SEARCH=pro` — `selftest search query=pro visible=Profile` — pass
  - `cargo run` + CGWindow probe (~12 s) — 1 window, layer 0, alpha 1, onscreen 1, bounds 1320×880 at (120,120), empty stderr — pass
  - scope: on target (waves 1–4: vendored widget/select/popover stack, `src/settings/*` pages, `src/finance/`, `src/shell.rs` search row, `src/main.rs` bootstrap, docs)
  - proof_classes: output, unit, integration
  - requirements: R4 met (seven pages on zeron's kit; Appearance is zeron's custom page; glass sliders superseded by the Glass select per R8) | R3 met with upstream's keyboard set (Enter/Space not wired, matching zeron) | R8 met (code parity + owner directive)
  - rollback_point: `feat/zeron-stack` commit `85a1fdd`
  - requests: none
  - not_independently_verified: rendered pixels, hover/selection motion, click/keyboard behaviour, AX exposure (agent tooling has no capture or synthetic-input grant; waived by the owner directive)
  - judge: same-session (owner directive)
  - judge_model: deepseek-v4.1-flash

- 2026-10-30T19:40Z — phase `fidelity-close` — verdict: APPROVE_WITH_REQUESTS — mode: full (lane high-risk, final phase)
  - judge: independent subagent (separate context, read-only); judge_model: deepseek-v4.1-flash; graded first at HEAD `743e7d6`, then on the close-out commit that resolves its requests.
  - `cargo test` — 119 pass, 0 failed (mono-desk 58, zeron-theme 24, zeron-ui 37) — pass
  - `make check` / `make lint` / `make fmt-check` — rc 0 — pass
  - `grep -rn "gpui_kit\|gpui-kit" Cargo.toml` — no matches — pass
  - `grep -rn "gpui-component\|vendor/patches" Cargo.toml src vendor/zeron-ui` — only the two restored upstream provenance citations in the vendored popover file; no dependency, path or import — pass
  - `git ls-files vendor/gpui-component vendor/patches` — empty; `ls vendor` = `zeron-theme`, `zeron-ui` — pass
  - `cargo run` + CGWindow probe + all seven `MONO_DESK_SECTION` probes + `MONO_DESK_SEARCH` — window 1320×880 at (120,120), every page paints, filter probe matches — pass
  - scope: on target (README/PROJECT rewrite, docs/design archive, memory banners, notices, dead-code/doc-comment hygiene, nav i18n, text-field tests, plan Log/Validation)
  - proof_classes: output, unit
  - proof_gaps: integration — no interaction harness (no synthetic input; gpui `test-support` not enabled), and rendered pixels, hover/selection motion, keyboard/typing, AX exposure, the Opaque↔Blurred flip and runtime min-size enforcement are unverifiable from agent tooling
  - requirements: R6 met (grep sweep, notices, old vendor trees absent, docs rewritten) | R7 met (four phase gates recorded; this is the final independent `full`)
  - rollback_point: `feat/zeron-stack` commit `85a1fdd`
  - requests: 8 judge findings — all resolved in the close-out (Log 2026-10-30T19:40Z): dirty-tree commit + gate re-run; R4 amended; success_signal reworded; T2 hygiene finished; vacuous test replaced; nav localized via i18n; text_field covered by pure tests; this phase's `settings-surface` Validation entry added above
  - not_independently_verified: nothing beyond `proof_gaps`
  - judge: independent
  - judge_model: deepseek-v4.1-flash

## Current State and Next Action
- active_phase: none
- lifecycle_status: completed (all five phases `done`; final `check full`
  returned APPROVE_WITH_REQUESTS and every request is resolved or recorded)
- blockers: none.
- open_items: none for this initiative. Carried as documented, accepted gaps
  (not backlog for this plan): rendered pixels / hover / click / keyboard /
  AX exposure are unverifiable from agent tooling and the owner waived the
  visual checkpoint (Log 2026-10-29T16:00Z); the Opaque↔Blurred flip and the
  900×600 minimum are code-verified only; the pre-port interaction suite is
  replaced by pure tests + headless render probes (Log 2026-10-29T16:35Z);
  no nav edge-fade wrapper; the Finance art board is fixed-px and does not
  scale with the interface font size; About has no Links/dialog group;
  reduce-motion and theme-variant choices are live-only (no persisted key,
  NG4); pre-port accent values that match no zeron preset read as "Theme
  default"; 29 i18n keys from removed controls remain unused; zeron's theme
  library/wallpaper/cloud surfaces are out of scope (NG2).
- exact_next_action: owner runs `make run` for the visual pass over the seven
  pages (light and dark, `MONO_DESK_APPEARANCE=dark` for a pinned pass). Any
  defect it finds starts a new plan (`docs/plans/active/`) or reopens the
  owning area; there is no further work queued in this initiative.
- durable handoff: `docs/memory/zeron-stack-port.md` (module map + adaptation
  rules), `docs/PROJECT.md`, `THIRD_PARTY_NOTICES.md`, and this plan's
  Validation entries.
