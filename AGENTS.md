# AGENTS.md

## Project

`mono-desk` — a settings-style desktop demo app (single binary crate, Rust
edition 2024) built on **zeron's GPUI stack**: `gpui` + `gpui_platform` from
`github.com/zeronsh/zui` (zed's gpui fork, Apache-2.0; rev pinned in
`Cargo.toml`), not the registry `gpui-kit` stack any more. Window chrome,
sidebar/settings layout, widgets and theme are ported from zeron's
MIT-licensed source (`github.com/zeronsh/zeron`) and vendored under
`vendor/zeron-ui/` (UI) and `vendor/zeron-theme/` (theme domain model) — see
`THIRD_PARTY_NOTICES.md` and `docs/plans/completed/zeron-stack-port.md`. Vendored
files are adapted copies of upstream source at a pinned rev; they are not
drop-in updates. The three crates form one cargo workspace
(`vendor/zeron-theme` and `vendor/zeron-ui` are `default-members`, so the gates
cover them); the module map and the adaptation rules live in
`docs/memory/zeron-stack-port.md`. The pre-port gpui-kit implementation (glass
feature + custom sidebar) is preserved on branch `wip/gpui-kit-shell`.

## Commands

Use the Makefile — it auto-injects the linker shim flag (see caveats):

- `make run` — build and launch the app
- `make app` — bundle `target/macos/mono-desk.app` with the vendored brand icon
  (`assets/brand/icon.icns`); macOS only, needed for the Dock/⌘-Tab icon —
  `cargo run` shows a generic one. `make open-app` builds and opens it.
- `make check` — fast type-check (all workspace members)
- `make lint` — clippy `-p mono-desk --all-targets -D warnings` (run before
  committing); vendored crates are compiled and tested by the gates but not
  re-linted, since their style lints are upstream's
- `make fmt` / `make fmt-check` — first-party formatting only; vendored files
  keep upstream formatting
- `make deps` — install system packages (sudo; Ubuntu 24.04 tested list)
- `make shim` — recreate linker shims without sudo
- Gates: `cargo test && make check && make lint && make fmt-check`.

## Environment caveats (this machine)

- macOS; the GPUI stack needs a GUI session for `make run` — headless gates
  are `cargo test`, `make check`, `make lint`, `make fmt-check`.
- First build compiles the zui gpui fork + wgpu (hundreds of crates, several
  minutes); dependency cache in `target/` is ~12 GB. Avoid `cargo clean` and
  delete a stale `Cargo.lock` only when the dependency graph is replaced
  wholesale (old lock pins can conflict with gpui's exact version pins).
- Changing the pinned `zui` rev recompiles the whole dependency tree; treat a
  rev bump as its own task with its own verification.
- Agent tooling has no macOS Screen Recording grant: `screencapture` returns
  wallpaper + menu bar only, and the zui fork's AX content tree exposes just
  the window and native traffic lights (not GPUI-painted nodes). Verify layout
  with the GPUI test support, window bounds probes, and the owner's eye.
  `Window::render_to_image()` exists in the fork but is gated behind the
  `test-support` feature and unimplemented on the macOS platform, so
  in-process screenshots are not available yet.
- Dev review hooks in `src/main.rs`/`src/shell.rs`:
  `MONO_DESK_APPEARANCE=dark|light` pins the palette without touching the OS
  setting, `MONO_DESK_SELFTEST=1` prints the resolved appearance, tokens,
  fonts, icon asset load, native window background, chrome geometry, nav,
  settings state, the resolved appearance/rem markers and the theme-library
  entry/warning counts, then quits, and `MONO_DESK_SECTION=<general|appearance|
  notifications|profile|finance-ui|reset|about>` opens the shell on that
  section (use it with the selftest to prove a page other than General
  paints), `MONO_DESK_SEARCH=<text>` seeds the sidebar query so a headless
  run prints the filtered section list, and `MONO_DESK_SIDEBAR=<collapsed|
  open>` seeds the sidebar collapse state so a headless run proves the
  collapsed shell (`selftest sidebar collapsed=… target=…`). Use the selftest
  as the machine-checkable half of a visual phase check; the owner supplies
  the pixels. Do not claim pixel evidence from an agent run.
- Missing dev symlinks `libxkbcommon-x11.so` and `libX11-xcb.so` (Linux only)
  are shimmed at `~/.local/lib/dev-shim/`; the Makefile adds `-L` only while
  that dir exists. Proper fix is `make deps`.

## Code conventions

- Bootstrap (`src/main.rs`): `gpui_platform::application().run(|cx| ...)` →
  `cx.open_window(WindowOptions {...}, |window, cx| cx.new(...))`. Window
  options mirror zeron's `open_main_window`: transparent titlebar, native
  traffic lights at (14,14), `app_owns_titlebar_drag`, 900×600 minimum, and
  `window_background` read from the already-installed `Theme`
  (`Theme::window_background_appearance()`) — install the theme before
  `open_window`, then re-sync it from the window's own appearance once the
  window is attached.
- Layout constants live as numbers (zeron's titlebar 38, top pad 4, cluster
  start 88, sidebar default 256); colors come from the ported zeron theme
  (`Theme::of(cx)`), never hardcoded once the theme lands. The shell is
  zeron's *Settings* route (`vendor/zeron-ui/src/chrome.rs`, `src/shell.rs`):
  glass root, sidebar `wash(0.05)` tone, collapsible sidebar column and the
  38px drag strip. Plan R2 (owner decision 2026-10-29) ported that route
  literally, i.e. without centered title or resize handle; the
  `sidebar-collapse` amendment (owner request 2026-10-31) then added the chat
  shell's sidebar toggle: one `toggle-sidebar` window-control button in the
  top-left overlay cluster (88px cluster start, `icons::sidebar_glyph` morph)
  that tweens the column between 256 and 0, bound to the primary modifier
  (⌘B on macOS, Ctrl+B elsewhere) through `shell_actions::ToggleSidebar` and
  persisted as `sidebar_collapsed`.
- App state lives in a gpui `Global` (`AppSettings`, installed from
  `persist::load()` before the window opens); a settings write mirrors the
  global through `to_stored()` into `persist::save` via
  `settings::persist_now(cx)` and then `cx.notify()`s the writing view.
  Anything that changes appearance, accent, glass/surface or font size must
  also call `settings::apply_appearance(window, cx)` — the single funnel into
  zeron's `Theme::install_selection` + window background + rem size; and
  anything that can change `launch_at_login` calls
  `settings::apply_launch_at_login(cx)` so the OS autostart entry converges.
- Text input: zeron's kit has no text field, so `src/settings/text_field.rs`
  wraps an adaptation of the zui fork's `crates/gpui/examples/input.rs`
  (Apache-2.0, attributed in `THIRD_PARTY_NOTICES.md`). Register its editing
  actions through `TextField::key_bindings()` (already bound in `main.rs` for
  the `TextField` key context); pages own `Entity<TextField>` and mirror edits
  into `AppSettings` via `on_change`.
- Settings pages (`src/settings/`): built from zeron's vendored kit
  (`settings::widgets` rows/cards + `settings::select`), one gpui *view* per
  page so it can own its `SelectState`s and `ScrollHandle`; the shell keeps the
  page entities and only hosts the pane (`flex_1`, no scroll of its own, since
  `page_column` carries the titlebar clearance). Strings come from `i18n::tr`
  keyed by `AppSettings.language`. Add role/aria/tab-index to every ported
  control the way upstream does, and publish `settings::set_sidebar_width` each
  frame (the shell does) or dropdowns cannot measure the pane.
- Appearance page (`src/settings/appearance/`): zeron's full page split into
  one file per section — `mod.rs` owns the scroll host/rail and assembles
  header → Color scheme → color card → Material → Motion → library card →
  Fonts and layout → library warning, with the single modal stack last
  (`dialogs/{mod,background,library}.rs` in upstream priority order). Section
  modules own their view state and return `section(...) -> AnyElement`;
  `AppearancePage::new()` stays cx-free, so entities are created lazily in the
  handlers/`ensure` helpers. Every appearance write persists and funnels
  through `settings::apply_appearance` (theme library mutations additionally
  run `theme_library::reconcile` on the persisted variant ids first); the
  background install/remove path keeps the checked-persist contract in
  `src/settings/background.rs`, and the page itself never touches the
  filesystem. Constants/ids/roles/aria follow zeron's line map; record every
  adaptation in the module header and the plan Log. The theme library data
  and copied background images live under the app data dir
  (`Application Support/mono-desk/`), not in `settings.conf`.
- i18n: `src/i18n.rs` is a flat `(key, en, vi, zh-CN, ja)` table with a
  completeness test; appearance keys are grouped by owning call site
  (`appearance.background.*`, `appearance.library.*`, `appearance.fonts.*`,
  `appearance.layout.*`, `appearance.surface.*`, `appearance.motion.*`).
  Settings-domain error constants (`background`/`wallpaper`/`theme_library`)
  are mapped through their `appearance.*.error.*` keys at the page boundary;
  unexpected I/O diagnostics may stay raw (recorded in the plan Log). The
  ReduceMotion option words are localized page-side (`appearance.motion.*`).
- Accessibility: GPUI-painted controls do not appear in the macOS AX tree
  unless they carry explicit role/aria-label/tab-index metadata. Add it when
  porting a control that tests or the owner should reach by keyboard.
- Fonts: the app's UI stack (Momo Trust Sans and friends) lives in
  `assets/fonts/`; the ported typography module registers them. zeron's Geist
  fonts are not vendored.
- Styling is GPUI's Styled/Tailwind-flavored chaining; the ported modules keep
  zeron's layout constants and call shapes so visual comparisons stay honest.

## References

- zeron source (MIT): https://github.com/zeronsh/zeron — clone at
  `/tmp/zeron-src` when porting more; the port plans are
  `docs/plans/completed/zeron-stack-port.md` and
  `docs/plans/completed/appearance-parity.md`; the appearance module map and
  adaptation rules live in `docs/memory/appearance-parity.md`.
- GPUI fork (Apache-2.0): https://github.com/zeronsh/zui
- Repo workflow: `docs/WORKFLOW.md`

<!-- ZHARNESS:BEGIN -->
## Harness

Start with the requested outcome and use the repository as the system of record.
Read `docs/WORKFLOW.md` and only relevant product, design, plan, code, and
validation material.

- Answers, explanations, reviews, diagnoses, plans, and status reports are
  read-only. Inspect only what is needed; change nothing.
- For a bounded change, inspect affected behavior and proof, implement, and
  validate. No plan file is required.
- Use one `docs/plans/active/` file when work spans sessions, coordinates
  contributors, has dependencies, or needs recovery. Move it to
  `docs/plans/completed/` only after validation.
- Before editing, identify repository authority for each new externally
  observable policy. If materially different choices remain open, stop before
  edits; configurable defaults are not authority.
- Claim completion only with executable or observable evidence. Report outcome,
  changes, validation, and unresolved risks.

The `zharness` binary is install / update / uninstall only. It does not run
the lifecycle. There is no task database. There is no parallel control-plane state.
<!-- ZHARNESS:END -->
