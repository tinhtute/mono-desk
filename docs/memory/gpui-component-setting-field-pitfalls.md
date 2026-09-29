---
id: mem-gpui-setting-field-pitfalls-20260923
created: 2026-09-23
---
# gpui-component SettingField pitfalls (mono-desk)

Two traps hit while wiring real effects into `SettingField` setters
(gpui-component 0.6.1, vendored):

1. **`NumberField` accepts non-finite input.** `"NaN"`.parse::<f64>()
   succeeds and `f64::clamp` passes NaN through, so the app setter
   receives `f64::NAN`. Guard at the app boundary
   (`val.is_finite()`), especially before feeding the value into
   `px()`/`window.set_rem_size` — NaN there poisons all rem-based
   layout. See `normalize_font_size` in `src/main.rs`.
2. **Setters never repaint.** `StringField`/`NumberField` change
   subscriptions only invoke the app setter — no `cx.notify`, no
   window refresh. Any cross-control effect (a Preview item, a gated
   button) needs `cx.refresh_windows()` inside the setter.
3. **`Theme::change` re-applies `font.size` from the ThemeConfig.**
   A runtime font-size override must be re-applied after every
   `Theme::change` call (both the settings switch and the rail
   toggle), or the mode flip silently resets it.
4. **Persisted defaults must mirror app defaults.** A derived
   `Default` on a stored-settings struct yields `0.0`/`""`/`false`;
   loading a missing file then collapses `set_rem_size` to 0px.
   Hand-write `Default` to match the app's `Default`.
