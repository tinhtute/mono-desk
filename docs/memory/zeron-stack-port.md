---
id: zeron-stack-port
date: 2026-10-28
status: active
---
# Zeron stack port (owner decision, 2026-10-28)

- Owner decision: after several gpui-kit/gpui-component attempts diverged from
  the reference, the owner said "drop custom của mình làm theo luôn zeron của
  họ đi, dùng của họ" (drop the custom approach, copy zeron's, use theirs).
  Earlier in the same session they chose "Port nguyên code zeron".
- Authority: zeron source is MIT (`github.com/zeronsh/zeron` @ 8b7afb3,
  Copyright 2026 Wing, cloned at `/tmp/zeron-src`). Its UI is **not
  drop-in**: `crates/ui` is ~255k lines over ~20 modules and depends on 19
  `zeron-*` domain crates (engine/proto/rpc/session/update/...), and its
  GPUI is a fork (`github.com/zeronsh/zui` @ b564a344, git dependency) plus
  a fork of gpui-component (`github.com/zeronsh/gpui-component` @ 59f8924,
  used only for `gpui_base::init`). The port therefore vendors/adapts the
  needed UI source and replaces zeron's app state with `AppSettings`.
- Feasibility proven: a `/tmp/zui-probe` bin depending on `gpui` +
  `gpui_platform` from the pinned `zeronsh/zui` rev builds on stable Rust
  (edition 2024) and opens a 900×600 CSD window. In the real app a direct AX
  probe sees one AXWindow (1320×880) and the native traffic lights, and the
  window enforces its 900×600 minimum; the AX *content* tree contains no
  GPUI-painted nodes, so tests/visual checks cannot rely on AX for app
  controls.
- Key port targets: `crates/ui/src/lib.rs` `open_main_window` (window
  options: 900×600 min, 1320×880 default, transparent titlebar, lights
  (14,14), `app_owns_titlebar_drag`, glass `window_background`),
  `shell.rs` `render_settings_page`/`render_sidebar`/`sidebar_tone`/titlebar
  cluster/`render_sidebar_footer_row`, `settings/widgets.rs` (page scaffold +
  section cards + toggle/select/dropdown), `crates/theme` + `theme.rs`
  (tokens/appearance), `icons.rs` + `assets/icons` (Solar Icons, CC BY 4.0),
  `motion.rs`, `frost.rs`, `glass.rs`.
- Rollback/lineage: the gpui-kit implementation (glass + zeron-style sidebar
  work) is preserved on branch `wip/gpui-kit-shell` (commit 50823e3); the
  port works on `feat/zeron-stack` and the plan is
  `docs/plans/active/zeron-stack-port.md`.

## Ported modules (phase `theme-foundation`, 2026-10-29)

`vendor/zeron-ui/` is a workspace member adapted from zeron `crates/ui` @
8b7afb3; `vendor/zeron-theme/` holds `crates/theme`. Port map and the
adaptations that keep the upstream import closure out:

| Vendored file | Upstream | Adaptation |
|---|---|---|
| `theme.rs` | `crates/ui/src/theme.rs` | `zeron_syntax::HighlightKind` → local `highlight.rs`; wallpaper tinting and the `gpui_base` scrollbar sync dropped (no gpui-component fork); tests dropped |
| `typography.rs` | `crates/ui/src/typography.rs` | rewritten around the bundled families (Momo Trust Sans, Hanken Grotesk, Google Sans Code, Xanh Mono); `fontdb`/`ttf-parser` inspection, zeron's settings-store writes and its `appearance` module dropped |
| `icons.rs` + `assets/icons/` | `crates/ui/src/icons.rs` + `assets/icons` | full 128-SVG set; file-icon theme fallback dropped |
| `motion.rs` | `crates/ui/src/motion.rs` | `zeron_proto` phase math vendored into `loader_math.rs`; OS reduce-motion detection and settings writes dropped |
| `frost.rs`, `glass.rs` | same names | verbatim |

Port rules that held: layout/animation constants and call shapes stay; only
domain imports change; colour/token math stays zeron's (light is "designed,
not inverted": content plane white, chrome grey, accents at the 600 level).

State after phase 2: `Theme` installs as a gpui `Global` from the window's
appearance (or `MONO_DESK_APPEARANCE`), the native window background follows
`Theme::window_background_appearance()` (`Blurred` under zeron's default
frosted surface), all four font families register, and the icon asset source
serves the SVGs. `MONO_DESK_SELFTEST=1` prints the resolved tokens/fonts/icon
as the machine-checkable half of a phase check — `window.render_to_image()` is
gated behind gpui's `test-support` feature and unimplemented on the macOS
platform, and macOS Screen Recording is not granted to agent tooling.

Gate wiring: the vendored crates are workspace `default-members` (so `cargo
test` runs their 26 unit tests; `vendor/zeron-theme` regains `tempfile` as a
dev-dependency), while `make lint`/`make fmt` stay scoped to `-p mono-desk` —
vendored files carry upstream clippy style lints and `cargo fmt` would rewrite
them away from the pinned rev.
