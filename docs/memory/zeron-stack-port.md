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
