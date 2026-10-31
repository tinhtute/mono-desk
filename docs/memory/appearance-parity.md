# Appearance parity — module map and adaptation rules

Companion to `docs/plans/completed/appearance-parity.md`. The Appearance page
is a 1:1 port of zeron's `crates/ui/src/settings/appearance.rs` @ `8b7afb3`
(MIT) onto mono-desk's vendored kit; this note records where everything lives
and the adaptations a future session must preserve.

## Module map (`src/settings/appearance/`)

| File | Zeron source | Responsibility |
|---|---|---|
| `mod.rs` | :4185-4289, :4238-4281, :4089-4092 | page view, scroll host + floating rail, assembly order, modal priority |
| `color.rs` | :3562-3704, :2144-2203, :4238-4251 | mode cards, theme selectors (218/260), accent row, wallpaper-colors row |
| `material.rs` | :3706-3957, :1141-1210 | Glass (148), New-thread background tile/actions/effect (128), Wallpaper folder + Shuffle |
| `motion.rs` | :3958-4083 | Reduce motion (128), Pause animations in background, Smooth scrolling |
| `library.rs` | :3321-3543, :4272-4280 | theme-library card (mt32), six entry actions, warning line (page tail) |
| `fonts.rs` | :4094-4182, :1958-2142, :735-1004 | three font rows, 220px searchable pickers, 128px size selects, error strips |
| `font_model.rs` | :129-296, :1411-1580 | FontKind model, size ladders, step/filter/first/last/fallback/format helpers |
| `layout.rs` | :515-678, :398-513 | Conversation width slider (240x28), Render math, Math font (160/220), Math size (160) |
| `preview.rs` | :1526-1705 | `bar`/`miniature`/`miniature_split`/`preview`/`palette_preview` |
| `dialogs/mod.rs` | :4089-4092 | modal stack dispatcher + host drag/mouse-up plumbing |
| `dialogs/background.rs` | :44-112, :1212-1366, :2747-3265 | background adjustment dialog (≤600 card, 260px well, 168x28 zoom slider) |
| `dialogs/library.rs` | :2205-2746, :3267-3319, :1715-1881 | import dialog (600), review dialog (660), scene/report helpers |

Settings/persistence backing: `src/app_settings.rs` (42-key `settings.conf`,
typed mirrors), `src/persist.rs` (frozen order, escaped paths),
`src/settings/mod.rs` (`apply_appearance` funnel, boot reconcile),
`src/settings/background.rs` (managed image install/remove + checked persist),
`src/settings/wallpaper.rs` (folder/source/random pick),
`src/settings/wallpaper_colors.rs` (async tint extraction),
`src/settings/theme_library.rs` (import/install bridge over
`vendor/zeron-theme`), `src/i18n.rs` (four locales). Vendored additions:
`vendor/zeron-ui/src/{typography.rs, theme.rs, settings/widgets.rs}`
(`PageScroll`/`rail`, installed-font catalog, tint variants) and
`vendor/zeron-ui/src/new_thread_background_{effects,mask}.rs`.

## Adaptation rules (do not "fix" these back)

- **No zeron settings service**: every control writes the `AppSettings` global
  and funnels through `settings::apply_appearance`. Theme-library mutations run
  `theme_library::reconcile` on the persisted `theme_light`/`theme_dark` ids
  first (the `apply_registry_change` substitute).
- **`AppearancePage::new()` is cx-free** (`shell.rs` owns the signature): modal
  state creates its `Entity<TextField>` lazily (`State::open`/`ensure`), so
  `Default`-constructible section states are the contract.
- **`dialogs` is private to its dispatcher**: dialogs are opened through
  `State::open(cx)` methods, not module re-exports.
- **Toast → inline**: background/wallpaper/library errors render as an inline
  strip/warning; settings-domain error constants are mapped through
  `appearance.*.error.*` at the page boundary (unexpected I/O diagnostics stay
  raw — recorded decision).
- **Preview colors**: `Theme::from_variant` is crate-private, so import/review
  previews paint from `ThemeVariant` model roles (palette/scene/report); they
  are semantically, not pixel-, identical to an installed `Theme`.
- **Vendored mirror types**: `Framing` and `BackgroundEffect` mirror the app's
  persisted adjustment/effect enums (vendored crates cannot depend on
  `src/app_settings.rs`); the dialog converts at the boundary.
- **Trimmed subsystems (plan NG1-NG6)**: no wallpaper painting layer, no
  transcript/composer/Markdown surface, no cloud sync, no settings-search
  anchors/`page_reveal`, no kinetic-scroll engine, no composer artwork/hero
  crossfade/preload in the vendored background modules.
- **UI vocabulary**: page copy through `tr`; ReduceMotion option words are
  localized page-side; vendor data words (theme/family names, paths) stay as
  data.
- **No mouse-up-out hook**: the host wires `on_drag_move` + `on_mouse_up`; the
  transcript width still flushes through its `on_next_frame` publish.

## Verification hooks

`MONO_DESK_SELFTEST=1` + `MONO_DESK_SECTION=appearance` prints nav,
appearance/rem markers, `selftest library entries=N warning=…`, boot order and
`rem_state`/`rem_window`. Background/dialog render paths need state, so they are
exercised with crafted `HOME` dirs (managed PNG, corrupt library, vi/dark) or
remain code/geometry-reviewed. There is no GPUI widget-interaction harness and
no pixel capture: the owner supplies the visual pass. Headless gates:
`cargo test` (206 = 119 mono-desk + 24 zeron-theme + 63 zeron-ui),
`make check`, `make lint`, `make fmt-check`.
