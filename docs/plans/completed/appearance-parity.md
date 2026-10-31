---
status: active
lane: high-risk
---
# Appearance page 1:1 port from zeron

## Goal
- outcome: mono-desk's Appearance page is a component-and-layout-exact port
  of zeron's own custom Appearance page (`/tmp/zeron-src/crates/ui/src/
  settings/appearance.rs` @ 8b7afb3, 4,896 lines; the page mono-desk's
  previous port reduced to 708 lines): the same page host (edge-faded scroll
  + floating scrollbar rail + drag-move host), the same section order and
  spacing, every row in Color scheme, Material and background, Motion, the
  unlabeled Theme library card and Fonts and layout, the same three modal
  overlays (theme import, background adjustment, theme mapping review),
  zeron's helper texts and select widths, with all controls functional
  against locally persisted state and text localized in the app's four
  locales. Scope decisions taken by the owner in this session (2026-10-30):
  full 1:1 including every subsystem, new state persisted in the existing
  `settings.conf`, all heavy subsystems included (theme library import, font
  catalog with installed system families, background effect previews,
  wallpaper randomizer), strings translated for en/vi/zh-CN/ja.
- success_signal: `MONO_DESK_SECTION=appearance MONO_DESK_SELFTEST=1` reports
  the ported page's structural markers (section order, row ids, select
  widths, dialog registrations) and the resolved new settings; the owner's
  `make run` visual pass over the page (light + dark, and with a chosen
  background image, an imported theme and a non-`None` background effect)
  matches zeron's page row-for-row; `cargo test && make check && make lint &&
  make fmt-check` are green; the independent `check full` judge records its
  verdict in `## Validation`.
- actors: the owner (visual authority for this demo), developers reading the
  repo for zeron's settings-surface patterns.
- authority: owner decisions 2026-10-30 (this session): "same 100%, full
  components, layout with it" plus the four scoping answers (full 1:1; extend
  `settings.conf`; include all heavy subsystems; translate all four locales).
  Reference source: zeron `github.com/zeronsh/zeron` @ 8b7afb3 (MIT,
  Copyright 2026 Wing), clone at `/tmp/zeron-src`; vendored UI/theme under
  `vendor/zeron-ui/` and `vendor/zeron-theme/`. Page inventory evidence
  (line numbers below) comes from the read-only component inventory against
  that pinned source. Repo rules: `AGENTS.md`, `docs/WORKFLOW.md`,
  `docs/playbooks/*`.
- requirements:
  - R1: Page structure and layout match zeron's render tree: page host
    `#appearance-page-host` (relative, drag-move handlers, mouse-up flush,
    list-hover hook), `edge_faded(16, top+bottom)` scroll wrapper,
    `PageScroll` + `popover::rail` scrollbar, `page_column` + "Appearance"
    header, then in order: "Color scheme" section (3 mode cards),
    color card (Light theme, Dark theme, Accent color, Match wallpaper
    colors), "Material and background" (Glass, New thread background,
    Wallpaper folder, conditional Background effect), "Motion" (Reduce
    motion, Pause animations in background, Smooth scrolling), unlabeled
    theme-library card at `mt 32`, "Fonts and layout" (Interface / Terminal /
    Code & diff fonts, Conversation width, Render math, Math font, Math
    size), unavailable-font error strips, then at most one modal (background
    adjust > import > review). | acceptance: structural markers in
    `MONO_DESK_SELFTEST` equal the checklist in the plan's Log evidence +
    code inspection against the zeron line map + owner visual pass.
    | source: zeron page :4185-4289, :4239-4281, :4089-4092
  - R2: Color scheme behaves like zeron's: mode cards (System/Light/Dark,
    `option_card` + `tab_selection_t` + miniature previews) unchanged in
    shape; Light/Dark theme selectors list every `ThemeRegistry::active()`
    variant for that appearance with `palette_preview` leading art, width
    218 / menu 260 / heading "Light themes"/"Dark themes"; selected variant
    ids persist (`theme_light`, `theme_dark`) and drive
    `settings::apply_appearance`; the accent row's meta is zeron's dynamic
    `accent_helper` (and the wallpaper-colors notice when enabled); "Match
    wallpaper colors" toggles wallpaper-derived tinting of the resolved theme
    (restores the branch stubbed at `vendor/zeron-ui/src/theme.rs:1400`).
    | acceptance: unit tests for variant-id codec/fallback and accent helper
    strings; selftest prints both selected variant ids; tint test on a
    resolved theme. | source: zeron :3562-3704, :2144-2203; owner answer 1
  - R3: Material and background is complete and functional: Glass row with
    zeron's `surface_helper` text and width 148; New thread background row
    (36px tile rendered from the installed image, meta states "name" /
    "Image unavailable · Choose a replacement or remove it." / "No image
    selected", actions Adjust · Replace image · Remove or Choose image,
    image copied into the app data dir's `new-thread-backgrounds/`); the
    background adjustment dialog (260px preview well, pan/zoom/pinch,
    168×28 zoom slider, Reset, Cancel/Apply, keyboard pan/zoom, focus
    return); conditional Background effect select (128) with effect
    previews; Wallpaper folder row (folder meta incl. shortcut, "Choose
    folder" + conditional "Shuffle"). Removing a background deletes only
    files inside the managed directory. | acceptance: unit tests for
    adjustment normalize, managed-path guard, effect codec; image-tile and
    preview helpers compile against gpui image APIs; selftest reports
    background state; owner visual pass. | source: zeron :3730-3957,
    :2747-3265
  - R4: Motion section has zeron's three rows: Reduce motion (width 128,
    dynamic `reduce_motion_helper`), Pause animations in background toggle,
    Smooth scrolling toggle with zeron's helper text; all three persist
    (`reduce_motion`, `pause_in_background`, `smooth_scrolling`) and boot
    initializes `motion::init` from them. | acceptance: codec tests;
    selftest prints the three values; changing them survives restart (file
    inspection). | source: zeron :3958-4083; motion API in
    `vendor/zeron-ui/src/motion.rs:1161-1268`
  - R5: Theme library matches zeron: unlabeled card with "Theme library" row
    + "Add theme", "Imported"/"Linked" group headers, per-entry status line
    and actions (Reload when linked, Reveal, Review, Duplicate as editable,
    Unlink when linked, Remove in danger tone), load warning line; the
    import dialog (Source field + Browse via `cx.prompt_for_paths`, "Keep it
    up to date" copy/link modes, compile through the vendored
    `CustomThemeLibrary::compile`/vscode importer, Detected themes with
    palette + scene preview + report panel per variant, selection
    checkboxes, Analyze theme / Import selected footer, Esc/Enter keys) and
    the Theme mapping review dialog; library persisted in
    `Application Support/mono-desk/` and installed into
    `ThemeRegistry::active()` so the page selectors see custom families.
    | acceptance: tempdir integration test compiles a fixture VS Code theme,
    installs it, asserts the active registry exposes the variant and remove
    cleans up; codec tests for `source_name`/`slug`; selftest prints library
    entry count. | source: zeron :3321-3543, :2205-2746, :3267-3319;
    vendored `vendor/zeron-theme/src/{library,vscode}.rs`;
    `crates/ui/src/theme_library.rs` (217 lines, vendored UI bridge missing)
  - R6: Fonts and layout matches zeron: three font rows (Interface,
    Terminal, Code & diff) each with a 220px family picker (dropdown card
    220, search input, scrollable list with rail, keyboard open/step/commit/
    dismiss incl. the 400ms reopen guard) and a 128px size select (Ui ladder
    12-20px; terminal/code ladder 10-20 incl. 12.5); family catalog contains
    the bundled faces plus installed system families that pass zeron's
    Latin-metrics and fixed-width checks (fontdb + ttf-parser), with
    terminal restricted to fixed-width; Conversation width row with the
    240×28 slider (560-1200px, step 16, default 736, hover/keyboard detail
    rows, Reset, per-frame coalesced persistence); Render math toggle; Math
    font (160/220 with style detail) and Math size (160, 90-150% with
    "Default"); error strips when requested != effective. | acceptance:
    unit tests for `step_font`, `filter_families`, `first_available`,
    `last_available`, `format_px`, `nearest_math_size_ix`,
    `nearest_mono_ix`, transcript normalize/step, and availability
    fixed-width filtering; selftest prints resolved families/sizes.
    | source: zeron :398-513, :515-678, :1958-2142, :4094-4182;
    `vendor/zeron-ui/src/typography.rs` (needs the installed catalog)
  - R7: New state persists in the flat `settings.conf` key=value format:
    `theme_light`, `theme_dark`, `ui_font_family`, `terminal_font_family`,
    `code_font_family`, `terminal_font_size`, `code_font_size`,
    `transcript_width`, `render_math`, `math_font`, `math_size`,
    `smooth_scrolling`, `reduce_motion`, `pause_in_background`,
    `wallpaper_theme_colors`, `wallpaper_color`, `wallpaper_folder`,
    `wallpaper_source`, `new_thread_background_path`,
    `new_thread_background_name`, `new_thread_background_focal_x`,
    `new_thread_background_focal_y`, `new_thread_background_zoom`,
    `new_thread_background_effect` (~24 keys). Each key has a stable
    word/codec, a default that mirrors `AppSettings::default`, a non-finite
    guard where numeric, and tolerance for unknown/hand-edited values; the
    write order is frozen and tested; old files without the keys load with
    defaults; the foreign-file guard and atomic write stay intact. The theme
    library JSON and copied background images live in the same data dir
    (atomic writes). | acceptance: `cargo test` codec/default-mirror/
    unknown-key tests; `persist::save` order test updated; a hand-crafted
    file with one new key loads without resetting others.
    | source: owner answer 2 (extend settings.conf); `src/persist.rs`
  - R8: Every new visible string (row titles, helper texts, dialog copy,
    buttons, empty states, error strips) goes through `i18n::tr` and is
    translated in en, vi, zh-CN and ja; no key falls back to the key name.
    | acceptance: i18n completeness test asserting every `appearance.*`,
    `font.*`, `math.*`, `theme_library.*`, `background.*` key is non-empty
    in all four locales; selftest language pass. | source: owner answer 4;
    `src/i18n.rs`
  - R9: Interaction details match zeron where observable: modal stack order
    and Escape routing, focus return from dialogs, drag hosts for width/
    pan/zoom, window-frame coalescing for the width slider, select/menu
    keyboard models, 30×34 accent swatches, 36×36 background tile, focus
    rings/roles/aria on ported controls. | acceptance: pure-helper tests +
    code inspection against the zeron interaction inventory; owner pass.
    | source: zeron :348-1397, :4185-4289
  - R10: The port is documented and gated: `AGENTS.md` conventions
    (appearance state, theme library, background media, i18n keys),
    `docs/PROJECT.md` non-goals no longer exclude theme library/wallpaper,
    `THIRD_PARTY_NOTICES.md` lists every newly copied zeron source,
    `docs/memory/` captures the module map/adaptation rules, the pre-commit
    hook passes, and the final phase receives an independent `check full`
    with the verdict recorded in `## Validation`. | acceptance:
    `cargo test && make check && make lint && make fmt-check` rc 0 on the
    final commit; docs diff review; Validation entry present.
    | source: `docs/WORKFLOW.md`, `AGENTS.md`
- non-goals:
  - NG1: zeron's settings-search metadata (`SearchAnchor`, `SettingId`
    anchors) and the `page_reveal` glide — unseen when the page is opened
    directly, and mono-desk's sidebar search filters nav sections rather than
    settings rows; the page keeps its own localization.
  - NG2: zeron's kinetic scroll engine (`smooth_scroll.rs`/`overscroll.rs`)
    — the Smooth scrolling toggle persists but mono-desk's plain gpui scroll
    does not glide.
  - NG3: chat surfaces: no transcript, Markdown/LaTeX renderer or
    new-thread composer exists, so Conversation width, Render math, Math
    font/size and the background/effect choices persist and preview only
    inside the Appearance page (row + dialog behave; nothing else consumes
    them).
  - NG4: a wallpaper rendering layer — "Wallpaper folder"/"Shuffle" persist
    `wallpaper_folder`/`wallpaper_source` and surface errors, but no
    wallpaper is painted behind the app; wallpaper-derived colors come from
    the installed new-thread background image, exactly as zeron's
    `wallpaper_colors::active` reads it.
  - NG5: zeron's toast surface (`toast::take_error`) — background/import
    errors render as inline strips in the owning card (visual deviation
    recorded in `## Log`).
  - NG6: cloud theme sync, appshots, auto-update, chat-route chrome and any
    other zeron surface outside the Appearance page.
  - NG7: no change to the app's default bundled typography identity:
    Momo Trust Sans / Hanken Grotesk / Google Sans Code / Xanh Mono remain
    the bundled choices and the defaults; the installed-family catalog is
    additive.

## Phases and Verification
- approach: port zeron's page bottom-up in dependency order, keeping the app
  runnable after every phase. (1) complete the vendored kit (PageScroll/rail,
  installed-font catalog, wallpaper tint branch) plus the new direct deps;
  (2) grow `AppSettings`/persistence and the appearance/theme-library/
  background/wallpaper domain so the page has real state; (3) rebuild the
  page scaffold + Color scheme + Motion sections on the new state; (4)
  Material and background incl. the adjustment dialog and effect previews;
  (5) Theme library rows + import/review dialogs; (6) Fonts and layout incl.
  the three pickers, slider and math rows; (7) polish, docs, gates and the
  independent check. The page is split into `src/settings/appearance/
  {mod,color,motion,material,library,fonts,dialogs,preview,font_model}.rs`
  so independent sections can be written in parallel; each phase's mod.rs
  "traffic" wiring has a single owner. Rejected alternative: copy zeron's
  4,896-line file verbatim — it binds to zeron's `settings` service,
  `ComposerInput`, chat surfaces and cloud settings that mono-desk does not
  have; the adapted modular port keeps zeron's row code shape and constants
  while wiring to `AppSettings` (same adaptation principle as the previous
  port).
- constraints: zeron source is MIT — new copied files keep provenance
  headers and `THIRD_PARTY_NOTICES.md` entries; vendored files keep upstream
  formatting and `#![allow(clippy::all)]`; copied constants/layout numbers
  stay verbatim, adaptations are limited to imports, state source and
  removed chat-only branches; `make lint`/`make fmt` stay scoped to
  `-p mono-desk`; persistence stays the flat frozen-order `settings.conf`
  with the marker guard; no new runtime network calls; new direct deps must
  already exist in `Cargo.lock` (`fontdb 0.23`, `ttf-parser 0.25`,
  `image 0.25`, `tempfile` as dev-dep).
- risks: (a) gpui APIs used by the dialogs (`prompt_for_paths`,
  `paint_image_fitted_masked`, drag/ pinch/ scroll events, image decoding)
  may differ in the pinned zui fork — mitigated by P1/P2 compile probes
  before page work; stop and report if a needed platform API is missing.
  (b) The installed-font catalog changes typography startup and can affect
  every page's metrics — mitigated by keeping bundled defaults and by the
  existing selftest font markers; stop if any existing page's font identity
  shifts. (c) Theme library install mutates a global registry mid-session —
  mitigated by P2's reconcile/fallback tests and by reusing
  `Theme::reinstall_selection`; a variant that disappears must fall back,
  never panic. (d) Settings-file growth can corrupt user state — mitigated
  by R7 tests + the foreign-file guard; stop on any non-additive change to
  existing keys/order. (e) Image decode/effects work is the largest visual
  unknown — mitigated by keeping `image` on the background executor and by
  the owner's dialog preview pass; stop if a chosen image crashes or blocks
  the UI thread. (f) The page is one hot file while five phases edit it —
  mitigated by the split-module architecture and by per-phase traffic
  ownership.
- recovery: work happens on branch `feat/appearance-parity` cut from
  `feat/zeron-stack` HEAD `0cb89e1` (rollback point). To abandon:
  `git switch feat/zeron-stack && git branch -D feat/appearance-parity`;
  to roll back a phase: `git checkout <last-good-commit> -- .` on the
  branch. Existing plan artifacts (`docs/plans/completed/zeron-stack-port.md`,
  `docs/memory/zeron-stack-port.md`) are the prior port's authority and are
  not rewritten by this plan except additive notes.
- phase_slug: `kit-typography`
  status: checked
  - goal: R6, R9, R2 (tint hook) | depends_on: none
  - surfaces: `Cargo.toml`, `Cargo.lock`,
    `vendor/zeron-ui/src/typography.rs`,
    `vendor/zeron-ui/src/settings/widgets.rs`,
    `vendor/zeron-ui/src/settings/mod.rs` (re-exports),
    `vendor/zeron-ui/src/theme.rs`, `vendor/zeron-ui/src/lib.rs` (module
    list if a module is added) | avoided: `src/`, `vendor/zeron-theme/`,
    `assets/`, `docs/`
  - escalate_when: the pinned gpui fork lacks `text_system().all_font_names`
    / `add_fonts` behavior the catalog needs, `fontdb`/`ttf-parser` versions
    in the lock lack the required APIs, or `PageScroll` cannot be expressed
    on the vendored popover rail state.
  - wave 1:
    - T1 — add the direct deps (`fontdb = "0.23"`, `ttf-parser = "0.25"`,
      `image = { version = "0.25", default-features = false, features =
      ["png", "jpeg", "gif", "webp", "bmp", "tiff", "ico"] }`) and a
      `tempfile = "3"` dev-dep to the root `Cargo.toml`; `cargo check`.
      (owner: lead, writes `Cargo.toml`, `Cargo.lock`) — output: deps
      resolve against the existing lock versions. — check: `cargo check`. —
      stop_if: resolution pulls a different major of an existing pinned dep.
    - T2 — port `PageScroll` (upstream `settings/widgets.rs:181-242`) and
      `widgets::rail` (:250-303) into the vendored widgets module on the
      already-vendored `popover::{ScrollRailHost, MenuScrollbarState, rail,
      schedule_scrollbar_hide, reset_menu_scroll}`, re-export through
      `crate::settings`, and add a unit test for rail metrics/hover state.
      (owner: widget-scaffold, writes the two vendored files only) —
      output: pages can own a `PageScroll` and render a floating rail. —
      check: `cargo test -p zeron-ui`. — stop_if: the popover state machine
      lacks the methods the upstream body calls.
    - T3 — extend the vendored typography to zeron's catalog semantics:
      add `UiFontFamily::Installed(String)` (keeping the bundled variants and
      their labels), replace the fixed `ALL` availability with
      `FontAvailability` carrying bundled-registered + installed-Latin +
      fixed-width lists, port `installed_families_with_latin_metrics`,
      `face_is_fixed_width`, `register_fonts` enumeration (bundled first) and
      `is_available`/`is_fixed_width_available`; keep bundled families as
      defaults/fallbacks. (owner: state-persist, writes
      `vendor/zeron-ui/src/typography.rs`) — output: availability lists
      bundled + system families and classifies fixed width. — check:
      `cargo test -p zeron-ui`. — stop_if: font enumeration needs a platform
      service not exposed by the pinned fork.
  - wave 2:
    - T4 — restore the wallpaper-color branch in the vendored theme: add a
      public `Theme::install_selection_with_color(...)`/`for_selection_with_color`
      path that applies zeron's `tint_variant` mapping (surface mixes,
      accent derivation, hover/active/border/border_strong, terminal tint)
      before `from_variant`, thread it through `install_selection_inner`
      (`None` keeps today's behavior), and unit-test the tint. (owner: lead,
      writes `vendor/zeron-ui/src/theme.rs`) — output: a color tints the
      resolved theme exactly like zeron's `wallpaper_colors::tint_variant`. —
      check: `cargo test -p zeron-ui`. — stop_if: the tint needs zeron
      settings-domain types that cannot be expressed in `zeron_theme`.
  - phase check: `cargo test && make check` (vendored crates included) and
    `grep -n "Installed\|PageScroll\|with_color" vendor/zeron-ui/src/*.rs
    vendor/zeron-ui/src/settings/*.rs` listing the new symbols.
- phase_slug: `state-persistence`
  status: checked
  - goal: R7, R5 (library foundation), R2 (variant selection), R4
    (persistence), NG5 adapter | depends_on: `kit-typography`
  - surfaces: `src/app_settings.rs`, `src/persist.rs`, `src/settings/mod.rs`,
    `src/settings/theme_library.rs` (new), `src/settings/background.rs`
    (new), `src/settings/wallpaper.rs` (new),
    `src/settings/wallpaper_colors.rs` (new), `src/main.rs`,
    `src/shell.rs` (only the page construction/apply path) | avoided:
    `src/settings/appearance*` (phase 3-6), `vendor/`
  - escalate_when: a new key needs a value shape the flat key=value codec
    cannot express without escaping ambiguity, or boot order cannot init the
    theme library before the first theme install.
  - wave 1:
    - T1 — extend `StoredSettings`/`AppSettings` with the ~24 R7 keys
      (defaults, clamps/non-finite guards, word codecs for enums/fonts/
      variants/colors, `from_stored`/`to_stored`), keep the frozen write
      order test and add round-trip/default-mirror/unknown-key/non-finite
      tests; `tempfile` used for any file test. (owner: state-persist, writes
      `src/app_settings.rs`, `src/persist.rs`) — output: settings round-trip
      with defaults mirrored and old files loading. — check:
      `cargo test -p mono-desk`. — stop_if: an existing key's order/wording
      would have to change.
  - wave 2:
    - T2 — appearance domain: persist-selected variant ids resolve through
      `ThemeRegistry::active()` (fallback to `zeron-*`) in
      `settings::apply_appearance`; add `settings::apply_registry_change`
      equivalent; initialize typography from persisted families/sizes
      (bundled default when unknown), motion from `reduce_motion`/
      `pause_in_background`, and fix the vendored `typography::reinstall_theme`
      clobber so a family change keeps `variant_id`/surface selection; boot
      ordering: data dir + theme library init → typography → theme/appearance
      install. (owner: state-persist, writes `src/settings/mod.rs`,
      `src/main.rs`, `vendor/zeron-ui/src/typography.rs` reinstall fn) —
      output: restart keeps theme variant/family/motion choices. — check:
      `cargo test -p mono-desk` + `MONO_DESK_SELFTEST=1`. — stop_if: theme
      library init must run after the first theme install (record and
      reorder via decision entry).
    - T3 — port the 217-line theme-library UI bridge as
      `src/settings/theme_library.rs` (Adapted: data dir from
      `persist::config_dir()`, `zeron_theme::{CustomThemeLibrary,
      InstallMode, ThemeRegistry, replace_custom_families}`, persistence via
      the library file, reconcile to the app's appearance state); T2 calls
      `theme_library::init(data_dir, cx)` at boot. Add tempdir tests for
      install/persist/remove and reconcile fallback. (owner: profile-about,
      writes `src/settings/theme_library.rs` only) — output: custom families
      load into the active registry at boot and survive restart. — check:
      `cargo test -p mono-desk`. — stop_if: vendored library save path
      semantics differ from the assumption (record and adapt).
    - T4 — background/wallpaper/wallpaper-color domain: new
      `src/settings/background.rs` (NewThreadBackground + adjustment +
      effect enum with zeron labels/descriptions, install/replace/remove
      copying into `config_dir()/new-thread-backgrounds/`, managed-path-only
      deletion, decode validation through the `image` crate, 64×64 thumbnail
      color extraction), `src/settings/wallpaper.rs` (folder scan/shuffle
      picking a random supported image, error strings, persisted source),
      `src/settings/wallpaper_colors.rs` (port `extract` + `tint_variant`
      adapter + enabled toggle applying the theme tint). (owner:
      select-stack, writes those three files only) — output: background and
      wallpaper operations are testable without UI. — check:
      `cargo test -p mono-desk`. — stop_if: image decoding on the app path
      requires a platform service not available in tests.
  - phase check: `cargo test && make check && MONO_DESK_SELFTEST=1` (the
    selftest must print the new settings words and the boot order marker).
- phase_slug: `page-base-color-motion`
  status: checked
  - goal: R1 (scaffold + section order), R2, R4, R8 (part 1) |
    depends_on: `state-persistence`
  - surfaces: `src/settings/appearance.rs` → `src/settings/appearance/
    mod.rs`, `src/settings/appearance/{color,motion,preview}.rs`,
    `src/i18n.rs`, `src/shell.rs`, `src/main.rs` (selftest markers) |
    avoided: `src/settings/{general,notifications,profile,about,reset,
    finance}*`, `vendor/`
  - escalate_when: the shell cannot host the page's new per-frame state
    (drag/scroll handles, dialog entities) without changing the shell's page
    ownership contract.
  - wave 1:
    - T1 — rebuild the page view as `appearance/mod.rs` + submodules:
      host div + edge fade + `PageScroll` + `popover::rail` + hover hook,
      `page_column` with the exact R1 order (empty material/library/fonts
      section bodies to be filled in later phases are rendered as
      `section_card` placeholders only if needed — final rows land in phases
      4-6), Color scheme section (mode cards unchanged), the color card with
      Light/Dark theme selectors (registry variants, palette previews, width
      218/menu 260/heading), Accent row with dynamic helper + 8 swatches,
      Match wallpaper colors row + toggle, Motion section's three rows;
      `color.rs` holds miniature/palette previews/helpers, `motion.rs` the
      rows, `preview.rs` the shared miniature helpers. (owner:
      widget-scaffold, writes `src/settings/appearance.rs` deletion,
      `src/settings/appearance/{mod,color,motion,preview}.rs`,
      `src/settings/mod.rs` module list) — output: the page renders the new
      scaffold with the real color/motion sections. — check:
      `cargo check -p mono-desk` + `MONO_DESK_SECTION=appearance
      MONO_DESK_SELFTEST=1`. — stop_if: an API used by zeron's scaffold
      (edge_fade/page_reveal/rail) is unavailable in the vendored kit.
    - T2 — i18n part 1: add `appearance.*` keys for the new rows/helpers
      (scheme/accent/wallpaper-colors/glass/new-thread-background/wallpaper
      folder/effect/motion/theme-library/fonts section titles and helper
      sentences) in en/vi/zh-CN/ja. (owner: finance-ui, writes
      `src/i18n.rs` only) — output: no hardcoded English in the new rows. —
      check: `cargo test -p mono-desk` i18n completeness test. — stop_if: a
      string needs a product decision (ask owner, record decision).
  - wave 2:
    - T3 — shell/main wiring + selftest: construct the page with its state,
      keep the footer appearance toggle path on `apply_appearance`, extend
      `MONO_DESK_SELFTEST` with appearance markers (selected variants, accent,
      surface, motion, background/wallpaper/state words). (owner: lead,
      writes `src/shell.rs`, `src/main.rs`) — output: open page shows the new
      sections; selftest prints the markers. — check: `cargo test -p
      mono-desk` + `MONO_DESK_SECTION=appearance MONO_DESK_SELFTEST=1`. —
      stop_if: shell page ownership has to change (escalate).
  - phase check: `cargo test && make check && make lint && make fmt-check`
    plus the appearance selftest output recorded in `## Log`.
- phase_slug: `material-background`
  status: checked
  - goal: R3, R9 (drag/pan/zoom), R8 (part 2) |
    depends_on: `page-base-color-motion`
  - surfaces: `src/settings/appearance/material.rs` (new),
    `src/settings/appearance/dialogs.rs` (new, background dialog),
    `src/settings/appearance/mod.rs`,
    `vendor/zeron-ui/src/new_thread_background_mask.rs` (new, adapted),
    `vendor/zeron-ui/src/new_thread_background_effects.rs` (new, adapted),
    `vendor/zeron-ui/src/lib.rs`, `src/settings/appearance/preview.rs`,
    `src/i18n.rs` | avoided: font/library sections, `src/app_settings.rs`
  - escalate_when: gpui lacks a needed primitive (`paint_image_fitted_masked`,
    scroll-wheel/pinch events, canvas paint) or the effects port requires
    zeron's composer/artwork pipeline that cannot be isolated.
  - wave 1:
    - T1 — Material and background rows: Glass (helper text + width 148),
      New thread background (tile image via `img()`, meta states, action
      buttons wired to `settings::background`), Background effect
      (conditional select 128), Wallpaper folder (meta + folder/shuffle,
      shortcut label via a small local keymap constant matching zeron's
      default), plus the host-level drag-move/mouse-up plumbing the dialog
      needs. (owner: widget-scaffold, writes
      `src/settings/appearance/material.rs`,
      `src/settings/appearance/mod.rs`) — output: rows render and
      choose/replace/remove work against the data dir. — check:
      `cargo test -p mono-desk` + appearance selftest. — stop_if: image
      loading (`img(PathBuf)`) needs an image format feature the app does
      not link (record and enable in Cargo.toml).
    - T2 — background effect previews + adjustment painting: port zeron's
      `new_thread_background_effects.rs` (`prepare`, effect frames) and
      `new_thread_background_mask.rs` (`fitted_geometry`,
      `adjusted_image_paint`, `paint_adjusted`, `pan_adjustment`,
      `zoom_adjustment_around`) into the vendored crate, trimming
      composer-only masks/cutout branches with provenance notes. (owner:
      select-stack, writes those two vendored files +
      `vendor/zeron-ui/src/lib.rs`) — output: effect-processed
      `Arc<RenderImage>` and crop geometry available to the dialog. — check:
      `cargo test -p zeron-ui`. — stop_if: the modules need zeron's
      composer state; port only the needed functions and record the trim.
    - T3 — i18n part 2 (material/background/dialog strings) in four locales.
      (owner: finance-ui, writes `src/i18n.rs` only) — output: no hardcoded
      English. — check: i18n completeness test. — stop_if: product wording
      question (ask owner).
  - wave 2:
    - T4 — background adjustment dialog: card anatomy (header, 260px
      preview well, guides while dragging, zoom slider 168×28 + percent,
      Reset, Cancel/Apply, close), pan/zoom/pinch drags, keyboard
      pan/zoom/tab wrap/escape/Mod-Enter, focus return, apply through
      `settings::background`; wire opening from the row. (owner: select-stack,
      writes `src/settings/appearance/dialogs.rs`,
      `src/settings/appearance/mod.rs` dialogs field) — output: Adjust opens
      the dialog, edits the draft, Apply persists and the tile re-renders. —
      check: `cargo test -p mono-desk` + appearance selftest + owner visual
      pass (recorded, not claimed by agent). — stop_if: preview painting
      cannot be exercised in the test environment (record the proof gap).
  - phase check: `cargo test && make check && make lint && make fmt-check`
    + appearance selftest markers for background/effect state.
- phase_slug: `theme-library`
  status: checked
  - goal: R5, R2 (active registry), R8 (part 3) |
    depends_on: `state-persistence` (glue), `page-base-color-motion`
  - surfaces: `src/settings/appearance/library.rs` (new),
    `src/settings/appearance/dialogs.rs` (import/review),
    `src/settings/appearance/mod.rs`, `src/i18n.rs`,
    `src/settings/theme_library.rs` (only if the bridge needs an
    `install`/`review` accessor added) | avoided: material/fonts sections,
    `vendor/`
  - escalate_when: the vendored importer needs platform/file-dialog
    behavior that cannot run headlessly, or install must mutate the registry
    before the page's first render (order conflict).
  - wave 1:
    - T1 — Theme library card: header row + Add theme action, group headers,
      entry rows with status line and six actions, warning line; rows read
      `theme_library::entries`/`load_warning` and call the bridge. (owner:
      state-persist, writes `src/settings/appearance/library.rs`) — output:
      library card renders real entries and actions. — check: `cargo test -p
      mono-desk`. — stop_if: bridge API lacks an action the row needs
      (add it in the same task, record).
    - T2 — import + review dialogs: source field (TextField adaptation with
      browse via `cx.prompt_for_paths`), copy/link mode controls, compile +
      detected-themes list with palette/scene preview/report panel,
      selection checkboxes, Analyze/Import footer, error/empty states,
      Escape/Enter keyboard; review dialog (660px, per-variant scene +
      report, Done). (owner: select-stack, writes
      `src/settings/appearance/dialogs.rs`) — output: a VS Code theme file
      can be compiled, previewed, selected and imported. — check:
      `cargo test -p mono-desk` + tempdir import test from phase 2. —
      stop_if: `CustomThemeLibrary::compile` needs source file shapes the
      fixture cannot provide headlessly (use a minimal JSON theme fixture).
    - T3 — i18n part 3 (theme library/import/review strings) in four
      locales. (owner: finance-ui, writes `src/i18n.rs` only) — output: no
      hardcoded English. — check: i18n completeness test. — stop_if: wording
      decision needed.
  - wave 2:
    - T4 — wire the library card + dialogs into the page (traffic in
      `mod.rs`), refresh the theme selectors after install/remove through
      `apply_registry_change`, and assert in a test that an installed custom
      variant appears in `ThemeRegistry::active().variants_for()`. (owner:
      state-persist, writes `src/settings/appearance/mod.rs`) — output:
      imported themes appear in the Appearance selects and library card.
      — check: `cargo test -p mono-desk` + appearance selftest. — stop_if:
      registry refresh cannot repaint (record and call
      `cx.refresh_windows`).
  - phase check: `cargo test && make check && make lint && make fmt-check`
    + selftest library markers.
- phase_slug: `fonts-layout`
  status: checked
  - goal: R6, R9 (keyboard/menu details), R8 (part 4) |
    depends_on: `page-base-color-motion`, `kit-typography`
  - surfaces: `src/settings/appearance/fonts.rs` (new),
    `src/settings/appearance/font_model.rs` (new),
    `src/settings/appearance/dialogs.rs` (font menus),
    `src/settings/appearance/mod.rs`, `src/main.rs` (page-level key
    bindings only if needed), `src/i18n.rs` | avoided: material/library
    sections, `src/app_settings.rs` (fields already exist)
  - escalate_when: the font menu needs a text-input contract the vendored
    TextField cannot provide (focus/key-context collisions), or the slider
    needs gpui APIs the fork lacks.
  - wave 1:
    - T1 — font model + rows: `font_model.rs` (FontKind, ladders
      `MONO_FONT_SIZES`/`MATH_SIZES`, `step_font`, `filter_families`,
      `first_available`, `last_available`, `fallback_selection`,
      `format_px`, nearest-ix helpers, transcript constants/normalize) and
      `fonts.rs` (three rows with 220px family picker triggers + 128px size
      selects; menus with search input, list + rail, picker rows, keyboard
      model, reopen guard; error strips). (owner: select-stack, writes the
      two new files) — output: family/size selection works against the
      vendored catalog and persists. — check: `cargo test -p mono-desk`. —
      stop_if: menu needs a new gpui popup primitive (record and adapt).
    - T2 — Conversation width + math rows: the 240×28 slider with hover/
      keyboard detail rows + Reset + per-frame coalesced persistence; Render
      math toggle; Math font/size selects with details. (owner:
      reset-notify, writes `src/settings/appearance/layout.rs` new) —
      output: width/math settings work and persist. — check: `cargo test -p
      mono-desk`. — stop_if: `window.on_next_frame` unavailable (fall back
      to per-event persist and record the deviation).
    - T3 — i18n part 4 (fonts/layout/dialog strings) in four locales.
      (owner: finance-ui, writes `src/i18n.rs` only) — output: no hardcoded
      English. — check: i18n completeness test. — stop_if: wording
      decision needed.
  - wave 2:
    - T4 — wire the fonts section + menus into the page (`mod.rs`), register
      any TextField key bindings, and extend selftest with resolved
      families/sizes. (owner: select-stack, writes
      `src/settings/appearance/mod.rs`, `src/main.rs` selftest) — output:
      Fonts and layout renders in zeron's order with working pickers. —
      check: `cargo test -p mono-desk` + appearance selftest. — stop_if: key
      binding registration conflicts with the shell's global bindings.
  - phase check: `cargo test && make check && make lint && make fmt-check`
    + appearance selftest font markers.
- phase_slug: `polish-docs-gates`
  status: checked
  - goal: R10 (+ residual R1-R9 proof) | depends_on: all previous phases
  - surfaces: `docs/PROJECT.md`, `AGENTS.md`, `README.md`,
    `THIRD_PARTY_NOTICES.md`, `docs/memory/appearance-parity.md` (new),
    `src/settings/appearance/*` (dead-code/i18n cleanup only),
    `src/main.rs` selftest final, `vendor/zeron-ui/src/*` headers |
    avoided: behavior changes outside cleanup
  - escalate_when: the final gate fails twice on the same command, or the
    independent judge requests a change that would alter locked requirements.
  - wave 1:
    - T1 — documentation: update `docs/PROJECT.md` non-goals (theme library/
      wallpaper/background now in scope for the Appearance page),
      `AGENTS.md` conventions (new state keys, theme library data dir,
      background media, i18n key families, appearance submodules),
      `README.md` feature list, `THIRD_PARTY_NOTICES.md` (every newly copied
      zeron file), and write
      `docs/memory/appearance-parity.md` (module map + adaptation rules +
      proof gaps). (owner: lead) — output: docs describe the delivered
      surface. — check: docs diff review + `grep -n "theme library" docs/
      PROJECT.md AGENTS.md`. — stop_if: a doc claim contradicts code.
    - T2 — cleanup: remove dead helpers/imports/`#[allow(dead_code)]`
      introduced during phases, collapse duplicated preview helpers, ensure
      every new i18n key is used and every used key exists (scripted check),
      and re-read the page header docs to match the final structure. (owner:
      state-persist) — output: `make lint` clean, no stale comments. — check:
      `make lint && make fmt-check`. — stop_if: cleanup would change
      behavior (move it to a decision entry instead).
  - wave 2:
    - T3 — final verification: run the full gate, capture
      `MONO_DESK_SECTION=appearance MONO_DESK_SELFTEST=1` (light + dark) and
      the runtime window probe; record results in `## Validation`. (owner:
      lead) — output: green gates and recorded evidence. — check:
      `cargo test && make check && make lint && make fmt-check`. — stop_if:
      any command fails (one targeted fix; then blocked entry).
    - T4 — independent `check full` on the final phase with the judge
      (fresh context, read-only): requirements R1-R10 vs the diff; verdict
      and requests recorded, requests resolved or recorded as accepted gaps.
      (owner: glass-arbiter as independent judge) — output: verdict in
      `## Validation`. — check: judge message/record exists and gates still
      green after any fix. — stop_if: verdict `BLOCK` (stop and report).
  - phase check: `cargo test && make check && make lint && make fmt-check`
    rc 0 + driver verdict recorded.

## Log
- 2026-10-30T12:41:26Z — kit-typography — start — lane high-risk;
  branch `feat/appearance-parity` cut from `feat/zeron-stack` @ `0cb89e1`;
  wave 1 = T1 deps (lead), T2 PageScroll/rail (widget-scaffold), T3
  installed-font catalog (state-persist).
- 2026-10-30T12:50:36Z — kit-typography/T1 — done — `fontdb 0.23` /
  `ttf-parser 0.25` / `image 0.25` added to `vendor/zeron-ui/Cargo.toml`,
  `image` + `tempfile` dev-dep to root `Cargo.toml`; check `cargo check` rc 0
  (6.18s, lock resolves the versions already pinned by gpui).
- 2026-10-30T12:50:36Z — kit-typography/T2 — done — ported `PageScroll`
  (+`ScrollRailHost`, `set_list_hovered`, `reset`, rail metrics) and
  `widgets::rail` verbatim from upstream widgets.rs:168-304 (+151/−6) onto
  the vendored popover rail primitives; check `cargo test -p zeron-ui` rc 0
  (37 passed) + `cargo check [-p zeron-ui]` rc 0.
- 2026-10-30T12:50:36Z — kit-typography/T3 — done — vendored typography
  carries zeron's catalog: `UiFontFamily::Installed(String)`,
  `FontAvailability` bundled/installed/fixed-width lists,
  `FIXED_WIDTH_TOLERANCE`, `face_is_fixed_width`,
  `installed_families_with_latin_metrics`, `register_fonts` enumeration
  (+398/−52, 9 tests; boot font scan measured ~0.4s debug on this device);
  check `cargo test -p zeron-ui` rc 0 (46 passed). Adaptation: kept the four
  bundled families as defaults/fallbacks per plan NG7.
- 2026-10-30T12:50:36Z — kit-typography/T4 — done — restored zeron's
  wallpaper tint in the vendored theme (`tint_variant` from upstream
  `settings/wallpaper_colors.rs:43-78`, `for_selection_with_color`,
  `install_selection_with_color`, `install_selection_inner` color parameter;
  `None` path unchanged; 2 tint tests); check `cargo test -p zeron-ui` rc 0
  (48 passed).
- 2026-10-30T12:50:36Z — kit-typography — decision — vendored
  `FontAvailability::fixed_width_choices()` now returns upstream's
  `&[UiFontFamily]` slice instead of an iterator: zero callers existed
  (grep-verified) and the phase-6 page port consumes slices like upstream.
- 2026-10-30T12:50:36Z — kit-typography — decision — T3 deliberately left
  `typography::reinstall_theme` → `Theme::install_preferences` untouched:
  that path drops `variant_id`/`surface_preference` when a family changes,
  and is re-pointed at the full selection in `state-persistence` T2 (per the
  plan's T2 scope), not in this phase.
- 2026-10-30T12:50:36Z — kit-typography — done — phase check `cargo test &&
  make check` rc 0 (mono-desk 58 + zeron-theme 24 + zeron-ui 48 = 130 tests);
  independent phase gate requested (shared task task-11).
- 2026-10-30T13:06:26Z — kit-typography — decision — dependency manifests
  split along crate boundaries: `fontdb 0.23` / `ttf-parser 0.25` /
  `image 0.25` are declared in `vendor/zeron-ui/Cargo.toml` (the typography
  and theme code that consumes them is vendored), while root `Cargo.toml`
  carries `image` for `src/settings/` media work plus `tempfile` as a
  dev-dep. All resolve to versions already in `Cargo.lock` (no new packages
  or majors). The plan's T1 text named only the root manifest; this split is
  the correct crate ownership and is recorded as the adaptation.
- 2026-10-30T13:06:26Z — kit-typography — decision — `register_fonts`
  advertises only bundled families that actually registered (upstream lists
  Geist/GeistMono unconditionally). Keeps mono-desk's existing fallback
  behavior and avoids offering a family that failed to load; the installed
  list, fixed-width classification and fallbacks are upstream's.
- 2026-10-30T13:06:26Z — kit-typography — done — independent gate requests
  (task-11) resolved: T2 rail tests added (`set_list_hovered` transitions,
  `ScrollRailHost` handle/bar wiring, pre-layout `metrics()`-hidden + reset
  safety; 3 new, `cargo test -p zeron-ui` 51 passed); T4 tint path now
  mirrors upstream `theme.rs:1388-1405` (`AccentSelection::ThemeDefault`
  resolution, recorded caller selection, Frosted hover/active/band override)
  with corrected docs and tint tests pinned to the Frosted constants and to
  real surface changes in both appearances; vendored `lib.rs` header
  refreshed. Gates: `cargo test` 133 passed (58 + 24 + 51), `make check`,
  `make lint`, `make fmt-check` rc 0.
- 2026-10-30T13:09:06Z — kit-typography — done — re-check verdict
  APPROVE_WITH_REQUESTS with one remaining doc line; fixed the line
  (`for_selection_with_color` now names `Theme::install_selection_inner`) and
  two doc nits (`lib.rs` future tense for the still-to-land theme-library
  bridge; the rail test comment no longer claims `popover::rail` coverage).
  Final gates: `cargo test` 133 passed (58 + 24 + 51), `make check`,
  `make lint`, `make fmt-check` rc 0; phase-check grep evidence:
  `Installed(` x19 in typography.rs, `pub struct PageScroll` at
  widgets.rs:59, `pub fn install_selection_with_color` at theme.rs:1594.
- 2026-10-30T13:09:06Z — kit-typography — done — phase closed `checked` after
  the independent two-round gate; proof gaps recorded in `## Validation`.
- 2026-10-30T13:09:06Z — state-persistence — start — wave 1 T1 settings
  model + 24-key codec dispatched (shared task task-12, owner
  state-persist); waves 2 queued: T2 appearance domain
  (`src/settings/mod.rs` + `src/main.rs`), T3 theme-library bridge
  (`src/settings/theme_library.rs`), T4 background/wallpaper/wallpaper-color
  helpers.
- 2026-10-30T13:18:44Z — state-persistence/T1 — done — 24 keys appended
  after `timezone` (18 old keys/order untouched; 42 keys + marker = 43 lines)
  with `StoredSettings` mirrors, typed `AppSettings` fields and codecs
  (`theme_light`/`theme_dark`, 3 font families, terminal/code sizes,
  `transcript_width` 560-1200 16px grid, `render_math`, `math_font`,
  `math_size` 90-150, `smooth_scrolling`, `reduce_motion`,
  `pause_in_background`, `wallpaper_theme_colors`, `wallpaper_color`,
  `wallpaper_folder`, `wallpaper_source`, 5 `new_thread_background_*` keys,
  `new_thread_background_effect`), plus background/adjustment/effect/math
  types in `src/app_settings.rs`; check `cargo test -p mono-desk` rc 0 (71
  passed, +13), `make check` rc 0, `make lint`/`make fmt-check` rc 0.
  Surfaces: `src/app_settings.rs` +746, `src/persist.rs` +513/−6 only.
- 2026-10-30T13:18:44Z — state-persistence — decision — `math_size` clamps to
  90-150 (the Appearance page's `MATH_SIZES` ladder) rather than upstream's
  wider settings-store range 75-200; the page can only pick ladder values, so
  the narrower clamp keeps persisted files on selectable values.
- 2026-10-30T13:18:44Z — state-persistence — decision — `wallpaper_color`
  persists as 6-digit uppercase `#RRGGBB` (wallpaper samples are opaque);
  parsing still accepts the wider zeron `Color` forms (`#rgb/#rgba/#rrggbb/
  #rrggbbaa`), and a hand-edited alpha is dropped on the next write.
- 2026-10-30T13:18:44Z — state-persistence — decision — `NewThreadBackground`,
  `NewThreadBackgroundAdjustment`, `NewThreadBackgroundEffect` and `MathFont`
  live in `src/app_settings.rs` (settings-domain types alongside
  `Appearance`/`DigestCadence`), so `background.rs` (T4) imports them instead
  of defining a second copy; T4 was messaged with the correction.
- 2026-10-30T13:18:44Z — state-persistence — start — wave 2 dispatched: T3
  theme-library bridge (task-15, profile-about), T4 background/wallpaper/
  wallpaper-color helpers (task-16, select-stack); T2 (task-17) waits on T3's
  module.
- 2026-10-30T13:28:26Z — state-persistence/T3 — done — ported zeron's
  theme-library bridge into `src/settings/theme_library.rs` (443 lines incl.
  module doc + 6 headless tests; upstream 217 lines): `init(data_dir)`,
  `entries`/`load_warning`/`compile`/`install`/`reload`/`unlink`/
  `duplicate_as_snapshot`/`duplicate_as_editable`/`remove`/`reveal`,
  `mutate`/`persist_and_activate` (upstream failure-atomic semantics) and a
  pure `reconcile(&mut ThemeSelection) -> bool` replacing upstream's
  `appearance::*` calls; `eprintln!` instead of `tracing`; root `anyhow = "1"`
  added (already in lock). Authorized write-scope exception: one line
  `pub(crate) mod theme_library;` in `src/settings/mod.rs`. Check
  `cargo test -p mono-desk` 77 passed (+6), `make check`/`make lint`/
  `make fmt-check` rc 0. Handoff: boot/page must run reconcile → persist
  corrected ids → `apply_appearance` after any library mutation.
- 2026-10-30T13:28:26Z — state-persistence/T4 — done — background/wallpaper/
  wallpaper-color helpers: `src/settings/background.rs` (decode-validate the
  exact bytes, content-hash+sequence+nanos managed file under
  `config_dir()/new-thread-backgrounds/`, 64x64 color sampling, persist
  before retiring the old file, managed-path-only deletion, normalized
  adjustment/effect setters, zeron error strings with brand adaptation),
  `src/settings/wallpaper.rs` (`SUPPORTED_EXTENSIONS`, sorted scan,
  dependency-free random pick, zeron error strings, folder/source persist),
  `src/settings/wallpaper_colors.rs` (`extract` verbatim, decode+thumbnail
  sampling, `active(settings)` gate, `apply_enabled` returning
  `Task<Option<Color>>` for the caller to apply); 15 new tests. Authorized
  write-scope exception: three `mod` declarations in `src/settings/mod.rs`.
  Check `cargo test -p mono-desk` 92 passed (+15), `make check`/`make lint`/
  `make fmt-check` rc 0.
- 2026-10-30T13:28:26Z — state-persistence — decision — T4 trims recorded:
  no uuid (content hash + in-process sequence + unix nanos), no wallpaper
  renderer/preload queue/history (NG4: mono-desk paints no wallpaper), `tif`
  accepted alongside `tiff` (upstream accepts both), no separate
  `tint_variant` copy (vendored theme already applies zeron's tint when given
  a color), and `apply_enabled` persists the toggle/color and hands the
  caller a `Task<Option<Color>>` seam instead of owning the theme reinstall.
- 2026-10-30T13:28:26Z — state-persistence — start — T2 dispatched (task-17,
  state-persist): appearance funnel uses persisted variant ids +
  `install_selection_with_color` + `wallpaper_colors::active`, typography
  before theme, vendored typography clobber removed, boot wiring
  (persisted fonts/motion, `theme_library::init` before first install,
  reconcile persisted variant ids).
- 2026-10-30T13:36:11Z — state-persistence/T2 — done — appearance funnel +
  boot order + typography clobber: `settings::apply_appearance` applies
  typography first (`set_family`/`set_terminal_family`/`set_code_family`/
  `set_terminal_font_size`/`set_code_font_size`/`set_font_size`), then
  installs the theme via `Theme::install_selection_with_color` with the
  persisted per-appearance variant id, accent/surface and the
  `wallpaper_colors::active` tint gate; new pure helpers
  (`typography_seed`, `motion_seed`, `variant_id`, `theme_selection`,
  `appearance_plan`, `reconcile_persisted_selection`, `boot_appearance`) with
  4 tests. Boot order: fonts registered → persisted settings loaded →
  typography init from persisted families/sizes → motion init from persisted
  preferences → `theme_library::init(config_dir)` → reconcile persisted
  variant ids (persist only when changed) → first full theme install.
  Vendored typography: removed `reinstall_theme`'s
  `Theme::install_preferences` rebuild (the variant/surface/accent clobber);
  setters now update `TypographyState` + repaint, with a provenance comment
  that the app owns the reinstall. Selftest markers added: `selftest boot
  stage=…` (7 stages) and `selftest appearance theme_light=… theme_dark=…
  variant=… surface=… wallpaper=… family=… terminal=… code=…
  reduce_motion=… pause=… wallpaper_theme_colors=… background=… effect=…`.
  Check: `cargo test` 171 passed (96 + 24 + 51, +4 mono-desk),
  `make check`/`make lint`/`make fmt-check` rc 0, selftest rc 0 with the
  stage order and persisted values reported.
- 2026-10-30T13:36:11Z — state-persistence — done — phase tasks complete;
  independent phase gate requested (shared task task-18).
- 2026-10-30T13:36:11Z — state-persistence — decision — the pre-window
  install no longer uses `Theme::install_preferences`; it installs the full
  reconciled persisted selection through `settings::install_appearance`, and
  the boot appearance resolves as env pin → persisted Light/Dark → OS for
  `System` (`settings::boot_appearance`), so the first frame matches the
  persisted choice; main's old `forced_appearance`/`desired_appearance`
  helpers were removed and `MONO_DESK_APPEARANCE` preserved via
  `settings::pinned_appearance`.
- 2026-10-30T13:36:11Z — state-persistence — decision — the theme-library
  bridge and the background/wallpaper APIs carry module-level
  `#![allow(dead_code)]` until the Appearance page consumes them (phases 3-4);
  every other gate (`make lint -D warnings`) is green with those allows.
- 2026-10-30T14:02:00Z — state-persistence — decision — gate round 1 returned
  BLOCK on two defects, both fixed before close: (B1) the boot-seeded
  typography state made `typography::set_font_size` early-return, so the
  window kept GPUI's 16px rem and the selftest measured state instead of the
  window — `settings::apply_appearance` now writes
  `window.set_rem_size(px(seed.ui_size.pixels()))` unconditionally and the
  selftest prints `rem_state`/`rem_window` (default 14/14; crafted-HOME
  12/18/20 windows track the persisted size); (B2) `background::install/
  remove` discarded `persist::save`'s bool and retired managed files anyway —
  both now snapshot `PreviousState`, call `persist::save` directly, restore
  state + discard only the staged copy on failure, return `ERROR_SAVE`/
  `ERROR_REMOVE_SAVE`, and retire files only after the durable save (4 new
  failure/success tests). Also recorded: boot order is typography → motion →
  theme_library → reconcile → install (plan T2 text said library first; the
  landed order still puts the library before reconcile/first install, so it is
  functionally equivalent); `apply_registry_change` is substituted by pure
  `reconcile_persisted_selection` + `apply_appearance` (no forced-generation
  consumer exists yet); `remove` with no installed background is now the
  upstream clone-no-op `Ok(())` (does not clear stale source/color).
- 2026-10-30T14:02:00Z — state-persistence — done — phase closed `checked`
  after the round-2 APPROVE (task-22); Validation entry recorded.
- 2026-10-30T14:02:00Z — page-base-color-motion — start — fan-out wave:
  T1 page scaffold/Color scheme/Motion (task-19, widget-scaffold) and T2 i18n
  part 1 (task-20, finance-ui) dispatched; the Lead owns T3 shell/main
  wiring + the phase check; phases 4-6 are queued to run in parallel once T1
  lands (their plan dependencies only require this phase + the closed kit).
- 2026-10-30T14:02:00Z — page-base-color-motion/T2 — done — i18n part 1: 45
  new `appearance.*` keys in en/vi/zh-CN/ja (+352 lines; table 128 → 173
  keys, appearance 19 → 64, no duplicates) covering theme-selector headings,
  accent/wallpaper-colors copy, Glass surface helper+option labels, New thread
  background meta/actions, Wallpaper folder meta/actions, Background effect
  labels+descriptions, and the reduce/pause/smooth helper sentences; test
  `appearance_keys_are_translated_in_every_locale` asserts every
  `appearance.*` key is non-empty in all four locales. Adaptations: pause
  helper brands "mono-desk"; technical effect terms stay English in vi;
  dialog/font/library strings are deferred to i18n parts 2-4. Check:
  `cargo test -p mono-desk` 101 passed, `make check`/`make lint`/
  `make fmt-check` rc 0.
- 2026-10-30T14:04:31Z — page-base-color-motion/T1 — done — rebuilt the
  Appearance page as `src/settings/appearance/{mod,color,motion,preview}.rs`
  (1,053 lines) replacing the old 708-line `appearance.rs`: host
  `#appearance-page-host` (relative, list-hover hook) → `edge_faded(16,true,
  true)` → `#appearance-page` scroller on the ported `PageScroll` +
  `popover::rail`; section order header → Color scheme (3 mode cards,
  mt24/gap12) → color card (Light/Dark theme selectors with registry palette
  previews + headings + 218/260 widths, Accent row with dynamic helper +
  wallpaper-colors notice, Match wallpaper colors toggle) → Motion (Reduce
  motion 128 + dynamic helpers, Pause in background, Smooth scrolling).
  Handlers persist then funnel through `settings::apply_appearance`; variant
  picks write `theme_light`/`theme_dark`; the wallpaper-colors toggle awaits
  `wallpaper_colors::apply_enabled` and re-applies the funnel;
  `AppearancePage::new()` preserved so `src/shell.rs` is untouched (T3 is
  therefore a no-op wiring + the existing phase-2 selftest markers). 5 new
  page tests replace the old page's tests. Check `cargo test` 177 passed
  (102+24+51), `make check`/`make lint`/`make fmt-check` rc 0, appearance
  selftest rc 0 (`selected=Appearance`, no panic, `rem_state=14
  rem_window=Some(14.0)`).
- 2026-10-30T14:04:31Z — page-base-color-motion — decision — deferred to the
  parallel phases 4-6 by plan order: Material/background section (phase 4),
  Theme library card (phase 5), Fonts and layout section (phase 6);
  `surface: SelectState` is intentionally reserved with a dead-code allow and
  a consumer-phase comment. The 45 part-1 i18n keys cover only the built
  sections; parts 2-4 are queued with their phases.
- 2026-10-30T14:04:31Z — page-base-color-motion — done — T1/T2/T3 complete;
  independent phase gate requested (task-23). Fan-out for phases 4-6 will
  start right after the verdict (Lead writes the shared `mod.rs` skeleton so
  the three section owners only write their own module files).
- 2026-10-30T14:16:45Z — page-base-color-motion — done — gate requests
  resolved: (R1) the appearance selftest marker now prints
  `smooth_scrolling={}` after `pause={}` plus the persisted accent word
  (`accent_word=#2563EB` on the owner's file, `theme` when unset), so R4's
  "selftest prints the three values" holds; (R2) mode-card/selection ids now
  use zeron's label-cased locale-stable words (`appearance-System`,
  `appearance-mode-System-selection`; upstream :3566-3580); (R3) added the
  `accent_helper` test covering ThemeDefault + every preset in all four
  locales, and recorded the deliberate exceptions (ReduceMotion option words
  stay vendored English "System/On/Off" — part-2 i18n candidate; the old
  page's Glass row and interface font-size row are intentionally absent until
  phases 4 and 6). Gates: `cargo test -p mono-desk` 103 passed,
  `make check`/`make lint`/`make fmt-check` rc 0, appearance selftest rc 0
  with the new marker.
- 2026-10-30T14:16:45Z — page-base-color-motion — done — phase closed
  `checked` after the round-2 APPROVE_WITH_REQUESTS; Validation entry
  recorded. Independent gate: task-23.
- 2026-10-30T14:16:45Z — material-background — start — fan-out wave for the
  remaining phases: the Lead writes the shared `src/settings/appearance/
  mod.rs` skeleton (per-section stub modules + state fields, split dialogs
  into `dialogs/background.rs` and `dialogs/library.rs`), then spawns three
  parallel owners (material+background dialog, theme-library card+import/
  review dialogs, fonts/layout) plus a single i18n writer for parts 2-4;
  phase-3 files stay untouched by them.
- 2026-10-31T03:30:25Z — material-background/T1-T4 — done — host interruption
  killed the first fan-out run at the host level (no files changed for ~12h);
  the wave was re-dispatched and all four remaining phase owners landed
  their scopes in parallel on the Lead's skeleton:
  `material.rs` (544 lines: Glass 148 + helper, New-thread-background 36px
  tile + meta states + Choose/Replace/Remove/Adjust via `prompt_for_paths` +
  `settings::background`, conditional Background effect 128, Wallpaper
  folder + Shuffle via `wallpaper::{pick_random,record_source}`; inline
  error strip instead of zeron's toast — NG5), `dialogs/background.rs` (954
  lines: 260px preview well, crosshair guides, pan/zoom/pinch, keyboard
  arrows/±/Esc/ModEnter/Tab, 168x28 zoom slider + %, Reset/Cancel/Apply,
  guarded apply → `background::set_adjustment`), and the vendored
  `new_thread_background_effects.rs` (415) +
  `new_thread_background_mask.rs` (433) exposing `prepare`,
  `paint_adjusted`, `pan_adjustment`, `zoom_adjustment_around`,
  `Framing`/`BackgroundEffect` mirrors (composer/hero/preload pipelines
  trimmed with provenance notes) + 12 ported geometry/effect tests.
- 2026-10-31T03:30:25Z — theme-library/T1-T4 — done — `library.rs` (353
  lines: `.mt(32)` card, Theme library + Add theme, Imported/Linked groups,
  entry status + six actions, warning line; mutations run
  `theme_library::*` then `reconcile_persisted_selection` → persist →
  `apply_appearance` — the recorded `apply_registry_change` substitution) and
  `dialogs/library.rs` (1332 lines: 600px import dialog with lazy TextField
  source + Browse, copy/link radios, compile + detected-themes list with
  palette/scene preview + report panel, selection, Analyze/Import footer,
  Esc/Enter, click-out; 660px review dialog; ported
  `import_scene_preview`/`report_panel`/`source_name`/`slug`; 3 pipeline
  tests). Adaptations: dialog opens via `State::open(cx)` because
  `dialogs::library` is private to the Lead-owned dispatcher; previews paint
  from `ThemeVariant` model roles because `Theme::from_variant` is
  crate-private; Escape/Enter are card-scoped (shell-level
  `dismiss_on_escape` not wired).
- 2026-10-31T03:30:25Z — fonts-layout/T1-T4 — done — `font_model.rs` (474
  lines: FontKind + slug/label/description, terminal fixed-width narrowing,
  requested/effective/apply accessors, UiFontSize + MONO_FONT_SIZES +
  MATH_SIZES ladders, step/filter/first/last/fallback/format helpers; 8
  tests) and `fonts.rs` (704 lines: three Interface/Terminal/Code rows with
  220px searchable family menus — lazy TextField filter, `popover::picker_row`
  list, `widgets::rail` + `PageScroll`, capture-phase ↑/↓/Home/End/Enter/Esc,
  400ms reopen guard, outside-press dismiss, dimmed unavailable rows — plus
  128px size selects, persist + `persist_now` + `apply_appearance`, and the
  unavailable-font error strips); `layout.rs` (663 lines: Conversation width
  row 240x28 with hover/press/keyboard details + Reset + per-frame
  `on_next_frame` coalescing + mouse-up flush, Render math toggle, Math font
  160/220 with localized style details, Math size 160 over the 90-150 ladder;
  5 tests). Host `on_mouse_up_out` is not wired (Lead-owned mod.rs): the
  pending frame still publishes the value, recorded as a non-loss deviation.
- 2026-10-31T03:30:25Z — i18n parts 2-4 (task-28) — done — `src/i18n.rs`
  +745 lines this pass (table 173 → 278 keys, `appearance.*` 64 → 168, no
  duplicates), keys reconciled against the parallel owners' live call sites:
  adjustment-dialog copy, background/wallpaper/library error sentences,
  library card + import/review + report fragments, fonts rows/pickers,
  layout/math copy, `unit.px`; completeness test extended to `appearance.*`
  and `unit.*`; mismatched names were resolved by adopting the call-site
  names and deleting duplicate bridge keys.
- 2026-10-31T03:30:25Z — material-background/theme-library/fonts-layout —
  decision — aggregate cleanup after the wave: domain action errors from
  `settings::{background,wallpaper,theme_library}` are mapped through the new
  `appearance.background.error.*`/`appearance.wallpaper_folder.error.*`/
  `appearance.library.error.*` keys at the page boundary (unknown I/O
  diagnostics stay raw — developer-facing edge cases, recorded); the
  now-consumed reserved `AppearancePage.surface` field and all
  `#[allow(dead_code)]` markers under `src/settings/appearance/` were removed;
  the unused `background_dialog_path` helper was deleted.
- 2026-10-31T03:30:25Z — material-background/theme-library/fonts-layout —
  done — aggregate gates on the combined tree: `cargo test` 206 passed
  (119 mono-desk + 24 zeron-theme + 63 zeron-ui, 0 failed);
  `make check`/`make lint`/`make fmt-check` rc 0; appearance selftest rc 0
  with `selected=Appearance`, the appearance/rem markers and no panic.
  Combined independent phase gate requested for phases 4-6 (file-scoped
  verdicts, one judge run).

- 2026-10-31T03:45:01Z — material-background/theme-library/fonts-layout —
  done — combined independent gate (task-29): `fonts-layout` APPROVE;
  `material-background` and `theme-library` APPROVE_WITH_REQUESTS. Requests
  fixed in the same session: Glass row now uses zeron's `surface_helper`
  copy via `appearance.surface.frosted`/`.opaque`; the library warning moved
  to the page-column tail after the Fonts section (upstream :4272-4280, new
  `library::warning`); the import path maps known bridge save/initialization
  words through `appearance.library.error.*` (`bridge_error_text`), with
  importer compile diagnostics staying raw per the recorded decision; the
  empty-selection copy says "variant" in all four locales; and the selftest
  gained `selftest library entries=N warning=…` (default
  `entries=0 warning=false`). Gate re-run: `cargo test` 206 passed,
  `make check`/`make lint`/`make fmt-check` rc 0, appearance selftest rc 0.
  All three phases closed `checked`.
- 2026-10-31T03:45:01Z — polish-docs-gates — start — final phase: docs
  (THIRD_PARTY_NOTICES paths + vendored modules, PROJECT non-goals,
  AGENTS conventions, README, new `docs/memory/appearance-parity.md`),
  cleanup (stale helper-module `#![allow(dead_code)]`, stale headers, i18n
  unused keys + the pre-existing `item.channels`/`item.digest` raw-key gap,
  ReduceMotion option words, no-op dispatcher), then the full gate and the
  independent `check full`.
- 2026-10-31T03:48:50Z — polish-docs-gates/T1 — done — docs refreshed:
  `THIRD_PARTY_NOTICES.md` names the `src/settings/appearance/` module tree
  (`{mod,color,motion,preview,material,library,fonts,font_model,layout}` +
  `dialogs/{mod,background,library}`) and the vendored wallpaper/
  wallpaper_colors/theme_library/new-thread-background-{effects,mask,image}
  sources, with the zeron-stack plan path moved to `completed`;
  `docs/PROJECT.md` non-goals updated (theme library/import, background
  image/effect and wallpaper folder rows are ported; still no wallpaper
  painting layer, transcript/composer surface, cloud sync or search anchors);
  `AGENTS.md` gained the appearance-page and i18n convention bullets, the
  extended selftest description and updated references; `README.md`
  features/persistence text matches the page; new
  `docs/memory/appearance-parity.md` records the module map, adaptation
  rules and verification hooks.
- 2026-10-31T03:48:50Z — polish-docs-gates/T2 — done — cleanup: removed the
  stale module-level `#![allow(dead_code)]` from
  `src/settings/{background,wallpaper,wallpaper_colors,theme_library}.rs`
  (all consumers landed; the one remaining allow documents the
  upstream-parity `duplicate_as_snapshot`, which zeron's page does not
  expose), deleted the no-op `dialogs::library::mouse_up` and the unused
  `background_dialog_path`, refreshed the `appearance/mod.rs` section map and
  the theme-library consumer comments, pruned five unused new `appearance.*`
  i18n keys, added the previously-missing `item.channels`/`item.digest` keys,
  and localized the ReduceMotion option words
  (`appearance.motion.system/on/off`).
- 2026-10-31T03:48:50Z — polish-docs-gates/T3 — done — final gates on the
  closing tree: `cargo test` 206 passed (119 + 24 + 63),
  `make check`/`make lint`/`make fmt-check` rc 0; appearance selftest rc 0
  with nav/appearance/library/rem markers and no panic. Independent
  `check full` requested (task-30) before the plan moves to
  `docs/plans/completed/`.

- 2026-10-31T03:52:00Z — polish-docs-gates — decision — phase-7 T1's
  "vendored vim-style adaptation note" clause has no counterpart in this
  repository: no vim-style code, module or attribution exists in zeron's
  pinned source at this rev or in mono-desk, so there was nothing to add to
  `THIRD_PARTY_NOTICES.md`; the T1 text now omits the clause. The optional
  pre-existing `appearance.cycle` i18n key stays (unused since before this
  feature; pruning it is unrelated to the parity port) — recorded instead of
  sweeping it into this change.

## Validation
- 2026-10-30T13:06Z — phase `kit-typography` — verdict:
  APPROVED_WITH_REQUESTS → requests resolved — mode: gate, independent judge
  (high-risk lane)
  - judge: independent subagent `glass-arbiter` (separate context, read-only);
    two rounds — task-11 full gate, task-14 re-check of the fixes. Graded the
    working tree on branch `feat/appearance-parity` (base `0cb89e1`).
  - round 1 requests: (1) missing T2 rail/hover unit test; (2) tint path
    omitted upstream's Frosted hover override (`element_hover/active/band`);
    (3) accent precedence + docs. All three fixed before phase close;
    round 2 confirmed 1-2 resolved and 3 functionally resolved, with one doc
    line left — corrected afterwards (judge deemed a third round
    unnecessary for a comment edit).
  - `cargo test` rc 0 — 133 passed, 0 failed: mono-desk 58, zeron-theme 24,
    zeron-ui 51 (48 + 3 rail tests) — pass
  - `make check` / `make lint` / `make fmt-check` rc 0 — pass
  - phase-check grep: `Installed(` x19 in `vendor/zeron-ui/src/typography.rs`;
    `pub struct PageScroll` in `settings/widgets.rs:59`;
    `pub fn install_selection_with_color` in `theme.rs:1594` — pass
  - surfaces: root `Cargo.toml`, `vendor/zeron-ui/Cargo.toml`,
    `Cargo.lock`, `vendor/zeron-ui/src/{lib,theme,typography}.rs`,
    `vendor/zeron-ui/src/settings/widgets.rs` — on target; no `src/`,
    `docs/`, or `assets/` edits — pass
  - recorded deviations: deps split across root/vendor manifests (T1 task
    text named only the root; Log decision); bundled families registered
    first and only advertised when registered (Log decision + NG7);
    `fixed_width_choices()` takes upstream's slice shape (Log decision);
    `reinstall_theme` clobber deferred to `state-persistence` T2 (Log
    decision).
  - proof gaps (honest, carried): no pixel evidence for a wallpaper-tinted
    Frosted theme (owner visual pass required); boot font enumeration against
    a live text system unasserted (helpers/fixtures only); `popover::rail`
    host pointer listeners untested (pre-existing popover tests cover the
    scrollbar state machine only); `install_selection_with_color`'s App-level
    install path untested; the page itself remains unported until phases 3-6.
  - rollback_point: `feat/zeron-stack` @ `0cb89e1`; phase changes are an
    uncommitted working tree (no commit made — git only on owner request).

- 2026-10-30T14:02Z — phase `state-persistence` — verdict: APPROVED (one
  BLOCK loop) — mode: gate, independent judge (high-risk lane)
  - judge: independent subagent `glass-arbiter` (separate context, read-only);
    round 1 task-18 = BLOCK (B1 window rem at boot, B2 unchecked persist before
    managed-file retirement), round 2 task-22 = APPROVE after the fixes.
  - round-1 findings fixed: (B1) `settings::apply_appearance` sets the window
    rem unconditionally after the typography setters, mirroring upstream
    `lib.rs:490`; the selftest now prints `rem_state` and the real
    `rem_window` read from the window (`AnyWindowHandle::update`). (B2)
    `background::install`/`remove` snapshot the replaced state, call
    `crate::persist::save` directly, restore state + discard only the staged
    copy on failure, return `ERROR_SAVE`/`ERROR_REMOVE_SAVE`, and retire
    managed files only after a durable save; 4 failure/success tests.
  - round-2 gates: `cargo build` rc 0; `MONO_DESK_SELFTEST=1` rc 0 with the
    7-stage boot order and `rem_state=14 rem_window=Some(14.0)`; crafted-HOME
    boots with `font_size` 12/18/20 show `rem_window` 12/18/20; `cargo test`
    rc 0 — 176 passed (101 mono-desk + 24 zeron-theme + 51 zeron-ui);
    `make check`/`make lint`/`make fmt-check` rc 0; `git diff --check` clean.
  - surfaces: `src/app_settings.rs`/`src/persist.rs` (24 keys, 42-key frozen
    order), `src/main.rs`/`src/settings/mod.rs` (funnel + boot order + rem),
    `src/settings/theme_library.rs` (new bridge), `src/settings/
    {background,wallpaper,wallpaper_colors}.rs` (new helpers), vendored
    `typography.rs` reinstall-path removal, root `anyhow` — on target; no
    `src/shell.rs`/pages/docs/`vendor/zeron-theme` edits.
  - recorded decisions: `math_size` clamps to the page ladder 90-150;
    `wallpaper_color` persists 6-digit uppercase (alpha accepted on read,
    dropped on write); background/adjustment/effect/math types live in
    `src/app_settings.rs`; boot order typography → motion → library →
    reconcile → install (plan text variant recorded); `apply_registry_change`
    substituted by `reconcile_persisted_selection` + `apply_appearance`;
    `remove` with no background is the upstream clone-no-op.
  - proof gaps (honest): no pixel/visual evidence (owner pass required); the
    rem fix is proven by the selftest window read + crafted-HOME boots, not a
    widget unit test; the B2 save gate is tested with an injected save result
    (production wiring code-reviewed); real custom-family boot activation is
    not exercised end-to-end; `apply_enabled` async sampling and real-photo
    decode untested; theme re-apply after background/wallpaper ops is the
    phase-4 page caller's contract.
  - rollback_point: branch base `feat/zeron-stack` @ `0cb89e1`; uncommitted
    working tree (no commit made).

- 2026-10-30T14:16Z — phase `page-base-color-motion` — verdict:
  APPROVED_WITH_REQUESTS → requests resolved — mode: gate, independent judge
  (high-risk lane)
  - judge: independent subagent `glass-arbiter` (separate context, read-only);
    task-23. Graded the phase-3 delta on branch `feat/appearance-parity`.
  - PASS: scaffold/section order/layout numbers vs upstream :4185-4289,
    :3562-3704, :3958-4083; theme selectors from the active registry with
    palette previews + 218/260 widths + headings and persisted variant ids
    funnelled through `apply_appearance`; accent row dynamic helper + wallpaper
    notice; Match-wallpaper toggle awaiting `wallpaper_colors::apply_enabled`;
    Motion three rows with upstream helpers/ids/aria and runtime calls;
    previews verbatim; 5 new page tests; i18n 45 keys × 4 locales with genuine
    translations; scope delta limited to the deleted page file, the new module
    and `src/i18n.rs`.
  - requests resolved: (R1) selftest marker gained `smooth_scrolling` (and the
    persisted accent word), evidence
    `… pause=false smooth_scrolling=true … accent_word=#2563EB`; (R2) mode ids
    now label-cased (`appearance-System` etc., locale-stable); (R3) Log
    decisions recorded for the English ReduceMotion option words and the
    phases-4/6-deferred Glass/font-size rows, plus the new `accent_helper`
    test (R2 acceptance).
  - gates after the fixes: `cargo test -p mono-desk` 103 passed (+1 helper
    test), `make check`/`make lint`/`make fmt-check` rc 0, appearance
    selftest rc 0 with the new marker; workspace totals at gate time were
    102+24+51 = 177 (the +1 accent test landed after the judge's run).
  - surfaces: `src/settings/appearance/{mod,color,motion,preview}.rs` (new
    directory module, old `appearance.rs` deleted), `src/i18n.rs` (+352). No
    `src/shell.rs`/`src/main.rs`/vendor changes in this phase (the main.rs
    marker edit is the gate-request fix).
  - proof gaps: no pixels (owner visual pass required); no widget-level
    interaction tests; async wallpaper sampling untested end-to-end;
    custom-library selector variants unexercised until phase 5; the
    `language=vi`/dark-appearance render was evidenced by a crafted-HOME
    selftest (no panic), not by string assertions.
  - phase-7 note: `THIRD_PARTY_NOTICES.md:16-18` still points at the deleted
    `src/settings/appearance.rs` path — the docs phase must correct it.
  - rollback_point: branch base `feat/zeron-stack` @ `0cb89e1`; uncommitted
    working tree (no commit made).

- 2026-10-31T03:45Z — phases `material-background`, `theme-library`,
  `fonts-layout` — verdicts: APPROVE / APPROVE_WITH_REQUESTS (requests
  resolved) / APPROVE — mode: combined gate, independent judge (high-risk
  lane)
  - judge: independent subagent `glass-arbiter` (separate context, read-only);
    task-29, one run with per-phase verdicts on branch `feat/appearance-parity`.
  - PASS evidence: row/dialog/anatomy parity vs upstream line maps
    (material :3706-3957 + dialog :2747-3265 + vendored effects/mask ports
    with trims and 12 tests; library :3321-3543 + import :2205-2746 + review
    :3267-3319 + failure-atomic bridge; fonts/layout :4094-4182, :1958-2142,
    :515-678, :398-513 with 8+5 model/helper tests); section order and modal
    priority match :4238-4281/:4089-4092; i18n parts 2-4 pure additive
    (173→278 keys, all four locales, no used-but-missing wave keys); no
    filesystem writes from the page; scope limited to the declared files.
  - requests resolved before close: Glass `surface_helper` copy; library
    warning position + import save-word mapping + "variant" wording;
    `selftest library entries=N warning=…` marker added.
  - gates after fixes: `cargo test` 206 passed (119 + 24 + 63); `make check`/
    `make lint`/`make fmt-check` rc 0; appearance selftest rc 0
    (`selected=Appearance`, `library entries=0 warning=false`,
    `rem_state=14 rem_window=Some(14.0)`); crafted-HOME judge runs exercised
    installed-background rows+tint, corrupt-library and dark/vi renders
    without panic.
  - recorded adaptations: settings service → `AppSettings` funnel; toast →
    inline strip; ComposerInput → `TextField`; `Theme::from_variant`
    crate-private → model-role previews; vendored `Framing`/`BackgroundEffect`
    mirrors; trimmed composer/hero pipelines; card-scoped Escape; no
    `on_mouse_up_out` (next-frame publish keeps the width).
  - proof gaps: no pixels (owner visual pass required); the two modals and
    the effect paint path need a click/installed image so remain
    code-reviewed + geometry-tested only; library-with-entries, real import/
    registry activation, real-photo decode/effects and async wallpaper
    sampling are in-process/unit-proven only.
  - rollback_point: branch base `feat/zeron-stack` @ `0cb89e1`; uncommitted
    working tree (no commit made).

- 2026-10-31T03:52Z — phase `polish-docs-gates` — verdict:
  APPROVE_WITH_REQUESTS → close chores resolved — mode: check full, independent
  judge (high-risk lane)
  - judge: independent subagent `glass-arbiter` (separate context, read-only);
    task-30, one full-feature run on branch `feat/appearance-parity`.
  - PASS evidence: gates re-run by the judge (`cargo build` + appearance
    selftest rc 0 with all markers incl. `selftest library entries=0
    warning=false` and `rem_state=14 rem_window=Some(14.0)`, boot order;
    `cargo test` 206 = 119 + 24 + 63; `make check`/`lint`/`fmt-check` rc 0;
    `git diff --check` clean); whole-feature review against zeron's
    `appearance.rs` @ 8b7afb3 (assembly :4238-4281, modal priority
    :4089-4092, R1-R10 met or recorded, NG1-NG7 honored); persistence
    surface unchanged in phase 7 (42-key frozen test still green); i18n 278
    unique keys, no used-but-missing keys, four-locale coverage, pruned keys
    confirmed unused; docs/attribution/memory map accurate; no pixel claims.
  - requests resolved before close: (1) `docs/PROJECT.md` current-work
    section rewritten (no nonexistent active plan; points at the two
    completed plans + residual proof gaps); (2) plan moved to
    `docs/plans/completed/appearance-parity.md` with all active→completed
    references updated (`AGENTS.md`, `THIRD_PARTY_NOTICES.md`,
    `docs/memory/appearance-parity.md`; plus the pre-existing stale
    `docs/design/README.md` / `docs/memory/zeron-stack-port.md` references);
    (3) Log decision recorded for the inapplicable "vim-style adaptation
    note" T1 clause (no such code/attribution exists in this repo).
    Optional unused `appearance.cycle` key kept deliberately (pre-existing,
    unrelated — recorded in Log).
  - proof gaps (unchanged and honest): no pixels — the owner supplies the
    visual pass; the two modals and the background-effect paint path need a
    click/installed image so remain code-/geometry-tested only;
    library-with-entries, import/registry activation, real-photo decode and
    async wallpaper sampling are in-process/unit-proven; no GPUI
    widget-interaction harness.
  - rollback_point: branch base `feat/zeron-stack` @ `0cb89e1`; uncommitted
    working tree (no commit made — owner instruction).

## Current State and Next Action
- active_phase: none — `polish-docs-gates` is checked and the whole
  `appearance-parity` plan is complete; this file moved to
  `docs/plans/completed/appearance-parity.md`.
- lifecycle_status: complete
- blockers: none
- open_items: none for this plan. Residual proof gaps are listed in the
  phase-7 Validation entry and `docs/memory/appearance-parity.md`. No git
  commit/push/PR was made (owner instruction; branch
  `feat/appearance-parity`, base `0cb89e1`, working tree uncommitted).
- exact_next_action: report the completion summary to the owner; commit only
  when the owner explicitly asks.
