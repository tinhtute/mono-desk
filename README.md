# mono-desk

A settings-style desktop demo app built with [GPUI Kit](https://gpui-kit.com/) — the Rust desktop UI framework from Longbridge, built on Zed's GPUI engine.

## Features

- Multi-page settings UI with sidebar navigation (`Settings` / `SettingPage` / `SettingGroup` / `SettingItem`)
- Live dark/light theme switching
- Switch, checkbox, dropdown, number-input, and text-input fields
- In-app notifications and modal dialogs via the `Root` overlay layers
- Global application state pattern (`impl Global`)
- Per-page reset-to-defaults via `default_value` on fields

## Requirements

- Rust toolchain (edition 2024 — Rust 1.85+)
- Linux/macOS/Windows with a desktop session (GPU-rendered window)
- System packages on Ubuntu/Debian: `make deps` installs the full list

## Quick start

```bash
make deps   # first time only — system packages (sudo)
make run    # build and run the app
```

## Make targets

| Command | Description |
| --- | --- |
| `make run` / `make build` | Run / build debug binary |
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
auto-applies `~/.local/lib/dev-shim` while it exists.

## License

Apache-2.0
