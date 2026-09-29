# PROJECT — identity

## What is this project?
- `mono-desk` is a settings-style desktop demo app: a single-window GPUI
  application built on `gpui-kit` 0.6, showcasing the `Settings` component
  with grouped pages, fields, and overlay layers — plus app-owned rail
  chrome (brand block, search, item list, footer theme toggle) on a
  vendored, minimally patched `gpui-component`.

## Who is it for?
- Its owner (lab project) and developers evaluating gpui-kit patterns for
  real desktop apps.

## Non-goals
- Not a production settings/productivity app; no persistence backend, no
  multi-window or multi-view product surface. UI strings switch live across
  the four offered languages (en/vi/zh/ja), but there is no OS-locale
  negotiation, no pluralization, and no RTL support.

## What are the gate commands?
- run from: repository root (host-local: `/Users/tinhtute/Lab/mono-desk`,
  Linux sessions `/home/tinhpt/Lab/mono-desk`)
- tests: `cargo test` (embedded theme + font-size clamp in `src/theme.rs`,
  locale coverage in `src/main.rs`)
- types: `make check`
- lint: `make lint`
- build: `make run` (compiles; window needs a GUI session)
- format: `make fmt`

## Architecture in one breath
- runtime shape: single binary crate, Rust 2024, direct deps `gpui-kit` 0.6
  and `rust-i18n` 4.2 (the same i18n stack the vendored component uses), plus
  `[patch.crates-io] gpui-component = { path = "vendor/gpui-component" }` —
  upstream 0.6.1 carrying the full rail/cursor/tree patch set (deltas in
  `vendor/patches/`, apply order listed in `Cargo.toml`)
- i18n: `locales/app.yml` (en/vi/zh/ja) loaded by `rust_i18n::i18n!` in
  `src/main.rs`; the Language setting calls `rust_i18n::set_locale`, which
  switches the app strings and the component-internal strings together
- assets: `assets/themes/mono.json` (embedded ThemeSet), `assets/fonts/`
  (8 static TTFs registered at startup), `assets/brand/` (SVG + PNG mark +
  `icon.icns`) served by an app `AssetSource` that delegates other paths
  to `gpui_kit::assets::Assets`
- where state lives: `AppSettings` in a GPUI `Global` set inside the view
  constructor (`cx.set_global`)
- entrypoints: `src/main.rs` — `gpui_kit::application()` → `gpui_kit::init`
  → `Root::new(SettingsWindow)` with dialog/sheet/notification layers

## What are we working on right now?
- plan: docs/plans/active/mono-design-port.md (active) — refined
  2026-09-18 with requirements 8–14 (rail chrome: brand block, footer
  theme toggle, vendored `gpui-component` patch); next stage `to-plan`
  authors their phases, the three checked phases stay as recorded.
