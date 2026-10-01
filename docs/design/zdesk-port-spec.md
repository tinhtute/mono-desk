# zdesk Port Spec

Records the applied state of the zdesk → mono-desk (gpui-kit) UI port:
token derivation, shell metrics, font roles, interactions, deviations,
and vendored-patch state. Source of truth for source values is zdesk
(read-only): `docs/design-system.md`,
`packages/design-system/src/tokens/{colors,spacing,typography}.css`,
`packages/design-system/src/shell/{AppShell,TitleBar}.tsx`,
`packages/design-system/src/navigation/SidebarItem.tsx`,
`apps/desktop/src/features/shell/{NavView.tsx,useGlobalShortcuts.ts}`.
Applied state lives in `assets/themes/mono.json` + `src/theme.rs` +
`src/shell.rs` + `src/main.rs`. Locked owner decisions: keep `mono-light`
/`mono-dark` theme names carrying zdesk values; keep the Xanh Mono italic
wordmark; theme toggle lives only in Settings (Dark Mode switch on the
General page).

**Two-region layout (phases `zdesk-correction` + `zdesk-layout-fix`).**
`AppShell` is a `v_flex` of exactly two children
(`src/shell.rs:93-117`): a borderless transparent `TitleBar`
(`refine_style(border_0)` kills its built-in `border_b_1` seam — the
refinement lands after the component's own border in
`vendor/gpui-component/src/title_bar.rs:336-337`, so it wins) whose one
child is a ghost `PanelLeft` `Button` that toggles
`ShellState::collapsed`, parked right of the macOS traffic lights by the
title bar's built-in 80px left padding
(`TITLE_BAR_LEFT_PADDING`, `title_bar.rs:17`); and the `Settings`
element rendered directly under it. Settings' internal sidebar IS the
app sidebar — now collapsible between 240px and a 48px icon rail via
`ShellState` (a derived-`Default` `Global`, `src/shell.rs:20-27`);
there is still no separate nav column, destinations, StatusBar, or
EmptyState placeholders.

## Token derivation

zdesk derives every neutral from three primitives — `--theme-hue: 240`,
`--theme-saturation: 0%` (pure graphite), and a per-mode lightness pair
(`colors.css:15-87`):

- **dark (default):** `--background-lightness: 9%` → base `hsl(240 0% 9%)`
  = `#171717`; raised = base + 4% → `hsl(240 0% 13%)` = `#212121`;
  `--content-lightness: 92%` → `hsl(240 0% 92%)` = `#ebebeb`.
- **light:** `--background-lightness: 97%` → `#f7f7f7`; raised = base
  + 4% clamps → `#ffffff`; `--content-lightness: 18%` → `#2e2e2e`.

Every `color-mix(in srgb, X N%, transparent)` was resolved to a static
hex by compositing over the mode's base/raised surface (GPUI has no
`color-mix`); the resolved values below are what landed in
`assets/themes/mono.json`.

| zdesk token (`colors.css`) | Mix (light / dark) | Hex light | Hex dark | `ThemeConfigColors` key(s) |
|---|---|---|---|---|
| `background-base` | hsl 240 0% 97% / 9% | `#f7f7f7` | `#171717` | `background`, `sidebar.background`, `status_bar.background`, `title_bar.background`, `tab.background` |
| `background-raised` | +4% lightness | `#ffffff` | `#212121` | `group_box.background`, `popover.background`, `list.background`, `tab.active.background`, `tab_bar.background`, `accordion.background`, `table.background` |
| `content` | hsl 240 0% 18% / 92% | `#2e2e2e` | `#ebebeb` | `foreground`, `group_box.foreground`, `popover.foreground`, `button.foreground`, `secondary.foreground`, `tab.active.foreground`, `switch.thumb.background` (dark), `sidebar.accent.foreground` |
| `content-muted` (75% / 60%) | mix | `#606060` | `#969696` | `muted.foreground`, `tab.foreground`, `table.head/foot.foreground` |
| `content-subtle` (70% / 50%) | mix | `#6a6a6a` | `#888888` | `description_list.label.foreground` (dark raised from zdesk's `#818181` — 4.13:1 → 4.54:1), `scrollbar.thumb.hover.background` (dark variant) |
| `stroke` (content 7%) | mix | `#e9e9e9` | `#262626` | `border`, `drag.border`, `window.border`, `sidebar.border`, `status_bar.border`, `title_bar.border`, `list.active.border`, `table.active.border`, `table.row.border`, `button.active.background` (light), `slider.background` — **`input.border` is WCAG-adjusted** to `#8f8f8f` / `#6d6d6d` (see below) |
| `selection-subtle` (5% / 8%) | mix | `#ededed` | `#282828` | `muted.background`, `secondary.background`, `button.background`, `skeleton.background`, `tiles.background`, `list.even/head.background`, `table.even/foot/head.background`, `tab_bar.segmented.background`, `button.secondary.background`, `sidebar…` surface steps — **`switch.background` is WCAG-adjusted** to `#949494` / `#6b6b6b` (see below) |
| `selection` (6% / 10%) | mix | `#ebebeb` | `#2c2c2c` | `list.active.background`, `table.active.background`, `button.hover.background`, `secondary.hover.background`, `overlay` |
| `selection-hover` (10% / 15%) | mix | `#e3e3e3` | `#373737` | `list.hover.background`, `table.hover.background`, `accent.background`, `sidebar.accent.background` |
| `accent` | hsl 200 70% 24% / hsl 193 100% 29% | `#124b68` | `#007494` | `primary.background`, `button.primary/info.background`, `link`, `ring`, `caret`, `progress.bar.background`, `slider.thumb.background`, `sidebar.primary.background`, `info.foreground` (light) |
| `accent-hover` (accent 90%) | mix | `#295d76` | `#026b87` | `primary.hover.background`, `link.hover`, `button.primary/info.hover.background`, `info.hover.background` |
| `accent-tint` (accent 15%) | mix | `#d5dee2` | `#14252a` | `selection.background`, `drop_target.background`, `info.background` |
| `accent-content` | hsl 200 70% 24% / hsl 188 81% 49% | `#124b68` | `#18c7e2` | `link`/`link.active`/`link.hover` (dark), `info.foreground` (dark), `accent.foreground` |
| `success` | hsl 145 60% 30% / 45% | `#1f7a45` | `#2eb867` | `success.foreground`, `success.active.background`, `button.success.*`, `chart_bullish`, `base.green` |
| `success-tint` (15%) | mix | `#d7e5dd` | `#1a2f23` | `success.background` |
| `warning` | hsl 38 92% 30% / 55% | `#935f06` | `#f6a823` | `warning.foreground`, `warning.active.background`, `button.warning.*`, `base.yellow` |
| `warning-tint` (15%) | mix | `#e8e1d3` | `#382d19` | `warning.background` |
| `danger` | hsl 0 75% 42% / 60% | `#bb1b1b` | `#e64c4c` | `danger.foreground`, `danger.active.background`, `button.danger.*`, `chart_bearish`, `base.red` |
| `danger-tint` (15%) | mix | `#eed6d6` | `#361f1f` | `danger.background` |
| `scrim` (content 10%) | mix | `#e9e9e9` | `#2c2c2c` | `overlay` (shared with `selection` step where equal) |
| `scrollbar-thumb` (content 25%) | mix | `#8f8f8f` | `#6b6b6b` | `scrollbar.thumb.background` — WCAG-adjusted (see below) |
| `scrollbar-thumb-hover` (content 50%) | mix | `#7a7a7a` | `#818181` | `scrollbar.thumb.hover.background` — light WCAG-adjusted (see below) |
| `accent-foreground` | hsl 0 0% 100% | `#ffffff` | `#ffffff` | `primary.foreground`, `sidebar.primary.foreground`, `button.primary/info.foreground` |

Charts (`chart.1–5`) keep the prior mono palette (`#e5a14b`, `#3fb785`,
`#52b3d1`, `#8d6be7`, `#e95048` in both modes) — zdesk has no chart
tokens; `base.*` accent shades (`base.cyan` `#124b68`/`#18c7e2`, etc.)
track the zdesk accent/status hues.

Theme-level keys in both configs (`mono.json`): `"radius": 8`,
`"radius.lg": 16`, `"shadow": false`, `"font.family": "Inclusive Sans"`,
`"font.size": 14`, `"mono_font.family": "Google Sans Code"`,
`"mono_font.size": 12`.

**beUI pilot (2026-09-30).** The owner retuned the three non-colour keys
above — `font.size` 13 → 14, `radius` 6 → 8, `radius.lg` 10 → 16 — while
keeping every colour and `shadow: false`. `font.size` is also the window
rem base (`Root::render` → `set_rem_size`), so every rem-derived helper
(`h_7`, `text_sm`, `rounded_lg`, …) now renders at 14/16 of its Tailwind
nominal instead of 13/16; the px annotations in the tables below are the
13px-era values. `theme::tests::default_and_reset_agree_with_the_config`
pins the app's startup default to the config.

**beUI token layer (2026-10-01).** The non-colour token layer is decided
and pinned; colours, `font.size` 14, and the vendored patch set are
frozen (owner decisions O1–O3).

Radius roles (D3):

| Role | Token | Value | Consumers |
|---|---|---|---|
| control | `theme.radius` | 8px | inputs, sidebar items, buttons, tooltip (`tooltip.rs:120`) |
| large surface | `theme.radius_lg` | 16px | dialogs (`dialog/dialog.rs:580`), notifications (`notification.rs:425`) |
| pill | `rounded_full()` helper (app) / `theme.radius_full()` (vendored) | 9999 | status chip (`src/status_chip.rs:249`), vendored badges |
| card | `.rounded_lg()` rem helper | 7px @ rem 14 | the content card (`src/main.rs:1696-1703`) |

Derived `RadiusTokens`: none 0 · sm 4 · md 8 · lg 16 · **xl 16** · full
9999. `lg` and `xl` collide at 16 (`xl` = `radius * 2`); accepted (O3) —
nothing in the app reads `radius.xl` (its only vendored reader is
`Attachment`), and the derived values are pinned by
`theme::tests::theme_token_values_are_pinned`. The card keeps the 7px
rem helper (O2): faithful to zdesk's `rounded-lg` at the approved rem
base, and it scales with the Font Size setting.

Shadow policy (D4): `theme.shadow` stays `false` — the legacy flag gates
surface-control shadows, not beUI. The app-owned elevation scale
(`sm`/`md`/`lg`, 10% black ink, Tailwind radii halved for GPUI's sigma)
lives in `src/tokens.rs` (`UiTokens`), resolved from the live theme via
`Theme::resolve_semantic_config` (non-mutating; `apply_semantic_config*`
is never called) and re-resolved by `theme::apply_overrides` after every
theme rebuild and font-size change. Its consumer is the active sidebar
pill, which carries `sm` (`active_item_style`, `src/main.rs:1685-1692`).
The vendored overlays — Dialog, Sheet, Notification, Popover, Tooltip —
paint their own hardcoded elevations at the same scale and are not
restyled. The switch-thumb exception (beUI's `shadow-md` on the switch
thumb) is **not adopted**: controls stay flat, and a thumb shadow would
need a vendor patch.

### Accessibility (WCAG) adjustments

The zdesk palette resolves several token pairs below WCAG AA. The applied
values in `mono.json` are adjusted; the ratios are asserted by
`theme::tests::theme_tokens_meet_wcag_contrast` (4.5:1 text, 3:1 component
boundaries per 1.4.11).

| Token | zdesk | Applied light | Applied dark | Pair | Before → after |
|---|---|---|---|---|---|
| `input.border` | `stroke` `#e9e9e9` / `#262626` | `#8f8f8f` | `#6d6d6d` | vs `group_box.background` (the card controls sit on) | 1.21 → 3.23 / 1.06 → 3.11 |
| `switch.background` | `selection-subtle` `#ededed` / `#282828` | `#949494` | `#6b6b6b` | vs `group_box.background` | 1.17 → 3.03 / 1.09 → 3.02 |
| `description_list.label.foreground` | `content-subtle` `#6a6a6a` / `#818181` | `#6a6a6a` (unchanged) | `#888888` | vs `group_box.background` | 5.41 / 4.13 → 4.54 |
| `scrollbar.thumb.background` | `scrollbar-thumb` `#c5c5c5` / `#4c4c4c` | `#8f8f8f` | `#6b6b6b` | vs `group_box.background` and `background` | 1.73 → 3.23 / 1.88 → 3.02 |
| `scrollbar.thumb.hover.background` | `scrollbar-thumb-hover` `#939393` / `#818181` | `#7a7a7a` | `#818181` (unchanged) | vs `group_box.background` and `background` | 3.07 → 4.29 / 4.13 |

`input.border` is the unchecked-checkbox outline and the text-input / select
frame; `switch.background` is the switch's off track; the scrollbar pair is
the scroll-position indicator. All were below 3:1 before — the checkbox and
switch effectively invisible (≈1.1:1). `border` (the card frame and dividers)
is left at the zdesk value: it bounds containers, not interactive components,
so 1.4.11 does not apply. The scrollbar is not rendered by the app's current
pages (they are short), so its adjustment is token-level only.

App-side (not token) fixes: the sidebar wordmark and the About wordmark now
use `theme.foreground` at 17px (12.7:1 / 15.0:1 and 13.6:1 / 13.5:1), and
inactive sidebar labels use `muted.foreground` instead of
`sidebar_foreground.opacity(0.7)` (3.74 → 5.87 light, 5.40 → 6.06 dark).

## Shell metrics

| Element | zdesk source | Applied |
|---|---|---|
| Title bar | 36px (`--spacing-titlebar: 2.25rem`, `spacing.css:8`) | vendored `TitleBar` at `TITLE_BAR_HEIGHT = 34px` (`vendor/gpui-component/src/title_bar.rs:15`) — **deviation**; bottom border removed via `refine_style(border_0)` which lands after the component's own `.border_b_1()` (`title_bar.rs:336-337`), so the zeroed border wins (`src/shell.rs:95-114`) |
| Traffic lights | macOS overlay at (16, 18), 72px reserved (`--spacing-traffic-lights: 4.5rem`) | `traffic_light_position: Some(shell::traffic_light_origin())` — 16px left, top edge at 11px so the 14px button centers on zdesk's y=18 — over `TitleBar::title_bar_options()` (`src/main.rs:1825`) |
| Sidebar | `--spacing-nav: 15rem` = 240px (`NavView.tsx:40`) | `sidebar_width(sidebar_w)` + `sidebar_size_range(sidebar_w..sidebar_w)` where `sidebar_w` = 240 expanded / 48 collapsed — the dynamic pin keeps the `resizable_panel` seam from dragging; `show_handle(false)` (settings-collapse.patch) suppresses the painted 1px line via an empty `ResizeHandleRenderer` but the hit area remains — **deviation (residual invisible hit area)** (`src/main.rs:1677-1683`) |
| Brand row | 20px mark (`--spacing-brand: 1.25rem`) + wordmark | `Settings::sidebar_header` slot (the header always renders; only the search input is gated on `!sidebar_collapsed` by `settings-collapsed-header.patch`, so the mark stays on the rail): expanded = `h(32px)` row, 20px PNG mark (raster `img` — `svg()` would flatten the gradient), Xanh Mono italic `text_size(17px)` wordmark "mono-desk" in `theme.foreground` (12.7:1 / 15.0:1), `gap_2 px_2`; collapsed = centered 20px mark on a 32px box (`src/shell.rs:45-67`) |
| Sidebar item | `SidebarItem` `h-7` = 28px, `rounded-md`, `px-2 gap-2` (`SidebarItem.tsx:39`) | vendored `SidebarMenuItem`: `h_7`, `p_2`, `gap_x_2`, `text_sm`, `rounded(theme.radius)` = 8px (was 6px before the beUI pilot); collapsed items render as centered icons (`SidebarCollapsible::Icon`); **page** items carry icons via `SettingPage::icon` — Settings2/Bell/User/Info (`src/main.rs`); child (group) rows are iconless text (`settings-static-tree.patch`) |
| Selected state | `border-stroke` + `bg-background-raised` + `shadow-raised` (`SidebarItem.tsx:42`) | `active_item_style` patch: `bg(group_box)` + `border_1` `border_color(theme.border)` + the app-owned `sm` elevation (`UiTokens`, O1) — the raised pill. `SidebarMenuItem::active_style` is refined **after** the built-in active styling (`when(is_active)` lands last, `settings-item-style.patch`), so the custom style wins in both modes; `theme.shadow` stays `false` (`src/main.rs:1685-1692`) |
| Inactive items | muted foreground | `sidebar_style` `.w(sidebar_w)` + `.border_0()` (rail's right border removed) + `text_color(muted_foreground)` — inherited by inactive labels only (active/hover set `sidebar.accent.foreground` explicitly). zdesk's `sidebar_foreground.opacity(0.7)` resolved to 3.7:1 in light mode, below WCAG AA; `muted.foreground` is 5.9:1 / 6.1:1 (`src/main.rs:1677-1683`) |
| Hover | `bg-selection-hover` | native hover: `sidebar.accent` at 0.8 opacity + `sidebar.accent.foreground` (`sidebar/menu.rs:256-261`) |
| Content card | `rounded-lg border border-stroke bg-background-raised` + inset margins (`AppShell.tsx:40`) | `content_panel_style` on the `resizable_panel` content panel: `rounded_lg()` — the GPUI style helper (`rems(0.5)`), **7px at rem 14**, not `theme.radius_lg`/16px (O2) — `border_1` `border_color(theme.border)`, `bg(theme.group_box)`, `m(6px)` — equal 4-side inset — `overflow_hidden` (`src/main.rs:1696-1703`) |

## Font roles

| Role | Family | Weights vendored | Applied as |
|---|---|---|---|
| UI sans (body, labels, controls) | Inclusive Sans | 400 / 500 / 600 / 700 | `font.family`, `font.size` = 14 (zdesk `text-body`; raised from 13 by the beUI pilot); sidebar labels are native `SidebarMenuItem` `text_sm` |
| Mono (code, telemetry) | Google Sans Code | 400 / 500 | `mono_font.family`, `mono_font.size` = 12 (zdesk `text-code`) |
| Wordmark | Xanh Mono | 400 regular + 400 italic | `sidebar_brand()` wordmark — 17px italic in `theme.foreground` (`src/shell.rs:45-67`); zdesk specifies a 15px semibold sans wordmark — locked deviation per owner decision. Xanh Mono ships no 600 face, so the requested `SEMIBOLD` resolves to Regular; presence comes from size + colour |

Vendoring: Inclusive Sans was copied as static TTFs from
`~/Library/Fonts` (installed upstream statics — no instancer pass
needed); registered alongside Hanken Grotesk, Google Sans Code, and Xanh
Mono in `theme::install` (`src/theme.rs:85-100`) via
`cx.text_system().add_fonts` before `Theme::change`. Hanken Grotesk
remains registered but is no longer the theme family.

## Interactions

| zdesk behavior (`useGlobalShortcuts.ts`) | Applied |
|---|---|
| ⌘/Ctrl+Q → window close | `cmd-q` → `Quit` → `cx.quit()`; the app-menu item is registered in `set_menus` so macOS shows the key equivalent (`src/main.rs:1788,1792-1794`) |
| ⌘/Ctrl+W → close active tab, else close window | `cmd-w` → `CloseWindow` → `cx.quit()` — **no tabs exist, so it quits** (`src/main.rs:1789,1798-1799`); semantics recorded as deviation |
| ⌘/Ctrl+K → search view | **not bound** — the multi-view shell was removed; there is no Search view |
| ⌘/Ctrl+, → settings view | **not bound** — Settings is the only view |
| ⌘/Ctrl+B → toggle nav | `cmd-b` → `ToggleSidebar` flips `ShellState::collapsed` + `cx.refresh_windows()` — toggles 240px ⇄ 48px icon rail; same affordance as the titlebar `PanelLeft` ghost button (`src/main.rs:1790,1801-1805`) |
| Theme toggle | zdesk offers a controller; mono-desk keeps it Settings-only (Dark Mode switch, General page) — locked decision |

## Settings control contracts (beUI P1 adoption)

- **Input field** (`src/input_field.rs`, Profile → Username): the
  vendored `Input` keeps its own chrome; the app composes label +
  input + validation message. `FieldState` seeds validation at
  mount (no shake) and re-validates whenever the value syncs from the
  app (`sync_from_app`), so external writes/reset never leave a stale
  error. The message row reserves a minimum line (`min_h`) and grows
  for wrapped/localized text; it carries `Role::Alert` while it holds
  an error. `InputFieldConfig` is construction-time only — remount on
  page navigation picks up new labels. The input's accessible name is
  `aria_label` (the row title), not the placeholder.
- **Switch / checkbox rows** (General → Dark Mode, Startup → Launch
  at login, Notifications → Enabled): app-side `Switch`/`Checkbox` via
  `SettingField::render` carrying `.accessibility_label(row title)` +
  `.on_reset(...)`; each has a unique `ElementId` and a
  `debug_selector` on the wrapper for headless tests. Toggle is
  click + keyboard (Space/Enter) driven and lands under reduced motion.
- **Collapsed-rail tooltips**: vendored `SidebarMenuItem::collapsed_tooltip`
  wires `managed_tooltip_at(Placement::Right, …)` for icon-only items —
  the earlier "unavailable" note was wrong; the managed tooltip stack
  drives the show/hide delay, verified in `src/view_tests.rs`.
- **Notifications** (Notifications → Send test notification): push →
  `window.notifications()` mounts on the window's notification layer;
  the close button calls `Notification::dismiss`; the 5 s autohide is
  driven by the toast advance task. A11y gap recorded: GPUI has no
  `aria_live`/`aria_describedby`, so beUI's `role=alert` announcement
  parity is **not** promised — same limit as the input message row.

## Deviations & non-portable

- **`color-mix` frozen to hex.** zdesk re-derives all neutrals from
  `--theme-hue/-saturation/-dark-lightness`; GPUI JSON carries static
  hex, so a knob change requires re-resolving every row in the table
  above (both modes).
- **Selected rail items carry a raised pill** (`active_item_style`: `bg`
  group_box + 1px border stroke + the app-owned `sm` elevation from
  `UiTokens`) applied after the vendored accent fill so it overrides;
  zdesk's `shadow-raised` is now matched by the app-owned scale while
  `theme.shadow` stays `false` (the legacy flag gates the vendored
  surface-control shadows).
- **No keyframe motion**: zdesk `motion.css` (120–200ms easings,
  transitions on `SidebarItem`) has no GPUI analog — non-portable.
- **Residual resize hit area**: `sidebar_size_range` pins
  `width..width` so the seam cannot drag and `show_handle(false)` paints
  nothing, but `h_resizable` still registers the invisible hit strip —
  vendored-component deviation.
- **`sidebar_footer` slot dormant**: `sidebar-slots.patch` restores both
  element slots, but only `sidebar_header` has a call site — the footer
  theme toggle was cut by owner decision (theme toggle is Settings-only).
- **Settings-internal rail gaps** (10px uppercase tracking-wider group
  labels, per-item density — `h_7`/`p_2`/`text_sm` hardcoded in
  `sidebar/menu.rs`): component-internal to vendored `Settings`, not
  expressible without another patch. The inactive-item ~70% opacity gap
  is closed via `sidebar_style`.
- **TitleBar 34px vs spec 36px** — vendored-component deviation, no
  patch added.
- **`Cmd+W` semantics**: zdesk closes the active tab before falling back
  to window close; mono-desk has no `TabBar`, so ⌘W quits outright.

## Vendored patch state

`vendor/gpui-component` is stock 0.6.1 plus exactly thirteen patches
(`Cargo.toml:18-39`):

| Patch | Lines | State | What it does |
|---|---|---|---|
| `vendor/patches/sidebar-cursor.patch` | 14 | **KEPT** | `cursor_pointer()` on rail items / `cursor_default()` when disabled (`src/sidebar/menu.rs`), matching the `Button` convention |
| `vendor/patches/sidebar-slots.patch` | 101 | **RESTORED** (phase `zdesk-correction`) | `Settings::sidebar_header`/`sidebar_footer` element slots on `src/setting/settings.rs`; `sidebar_header` carries `shell::sidebar_brand()`, `sidebar_footer` is dormant (no call site by owner decision) |
| `vendor/patches/settings-content-style.patch` | 59 | **KEPT** | `Settings::content_panel_style(&StyleRefinement)` → `refine_style` on the content `resizable_panel` — carries the raised zdesk card chrome |
| `vendor/patches/settings-group-icon.patch` | 86 | **SUPERSEDED** (`settings-static-tree`) | Added `SettingGroup::icon` + child-item `.icon()`; group icons were dropped when the tree went static — `settings-static-tree.patch` removes the field/setter/call site again |
| `vendor/patches/settings-collapse.patch` | 116 | **NEW** (`zdesk-layout-fix`) | `Settings::sidebar_collapsed` + `show_handle`; `Sidebar` gets `.collapsible(SidebarCollapsible::Icon)` + `.collapsed(sidebar_collapsed)`; the header gating it introduced is refined by `settings-collapsed-header.patch` |
| `vendor/patches/settings-item-style.patch` | 97 | **NEW** (`zdesk-layout-fix`) | `Settings::active_item_style` → `SidebarMenuItem::active_style`; the refinement is applied **after** the built-in `is_active` accent styling (`menu.rs`), so the custom pill wins in both modes |
| `vendor/patches/settings-search-placeholder-locale.patch` | 34 | **NEW** (i18n) | `SettingsState::placeholder_locale` + a re-apply of `Settings.search_placeholder` in `render` when the active locale changes — the search input is created once per keyed state, so its placeholder would otherwise stay in the first-render locale |
| `vendor/patches/controls-cursor.patch` | 37 | **NEW** (WIP port) | `cursor_pointer()` on every interactive `Button` (was link/text variants only) and on enabled `Checkbox`/`Select`/`Switch` |
| `vendor/patches/settings-collapsed-header.patch` | 44 | **NEW** (WIP port) | The sidebar header is always rendered; only the search `Input` is gated on `!sidebar_collapsed`, so the `sidebar_header` slot (brand mark) stays visible on the 48px rail |
| `vendor/patches/settings-static-tree.patch` | 437 | **NEW** (mdtree) | Static sidebar tree (`src/sidebar/menu.rs`, `src/setting/{settings,page,group}.rs`): no caret/toggle (`SidebarMenuItem::{default_open,click_to_open,click_to_toggle}` + `SettingPage::default_open` removed), children always render when the rail is expanded, page labels bold via `label_style`, child items only for pages with >=2 titled groups — indices address `page.groups` so untitled groups keep their slots; child rows are iconless, so `SettingGroup::icon` (added by `settings-group-icon.patch`) is removed again |
| `vendor/patches/context-menu-release.patch` | 18 | **NEW** (beUI P1) | `ContextMenuSharedState.menu_view` is cleared on `DismissEvent` — the subscription closure lives inside the state it captures, so retaining the built `PopupMenu` there is a retain cycle that leaks the menu entity (and keeps a stale entity alive until element unmount) |
| `vendor/patches/searchable-list-test-support.patch` | 20 | **NEW** (beUI P1) | `.test_support()` on `SearchableListItemElement` rows (the `ListItem` element already has it) so the gpui-kit test backend can observe Select/Combobox popup rows |
| `vendor/patches/combobox-value-change.patch` | 63 | **NEW** (beUI P1) | Combobox selection-change detection compares item **values**, not row indices — a filtered list reuses indices, so clicking a filtered row that landed on the selected row's index was treated as "no change" (no `Change`/`Confirm`, no close) while still mutating the selection; matches the value identity `selection_changes`/`is_item_checked` already use |

Upgrade rule: re-copy the upstream crate, re-apply all thirteen patches.
**Reversal order** (the hunks stack — `git apply -R` in:
`combobox-value-change` → `searchable-list-test-support` →
`context-menu-release` → `settings-static-tree` →
`settings-collapsed-header` → `controls-cursor` →
`settings-search-placeholder-locale` → `settings-item-style` →
`settings-collapse` → `settings-group-icon` → `settings-content-style` →
`sidebar-slots` → `sidebar-cursor`; apply forward in the reverse
sequence).

## Known flags for owner review

- `make run` visual check pending: traffic-light position, borderless
  titlebar with the `PanelLeft` toggle right of the lights, ⌘B collapse
  icons), active pill (bg raised + 1px stroke + `sm` elevation), static
  tree (bold pages,
  iconless children), equal 6px
  card inset, residual (invisible) resize hit area.
- ⌘W = quit is a destructive binding kept per plan; revisit if tabs land.
- Revert path (per plan): `git checkout -- assets src` +
  `git clean -f src/shell.rs` removes the port; `git checkout -- vendor`
  restores patches.
