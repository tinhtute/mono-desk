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
(`src/shell.rs:78-101`): a borderless transparent `TitleBar`
(`refine_style(border_0)` kills its built-in `border_b_1` seam — the
refinement lands after the component's own border in
`vendor/gpui-component/src/title_bar.rs:336-337`, so it wins) whose one
child is a ghost `PanelLeft` `Button` that toggles
`ShellState::collapsed`, parked right of the macOS traffic lights by the
title bar's built-in 80px left padding
(`TITLE_BAR_LEFT_PADDING`, `title_bar.rs:17`); and the `Settings`
element rendered directly under it. Settings' internal sidebar IS the
app sidebar — now collapsible between 240px and a 48px icon rail via
`ShellState` (a derived-`Default` `Global`, `src/shell.rs:15-21`);
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
| `content-subtle` (70% / 50%) | mix | `#6a6a6a` | `#818181` | `description_list.label.foreground`, `scrollbar.thumb.hover.background` (dark variant) |
| `stroke` (content 7%) | mix | `#e9e9e9` | `#262626` | `border`, `input.border`, `drag.border`, `window.border`, `sidebar.border`, `status_bar.border`, `title_bar.border`, `list.active.border`, `table.active.border`, `table.row.border`, `button.active.background` (light), `slider.background` |
| `selection-subtle` (5% / 8%) | mix | `#ededed` | `#282828` | `muted.background`, `secondary.background`, `button.background`, `skeleton.background`, `switch.background`, `tiles.background`, `list.even/head.background`, `table.even/foot/head.background`, `tab_bar.segmented.background`, `button.secondary.background`, `sidebar…` surface steps |
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
| `scrollbar-thumb` (content 25%) | mix | `#c5c5c5` | `#4c4c4c` | `scrollbar.thumb.background` |
| `scrollbar-thumb-hover` (content 50%) | mix | `#939393` | `#818181` | `scrollbar.thumb.hover.background` |
| `accent-foreground` | hsl 0 0% 100% | `#ffffff` | `#ffffff` | `primary.foreground`, `sidebar.primary.foreground`, `button.primary/info.foreground` |

Charts (`chart.1–5`) keep the prior mono palette (`#e5a14b`, `#3fb785`,
`#52b3d1`, `#8d6be7`, `#e95048` in both modes) — zdesk has no chart
tokens; `base.*` accent shades (`base.cyan` `#124b68`/`#18c7e2`, etc.)
track the zdesk accent/status hues.

Theme-level keys in both configs (`mono.json`): `"radius": 6`,
`"radius.lg": 10`, `"shadow": false`, `"font.family": "Inclusive Sans"`,
`"font.size": 13`, `"mono_font.family": "Google Sans Code"`,
`"mono_font.size": 12`.

## Shell metrics

| Element | zdesk source | Applied |
|---|---|---|
| Title bar | 36px (`--spacing-titlebar: 2.25rem`, `spacing.css:8`) | vendored `TitleBar` at `TITLE_BAR_HEIGHT = 34px` (`vendor/gpui-component/src/title_bar.rs:15`) — **deviation**; bottom border removed via `refine_style(border_0)` which lands after the component's own `.border_b_1()` (`title_bar.rs:336-337`), so the zeroed border wins (`src/shell.rs:82-98`) |
| Traffic lights | macOS overlay at (16, 18), 72px reserved (`--spacing-traffic-lights: 4.5rem`) | `traffic_light_position: point(px(16.), px(18.))` over `TitleBar::title_bar_options()` (`src/main.rs:426-429`) |
| Sidebar | `--spacing-nav: 15rem` = 240px (`NavView.tsx:40`) | `sidebar_width(sidebar_w)` + `sidebar_size_range(sidebar_w..sidebar_w)` where `sidebar_w` = 240 expanded / 48 collapsed — the dynamic pin keeps the `resizable_panel` seam from dragging; `show_handle(false)` (settings-collapse.patch) suppresses the painted 1px line via an empty `ResizeHandleRenderer` but the hit area remains — **deviation (residual invisible hit area)** (`src/main.rs:340-349`) |
| Brand row | 20px mark (`--spacing-brand: 1.25rem`) + wordmark | `Settings::sidebar_header` slot (gated `when(!sidebar_collapsed)` inside the collapse patch — the whole header, brand + search, disappears on the rail): expanded = `h(32px)` row, 20px PNG mark (raster `img` — `svg()` would flatten the gradient), Xanh Mono italic semibold `text_size(15px)` wordmark "mono-desk", `gap_2 px_2`; collapsed = centered 20px mark on a `w_8 justify_center` box (`src/shell.rs:32-52`) |
| Sidebar item | `SidebarItem` `h-7` = 28px, `rounded-md`, `px-2 gap-2` (`SidebarItem.tsx:39`) | vendored `SidebarMenuItem`: `h_7`, `p_2`, `gap_x_2`, `text_sm`, `rounded(theme.radius)` = 6px; collapsed items render as centered icons (`SidebarCollapsible::Icon`); group icons via `SettingGroup::icon` — Appearance→`Palette`, Startup→`Play`, Delivery→`Bell`, Actions→`SquareTerminal`, Account→`CircleUser`, Links→`ExternalLink` (`src/main.rs`) |
| Selected state | `border-stroke` + `bg-background-raised` + `shadow-raised` (`SidebarItem.tsx:42`) | `active_item_style` patch: `bg(group_box)` + `border_1` `border_color(theme.border)` — the raised pill. `SidebarMenuItem::active_style` is refined **after** the built-in active styling (`when(is_active)` lands last, `settings-item-style.patch`), so the custom style wins in both modes; still no `shadow-raised` (`theme.shadow = false`) — partial deviation (`src/main.rs:364-369`) |
| Inactive items | muted foreground | `sidebar_style` `.w(sidebar_w)` + `.border_0()` (rail's right border removed) + `text_color(sidebar_foreground.opacity(0.7))` — inherited by inactive labels only (active/hover set `sidebar.accent.foreground` explicitly) (`src/main.rs:357-362`) |
| Hover | `bg-selection-hover` | native hover: `sidebar.accent` at 0.8 opacity + `sidebar.accent.foreground` (`sidebar/menu.rs:288-293`) |
| Content card | `rounded-lg border border-stroke bg-background-raised` + inset margins (`AppShell.tsx:40`) | `content_panel_style` on the `resizable_panel` content panel: `rounded_lg()` (10px), `border_1` `border_color(theme.border)`, `bg(theme.group_box)`, `m(6px)` — equal 4-side inset — `overflow_hidden` (`src/main.rs:374-382`) |

## Font roles

| Role | Family | Weights vendored | Applied as |
|---|---|---|---|
| UI sans (body, labels, controls) | Inclusive Sans | 400 / 500 / 600 / 700 | `font.family`, `font.size` = 13 (zdesk `text-body`); sidebar labels are native `SidebarMenuItem` `text_sm` |
| Mono (code, telemetry) | Google Sans Code | 400 / 500 | `mono_font.family`, `mono_font.size` = 12 (zdesk `text-code`) |
| Wordmark | Xanh Mono | 400 regular + 400 italic | `sidebar_brand()` wordmark — 15px semibold italic (`src/shell.rs:32-52`); zdesk specifies a 15px semibold sans wordmark — locked deviation per owner decision |

Vendoring: Inclusive Sans was copied as static TTFs from
`~/Library/Fonts` (installed upstream statics — no instancer pass
needed); registered alongside Hanken Grotesk, Google Sans Code, and Xanh
Mono in `theme::install` (`src/theme.rs:31-40`) via
`cx.text_system().add_fonts` before `Theme::change`. Hanken Grotesk
remains registered but is no longer the theme family.

## Interactions

| zdesk behavior (`useGlobalShortcuts.ts`) | Applied |
|---|---|
| ⌘/Ctrl+Q → window close | `cmd-q` → `Quit` → `cx.quit()`; the app-menu item is registered in `set_menus` so macOS shows the key equivalent (`src/main.rs:429,433-436`) |
| ⌘/Ctrl+W → close active tab, else close window | `cmd-w` → `CloseWindow` → `cx.quit()` — **no tabs exist, so it quits** (`src/main.rs:430,439`); semantics recorded as deviation |
| ⌘/Ctrl+K → search view | **not bound** — the multi-view shell was removed; there is no Search view |
| ⌘/Ctrl+, → settings view | **not bound** — Settings is the only view |
| ⌘/Ctrl+B → toggle nav | `cmd-b` → `ToggleSidebar` flips `ShellState::collapsed` + `cx.refresh_windows()` — toggles 240px ⇄ 48px icon rail; same affordance as the titlebar `PanelLeft` ghost button (`src/main.rs:431,441-446`) |
| Theme toggle | zdesk offers a controller; mono-desk keeps it Settings-only (Dark Mode switch, General page) — locked decision |

## Deviations & non-portable

- **`color-mix` frozen to hex.** zdesk re-derives all neutrals from
  `--theme-hue/-saturation/-dark-lightness`; GPUI JSON carries static
  hex, so a knob change requires re-resolving every row in the table
  above (both modes).
- **Selected rail items carry a raised pill** (`active_item_style`: `bg`
  group_box + 1px border stroke) applied after the vendored accent fill
  so it overrides; zdesk's `shadow-raised` is still not applied
  (`theme.shadow = false` gates surface shadows).
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
- **Collapsed tooltips unavailable**: on the 48px rail items shrink to
  bare icons; adding hover tooltips would need crate-internal API the
  vendored `SidebarMenuItem` doesn't expose — deviation.
- **TitleBar 34px vs spec 36px** — vendored-component deviation, no
  patch added.
- **`Cmd+W` semantics**: zdesk closes the active tab before falling back
  to window close; mono-desk has no `TabBar`, so ⌘W quits outright.

## Vendored patch state

`vendor/gpui-component` is stock 0.6.1 plus exactly nine patches
(`Cargo.toml:15-25`):

| Patch | Lines | State | What it does |
|---|---|---|---|
| `vendor/patches/sidebar-cursor.patch` | 14 | **KEPT** | `cursor_pointer()` on rail items / `cursor_default()` when disabled (`src/sidebar/menu.rs`), matching the `Button` convention |
| `vendor/patches/sidebar-slots.patch` | 101 | **RESTORED** (phase `zdesk-correction`) | `Settings::sidebar_header`/`sidebar_footer` element slots on `src/setting/settings.rs`; `sidebar_header` carries `shell::sidebar_brand()`, `sidebar_footer` is dormant (no call site by owner decision) |
| `vendor/patches/settings-content-style.patch` | 59 | **KEPT** | `Settings::content_panel_style(&StyleRefinement)` → `refine_style` on the content `resizable_panel` — carries the raised zdesk card chrome |
| `vendor/patches/settings-group-icon.patch` | 86 | **KEPT** | `SettingGroup::icon(impl Into<Icon>)` (`src/setting/group.rs`) → `.icon()` on the group's `SidebarMenuItem` (`src/setting/settings.rs`) |
| `vendor/patches/settings-collapse.patch` | 116 | **NEW** (`zdesk-layout-fix`) | `Settings::sidebar_collapsed` + `show_handle`; `Sidebar` gets `.collapsible(SidebarCollapsible::Icon)` + `.collapsed(sidebar_collapsed)`; the header gating it introduced is refined by `settings-collapsed-header.patch` |
| `vendor/patches/settings-item-style.patch` | 97 | **NEW** (`zdesk-layout-fix`) | `Settings::active_item_style` → `SidebarMenuItem::active_style`; the refinement is applied **after** the built-in `is_active` accent styling (`menu.rs`), so the custom pill wins in both modes |
| `vendor/patches/settings-search-placeholder-locale.patch` | 34 | **NEW** (i18n) | `SettingsState::placeholder_locale` + a re-apply of `Settings.search_placeholder` in `render` when the active locale changes — the search input is created once per keyed state, so its placeholder would otherwise stay in the first-render locale |
| `vendor/patches/controls-cursor.patch` | 37 | **NEW** (WIP port) | `cursor_pointer()` on every interactive `Button` (was link/text variants only) and on enabled `Checkbox`/`Select`/`Switch` |
| `vendor/patches/settings-collapsed-header.patch` | 44 | **NEW** (WIP port) | The sidebar header is always rendered; only the search `Input` is gated on `!sidebar_collapsed`, so the `sidebar_header` slot (brand mark) stays visible on the 48px rail |

Upgrade rule: re-copy the upstream crate, re-apply all nine patches.
**Reversal order** (the hunks stack — `git apply -R` in:
`settings-collapsed-header` → `controls-cursor` →
`settings-search-placeholder-locale` → `settings-item-style` →
`settings-collapse` → `settings-group-icon` → `settings-content-style` →
`sidebar-slots` → `sidebar-cursor`; apply forward in the reverse
sequence).

## Known flags for owner review

- `make run` visual check pending: traffic-light position, borderless
  titlebar with the `PanelLeft` toggle right of the lights, ⌘B collapse
  (240 ⇄ 48 icon rail, brand/search hidden when collapsed, centered
  icons), active pill (bg raised + 1px stroke), group icons, equal 6px
  card inset, residual (invisible) resize hit area.
- ⌘W = quit is a destructive binding kept per plan; revisit if tabs land.
- Revert path (per plan): `git checkout -- assets src` +
  `git clean -f src/shell.rs` removes the port; `git checkout -- vendor`
  restores patches.
