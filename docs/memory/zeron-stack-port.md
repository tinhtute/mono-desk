---
id: zeron-stack-port
date: 2026-10-28
status: completed
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
  (tokens/appearance), `icons.rs` + `assets/icons` (Phosphor Icons, MIT; slot
  names kept from zeron, map per line in `icons.rs`),
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
test` runs their unit tests: 34 today — 24 `vendor/zeron-theme`, 10
`vendor/zeron-ui`; `vendor/zeron-theme` regains `tempfile` as a
dev-dependency), while `make lint`/`make fmt` stay scoped to `-p mono-desk` —
vendored files carry upstream clippy style lints and `cargo fmt` would rewrite
them away from the pinned rev.

## Ported chrome (phase `chrome`, 2026-10-29)

Owner decision of 2026-10-29: the app has exactly one surface, so its chrome
is zeron's **Settings route**, literally — not the chat route. zeron's
settings route paints no titlebar control cluster, no centered title and no
sidebar resize/collapse (those belong to the chat shell), and its `Back` row
only exists to return to the chat. The owner chose "copy y màn Settings của
zeron" and "bỏ nút Back, sidebar bắt đầu bằng ô search"; both are recorded as
R2/R3 amendments in the plan.

| Vendored / first-party | Upstream | Adaptation |
|---|---|---|
| `vendor/zeron-ui/chrome.rs` | `crates/ui/src/shell.rs` (`titlebar_cluster_start`, `titlebar_spacer_width`, `caption_buttons_width`, `cluster_buttons_start`, `cluster_clearance`, `titlebar_right_padding`, `TITLEBAR_*`/`CLUSTER_BUTTONS_WIDTH`, `WINDOWS_CAPTION_*`) | geometry + upstream's five tests only: the caption buttons (NG3) and the titlebar *element* helpers (`titlebar_spacer`, `window_control_button`/`_with`) are not ported — R2's settings surface paints no cluster, so they would be dead code; `TITLEBAR_CLUSTER_PAD` stays private like upstream |
| `vendor/zeron-ui/settings/mod.rs` | `shell.rs` `SettingsSection` | mono-desk's seven sections in zeron's order (General, Appearance, Notifications, Profile, Finance UI, Reset, About) with zeron's label/icon/`starts_nav_group` conventions; the `Appearance` row exists now, its page is `settings-surface` T3 |
| `vendor/zeron-ui/settings/widgets.rs` | `settings/widgets.rs` (`section_tab`, `tab_selection_t`, `TabSelectionTravel`) | verbatim; `scroll_faded`/edge-fade wrappers not ported yet |
| `src/shell.rs` (first-party app wiring) | `shell.rs` (`Shell::render` root, `sidebar_tone`, `titlebar_drag_region`, `render_settings_page` skeleton, `render_settings_nav`, `render_sidebar_footer_row` user-menu trigger) | app state reduced to the selected section + roving focus handles + the drag flag; chat chrome not ported (see above); no account popover — the footer pill is identity only, and the footer's square action is the appearance toggle |

Settings-route layout recipe as ported: root `id("shell-root")` +
`.relative().flex().flex_row().size_full().bg(theme.glass())` +
`text_color(theme.text)` + `font_family(theme.font_sans)` +
`text_size(ui_rems(14))`; absolute `sidebar_tone` (`wash(0.05)` + `border_r_1`
with `theme.border`) at 256px; sidebar column `pt(TITLEBAR_HEIGHT)` (no Back
row); page pane `flex_1` whose scroll carries `TITLEBAR_HEIGHT + SPACE_LG` top
clearance; a full-width 38px `WindowControlArea::Drag` strip above the page
(`start_window_move` guarded by `pressed_button`, double-click
`titlebar_double_click`).

Section list (T2), zeron `render_settings_nav`: `Role::TabList` +
`aria_label("Settings sections")` column, 2px gaps, rows built by
`widgets::section_tab` with `aria_selected`, `Role::Tab`, roving
`tab_stop(selected)` handles, `focus_visible` accent ring, up/left →
previous, down/right → next, home/end, and the workspace group
(`FinanceUi`) starting a new group with `mt(SPACE_LG)`. Footer (T2): zeron's
`SIDEBAR_FOOTER_*` metrics (28px pill/action, 16px avatar, 4px action gap),
initial-avatar pill + name, appearance toggle in the action slot.
`vendor/zeron-ui/assets/icons/user.svg` is the Phosphor `user` glyph in that
slot — no icon slot is a hand-drawn mono-desk addition any more (the set was
swapped from Solar to Phosphor on 2026-10-31; see the swap record below).
Known gaps in this recipe (gate M6, recorded in the
plan's `settings-surface` known-gaps list): the nav still has no
`scroll_faded`/edge-fade wrapper, and the footer pill is identity-only (no
role/aria/popover, no `sidebar_faded_label`).


## Settings surface (phase `settings-surface`, wave 1, 2026-10-29)

Wave 1 rebuilt the two simplest pages on zeron's own page kit and vendored the
real select stack, so no page control is an approximation:

| Vendored file | Upstream | Adaptation |
|---|---|---|
| `vendor/zeron-ui/settings/widgets.rs` | `settings/widgets.rs` (page scaffolding: `page_column`, `page_header`, `page_subtitle`, `field_label`, `option_card*`, `section*`, `section_card`, `card_row`, `row_tile`, `row_title`, `meta_line`, `block_fill`, `row_divider`, `badge*`, `toggle_switch` + switch tests) | verbatim; only imports + header changed. Popover-dependent widgets (`dropdown`, `select`, `scroll_faded`, `text_tooltip`, action buttons, strips) live in the other files below |
| `vendor/zeron-ui/settings/menu.rs` | `settings/widgets.rs` lines 14–166 (`SettingsSidebarWidth`/`set_sidebar_width`, `pane_bounds`, `column_width`, `dropdown`, `dropdown_limits`, `dropdown_rows`, `dropdown_list_height`, `DropdownRows`) | new file because `widgets.rs` is owned separately; `SIDEBAR_DEFAULT` (256.0) defined here citing `crates/ui/src/settings.rs`, `SettingsDropdown` + fields widened to `pub(crate)` |
| `vendor/zeron-ui/settings/select.rs` | `settings/widgets.rs` lines 1101–1537 (`select`/`Select`, `SelectState`, `SelectOption`, `select_*` helpers, `close_select`) | new file, verbatim otherwise |
| `vendor/zeron-ui/popover.rs` + `popover/{contained,hover_intent}.rs` | `crates/ui/src/popover.rs` + submodules | verbatim except the `search_highlight*` helpers + their tests, which paint through the unvendored `crate::markdown`; the anchored/deferred menu layer, reducers, rails and 17 pure tests are in |
| `vendor/zeron-ui/edge_fade.rs` | `crates/ui/src/edge_fade.rs` | verbatim; required by `dropdown_rows`'s scrolling list (`edge_faded(...).fade_overflow_y(...)`); the 2 windowed `#[gpui::test]`s are dropped (vendored crate does not enable gpui `test-support`) |

First-party layout:

| File | Role |
|---|---|
| `src/app_settings.rs` | `AppSettings` (18 fields, `Global`), `Appearance`, `DigestCadence`, `from_stored`/`to_stored`/`save_settings`, and the old app-theme defaults/clamps/accent codec (old `theme.rs:25–186`) |
| `src/persist.rs` | the old settings file codec, ported verbatim: marker `# managed-by: mono-desk`, unknown-key tolerance, legacy `dark_mode` fallback, foreign-file guard, atomic write, autostart plists |
| `src/i18n.rs` | the old 107-key × 4-locale table (`tr(key, lang)`) |
| `src/settings/{mod,general,notifications}.rs` | the pages; `mod.rs` holds `persist_now` (write-through: mirror global → `to_stored` → `persist::save`) and `control_row` (zeron's title/meta/control row shape) |
| `src/shell.rs` | page pane hosts `Entity<GeneralPage>`/`Entity<NotificationsPage>` (lazily created, kept like zeron's shell keeps page entities); `menu::set_sidebar_width(256.0, cx)` is published every frame so dropdowns measure the pane; footer reads `AppSettings.display_name` |
| `src/main.rs` | installs `AppSettings::from_stored(persist::load())` as a global before the window opens |

Page conventions that must hold when adding the next pages:
- a page is a `gpui` *view* (its own `Entity`), because zeron's `select` is
  generic over the page type and stores open/highlight state in a
  `SelectState` field on that page;
- the page owns its `ScrollHandle` and its scroll host; the pane in
  `src/shell.rs` is only `flex_1` + full height (no scroll of its own), so
  `page_column`'s `TITLEBAR_HEIGHT + 16` clearance scrolls with the content as
  in zeron;
- rows are `widgets::section` / `section_card().mt_0()` / `super::control_row`;
  a write does `AppSettings::global_mut(cx).field = value;`
  `super::persist_now(cx); cx.notify();`
- there is no checkbox/radio/segmented widget in zeron's kit: booleans are
  `toggle_switch` rows, choices are `select`.

Gates today: `cargo test` = 94 tests (mono-desk 33 = 9 `app_settings` +
3 `i18n` + 21 `persist`; `vendor/zeron-theme` 24; `vendor/zeron-ui` 37).
`make lint`/`fmt-check` remain `-p mono-desk`. Dev hooks grew one:
`MONO_DESK_SECTION=<general|appearance|notifications|profile|finance-ui|reset|about>`
opens the shell on that section, so a headless selftest renders that page
(`MONO_DESK_SELFTEST=1` prints `selftest settings language=… timezone=…`).

## Settings surface (phase `settings-surface`, waves 2–4, 2026-10-29)

Waves 2–4 finished the phase: every page renders on zeron's kit, and the
theme/accent/surface/font-size state has a single apply path.

Vendored additions (same file ownership as wave 1):

| Vendored file | Upstream | Adaptation |
|---|---|---|
| `settings/widgets.rs` +T2 | `action_button`/`text_action`/`ghost_action` + `ActionTone` (1553-1598), `error_strip` (1600-1628), `warning_strip` (1630-1657), `scroll_faded` + `SettingsScroll` (1659-1732), `TextTooltip`/`text_tooltip`/`text_tooltip_above` (1734-1786) | verbatim; `select_fill` reached through `crate::settings::select::select_fill` (it lives in the split-out select module) |
| `settings/widgets.rs` +T3 | `accent_swatch` (`settings/appearance.rs` 1883-1960) | verbatim minus the `widgets::` path; made `pub` and typed on `zeron_theme::AccentSelection`, because the Appearance page is a first-party crate |
| `theme.rs` +T3 | `model_color` | made `pub` so the first-party page can compare an accent preset's authored color with the persisted `#RRGGBB` |
| `fidelity-close` follow-up note | `PageScroll`/`rail` remain unported: this app's shell paints its own nav and uses gpui's default scrollbars |

First-party additions:

| File | Role |
|---|---|
| `src/settings/text_field.rs` | single-line text input adapted from the zui fork's `crates/gpui/examples/input.rs` (Apache-2.0, attributed in `THIRD_PARTY_NOTICES.md`): caret/selection/clipboard/IME preserved, colors from `Theme`, `TextField::new(id, placeholder, cx)`, `.with_value()`, `.on_change()`, `value`/`set_value`; `TextField::key_bindings()` registers the editing actions for the `TextField` key context (bound in `main.rs`; Escape = `Clear`) |
| `src/settings/{profile,about,reset,appearance,finance}.rs` | the remaining pages; each is a view owning its `SelectState`s/`ScrollHandle`; the shell keeps all seven entities and its placeholder renderer is gone |
| `src/finance/` | the pre-port `src/cadence/` showcase rebuilt on zeron widgets/tokens: `mod` (DemoState + data + accent ramp), `blocks`, `charts`, `controls`, `showcase`; 3059 lines vs 3180 old |
| `src/shell.rs` +T4 | search row (zeron `shell/settings_search.rs` layout): 32px field, magnifier in a nav glyph's slot, ⌘F hint/`shell_actions::FocusSearch`, clear button, Escape clears via the field action; `sections_for_query`/`section_matches` filter the seven rows; `MONO_DESK_SEARCH=<text>` seeds the query so a headless run can prove the filter |
| `src/main.rs` +T2/T3 | `motion::init` before the first window; startup `persist::autostart_converge`; `sync_appearance` and the footer toggle now call `settings::apply_appearance` |

Theme apply funnel (`src/settings/mod.rs`), the recipe every appearance
control uses:
- `apply_appearance(window, cx)`: resolve the appearance (honoring the
  `MONO_DESK_APPEARANCE` pin), map `AppSettings.accent` → `AccentSelection`
  (preset authored colors only; anything else = ThemeDefault), map
  `background_translucent` → `SurfacePreference::{Frosted,Opaque}`, call
  `Theme::install_selection(appearance, builtin_variant_id, accent, surface,
  cx)` (handles the cached-style generation bump and the typography font
  fields), then `typography::set_font_size(ui_size(font_size), window, cx)`
  and `window.set_background_appearance(...)`.
- `persist_now(cx)` after every write; `apply_launch_at_login(cx)` wherever
  `launch_at_login` can change (autostart entry converges, marker-guarded).
- Persisted 8–32 px font sizes snap into zeron's 12–20 catalog; the accent
  hex codec and all 18 keys/order are unchanged (NG4).
- Live-only (no persisted key, by design): the reduce-motion preference and
  the per-appearance theme-variant selects (built-in variant only).

Gates today: `cargo test` = 115 (mono-desk 54 = 9 `app_settings` + 3 `i18n` +
21 `persist` + 2 search + 4 appearance + 4 reset + 5 finance + 6 other;
`zeron-theme` 24; `zeron-ui` 37). Dev hooks: `MONO_DESK_SECTION=<slug>` (seven
pages), `MONO_DESK_APPEARANCE=light|dark` now also analyzed through
`apply_appearance` (the pin wins over the persisted choice),
`MONO_DESK_SEARCH=<text>` prints `selftest search query=… visible=…`.

## Icon set swap — Phosphor Icons (2026-10-31)

The UI glyph set was replaced wholesale: every non-brand slot under
`vendor/zeron-ui/assets/icons/` now carries a **Phosphor Icons** v2.1.1 glyph
(MIT, <https://phosphoricons.com/>), copied verbatim from
`@phosphor-icons/core@2.1.1` — `assets/regular/` for the line set, `assets/fill/`
for the two filled twin slots (`fast-tier-bold.svg`, `star-bold.svg`).
Phosphor's 256-unit canvas with a 16-unit outline matches the old Solar Linear
1.5-on-24 stroke, so icon sizes/colors at the call sites did not change.

Rules that keep the swap honest:
- Asset filenames keep zeron's slot names (`MAGNIFER` → `magnifying-glass` is
  still `icons/magnifer.svg`), so no vendored or first-party call site churned.
  The slot → Phosphor-glyph map is the per-line `// phosphor:` comments in
  `vendor/zeron-ui/src/icons.rs`; keep it in sync when changing a glyph.
- Brand marks keep zeron's artwork (no Phosphor equivalent): `antigravity-mark`,
  `claude-mark`, `cursor-mark`, `devin-mark`, `grok-mark`, `hermes-mark`,
  `muse-mark`, `openai-mark`, `opencode-mark`, `pi-mark`, `zeron-logo`.
  `assets/brand/` (app mark + `icon.icns`) is first-party artwork and untouched.
- The two animated panel glyphs (`icons::sidebar_glyph`,
  `icons::corner_card_glyph`) are drawn from quads, not assets, and keep
  zeron's geometry because the panel width morphs with motion state.
- The unreferenced `assets/icons/status/*` chip artwork (left over from the
  removed status-chip module) was converted to Phosphor too, so no icon file in
  the repo is from the old set.
- Guard: `vendor/zeron-ui/src/icons.rs::tests::ui_icons_are_phosphor_glyphs`
  loads every slot and asserts each non-brand glyph sits on the 256-unit canvas
  (the red/green proof for the swap). Gates after it: `cargo test` = 120
  (mono-desk 58 + zeron-theme 24 + zeron-ui 38), plus `make check`,
  `make lint`, `make fmt-check` — all green.
