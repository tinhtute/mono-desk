# PROJECT — identity

## What is this project?
- `mono-desk` is a settings-style desktop demo app: a single-window GPUI
  application built on **zeron's GPUI stack** — `gpui`/`gpui_platform` from
  `github.com/zeronsh/zui` (Zed's GPUI fork, Apache-2.0, pinned rev) — with the
  settings shell, pages, widgets and theme ported from zeron's MIT-licensed
  source (`github.com/zeronsh/zeron`) into `vendor/zeron-ui/` and
  `vendor/zeron-theme/`. The pre-port gpui-kit implementation is preserved on
  branch `wip/gpui-kit-shell`.

## Who is it for?
- Its owner (lab project) and developers evaluating zeron's GPUI settings
  surface and theme system for real desktop apps.

## Non-goals
- Not a production settings/productivity app; no multi-window or multi-view
  product surface. No chat/workspace surface: zeron's chat-route chrome is
  deliberately not ported (the app is its Settings route only). The Appearance
  page does port zeron's local theme library (VS Code theme import, review and
  install), the managed new-thread background image with its adjustment/effect
  dialog, and the wallpaper folder/shuffle rows; it still has **no wallpaper
  painting layer, no transcript/Markdown surface, no cloud theme sync**, and
  the settings-search anchors/`page_reveal` and zeron's kinetic-scroll engine
  stay unported (appearance-parity NG1-NG6).

## What are the gate commands?
- run from: repository root (host-local: `/Users/tinhtute/Lab/mono-desk`,
  Linux sessions `/home/tinhpt/Lab/mono-desk`)
- tests: `cargo test` (workspace: `mono-desk` state/persistence/i18n/pages,
  `zeron-theme`, `zeron-ui` vendored tests)
- types: `make check`
- lint: `make lint` (clippy on `-p mono-desk`; vendored crates keep upstream
  style)
- build: `make run` (compiles; window needs a GUI session)
- format: `make fmt-check`

## Architecture in one breath
- runtime shape: one cargo workspace, Rust 2024: the app crate plus the two
  vendored crates (`vendor/zeron-theme`, `vendor/zeron-ui`) as
  `default-members`, so the gates cover them
- dependencies: `gpui` + `gpui_platform` from `zeronsh/zui` @ pinned rev; no
  registry UI kit
- chrome: zeron's Settings route (`vendor/zeron-ui/src/chrome.rs`,
  `src/shell.rs`) — glass root over the blurred window background, 38px
  titlebar drag strip, 256px sidebar tone column, seven-section list, footer
  identity row, sidebar search
- pages: `src/settings/` — one gpui view per page owning its `SelectState`s and
  `ScrollHandle`, built from zeron's vendored widgets (`settings::widgets`,
  `settings::select`, `settings::menu`) over `Theme::of(cx)` tokens
- the finance showcase: `src/finance/` (rebuilt from the pre-port app's
  `src/cadence/`)
- appearance: `settings::apply_appearance` is the single funnel from
  `AppSettings` into `Theme::install_selection` + the native window background
  + rem font size
- i18n: zero-dependency table in `src/i18n.rs` (en/vi/zh-CN/ja)
- assets: `assets/fonts/` (registered by `vendor/zeron-ui/src/typography.rs`),
  `assets/brand/` (SVG/PNG mark + `icon.icns`), the Phosphor Icons subset under
  `vendor/zeron-ui/assets/icons/` (slot → glyph map in `icons.rs`)
- where state lives: `AppSettings` in a GPUI `Global`, installed from
  `persist::load()` before the window opens; writes go through
  `settings::persist_now`
- entrypoint: `src/main.rs` — `gpui_platform::application()` → theme/fonts →
  `AppSettings` global → `open_window(..., Shell::new)`

## What are we working on right now?
- no active plan: the zeron-stack port
  (`docs/plans/completed/zeron-stack-port.md`) and the Appearance-parity port
  (`docs/plans/completed/appearance-parity.md`) are complete. mono-desk's
  Settings route now carries zeron's full Appearance page — Color scheme,
  Material/background (managed image, effect, wallpaper folder), Motion, the
  local theme library with import/review dialogs, and Fonts and layout — with
  its persistence, four-locale i18n and independent phase gates. The
  remaining known gaps are visual-only (owner pass) and are listed as proof
  gaps in the plan and `docs/memory/appearance-parity.md`.
- module map/adaptation rules: `docs/memory/zeron-stack-port.md` and
  `docs/memory/appearance-parity.md`; third-party licensing:
  `THIRD_PARTY_NOTICES.md`.
