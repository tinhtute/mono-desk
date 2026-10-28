---
id: mem-gpui-window-glass-20261027
created: 2026-10-27
updated: 2026-10-28
---
# Window transparency + glass in GPUI 0.3.5 (mono-desk)

Facts that cost a detour while adding the Background (glass) settings in
`src/main.rs` / `src/shell.rs` / `src/theme.rs`. Re-read before touching
window chrome.

1. **Only one window-level knob exists.** `WindowBackgroundAppearance`
   (`Opaque` / `Transparent` / `Blurred`, plus `MicaBackdrop` /
   `MicaAltBackdrop` on Windows) via `WindowOptions.window_background` and
   `Window::set_background_appearance` (runtime per window). macOS `Blurred`
   installs a fixed `NSVisualEffectView` (material `Selection`, saturation
   filters stripped); there is **no** radius/material API — monocode's
   1–64 px blur radius needs private AppKit. Linux blurs only on compositors
   that advertise a blur protocol; Windows acrylic under GPUI's
   DirectComposition windows is unverified. Non-opaque windows also lose
   subpixel text antialiasing.
   **macOS 26 risk:** zeron's `zui` fork switched the blurred view to
   `UnderWindowBackground` because macOS 26 stopped vending `CABackdropLayer`
   for `Selection` ("window blur went dead"). gpui-pre 0.3.5 still uses
   `Selection`, so the Blur toggle may silently stop working after an OS
   upgrade — verify on macOS 26 before claiming blur support.
2. **`theme.colors` and `theme.tokens` are two layers.** Vendored `Root`,
   dialogs and sheets paint `theme.tokens.background`, while app code reads
   `cx.theme().background` (deref to `colors`). `Theme::sync_base` projects
   only `colors` into gpui_base, so alphaing the palette does **not** reach
   those vendored surfaces; alphaing the `tokens` snapshot instead also
   makes dialogs translucent (unreadable — GPUI has no per-element blur).
   The shipped answer is paint-time alpha (`theme::glass_surface`) plus a
   `Root` style override, never palette mutation.
3. **`Root` owns the last opaque full-window paint.** A view inside `Root`
   cannot style it, so glass needs the `Entity<Root>` handle: `main()`
   publishes it in a `Global` (`WindowRoot`), and `apply_glass` sets
   `root.style().background = Some(transparent)` while glass is on and
   `None` to restore the token paint. `Root::new` returns `Root`, not an
   `Entity`, so the handle only exists where a closure can wrap it in
   `cx.new` (the app's `open_window`, or a dedicated test window).
4. **Stacked alpha compounds — one glass *base* per pixel.** Two bases at
   alpha α render `1-(1-α)²` over the backdrop (85 % panes read 97.75 %).
   The shipped model paints **one** full-window frost layer; every other
   app-owned surface above it is transparent. A tone wash on top of the
   frost is fine (it is ink, not a second base).
5. **Frost-shell model (current, zeron-style).** `AppShell` paints
   `shell_frost_bg` = `glass_surface(theme.background, window_opacity)`
   exactly once; the title bar paints nothing; the sidebar paints
   `sidebar_tone_bg` (theme foreground at `theme::sidebar_tone_alpha`,
   0–100 % of a 0.10 max, default 50 % = 0.05 wash) plus a 1px
   `border_r` hairline; the content is flush (no card/margin/shadow) while
   glass is on. Opaque mode keeps the shipped card and the vendored
   sidebar token paint. Sliders show the *stored* clamped values
   (`window_opacity` 40–100 default 85, `sidebar_tone` 0–100 default 50)
   while paint uses the effective ones (opaque / zero when translucency is
   off); page reset runs every field handler, so it must reset to the
   stored defaults.
6. **Re-apply the window background after theme/appearance swaps.** gpui's
   macOS backend removes the `NSVisualEffectView` the moment the background
   appearance is anything but `Blurred` and never puts it back; zeron
   defers `reapply_window_background` after every theme change (zed runs the
   same loop). mono-desk currently sets it only in `apply_glass` on the
   toggles — if theme-swap code ever writes it, re-apply unconditionally.
7. **The vendored `Sidebar` zeroes padding on its own root.** With the
   titlebar turned into an absolute overlay (zeron layout), a `.pt(38)` on
   `sidebar_refinement` is silently discarded by
   `vendor/gpui-component/src/sidebar/mod.rs`; the only working offset is the
   app-owned header element (`div().pt(sidebar_header_padding()).child(brand)`).
   The content panel's `ResizablePanel` style *does* honor its padding, so
   only the rail header needs the explicit wrapper — and a test must assert
   `sidebar_header_padding()`, not the discarded refinement field.
