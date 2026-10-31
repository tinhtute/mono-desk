# mono-desk

A settings-style desktop demo app built on **zeron's GPUI stack** — the
[`zeronsh/zui`](https://github.com/zeronsh/zui) fork of Zed's GPUI engine
(pinned rev) with the window chrome, settings layout, widgets and theme ported
from zeron's MIT-licensed source
([`zeronsh/zeron`](https://github.com/zeronsh/zeron)) into `vendor/`.

## Features

- Zeron's Settings-route shell: 38px titlebar drag strip, glass root surface
  over a blurred window background, sidebar tone, seven-section list and footer
  identity row
- Seven settings pages on zeron's own page kit: General (language, time zone,
  launch at login), Appearance (System/Light/Dark mode cards with previews,
  theme variants, accent swatches, Glass surface, motion, font size),
  Notifications (delivery, channels, digest, test notification), Profile
  (username / display name with validation and preview), Finance UI (the
  eight-card showcase rebuilt from the pre-port app), Reset (per-setting and
  two-press reset-all), About
- Sidebar search that filters the section list (⌘F to focus, Escape to clear)
- Live theme: appearance/accent/glass/font-size are applied immediately through
  zeron's theme registry; the native window background follows the surface
  material
- Persistence: `key=value` settings file with an ownership marker, atomic
  writes, and OS autostart convergence (macOS LaunchAgent / Linux `.desktop`)

## Requirements

- Rust toolchain (edition 2024 — Rust 1.85+)
- macOS or Linux with a desktop session (GPU-rendered window)
- System packages on Ubuntu/Debian: `make deps` installs the full list

## Quick start

```bash
make deps   # first time only — system packages (sudo)
make run    # build and run the app
```

Headless review hooks (no GUI capture needed): `MONO_DESK_SELFTEST=1` prints the
resolved appearance, theme tokens, fonts, chrome geometry and settings state;
`MONO_DESK_SECTION=<general|appearance|notifications|profile|finance-ui|reset|about>`
opens the shell on that page; `MONO_DESK_APPEARANCE=light|dark` pins the palette;
`MONO_DESK_SEARCH=<text>` seeds the sidebar search.

## Make targets

| Command | Description |
| --- | --- |
| `make run` / `make build` | Run / build debug binary |
| `make app` / `make open-app` | Bundle a macOS `.app` with the brand icon |
| `make release` / `make run-release` | Optimized build |
| `make check` | Type-check without codegen |
| `make test` | Run tests |
| `make lint` | Clippy with warnings denied |
| `make fmt` / `make fmt-check` | Format / format check |
| `make deps` | Install system dependencies (sudo) |
| `make shim` | Recreate linker shims, no sudo |
| `make clean` | Remove build artifacts |

## Linking note

If the build fails with `unable to find library -lxkbcommon-x11`, install
`libxkbcommon-x11-dev libx11-xcb-dev`, or run `make shim` — the Makefile
auto-applies `~/.local/libdev-shim` while it exists.

## Third-party notices

Vendored zeron UI/theme (MIT), the Phosphor Icons subset (MIT), the zui GPUI
fork (Apache-2.0), the input example adapted into `src/settings/text_field.rs`
(Apache-2.0), bundled fonts (OFL-1.1) and the theme palettes are recorded in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md). The port plan and module map
live in [`docs/plans/active/zeron-stack-port.md`](docs/plans/active/zeron-stack-port.md)
and [`docs/memory/zeron-stack-port.md`](docs/memory/zeron-stack-port.md).

## License

Apache-2.0
