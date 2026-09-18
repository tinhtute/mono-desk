# AGENTS.md

## Project

`mono-desk` — a settings-style desktop demo app. Single binary crate, Rust
edition 2024, one dependency: `gpui-kit = "0.6"` (re-exports `gpui`,
`gpui-base`, `gpui-component`, assets/icons — no other GPUI crates needed).

## Commands

Use the Makefile — it auto-injects the linker shim flag (see caveats):

- `make run` — build and launch the app
- `make check` — fast type-check (~30s cold)
- `make lint` — clippy `--all-targets -D warnings` (run before committing)
- `make fmt` — format before committing
- `make deps` — install system packages (sudo; Ubuntu 24.04 tested list)
- `make shim` — recreate linker shims without sudo

## Environment caveats (this machine)

- Missing dev symlinks `libxkbcommon-x11.so` and `libX11-xcb.so`. Shimmed at
  `~/.local/lib/dev-shim/`; the Makefile adds `-L` only while that dir exists.
  Proper fix is `make deps`, then delete the shim dir.
- GUI app: needs a Wayland/X11 session to open a window — don't expect
  `cargo run` to succeed headless; `make check`/`lint` are the headless gates.
- First build is ~400 crates (GPUI + wgpu); dep cache in `target/` is ~8 GB.
  Avoid `cargo clean` unless disk pressure demands it.
- Changing `RUSTFLAGS` or edition invalidates the whole dep cache — keep flags
  stable via the Makefile.

## Code conventions

- Bootstrap pattern (required, see `src/main.rs`):
  `gpui_kit::application().with_assets(gpui_kit::assets::Assets)` →
  `gpui_kit::init(cx)` before any component use → window root must be
  `Root::new(view, window, cx)` → render `Root::render_{dialog,sheet,notification}_layer`
  as children for overlays to work.
- App state lives in a `Global` (`cx.set_global` in the view constructor) —
  `SettingField` getters/setters only receive `&App`/`&mut App`, they cannot
  reach the view's `Context` directly. Clone the `Entity` handle into the
  closure if view access is needed (`view.update(cx, ...)` + `cx.notify()`).
- Every `SettingField` gets `.default_value(...)` so page reset buttons work.
- Styling is Tailwind-flavored chained methods (`div().v_flex().gap_2()...`);
  theme colors via `cx.theme().*` (`ActiveTheme`), never hardcoded colors.
- `WindowExt` trait gives `window.open_dialog/open_sheet/push_notification`.
- Icons: `IconName` variants are PascalCase names from `gpui-kit-assets`
  (Lucide subset — check `default-icons.txt` upstream before using a new one).

## References

- Docs: https://gpui-kit.com/docs · API: https://docs.rs/gpui-kit
- Upstream repo: https://github.com/longbridge/gpui-kit
  - `examples/ai_recipes/` — tested consumer patterns (bootstrap, settings, tests)
  - `crates/story/` — component gallery; `cargo run` in that repo shows all controls
- Agent skill pack for this stack: `npx skills add longbridge/gpui-kit`

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
