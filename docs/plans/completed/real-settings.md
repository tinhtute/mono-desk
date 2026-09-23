---
id: plan-real-settings-20260923
status: completed
lane: high-risk
created: 2026-09-23
updated: 2026-09-23
---
# Real settings

Make every remaining control in the `Settings` window produce a real,
observable effect — or remove it. Closes the gaps found in the 2026-09-23
audit: dead-state fields, the non-repainting dark-mode switch, no
persistence, no i18n.

## Goal
- outcome: every control in the settings window either drives an
  observable behavior (theme repaint, rem-scale font size, gated
  notification send, OS autostart entry, live locale switch, visible
  profile preview) or no longer exists; all values survive relaunch via
  a platform config file.
- success_signal: `cargo test` + `make check` + `make lint` green; owner
  `make run` confirms each control's effect and that values persist
  across relaunch.
- actors: the owner (lab demo) and developers evaluating gpui-kit
  settings patterns.
- authority: owner decisions in this session (2026-09-23): implement
  Launch at Login for real, remove Auto Update and Digest, real i18n,
  real persistence, zero new direct dependencies (AGENTS.md keeps
  `gpui-kit` as the sole direct dep). Supersedes `docs/PROJECT.md`
  non-goals "no persistence backend, no i18n" (updated in the same
  pass).
- requirements:
  - R1: The General-page Dark Mode switch repaints the window on toggle
    (current `Theme::change(mode, None, cx)` refreshes nothing — wiring
    bug). | acceptance: setter calls `cx.refresh_windows()`;
    `make check` green; owner visual check via `make run`. | source:
    audit + `vendor/gpui-component/src/theme/mod.rs` `Theme::change`
  - R2: Font Size (8–32) scales the live UI via `Theme.font_size` →
    `window.set_rem_size` in `Root::render`, using
    `Theme::global_mut` + `Theme::sync_base` + `cx.refresh_windows()`
    (setter signature is `(f64, &mut App)` — no `Window`); the value
    survives dark-mode flips because `Theme::change` re-applies
    `font.size` from `mono.json`, so both theme-toggle paths re-apply
    `AppSettings.font_size` afterward. | acceptance: `make check` green;
    owner sees UI scale change and persist across a mode flip via
    `make run`. | source: audit + `vendor/gpui-component/src/root.rs`
    `set_rem_size`
  - R3: "Enable Notifications" gates the test Send button: disabled and
    click is a no-op while off. | acceptance: `make check`; owner
    toggles and observes disabled state + no notification via
    `make run`. | source: audit finding (toggle did not gate)
  - R4: Auto Update and Digest items are removed (no backend exists;
    owner chose removal over stubs). | acceptance: neither string
    appears in `src/`; `make check` green. | source: owner decision
  - R5: Launch at Login writes/removes a real OS autostart entry —
    `~/.config/autostart/mono-desk.desktop` on Linux,
    `~/Library/LaunchAgents/com.mono-desk.plist` on macOS — pointing at
    `std::env::current_exe()`; startup converges the entry to the
    persisted value. Safety: the entry carries an ownership marker
    (`X-Mono-Desk-Managed=true` / plist `MonoDeskManaged` key); a file
    at the target path WITHOUT the marker is never modified or deleted
    (log + leave); writes go through temp-file + rename in the same
    directory; IO errors log and continue, never panic; other platforms
    no-op. | acceptance: unit tests on generated contents, marker
    enforcement, and foreign-file no-touch using temp dirs; owner
    toggles and inspects the file. | source: owner decision
  - R6: Language dropdown switches the app locale for real: a
    zero-dependency `tr(key, lang)` table in `src/i18n.rs` covers every
    app-owned string for en/vi/zh-CN/ja (page titles, group titles,
    item labels, descriptions, button labels, rail footer, notification
    and dialog text); the setter also calls
    `gpui_kit::component::set_locale` (already re-exported) so
    gpui-component's own strings follow for zh-CN and fall back to en
    otherwise; startup applies the persisted locale. | acceptance:
    table-completeness unit test (every key × 4 langs); `make check`;
    owner switches language and sees translated labels via `make run`. |
    source: owner decision + AGENTS.md dep constraint
  - R7: `AppSettings` (plus a `dark_mode` field capturing the theme
    choice) persists to `~/.config/mono-desk/settings.conf` on Linux,
    `~/Library/Application Support/mono-desk/settings.conf` on macOS —
    hand-rolled `key=value` codec (escapes `\` and newline) whose first
    line is the ownership marker `# managed-by: mono-desk`; saved
    inside every `SettingField` setter AND the rail-footer theme
    `on_click` (both paths update `dark_mode`), via temp-file + rename;
    an existing file WITHOUT the marker is treated as foreign — logged
    and never overwritten; loaded once in `main()` before the window
    opens — the view constructor must not `set_global` over the loaded
    state; theme mode, font size, and locale are re-applied from the
    loaded values; a missing or corrupt file yields defaults, never a
    panic. | acceptance: unit tests round-trip (incl. hostile strings,
    corrupt input, foreign-file no-touch) in temp dirs; owner changes
    values, relaunches, sees them retained. | source: owner decision +
    AGENTS.md dep constraint
  - R8: Username and Display Name are consumed by a visible "Preview"
    item on the Profile page; both input setters call
    `cx.refresh_windows()` because `StringField` change subscriptions
    only invoke the app setter — without an explicit repaint the
    Preview stays stale (same for page reset). | acceptance:
    `make check`; owner edits and sees the preview update via
    `make run`. | source: audit finding (dead state) +
    `fields/string.rs` subscription
- non-goals:
  - NG1: No real auto-updater, version checker, or update UI.
  - NG2: No digest scheduler or queued-notification machinery.
  - NG3: No Windows autostart (registry Run key) — Linux + macOS only.
  - NG4: No new settings pages, controls, or product surface beyond the
    Preview item in R8.
  - NG5: No changes to `vendor/gpui-component` (no third patch).
  - NG6: No upstreaming of any mechanism to gpui-kit.
  - NG7: No new direct dependencies — no rust-i18n, serde/serde_json
    (stays dev-dep), dirs, or tempfile in `[dependencies]`.

## Phases and Verification
- approach: three phases in dependency order. (1) `live-effects` rewires
  existing controls inside `src/main.rs` only — smallest diff, no file
  IO. (2) `i18n` wraps the now-final string set behind `src/i18n.rs`
  `tr()` tables plus `gpui_kit::component::set_locale`; it runs after
  live-effects so strings are translated exactly once. (3)
  `os-integration` adds `src/persist.rs` (config dir resolution, atomic
  write, `key=value` codec, autostart entry generators) then wires
  save/load/converge into the finalized setters and startup. Rejected:
  adding rust-i18n/serde/serde_json/dirs/tempfile as direct deps —
  violates the AGENTS.md single-dep constraint (NG7); rejected: one
  phase for everything — file-IO risk must land last and stay
  independently revertible.
- constraints: writes outside the repo are limited to the settings file
  and the autostart entry; only files carrying the mono-desk ownership
  marker (`# managed-by: mono-desk` for settings.conf,
  `X-Mono-Desk-Managed`/`MonoDeskManaged` for autostart entries) are
  created/modified/deleted — a foreign file at the same path is logged
  and left untouched; every write is temp-file + rename in the same
  directory; IO errors log via `eprintln!` and continue, never panic;
  no new direct dependencies; `RUSTFLAGS`/edition untouched (Makefile
  stays the flag source).
- recovery: revert only this initiative's files — `git checkout --
  src/main.rs docs/PROJECT.md`, `rm src/i18n.rs src/persist.rs
  docs/plans/active/real-settings.md`; remove `settings.conf` and the
  autostart entry ONLY after verifying the mono-desk ownership marker
  (foreign files stay); each phase is mergeable alone.
- risks: `Theme::change` re-applies `mono.json` `font.size` → both
  theme-toggle paths re-apply `AppSettings.font_size` after change
  (shared `apply_font_size` helper); every setter calls
  `cx.refresh_windows()` for cross-control repaint — required for the
  Preview item since `StringField` setters never notify; IME
  disturbance during refresh is accepted as minor (input entities and
  focus survive a window refresh); the hand-rolled codec could
  mishandle free text → `\`/newline escaping + hostile-string
  round-trip test; `tr()` table drift → completeness test asserts
  every key exists in all four languages.
- execution note (2026-09-23): owner requested parallel subagent
  execution. Parallel dispatch of dependent-phase work was rejected —
  `depends_on` is immutable after to-plan and would need an approved
  refinement. Parallelism stays inside `live-effects`: the integration
  owner edits `src/main.rs` while a read-only scout verifies vendored
  API contracts for all phases; the high-risk lane's phase gate runs
  under an independent reviewer. Headless environment: `make run` is
  owner-performed (GUI); every owner-observable acceptance is listed
  as an explicit owner check per phase.
- phase_slug: `live-effects`
  status: done
  - goal: R1, R2, R3, R4, R8 | depends_on: none
  - surfaces: `src/main.rs` | avoided: `vendor/`, `assets/`,
    `Cargo.toml`, `docs/`
  - escalate_when: `Theme::sync_base`, `cx.refresh_windows()`, or
    `Button::disabled` signatures differ from the vendored source, or
    rem-scaling does not reach `Settings` content.
  - wave 1:
    - T1 Fix dark-mode switch repaint — drop the redundant
      `Theme::global_mut(cx).mode` write, keep `Theme::change`, add
      `cx.refresh_windows()` — output: General switch repaints like the
      rail toggle — check: `make check` — stop_if: `refresh_windows`
      missing from `App`
    - T2 Remove Auto Update + Digest items and their `AppSettings`
      fields — output: both gone — check:
      `! grep -rn "Auto Update\|Digest\|auto_update\|digest" src/` —
      stop_if: a consumer outside `setting_pages` exists
    - T3 Gate Send on `notifications_enabled` — render closure reads the
      global, `.disabled(!enabled)`, `on_click` early-returns —
      output: disabled button + no-op click while off — check:
      `make check` — stop_if: `Button::disabled` signature differs
    - T4 Profile "Preview" item — `SettingField::render` showing
      `{display_name} (@{username})`; username/display_name setters
      gain `cx.refresh_windows()` — output: live preview — check:
      `make check` — stop_if: —
  - wave 2:
    - T5 Font size → `Theme.font_size` — setter writes
      `Theme::global_mut(cx).font_size = px(val)`, `Theme::sync_base`,
      `cx.refresh_windows()`; shared `apply_font_size(cx)` re-applied
      after `Theme::change` in BOTH the dark-mode setter and the rail
      footer toggle — output: UI scales 8–32 and survives mode flips —
      check: `make check` — stop_if: `sync_base` missing or rem size not
      consumed by `Root`
  - phase check: `cargo test && make check && make lint`
  - owner check (headless — `make run`, GUI-only): General → Dark Mode
    repaints the window (R1); Font Size 8↔32 scales all UI text and
    survives a dark-mode flip (R2); Enable Notifications off → Send
    disabled and silent (R3); Auto Update/Digest absent (R4); editing
    Username/Display Name updates Preview live (R8).
- phase_slug: `i18n`
  status: done
  - goal: R6 | depends_on: live-effects
  - surfaces: `src/i18n.rs` (new), `src/main.rs` | avoided: `vendor/`,
    `assets/`, `Cargo.toml`
  - escalate_when: `gpui_kit::component::set_locale` is not reachable, or
    a required string cannot route through `tr()` (e.g. built inside the
    vendored component).
  - wave 1:
    - T1 `src/i18n.rs` — `tr(key, lang) -> &'static str` tables for
      en/vi/zh-CN/ja covering every app-owned string (4 page titles,
      group titles, ~12 item labels + descriptions, button labels, rail
      footer, notification + dialog text) with en fallback — output:
      complete table — check:
      `rustc --edition 2024 --test src/i18n.rs -o /tmp/i18n_test && /tmp/i18n_test`
      (completeness: every key × 4 langs) — stop_if: —
  - wave 2:
    - T2 Wrap all app strings in `tr()`; language setter calls
      `gpui_kit::component::set_locale` + `cx.refresh_windows()`;
      startup applies the current `AppSettings.language` — output: live
      locale switch — check: `make check && make lint` — stop_if:
      `set_locale` not re-exported
  - phase check: `cargo test && make check && make lint`
  - owner check (headless — `make run`, GUI-only): switching Language
    re-labels every page/group/item/button in vi, zh-CN, ja (R6);
    gpui-component strings (e.g. rail search placeholder) translate for
    zh-CN and stay English otherwise.
- phase_slug: `os-integration`
  status: done
  - goal: R5, R7 | depends_on: live-effects, i18n
  - surfaces: `src/persist.rs` (new), `src/main.rs` | avoided:
    `vendor/`, `assets/`, `Cargo.toml`
  - escalate_when: a target path holds a file without the ownership
    marker (stop, report, never overwrite), or atomic rename is
    unavailable on the platform.
  - wave 1:
    - T1 `persist.rs` codec + IO — `config_dir()` (`$XDG_CONFIG_HOME` or
      `$HOME/.config`; macOS `~/Library/Application Support`),
      `atomic_write(dir, name, bytes)` via temp + rename, `key=value`
      serialize/parse escaping `\` and newline with
      `# managed-by: mono-desk` first-line marker, `StoredSettings`
      struct + `load`/`save` — output: tested codec — check:
      `rustc --edition 2024 --test src/persist.rs -o /tmp/persist_test && /tmp/persist_test`
      (round-trip incl. hostile strings; corrupt file → defaults;
      foreign file no-touch; temp dirs only) — stop_if: —
    - T2 Autostart entries — `autostart_path()` per OS,
      `desktop_entry()`/`plist()` generators embedding the ownership
      marker, guarded `install`/`remove` (foreign file → log + leave),
      `std::env::current_exe()` target — output: tested generators +
      guarded install/remove — check: same `rustc --test` run (contents,
      marker enforcement, foreign-file no-touch; temp dirs only) —
      stop_if: —
  - wave 2:
    - T3 Wire persistence — `AppSettings` gains `dark_mode`; every
      `SettingField` setter AND the rail-footer `on_click` update
      `dark_mode` where relevant and end with `persist::save(cx)`;
      `main()` loads the settings file before opening the window
      (`SettingsWindow::new` drops its `set_global(default)` so loaded
      state is not overwritten) and applies `dark_mode` via
      `Theme::change`, `font_size` via `apply_font_size`, `language`
      via `set_locale` — output: values survive relaunch — check:
      `cargo test && make check` — stop_if: load panics on
      missing/corrupt file
    - T4 Wire autostart — `launch_at_login` setter calls
      install/remove; startup converges the entry to the persisted
      value — output: OS entry follows the setting — check:
      `cargo test && make check && make lint` — stop_if: foreign file
      at target path
  - phase check: `cargo test && make check && make lint`
  - owner check (headless — `make run`, GUI-only): change several
    values → quit → relaunch → all retained incl. dark mode (R7);
    toggle Launch at Login on → `~/.config/autostart/mono-desk.desktop`
    exists with `X-Mono-Desk-Managed=true` and correct `Exec` → toggle
    off → file removed (R5); a hand-created markerless file at either
    path is left untouched.

## Log
- 2026-09-23T03:48:23Z — live-effects — decision — premature work-stage entry rolled back: src/main.rs restored via git checkout, phase status returned to planned; to-plan hands off here, work runs the phase next
- 2026-09-23T03:55:00Z — all — decision — owner requested parallel subagent execution: live-effects, i18n-T1 (src/i18n.rs), os-integration-T1/T2 (src/persist.rs) dispatched concurrently; all main.rs wiring stays serial under the integration owner after live-effects lands; phases marked in-progress together
- 2026-09-23T03:55:00Z — all — decision — settings.conf gains ownership marker `# managed-by: mono-desk` so the marker rule covers both written files; foreign markerless files are never overwritten
- 2026-09-23T03:54:43Z — all — decision — correcting prior entry: dependent-phase parallel dispatch rejected (immutable depends_on); i18n and os-integration returned to planned; parallel work limited to live-effects roles (implementer + read-only contract scout + independent reviewer gate)
- 2026-09-23T04:08:00Z — i18n — blocked — premature start rolled back: live-effects is checked-not-done with unresolved owner GUI proof gaps; handoff step 5 requires done before a dependent phase starts; awaiting owner make run observations; failure_class: MISSING_CONTEXT
- 2026-09-23T04:12:00Z — live-effects — blocked — reopened: vendored NumberField parses "NaN" to f64::NAN and clamp passes it through; font_size setter must reject non-finite values before px()/set_rem_size; failure_class: BAD_OUTPUT
- 2026-09-23T04:16:10Z — live-effects — decision — src/i18n.rs kept on disk from the cancelled I18nModule agent: tr() fixed to &'static str signature (no Box::leak, dead lang_index removed), standalone rustc test 3/3 pass; it will be wired when i18n phase starts
- 2026-09-23T04:24:14Z — i18n — decision — gate found zh locale mismatch: dropdown stored "zh" but tr() matches "zh-CN"; fixed by storing canonical "zh-CN" in the dropdown (component_locale "zh"→"zh-CN" arm kept for robustness); cargo test 6/6 + make lint re-verified
- 2026-09-23T04:41:21Z — os-integration — decision — autostart_converge routed through public install/remove (was converge_at) to keep public API used; converge_at helper removed, test updated to install_at/remove_at
- 2026-09-23T04:53:55Z — os-integration — decision — full-check found StoredSettings derived Default yielded font_size 0.0 → set_rem_size(0) invisible UI on first launch; fixed: hand-written Default mirroring AppSettings (14.0/en/ninja), parse clamps font_size to 8–32, desktop_entry escapes \ and " in Exec; stale test expectations updated; cargo test 20/20 + lint + check re-verified
- 2026-09-23T07:59:22Z — all — decision — absorb: memory docs/memory/gpui-component-setting-field-pitfalls.md (NumberField NaN passthrough, setter no-repaint, Theme::change font.size reset, persisted-defaults divergence)
- 2026-09-23T08:05:01Z — os-integration — decision — foreign-file no-touch verified by probe (not owner): rustc-compiled persist.rs main() with HOME=/tmp/probe-foreign wrote markerless settings.conf + mono-desk.desktop, then called save/autostart_install/autostart_remove — all returned false, logged refusal, both files byte-identical; satisfies gate request 3

## Validation
- 2026-09-23T04:05:18Z — phase `live-effects` — verdict: APPROVE_WITH_REQUESTS — mode: gate
  - `cargo test` — pass: 2 passed, 0 failed (theme + assets unit tests)
  - `make check` — pass: cargo check finished clean
  - `make lint` — pass: cargo clippy --all-targets -D warnings clean
  - `! grep -rn "Auto Update\|Digest\|auto_update\|digest" src/` — pass: no matches; R4 removal complete
  - scope: on target — src/main.rs only; matches phase surfaces and tasks T1–T5
  - requirements: R1 met (dark-mode setter calls cx.refresh_windows() after Theme::change, src/main.rs:92) | R2 met (font_size setter writes Theme.font_size + Theme::sync_base + refresh, src/main.rs:126-127; apply_font_size re-applied after Theme::change in both toggle paths, src/main.rs:91,342) | R3 met (Send gated via .disabled(!enabled) + on_click early-return, src/main.rs:176-179) | R4 met (items + AppSettings fields removed; grep clean) | R8 met (Preview item renders "{display_name} (@{username})", src/main.rs:220; both input setters call cx.refresh_windows(), src/main.rs:201,213)
  - rollback_point: cc559a2c9d4c6b6b68092730d56e79c5addc6160
  - requests: 1. Owner `make run`: General → Dark Mode repaints the window (R1, src/main.rs:92). 2. Owner `make run`: Font Size 8↔32 scales all UI text and survives a dark-mode flip via both the General switch and the rail-footer toggle (R2, src/main.rs:91,126,342). 3. Owner `make run`: Enable Notifications off → Send disabled and click is silent (R3, src/main.rs:176-179). 4. Owner `make run`: Auto Update and Digest items absent (R4). 5. Owner `make run`: editing Username/Display Name updates the Preview item live, incl. after page reset (R8, src/main.rs:201,213,220).
  - proof_gaps: owner GUI verification (`make run`) not run — headless environment; integration tests — none exist for this UI behavior; complete manual review — deferred to `full` on the final phase per gate mode
  - judge: independent
  - judge_model: devin/swe-2


- 2026-09-23T04:24:55Z — phase `i18n` — verdict: APPROVE_WITH_REQUESTS — mode: gate
  - `cargo test` — pass: 6 passed, 0 failed (i18n completeness 3/3, font-size NaN guard, theme, assets)
  - `make check` — pass: cargo check finished clean
  - `make lint` — pass: cargo clippy --all-targets -D warnings clean
  - `rustc --edition 2024 --test src/i18n.rs -o /tmp/i18n_test && /tmp/i18n_test` — pass: 3 passed, 0 failed (every key x 4 langs, en fallback, unknown-key passthrough)
  - `cp src/i18n.rs /tmp/i18n_probe.rs && printf '\nfn main() { println!("{}", tr("page.general", "zh-CN")); }\n' >> /tmp/i18n_probe.rs && rustc --edition 2024 /tmp/i18n_probe.rs -o /tmp/zh_probe && test "$(/tmp/zh_probe)" = "通用"` — pass: stored "zh-CN" (src/main.rs:106) now reaches the zh-CN table arm; prior "zh" mismatch fixed
  - scope: on target — src/i18n.rs (new) + src/main.rs; matches phase surfaces and tasks T1–T2
  - requirements: R6 met (tr() tables cover every app-owned string for en/vi/zh-CN/ja with en fallback; setter calls component::set_locale via component_locale + cx.refresh_windows(); startup sets the AppSettings global then applies the locale; render closures read language fresh; zh-CN verified end-to-end by probe)
  - rollback_point: cc559a2c9d4c6b6b68092730d56e79c5addc6160
  - requests: 1. Owner `make run` (deferred, headless): switching Language re-labels every page/group/item/button in vi, zh-CN, ja (R6); gpui-component strings (e.g. rail search placeholder) translate for zh-CN and stay English otherwise.
  - proof_gaps: owner GUI verification (`make run`) not run — headless environment; integration tests — none exist for this UI behavior; complete manual review — deferred to `full` on the final phase per gate mode
  - judge: independent
  - judge_model: devin/swe-2


- 2026-09-23T04:55:09Z — phase `os-integration` — verdict: APPROVE_WITH_REQUESTS — mode: full
  - `cargo test` — pass: 20 passed, 0 failed (persist 14, i18n 3, font-size NaN guard, theme, assets)
  - `make check` — pass: cargo check finished clean
  - `make lint` — pass: cargo clippy --all-targets -D warnings clean
  - `rustc --edition 2024 --test src/persist.rs -o /tmp/persist_test && /tmp/persist_test` — pass: 14 passed, 0 failed (round-trip incl. hostile strings, marker enforcement, foreign-file no-touch, atomic write, autostart install/remove guards)
  - `cp src/persist.rs /tmp/persist_probe.rs && printf '\nfn main() { println!("{}", load().font_size); }\n' >> /tmp/persist_probe.rs && rustc --edition 2024 /tmp/persist_probe.rs -o /tmp/load_probe && env -u XDG_CONFIG_HOME HOME=/tmp/probe-home /tmp/load_probe` — pass: prints `14`; missing settings.conf now yields the StoredSettings::default() that mirrors AppSettings::default() (font_size 14.0, language "en", notifications_enabled true, ninja/Ninja) — prior 0.0 collapse fixed
  - `cp src/persist.rs /tmp/persist_probe.rs && printf '\nfn main() { println!("{}", load().font_size); }\n' >> /tmp/persist_probe.rs && rustc --edition 2024 /tmp/persist_probe.rs -o /tmp/load_probe && mkdir -p /tmp/probe-home2/.config/mono-desk && printf '# managed-by: mono-desk\nfont_size=0\n' > /tmp/probe-home2/.config/mono-desk/settings.conf && env -u XDG_CONFIG_HOME HOME=/tmp/probe-home2 /tmp/load_probe` — pass: prints `8`; loaded font_size is clamped to the control's 8–32 range (font_size=500 → 32 verified in the same run)
  - scope: on target — src/persist.rs (new) + src/main.rs; matches phase surfaces and tasks T1–T4; Cargo.toml untouched (NG7)
  - requirements: R5 met (desktop_entry/plist generators carry ownership markers; install/remove/converge guarded — foreign file logged and left; temp+rename writes; current_exe target with freedesktop-escaped Exec; other platforms no-op; 14 unit tests green; owner file inspection deferred) | R7 met (codec round-trip incl. hostile strings, first-line marker, atomic write, foreign-file no-touch, every setter + rail-footer on_click saves, main() loads before the window opens and applies dark_mode/font_size/locale, missing/corrupt file yields usable defaults, no panics) | success_signal: automated portion met (cargo test + make check + make lint green); owner `make run` deferred (→ owner check)
  - rollback_point: cc559a2c9d4c6b6b68092730d56e79c5addc6160
  - requests: 1. Owner `make run` (deferred, headless): change several values → quit → relaunch → all retained incl. dark mode (R7). 2. Owner `make run` (deferred, headless): toggle Launch at Login on → ~/.config/autostart/mono-desk.desktop exists with X-Mono-Desk-Managed=true and correct Exec → toggle off → file removed (R5). 3. Owner `make run` (deferred, headless): a hand-created markerless file at either path is left untouched.
  - proof_gaps: owner GUI verification (`make run`) not run — headless environment; autostart file inspection not run — owner-observable; macOS plist path untested — Linux-only environment; `judge: independent` is testimony, not proof
  - judge: independent
  - judge_model: devin/swe-2

## Current State and Next Action
- active_phase: none
- lifecycle_status: completed
- blockers: none
- open_items: none — owner confirmed persistence + autostart create/remove via `make run`; foreign-file no-touch verified by probe (log 2026-09-23T08:05:01Z)
- exact_next_action: none — initiative complete; plan archived to docs/plans/completed/
