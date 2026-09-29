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
- Not a production settings/productivity app; no multi-window or
  multi-view product surface. (Persistence and i18n were non-goals
  until 2026-09-23 — shipped under
  docs/plans/completed/real-settings.md.)

## What are the gate commands?
- run from: repository root (host-local: `/Users/tinhtute/Lab/mono-desk`,
  Linux sessions `/home/tinhpt/Lab/mono-desk`)
- tests: `cargo test` (embedded theme + font-size clamp in `src/theme.rs`,
  locale table in `src/i18n.rs`, persistence in `src/persist.rs`)
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
- i18n: zero-dependency table in `src/i18n.rs` (en/vi/zh-CN/ja); the
  Language setting also calls `gpui_kit::component::set_locale` for the
  component-internal strings
- assets: `assets/themes/mono.json` (embedded ThemeSet), `assets/fonts/`
  (8 static TTFs registered at startup), `assets/brand/` (SVG + PNG mark +
  `icon.icns`) served by an app `AssetSource` that delegates other paths
  to `gpui_kit::assets::Assets`
- where state lives: `AppSettings` in a GPUI `Global` set in `main()`
  before window creation (`cx.set_global`, loaded from persisted config)
- entrypoints: `src/main.rs` — `gpui_kit::application()` → `gpui_kit::init`
  → `Root::new(SettingsWindow)` with dialog/sheet/notification layers

## What are we working on right now?
- plan: docs/plans/completed/real-settings.md — completed 2026-09-23;
  every settings control produces a real effect (repaint, rem-scale,
  gated send, OS autostart, live locale, profile preview) and values
  persist to a platform config file (`key=value` with an ownership
  marker, atomic write). No active plan.
