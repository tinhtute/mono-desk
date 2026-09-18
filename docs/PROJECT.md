# PROJECT — identity

## What is this project?
- `mono-desk` is a settings-style desktop demo app: a single-window GPUI
  application built on `gpui-kit` 0.6, showcasing the `Settings` component
  with grouped pages, fields, and overlay layers.

## Who is it for?
- Its owner (lab project) and developers evaluating gpui-kit patterns for
  real desktop apps.

## Non-goals
- Not a production settings/productivity app; no persistence backend, no
  i18n implementation, no multi-window or multi-view product surface.

## What are the gate commands?
- run from: repository root (`/home/tinhpt/Lab/mono-desk`)
- tests: n/a (no test suite)
- types: `make check`
- lint: `make lint`
- build: `make run` (compiles; window needs a GUI session)
- format: `make fmt`

## Architecture in one breath
- runtime shape: single binary crate, Rust 2024, one dep `gpui-kit`
- where state lives: `AppSettings` in a GPUI `Global` set inside the view
  constructor (`cx.set_global`)
- entrypoints: `src/main.rs` — `gpui_kit::application()` → `gpui_kit::init`
  → `Root::new(SettingsWindow)` with dialog/sheet/notification layers

## What are we working on right now?
- plan: docs/plans/active/mono-design-port.md (active)
