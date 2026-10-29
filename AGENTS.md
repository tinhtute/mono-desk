# AGENTS.md

## Project

`mono-desk` — a settings-style desktop demo app (single binary crate, Rust
edition 2024) built on **zeron's GPUI stack**: `gpui` + `gpui_platform` from
`github.com/zeronsh/zui` (zed's gpui fork, Apache-2.0; rev pinned in
`Cargo.toml`), not the registry `gpui-kit` stack any more. Window chrome,
sidebar/settings layout, widgets and theme are ported from zeron's
MIT-licensed source (`github.com/zeronsh/zeron`) and vendored under
`vendor/zeron-ui/` (UI) and `vendor/zeron-theme/` (theme domain model) — see
`THIRD_PARTY_NOTICES.md` and `docs/plans/active/zeron-stack-port.md`. Vendored
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
- Dev review hooks in `src/main.rs`: `MONO_DESK_APPEARANCE=dark|light` pins the
  palette without touching the OS setting, and `MONO_DESK_SELFTEST=1` prints
  the resolved appearance, tokens, registered font families, icon asset load
  and native window background, then quits. Use the selftest as the
  machine-checkable half of a visual phase check; the owner supplies the
  pixels. Do not claim pixel evidence from an agent run.
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
  glass root, sidebar `wash(0.05)` tone and the 38px drag strip, deliberately
  without a titlebar control cluster, centered title or resizable/collapsible
  sidebar — those are zeron's chat-route chrome (plan R2, owner decision
  2026-10-29).
- App state lives in a gpui `Global` (`AppSettings`); settings writes go
  through the app's save path and `cx.notify()`.
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
  `/tmp/zeron-src` when porting more; the port plan is
  `docs/plans/active/zeron-stack-port.md`.
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
