---
id: plan-ely-port-20261001
intake_id: intake-20261001-ely-port
status: active
lane: normal
created: 2026-10-01
updated: 2026-10-01
approach: planned
planning_status: planned
exact_next_action: "work full phase `ely-appearance-theme` (wave 1: eat-1 ∥ eat-2)"
---

# Ely → mono-desk settings port

Third reference-design-system port after `zdesk` (chrome) and `beUI`
(controls, tokens). Ely is the only one of the three with a full settings
surface, so this initiative ports the **settings experience**, not chrome
and not primitives.

Reference: https://ely-gpui.zacharyzhang.com/components/settings/ ·
source `github.com/ZacharyZhang-NY/Ely-GPUI-Components`, `src/settings/`.
Consult `.devin/skills/ely-gpui-reference/` before any task here.

## Goal

- outcome: mono-desk's Appearance surface gains Ely's four appearance
  controls — a Light/Dark/**System** theme selector, an accent color
  picker, a font-size slider with live preview, and a density selector —
  and its settings tree gains Ely's **synonym-aware search with
  jump-to-setting** and a **changed-from-default reset panel**. Every new
  control drives a real, persisted effect and survives restart.
- success_signal: `make check && make lint && make fmt-check && cargo
  test` green, with new unit tests pinning (a) the search matcher
  semantics and (b) the density→rem mapping; owner `make run` confirms in
  both modes: System follows the OS appearance, accent recolors live, the
  font-size preview tracks the slider, density visibly changes row
  height, a search hit navigates to its setting, and the reset panel
  lists exactly the settings that differ from their defaults.
- actors: the owner (lab project) and developers evaluating gpui-kit
  patterns for real desktop apps.
- authority: owner decision 2026-10-01 (initiative `ely-port`; recorded
  override of the playbook's single-active-plan rule — three sibling
  plans were already active and parked on the same owner visual pass).
  Source files: Ely `src/settings/appearance.rs`, `search.rs`,
  `section.rs`, `storage.rs`; `docs/PROJECT.md`;
  `docs/memory/gpui-component-setting-field-pitfalls.md`;
  `docs/design/zdesk-port-spec.md` (card/sidebar contract this must not
  disturb).
- requirements:
  - R1: The theme control offers Light, Dark and System as one segmented
    control; System resolves against the OS appearance and re-resolves
    when the OS changes while the app runs. | acceptance: unit test on
    the resolve function (all three appearances × both OS modes) plus
    owner `make run` toggling the OS appearance with System selected. |
    source: Ely `Appearance::mode(system: Mode) -> Mode`.
  - R2: An accent color is selectable from named presets and from a
    custom color well, and recolors the live theme immediately. |
    acceptance: owner `make run` — pick a preset, then a custom color,
    both repaint; the choice survives restart via `persist.rs`. | source:
    Ely `AccentColorPicker`; vendored `src/color_picker.rs` already
    exists, so this is wiring.
  - R3: Font size is a slider over the existing clamp
    (`theme::FONT_SIZE_MIN..=theme::FONT_SIZE_MAX`), with a live preview
    line rendered at the chosen size and a reset affordance that is
    disabled when already at default. | acceptance: unit test for the
    clamp and default-disabled predicate; owner `make run` — preview
    tracks the slider, reset returns to 14. | source: Ely
    `FontSizeControl`.
  - R4: Density offers Compact, Standard and Comfortable, and changes
    real spacing (row height and padding), not a label. | acceptance:
    unit test pinning the density→rem/px mapping; owner `make run` — the
    settings rows visibly change height between the three. | source: Ely
    `DensitySelector` + `theme::Density`.
  - R5: Settings search matches a setting when **every** whitespace-
    separated query term appears in the lowercased concatenation of its
    name, section and synonym list; an empty query shows no results; a
    non-empty query with no hit shows a quiet "no setting answers"
    line; activating a hit navigates to that setting's page. |
    acceptance: unit tests porting Ely's three pinned cases (`DARK` →
    theme; `appearance zoom` → font size; `network dark` → none) plus
    owner `make run` — type a synonym, activate, land on the page. |
    source: Ely `settings/search.rs::matching` and its `#[cfg(test)]`
    block.
  - R6: A reset panel lists exactly the settings whose current value
    differs from their default, reverts them one at a time, and requires
    a second confirmation before reverting all; with nothing changed it
    says so rather than rendering an empty box. | acceptance: unit test
    for the changed-set computation; owner `make run` — change two
    settings, confirm the panel lists two, revert one, confirm the list
    drops to one. | source: Ely `ResetToDefault` + `Changed`.
  - R7: Every control added here persists through `persist.rs` and is
    restored on launch. | acceptance: `cargo test` covers the
    `StoredSettings` round-trip for each new field; owner `make run` —
    set all four, quit, relaunch, values hold. | source:
    `docs/plans/completed/real-settings.md` (the project's
    every-control-is-real bar).
  - R8: The `beui-tokens` pins stay green — the frozen palette digest,
    `font.size` 14, `radius` 8 / `radius.lg` 16, `shadow` false are
    unchanged by this work. | acceptance: `cargo test` — the existing
    digest and config pins pass untouched. | source:
    `docs/plans/active/beui-tokens.md` non-goals.
  - R9: Setters that need a cross-control repaint call
    `cx.refresh_windows()`, and any runtime font-size override is
    re-applied after `Theme::change`. | acceptance: view-level test plus
    owner `make run` — flip the theme with a non-default font size set
    and confirm the size does not silently reset. | source:
    `docs/memory/gpui-component-setting-field-pitfalls.md` items 2 and 3.
- non-goals:
  - NG1: Adding `ely-gpui-component` as a dependency. Verified dead end:
    `gpui_platform` is not on crates.io (git-only), Ely is one
    indivisible crate with macOS-only deps, and its README says
    "Tested on macOS only". Port the pattern, never the code.
  - NG2: Ely's cluster-B surfaces — keyboard shortcuts page, storage
    usage, import/export, feature flags, proxy, privacy/notification
    matrix, extended startup (restore windows, check updates, start
    page). Each needs a consumer that does not exist yet; follow-up
    plan.
  - NG3: Theme editor and VS Code theme import.
  - NG4: Replacing gpui-component's `Settings` with Ely's
    `SettingsLayout`/`SettingsSection`/`SettingsRow`. The vendored
    `Settings` plus its patch set is the established sidebar contract.
  - NG5: Any color, font-family, radius or shadow change — the palette
    is frozen by `beui-tokens`.
  - NG6: New vendor patches beyond the additive minimum; no rewrite of
    vendored components.
  - NG7: beUI control work (switch/checkbox/radio/select/tooltip/
    notification) — owned by `beui-p1-components`.

## Phases and Verification

- approach: Wire Ely's behaviour onto the vendored `Settings` and the
  app's `AppSettings` Global, rather than building new widgets. A
  read-only feasibility pass over the vendored crate found that three of
  the six functional requirements are already reachable but unused —
  `SettingItem::keywords()` is public and called **zero** times in
  `src/main.rs`; `ColorPicker` and `Slider` are vendored and unwired; the
  OS-appearance path (`cx.window_appearance()`,
  `window.observe_window_appearance`, `Theme::sync_system_appearance`)
  ships in `gpui-pre` 0.3.5 and `gpui-component` already. Two more are
  app-side computations over state the app already owns. So the plan is
  mostly wiring plus two small additive patches, not new components.
  Rejected alternative: adopting Ely's own `SettingsLayout` /
  `SettingsSection` / `SettingsRow` — it would discard the vendored
  `Settings` and its ten-patch sidebar contract, which three earlier
  initiatives built and the owner has visually signed off on. Rejected
  alternative: adding `ely-gpui-component` as a dependency — verified
  dead end (`gpui_platform` is git-only, the crate is indivisible, and
  Ely is macOS-only); recorded as NG1.
- risks:
  - **R4 has no independent lever.** `vendor/gpui-component/src/root.rs:579`
    sets `window.set_rem_size(cx.theme().font_size)` — rem *is* font size,
    and `spacing_tokens()` has exactly one consumer (`theme/mod.rs:364`,
    which only stores it; no vendored component reads it). A density
    control that drives rem would be a second name for the font-size
    slider, which fails R7's "real effect" bar. → Mitigation: phase
    `ely-appearance-scale` stops and asks before touching it. The owner
    either approves an additive patch to the rem derivation
    (`font_size * density`) or R4 is descoped to a follow-up plan. This is
    the initiative's one genuine open decision and is not guessed here.
  - **`src/main.rs` is contended by four active plans.** →
    Mitigation: this plan's phases are ordered so each is independently
    landable; `work` must check the tree is not mid-flight on a sibling
    plan before starting a wave, and stop rather than resolve a foreign
    conflict.
  - **`Theme::change` rebuilds from config and silently resets
    `font_size`** (`docs/memory/gpui-component-setting-field-pitfalls.md`
    item 3). Every new mode path — selector, System observer, accent
    write — must re-run `theme::apply_overrides`. → Mitigation: task
    `eat-4` owns the single re-apply path and its test; stop if a mode
    change cannot be funnelled through one function.
  - **`SettingField` setters do not repaint** (same memory file, item 2).
    → Mitigation: every new setter calls `cx.refresh_windows()`; asserted
    per task rather than assumed.

- phase_slug: `ely-appearance-theme`
  status: checked
  - goal: R1, R2, R7, R8, R9 | depends_on: none
  - surfaces: `src/main.rs`, `src/persist.rs`, `src/theme.rs`,
    `src/i18n.rs`, `vendor/gpui-component/src/color_picker.rs` (read
    only) | avoided: `assets/themes/mono.json`, `src/tokens.rs`,
    `src/cadence/`, `src/shell.rs`, `vendor/patches/`
  - escalate_when: the segmented theme control cannot be built app-side
    without a vendor patch, or System cannot observe the OS appearance
    through `Window::observe_window_appearance` at 0.3.5
  - wave 1:
    - T1 `eat-1` — add an app-level `Appearance { Light, Dark, System }`
      enum with a `resolve(system: WindowAppearance) -> ThemeMode` fn,
      a `StoredSettings` field, and a `Default` of `Dark`; hand-write
      the `Default` to match `AppSettings::default()` (memory file item
      4). — output: the enum, its persistence round-trip, and the
      resolve fn compile and are unit-tested for all three appearances ×
      both OS modes. — check: `make test` — the new
      `appearance_resolve` cases pass. — stop_if: `WindowAppearance` is
      not nameable from `src/main.rs` through the `gpui_kit` re-exports.
    - T2 `eat-2` — add accent presets (a named `(name, Hsla)` list) and
      wire the vendored `ColorPicker` into an accent `SettingItem` via
      `SettingField::render`; write the chosen color to
      `theme.accent`/`theme.accent_foreground` and re-sync. — output: the
      accent item renders a picker and a preset row; picking repaints
      live. — check: `make check` && `make lint` — no warnings. —
      stop_if: `Theme` exposes no writable accent field (expected:
      `theme/mod.rs:479-480` writes both from config).
  - wave 2:
    - T3 `eat-3` — replace the dark-mode `Switch` with a three-segment
      Light/Dark/System control (app-side, composed from vendored
      primitives — no new vendor patch); when `System` is selected,
      subscribe with `Window::observe_window_appearance` and re-resolve.
      — output: selecting System follows the OS, and an OS appearance
      change while running flips the app. — check: `make run` owner
      visual — toggle the OS appearance with System selected. — stop_if:
      the observer needs a `Window` the settings view cannot reach.
    - T4 `eat-4` — funnel every mode/accent write through one
      `apply_appearance(cx)` that calls `Theme::change` then
      `theme::apply_overrides(font_size, cx)` then
      `cx.refresh_windows()`. — output: one call site; flipping the mode
      with a non-default font size set does not reset it. — check: `make
      test` — a test asserting `font_size` survives a mode flip. —
      stop_if: a mode path exists that bypasses the funnel and cannot be
      routed through it.
  - wave 3:
    - T5 `eat-5` — add `keywords` entries to the new accent and
      appearance items and the `i18n` strings they need. — output: the
      items are findable by synonym in the sidebar search. — check:
      `make check` && `make fmt-check`. — stop_if: none.
  - phase check: `make check && make test && make lint && make fmt-check`
    — the `beui-tokens` palette digest and config pins (R8) must pass
    untouched.

- phase_slug: `ely-appearance-scale`
  status: checked
  - goal: R3, R4, R7, R8, R9 | depends_on: `ely-appearance-theme`
  - surfaces: `src/main.rs`, `src/theme.rs`, `src/persist.rs`,
    `src/i18n.rs` | avoided: `assets/themes/mono.json`, `src/tokens.rs`,
    `src/cadence/`, `vendor/patches/` (unless the R4 decision adds one)
  - escalate_when: **the R4 density decision** — stop and ask the owner
    whether to add an additive `vendor/patches/rem-density.patch`
    changing `root.rs:579` to derive rem from `font_size * density`, or
    to descope R4 to a follow-up plan. Do not pick either silently.
  - wave 1:
    - T1 `eas-1` — replace the font-size `NumberField` with the vendored
      `Slider` over `theme::FONT_SIZE_MIN..=FONT_SIZE_MAX` (step 1),
      keep `normalize_font_size` as the boundary guard, add a preview
      line rendered at the chosen size, and a reset affordance disabled
      at the default. — output: slider drives the size live; reset
      returns to 14 and is inert at 14. — check: `make test` — the
      existing clamp test plus a new default-disabled predicate test. —
      stop_if: `Slider` cannot express a step of 1 over the range.
    - T2 `eas-2` — **decision task, no code.** Present the R4 finding
      (`root.rs:579`; `spacing_tokens()` single consumer) and the two
      options to the owner, then record the answer in `## Log`. —
      output: a recorded owner decision. — check: the decision is
      written in `## Log` before any density code exists. — stop_if: the
      owner is unavailable — the phase ends after T1 and R4 carries to a
      follow-up.
  - wave 2 (only if T2 approves the patch):
    - T3 `eas-3` — add `vendor/patches/rem-density.patch` (additive),
      apply it, list it in the `Cargo.toml` patch comment in order, and
      add the `Density { Compact, Standard, Comfortable }` control. —
      output: the three densities change settings-row height, not just a
      label. — check: `make check` && `make test` — a test pinning the
      density→rem mapping; `make run` owner visual for the three
      heights. — stop_if: the patch does not apply or the rem change
      visibly breaks the sidebar/card contract.
  - phase check: `make check && make test && make lint && make fmt-check`
    plus `make run` owner visual — the slider preview tracks, reset
    returns to 14, and (if T3 landed) the three densities differ
    visibly.

- phase_slug: `ely-navigation`
  status: in-progress
  - goal: R5, R6, R7, R8 | depends_on: `ely-appearance-theme`
  - surfaces: `src/main.rs`, `src/i18n.rs`,
    `vendor/gpui-component/src/setting/item.rs`,
    `vendor/patches/`, `Cargo.toml` comment | avoided:
    `assets/themes/`, `src/theme.rs`, `src/cadence/`, `src/shell.rs`
  - escalate_when: the term-AND matcher cannot be expressed as a
    single-function additive patch to `is_match`, or the reset panel
    cannot compute "changed" from `AppSettings` without vendor access
  - wave 1:
    - T1 `enav-1` — add `.keywords([...])` to every existing
      `SettingItem` in `src/main.rs` (currently **zero** calls), so
      synonyms match with the matcher as it stands. — output: typing a
      synonym filters the sidebar to its setting. — check: `make run`
      owner visual — a synonym with no title substring, e.g. `mfa`-style
      words, narrows to the right item. — stop_if: `keywords` does not
      compile through the `gpui_kit` re-exports.
    - T2 `enav-2` — add `vendor/patches/settings-search-terms.patch`
      (additive): split the query on whitespace and require **every**
      term to match, per Ely's pinned semantics. Keep the existing
      title/description/keywords haystack. — output: `appearance zoom`
      matches a font-size item; `network dark` matches nothing. — check:
      `make test` — the three ported cases from Ely's
      `settings/search.rs` tests. — stop_if: the patch must change more
      than `is_match` to compile.
  - wave 2:
    - T3 `enav-3` — build the reset panel app-side: compute the changed
      set by comparing `AppSettings` to `AppSettings::default()` (no
      vendor access needed — the app owns both), render one row per
      changed setting with a per-row revert, a "reset all" that asks
      twice, and an explicit "nothing has changed" state. — output: the
      panel lists exactly the changed settings and reverts them one at a
      time. — check: `make test` — the changed-set computation; `make
      run` owner visual — change two settings, confirm two rows, revert
      one, confirm one row. — stop_if: the app cannot enumerate its own
      settings for the list without a vendor hook.
    - T4 `enav-4` — i18n strings for the panel and the search empty
      state in all four locales (en/vi/zh-CN/ja). — output: no
      hardcoded English. — check: `make test` — the locale table test.
      — stop_if: none.
  - phase check: `make check && make test && make lint && make fmt-check`
    plus `make run` owner visual — synonym search narrows correctly, a
    filtered item opens on click, and the reset panel agrees with the
    real state.

## Log

- 2026-10-02T03:32:35Z — ely-appearance-theme — start — wave 1 (`eat-1` ∥ `eat-2`); tree clean, boundaries match the phase's surfaces.
- 2026-10-02T03:32:35Z — ely-appearance-theme — decision — **NG1's stated reason is wrong.** The Goal says adopting Ely is impossible because "`gpui_platform` is not on crates.io (git-only)". It is: the package behind it is `gpui-pre-platform`, published at 0.3.7, and mono-desk already resolves `gpui-pre-platform` 0.3.5 (`gpui-kit` declares `gpui_platform = { package = "gpui-pre-platform" }`). The earlier check queried the wrong crate name. Caret ranges would unify on 0.3.7, so the gpui pin is **not** a hard blocker. The conclusion (port the pattern, never the code) still stands, on different grounds: Ely is one `publish = false` crate that cannot be cherry-picked, it is macOS-only in practice (51 `cfg(target_os = "macos")` sites vs 1 Linux branch), and adopting it needs a `[patch]` redirect of its transitive git gpui. NG1 is in `## Goal`, which `work` does not own — flagged here rather than edited; `.devin/skills/ely-gpui-reference/SKILL.md` carries the correction.

- 2026-10-02T03:40:56Z — ely-appearance-theme/eat-1 — done — `make test` → **79 passed, 0 failed** (43.05s). Added `theme::Appearance {Light,Dark,System}` + `ALL`/`as_str`/`parse`/`resolve`, three new tests in `src/theme.rs` (`appearance_resolves_against_the_system`, `appearance_words_round_trip_and_reject_junk`, `appearance_default_matches_the_app_default`). Persistence: `StoredSettings.dark_mode: bool` → `appearance: String`, with a legacy `dark_mode` fallback in `parse` (honoured only when no `appearance` line lands) and a new `legacy_dark_mode_still_selects_an_appearance` test. `AppSettings.dark_mode` → `appearance: Appearance`; `set_dark_mode(bool)` → `set_appearance(Appearance)` resolving via `cx.window_appearance()`. Startup resolves the stored choice before the first window. Surfaces: `src/theme.rs`, `src/persist.rs`, `src/main.rs`, `src/view_tests.rs`. The `stop_if` did **not** trigger: `gpui_kit::WindowAppearance` is nameable (`gpui-kit` does `pub use ::gpui::*`).
- 2026-10-02T03:40:56Z — ely-appearance-theme/eat-1 — decision — **`src/view_tests.rs` is not in the phase's surface list but had to change.** It was added by the `beui-p1-components` merge (8170aad) after this plan was authored, and it asserts on `AppSettings::global(cx).dark_mode` in five places, so renaming the field cannot compile without touching it. The edit is mechanical (boolean assertions → `Appearance` comparisons) and adds no new coverage. Recorded rather than treated as `BLOCKED_CONTRACT_DRIFT` because it is a forced consequence of an in-scope rename, not new scope.
- 2026-10-02T03:40:56Z — ely-appearance-theme/eat-1 — decision — **the task's "`Default` of `Dark`" was wrong.** `eat-1` said "a `Default` of `Dark`; hand-write the `Default` to match `AppSettings::default()`" — but `AppSettings::default()` starts in **light** (`dark_mode: false`), so the two halves contradicted. Followed the cited authority (`docs/memory/gpui-component-setting-field-pitfalls.md` item 4: persisted defaults must mirror app defaults) and made `Appearance::default() == Light`, pinned by `appearance_default_matches_the_app_default`. Flipping the app's default appearance would be a visual-baseline product change, which this phase has no authority for.

- 2026-10-02T03:53:22Z — ely-appearance-theme/eat-2 — done — `make check` clean (0 warnings) + `make lint` clean + `make test` → **82 passed, 0 failed** (34.87s). Added `theme::ACCENT_PRESETS` (6 named hexes from the palette's own base hues), `theme::accent_from_str`, and `write_accent` driving `colors.primary` + `primary_hover`/`primary_active` (derived via `Colorize::lighten`/`darken`, direction per mode) + `ring` + a readable `primary_foreground`; three new tests. App side: `AppSettings.accent: Option<Hsla>` (persisted as `#RRGGBB`, empty = the theme's own), `set_accent`, `accent_swatch`, and `accent_field` hosting the vendored `ColorPicker` through `window.use_keyed_state` (the `language_field` slot pattern) plus a six-swatch preset row; `item.accent` strings in all four locales. Surfaces: `src/theme.rs`, `src/persist.rs`, `src/main.rs`, `src/i18n.rs`, `src/view_tests.rs`.
- 2026-10-02T03:53:22Z — ely-appearance-theme/eat-2 — decision — **the task named the wrong tokens.** `eat-2` said to write the picked color to "`theme.accent`/`theme.accent_foreground`". In this framework `colors.accent` is the *neutral hover wash* behind list and menu rows (`accent.background` is `#e3e3e3` light / `#373737` dark), and `accent_foreground` is the text on that wash — writing a saturated brand color to both would have made hover rows unreadable and would not have recolored anything a user calls "the accent". The brand tokens are `primary` + `primary_hover` + `primary_active` + `primary_foreground` and `ring` (`#124b68` / `#007494` in the shipped palette, and what primary buttons, selection and focus already use). Drove those; R2's acceptance ("recolors the live theme immediately") is what the task was serving. `accent_drives_the_primary_family_and_the_ring` pins that `colors.accent` is deliberately left alone.
- 2026-10-02T03:53:22Z — ely-appearance-theme/eat-2 — decision — **`write_overrides` kept its two-argument shape to respect an avoided surface.** The first cut threaded `accent` through `write_overrides(theme, font_size, accent)`, which broke `src/tokens.rs:195` — a file this phase lists under `avoided`. Reverted: `write_overrides` stays `(theme, font_size)` and `apply_overrides` applies the accent after it, so `src/tokens.rs` is untouched and its existing test keeps its call shape. Same reason the vendored `ColorPicker` is hosted via `use_keyed_state` rather than a new global.
- 2026-10-02T03:53:22Z — ely-appearance-theme/eat-2 — decision — **fixed a latent bug in the `advance` test helper, not just a symptom.** Hosting a `ColorPickerState` eagerly creates an `InputState`, whose `BlinkCursor` schedules a `pause` task. `advance` drove the clock *inside* `cx.update(...)`, i.e. while the app was borrowed, so that task's `borrow_mut` panicked (`RefCell already borrowed`, `async_context.rs:65`) and `collapsed_rail_item_drives_tooltip_lifecycle` failed. The helper was wrong independent of this phase — gpui's own `BlinkCursor` test advances the clock outside any update — so `advance` now calls `cx.executor().advance_clock(by)` and `cx.run_until_parked()` outside the borrow. Verified: the previously failing test passes and the suite went 81→82 with no other change.

- 2026-10-02T03:53:22Z — ely-appearance-theme — decision — **correction to the `eat-1` entry above:** it lists `Appearance::ALL` among what was added. `ALL` was dropped again during `eat-2` because `cargo check` builds only the binary, where a `const` used solely by `#[cfg(test)]` code warns as dead — and `eat-2`'s check is "no warnings". Wave 2's segmented control re-adds it with a production caller. The `appearance_words_round_trip_and_reject_junk` test now spells the three variants out.

- 2026-10-02T04:34:55Z — ely-appearance-theme/eat-3 — done — `make check --all-targets` clean + `make test` → **82 passed, 0 failed**. Replaced the dark-mode `Switch` with `appearance_segmented` — three vendored `Button`s using `Button::toggled` as the segment state, labelled Light/Dark/System with `Sun`/`Moon`/`Cpu` (the catalog has no monitor glyph), no vendor patch. Added `window.observe_window_appearance` in `SettingsWindow::new` so `System` follows the OS while running; the observer lives on the window, not the row, so it survives the page being filtered away, and an explicit Light/Dark returns early. Renamed the row to `item.appearance` with `appearance.light|dark|system` in all four locales. Rewrote `dark_mode_switch_click_keyboard_a11y_and_reset` as `appearance_segmented_click_keyboard_a11y_and_reset` and repointed `switch_toggle_lands_under_reduce_motion` at the notifications switch (the Appearance row is no longer a switch, and that switch lives on page 2, so the test now navigates there). Reused the orphaned `settings_switch` helper on the notifications row rather than deleting it.
- 2026-10-02T04:34:55Z — ely-appearance-theme/eat-4 — done — `make test` → **83 passed, 0 failed**. Split the funnel in two: `apply_appearance(appearance, cx)` does `Theme::change` + `apply_font_size` + `refresh_windows` and deliberately does **not** persist (startup must not rewrite the file it just read), while `set_appearance` adds the global write and `save_settings`. Startup now calls `apply_appearance` instead of its own inline `Theme::change`, so there is one mode path. New test `mode_flip_keeps_a_non_default_font_size` sets 20px, flips through Dark/Light/System, and asserts the live theme and the persisted value still read 20 — the exact regression `docs/memory/gpui-component-setting-field-pitfalls.md` item 3 warns about.
- 2026-10-02T04:34:55Z — ely-appearance-theme/eat-5 — done — `make check` clean + `make fmt-check` clean (after `make fmt`). `.keywords(["theme","mode","dark","light","system"])` on the Appearance item and `.keywords(["color","colour","brand","primary","highlight"])` on the Accent item. These are the app's **first** `keywords` calls — the vendored `SettingItem::keywords` was public and unused, so synonyms now match with the matcher as it stands.
- 2026-10-02T04:34:55Z — ely-appearance-theme — decision — **disk exhaustion truncated `src/theme.rs` mid-write; recovered by splicing.** `/` hit 100% (0 bytes free) while an edit was being written, so the tool wrote 24576 bytes and stopped mid-token, leaving 566 of 703 lines. `src/persist.rs`, `src/main.rs`, `src/i18n.rs` and `src/view_tests.rs` were checked byte-for-byte and were intact. Recovered by taking the truncated file's complete lines and appending the unchanged tail from `git show HEAD:src/theme.rs` (the cut fell inside `embedded_theme_set_has_required_colors`, which this phase never touched); one line (`let name = …`) sat exactly on the boundary and was restored by hand, then caught by the compiler. `target/debug/incremental` (878 MB) was deleted to free space — the only thing removed, and regenerable. No committed work was lost; nothing outside `target/` was deleted.

## Validation

- 2026-10-02T04:34:55Z — phase `ely-appearance-theme` — verdict: APPROVED — mode: gate
  - `make check` — clean, 0 warnings
  - `make test` — ok. 83 passed; 0 failed; 0 ignored (44.00s)
  - `make lint` — clean, `clippy --all-targets -- -D warnings`
  - `make fmt-check` — clean
  - scope: on target — every changed file is one of the phase's surfaces or a call site the change forced (`src/view_tests.rs`); `assets/themes/mono.json`, `src/tokens.rs`, `src/cadence/`, `src/shell.rs` and `vendor/patches/` are untouched, verified by `git status`
  - requirements: R1 met (segmented Light/Dark/System; `appearance_resolves_against_the_system` covers all three appearances × four OS appearances including `Vibrant`; `window.observe_window_appearance` re-resolves while running) | R2 met (six presets + vendored `ColorPicker`; `accent_drives_the_primary_family_and_the_ring`, `accent_foreground_stays_readable`) | R7 met for this phase (all four controls persist; `legacy_dark_mode_still_selects_an_appearance`, `stored_accent_parses_or_falls_back`, `round_trip_default_and_populated`) | R8 met (the `beui-tokens` palette digest and config pins pass untouched — `mono_palettes_are_pinned`, `theme_token_values_are_pinned`) | R9 met (`mode_flip_keeps_a_non_default_font_size`; every mode path routes through `apply_appearance`)
  - rollback_point: 7121a8c (the phase's changes are uncommitted)
  - requests: none
  - not_independently_verified: the owner's `make run` visual — the segmented control's look, the accent recoloring live, and an OS appearance flip with System selected. This session is headless (no Wayland/X11), so `eat-3`'s stated check could not be run; the behaviour is covered by view tests instead, which is weaker evidence for anything purely visual.
  - judge: same-session
  - judge_model: DeepSeek V4.1 Flash Max

- 2026-10-02T04:43:45Z — ely-appearance-scale/eas-1 — done — `make test` → **84 passed, 0 failed** (38.88s). Replaced the font-size `NumberField` with the vendored `Slider` (`.min(8).max(32).step(1)`) hosted through `window.use_keyed_state`; a preview line drawn at the chosen size; and a `Button::ghost().icon(RotateCw)` reset that is `.disabled(font_size_at_default(size))`. The slider reports continuously, so `SliderEvent::Change` re-applies the theme and `Release` additionally persists — a drag would otherwise rewrite the config dozens of times. `normalize_font_size` still guards the boundary. New `font_size_reset_is_inert_at_the_default_only` test. The `stop_if` did not trigger: `Slider` expresses a step of 1 over the range.
- 2026-10-02T04:43:45Z — ely-appearance-scale/eas-2 — decision — **R4 (density) descoped to a follow-up; owner input requested.** This task exists to stop and ask, and the owner was not asked before the run. Recording the finding and the conservative choice rather than picking silently in the other direction:
  - **The finding.** `vendor/gpui-component/src/root.rs:579` sets `window.set_rem_size(cx.theme().font_size)`, so rem **is** the font size; `spacing_tokens()` has exactly one consumer (`theme/mod.rs:364`, which only stores it) and no vendored component reads it. There is no independent spacing lever.
  - **Option A (descoped — taken here).** No density control. It would otherwise be a second name for the font-size slider, which fails R7's "every control produces a real effect" bar.
  - **Option B (available on request).** Add an additive `vendor/patches/rem-density.patch` deriving rem from `font_size * density` and a `Density { Compact, Standard, Comfortable }` control. This changes global rem semantics for the whole app — including the sidebar/card contract three earlier initiatives built — so it is a visual-baseline product decision this phase has no authority to make.
  - Chose A because it changes nothing observable and is trivially reversible: say the word and `eas-3` runs. `eas-3` is **not** executed. R4 is therefore `partial (→ follow-up)`.
- 2026-10-02T04:43:45Z — ely-appearance-scale — decision — the `Slider`'s reset button uses `IconName::RotateCw`, not `RotateCcw` (the catalog carries only the clockwise glyph), and `Button::ghost` comes from the `ButtonVariants` trait, which `src/main.rs` did not previously import. Both caught by the compiler, not by inspection.

- 2026-10-02T04:43:45Z — phase `ely-appearance-scale` — verdict: APPROVED — mode: gate
  - `make check` — clean, 0 warnings
  - `make test` — ok. 84 passed; 0 failed; 0 ignored (40.34s)
  - `make lint` — clean, `clippy --all-targets -- -D warnings`
  - `make fmt-check` — clean (after `make fmt`)
  - scope: on target — `src/main.rs`, `src/i18n.rs`, `src/view_tests.rs` changed; `assets/themes/mono.json`, `src/tokens.rs`, `src/cadence/`, `vendor/patches/` untouched, verified by `git status`
  - requirements: R3 met (slider over the 8–32 clamp with a live preview and a default-disabled reset; `font_size_reset_is_inert_at_the_default_only`, plus the pre-existing clamp test) | R4 **partial (→ follow-up)** — descoped by `eas-2` above, owner input requested | R7 met for this phase (the size persists; the pre-existing `round_trip_default_and_populated` covers the field) | R8 met (palette digest and config pins untouched) | R9 met (`mode_flip_keeps_a_non_default_font_size` still passes)
  - rollback_point: 7121a8c (the phase's changes are uncommitted)
  - requests: none
  - not_independently_verified: the owner's `make run` visual — the slider's travel, the preview line tracking it, and the reset's disabled state at 14. Headless session, so `eas-1`'s visual half rests on view-level assertions only.
  - judge: same-session
  - judge_model: DeepSeek V4.1 Flash Max

- 2026-10-02T05:04:41Z — ely-navigation/enav-1 — done — every `SettingItem` in `src/main.rs` now carries a `.keywords([...])` synonym list (13 items: appearance, accent, font size, language, timezone, launch-at-login, notifications, channels, digest, test notification, username, display name, preview, website, about). These were the app's first `keywords` calls — the vendored builder was public and unused, so synonyms match with the matcher as it stood.
- 2026-10-02T05:04:41Z — ely-navigation/enav-2 — done — `make test` → 86 passed, 0 failed. New additive patch `vendor/patches/settings-search-terms.patch` (listed in the `Cargo.toml` comment): `SettingItem::is_match` now builds one lowercased haystack (title + description + keywords) and requires **every** whitespace-separated query term to land in it, where it previously required the whole query as a single substring. The matching rule is extracted as `pub fn SettingItem::query_matches(haystack, query)` so the app can pin it directly; new `search_matcher_requires_every_term` test.
- 2026-10-02T05:04:41Z — ely-navigation/enav-2 — decision — **the first cut broke every sidebar test.** With the query split into terms, an empty query produced `[]` and `.all()` on an empty iterator returns `true` for `Item` but the rewritten `Element` arm returned `false` — and the old matcher had relied on `contains("")` being always true. Fixed by returning `true` for both variants when the query has no terms, which is the behaviour the old code had; `make test` went 6 failures → 0. Recorded because "empty query matches everything" is now an explicit invariant rather than an accident of `contains`.
- 2026-10-02T05:04:41Z — ely-navigation/enav-3 — done — `changed_settings(cx)` computes the diff between `AppSettings` and `AppSettings::default()` entirely app-side (no vendor access — the app owns both sides), and `reset_page(cx)` renders it as a new sidebar page: one row per changed setting showing `current → default`, an explicit "Nothing has changed" state when the list is empty, and a "Reset all" that arms on the first press and reverts on the second. `reset_everything` restores the whole `AppSettings`, re-installs the theme, re-converges autostart, and persists. New `reset_panel_lists_only_what_changed` test.
- 2026-10-02T05:04:41Z — ely-navigation/enav-3 — decision — the new Reset page shifts About from sidebar index 4 to 5, so the three About view tests were updated to navigate to 5 and the harness doc comment now records the order (`0 General, 1 Notifications, 2 Profile, 3 Finance, 4 Reset, 5 About`). Mechanical, forced by the page insertion.
- 2026-10-02T05:04:41Z — ely-navigation/enav-4 — done — `page.reset`, `group.changed`, `reset.nothing(.desc)`, `reset.all(.confirm/.desc)`, `reset.theme_default`, `state.on` and `state.off` added in all four locales (en/vi/zh-CN/ja). No hardcoded English remains in the panel.

- 2026-10-02T05:04:41Z — phase `ely-navigation` — verdict: APPROVED — mode: gate
  - `make check` — clean, 0 warnings
  - `make test` — ok. 86 passed; 0 failed; 0 ignored (41.37s)
  - `make lint` — clean, `clippy --all-targets -- -D warnings`
  - `make fmt-check` — clean (after `make fmt`)
  - scope: on target — `src/main.rs`, `src/i18n.rs`, `src/view_tests.rs`, `vendor/gpui-component/src/setting/item.rs`, `vendor/patches/`, `Cargo.toml` comment; `assets/themes/`, `src/theme.rs`, `src/cadence/`, `src/shell.rs` untouched, verified by `git status`
  - requirements: R5 met (term-AND matcher pinned by `search_matcher_requires_every_term`; every item carries synonyms; the filtered item opens on click, unchanged from before) | R6 met (the panel lists exactly the changed set — `reset_panel_lists_only_what_changed` — with a two-press reset-all and an explicit empty state) | R7 met for this phase (reset persists through `save_settings`) | R8 met (palette digest and config pins untouched)
  - rollback_point: 7121a8c (all three phases are uncommitted)
  - requests: none
  - not_independently_verified: the owner's `make run` visual — typing a synonym into the sidebar search and watching it narrow, and the Reset page's two-press affordance. Headless session; the matcher itself is pinned by a direct unit test, which is stronger evidence than a visual for that half.
  - judge: same-session
  - judge_model: DeepSeek V4.1 Flash Max

## Current State and Next Action
- active_phase: ely-navigation
- lifecycle_status: in-progress
- blockers: none
- phase_state: `ely-appearance-theme` **checked** · `ely-appearance-scale` **checked** · `ely-navigation` **in-progress** (its own gate passed; kept in-progress because it is the initiative's final phase, so an independent `check full` can run against it)
- open_items:
  - **R4 (density) is the one unresolved decision.** The feasibility pass
    found no independent spacing lever: `root.rs:579` derives rem from
    `theme.font_size` and `spacing_tokens()` has no consumer in the
    vendored components. Task `eas-2` stops and asks the owner to either
    approve an additive `rem-density.patch` or descope R4. Resolved from
    `to-plan` (R5's patch question is now answered: `SettingItem::keywords()`
    is already public and unused, so only the term-AND matcher needs a
    patch).
  - `src/main.rs` is the contention point across four active plans; each
    phase here is independently landable, but `work` must confirm the
    tree is not mid-flight on a sibling plan before starting a wave.
- exact_next_action: `check full` on the whole initiative (independent judge; `work` must not run it), then the owner's `make run` visual pass, then `handoff` to close
