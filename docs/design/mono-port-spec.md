# mono-desk Design Port Spec

Records the applied state of the kumo-desk → mono-desk (gpui-kit) port:
token mapping, radii, shadow policy, sidebar metrics, font roles, and
control metrics. Source of truth for source values is kumo-desk
(read-only): `DESIGN.md`, `specs/tokens/token-reference.md`,
`specs/foundations/{color,typography}.md`, `src/app.css` (`@theme`),
`src/shell/AppShell.tsx`, `src/components/ViewShell.tsx`. Applied state
lives in `assets/themes/mono.json` + `src/theme.rs` + `src/main.rs`.

Source named rules (R2 token rule, 60% floor, CLS rules) informed values
only; they are not adopted as mono-desk policy (initiative non-goal).

## Token mapping

`ThemeConfigColors` serde keys → resolved hex per mode. Source value is
the authored `light-dark()`/oklch expression; hex is the resolved sRGB
port applied in `mono.json`.

| Source token | Source value (light / dark) | Hex light | Hex dark | `ThemeConfigColors` key(s) |
|---|---|---|---|---|
| brand (`--color-kumo-brand`, `--color-primary`) | `#124b68` / `oklch(0.5200 0.1050 223.1280)` | `#124b68` | `#007595` | `primary.background`, `ring`, `link`, `caret`, `progress.bar.background` |
| brand hover (`--color-kumo-brand-hover`) | `#0d3a52` / `color-mix(oklch(0.52…), white 15%)` | `#0d3a52` | `#3689a4` | `primary.hover.background`, `link.hover` |
| base canvas (`--color-kumo-base`) | `oklch(1.0000 0 0)` / `oklch(0.2679 0.0036 106.6427)` | `#ffffff` | `#262624` | `background`, `popover.background`, `group_box.background`, `tab_bar.background`, `list.background`, `tab.active.background` |
| recessed (`--color-kumo-recessed`) | `oklch(0.9809 0.0025 228.7836)` / `oklch(0.2213 0.0038 106.7070)` | `#f7f9fa` | `#1b1b19` | `muted.background`, `skeleton.background`, `secondary.background`, `switch.background` |
| line (`--color-kumo-line`) | `oklch(0.9219 0 0)` / `oklch(0.3618 0.0101 106.8928)` | `#e5e5e5` | `#3e3e38` | `border`, `input.border`, `drag.border` |
| default text (`--color-kumo-default`) | `oklch(0.2100 0.0060 285.8850)` / `oklch(0.9700 0 0)` | `#18181b` | `#f5f5f5` | `foreground`, `popover.foreground`, `secondary.foreground`, `tab.foreground` |
| muted text (`text-kumo-default/60`) | 60% blend of default text | `#747476` | `#a2a2a1` | `muted.foreground`, `description_list.label.foreground` |
| sidebar bg (`--color-sidebar`) | `oklch(1.0000 0 0)` / `oklch(0.2357 0.0024 67.7077)` | `#ffffff` | `#1f1e1d` | `sidebar.background` |
| sidebar fg (`--color-sidebar-foreground`) | `oklch(0.1913 0 0)` / `oklch(0.8074 0.0142 93.0137)` | `#141414` | `#c3c0b6` | `sidebar.foreground` |
| sidebar accent (`--color-sidebar-accent`) | `oklch(0.9219 0 0)` / `oklch(0.1680 0.0020 106.6177)` | `#e5e5e5` | `#0f0f0e` | `sidebar.accent.background`, `accent.background`, `list.hover.background` |
| sidebar accent fg (`--color-sidebar-accent-foreground`) | `#124b68` / `oklch(0.8074 0.0142 93.0137)` | `#124b68` | `#c3c0b6` | `sidebar.accent.foreground`, `accent.foreground`, `link.active` |
| sidebar border (`--color-sidebar-border`) | `oklch(0.9219 0 0)` / `oklch(0.9401 0 0)` | `#e5e5e5` | `#ebebeb` | `sidebar.border` |
| sidebar primary (`--color-sidebar-primary`) | `#124b68` / `oklch(0.3250 0 0)` | `#124b68` | `#343434` | `sidebar.primary.background` |
| sidebar primary fg (`--color-sidebar-primary-foreground`) | `oklch(1.0000 0 0)` / `oklch(0.9881 0 0)` | `#ffffff` | `#fbfbfb` | `sidebar.primary.foreground`, `primary.foreground` |
| selection (20% brand blend) | derived | `#d0dbe1` | `#1e363b` | `selection.background`, `list.active.background` |
| scrollbar thumb (18% default text, `.pat-scrollbar`) | derived | `#d5d5d6` | `#4b4b4a` | `scrollbar.thumb.background`, `scrollbar.thumb.hover.background` |
| danger / status red (`--pat-hue-red`) | `oklch(0.64 0.19 27)` both modes | `#e95048` | `#e95048` | `danger.background`, `danger.foreground` |
| warning / status amber (`--pat-hue-amber`) | `oklch(0.76 0.13 70)` both modes | `#e5a14b` | `#e5a14b` | `warning.background`, `warning.foreground` |
| success / status green (`--pat-hue-green`) | `oklch(0.70 0.13 162)` both modes | `#3fb785` | `#3fb785` | `success.background`, `success.foreground` |
| info / status cyan (`--pat-hue-cyan`) | `oklch(0.72 0.10 221)` both modes | `#52b3d1` | `#52b3d1` | `info.background`, `info.foreground` |
| status hues → charts | amber / green / cyan / purple `oklch(0.62 0.18 293)`→`#8d6be7` / red | `#e5a14b` `#3fb785` `#52b3d1` `#8d6be7` `#e95048` | same | `chart.1`, `chart.2`, `chart.3`, `chart.4`, `chart.5` |

Theme-level keys in both configs: `"radius": 6`, `"radius.lg": 10`,
`"shadow": false`, `"font.family": "Hanken Grotesk"`, `"font.size": 14`,
`"mono_font.family": "Google Sans Code"`, `"mono_font.size": 13`.

## Radii

| Source (`specs/foundations/radius.md`, `DESIGN.md` `rounded`) | Value | Applied as |
|---|---|---|
| control / chip (`--pat-radius-control`, `--pat-radius-chip`) | 6px | `radius` = 6 — buttons, inputs, menu items (`rounded(theme.radius)`) |
| card (`--pat-radius-card`) | 10px | `radius.lg` = 10 — dialogs, notifications, group boxes |
| full pill | 9999px | gpui hardcoded pill constant where used (sliders, dots) |

## Shadow policy

Source: flat-by-default — in-flow surfaces use 1px `--color-kumo-line`
borders and tonal contrast; shadows appear exclusively on floating
overlays (popover `shadow-lg`, spotlight `shadow-2xl`, subtle `shadow-xs`
on inputs/segmented pills).

Applied: `"shadow": false` in both configs. Verified scope (gpui-component
0.6.1): `theme().shadow` gates surface-control shadows only (e.g.
`button.rs`); dialog/popover/notification shadows are hardcoded and
unaffected — faithful to "flat surfaces; overlays excepted".

## Sidebar metrics

Source `AppShell.tsx`: 15rem expanded rail (= 240px), items
`rounded-md px-2 py-2 text-sm gap-x-2`, active =
`bg-sidebar-accent font-medium text-sidebar-accent-foreground`,
inactive = `text-sidebar-foreground/70` with hover → accent pair,
section labels `text-[10px] font-semibold uppercase tracking-wider
text-kumo-default/60`.

Applied in mono-desk `Settings`:

- `sidebar_width(px(240.))`; `sidebar_size_range` kept at defaults
  (`160..360`).
- Active item look is native `SidebarMenuItem` behavior:
  `font_medium` + `sidebar.accent.background` +
  `sidebar.accent.foreground`; hover uses `sidebar.accent.*` at 0.8
  opacity — matches the source active/hover pair; `sidebar_style`
  refinement therefore not added.
- **Gaps (recorded, not styled):** inactive-item 70% foreground opacity,
  `text-[10px] uppercase` group labels, and per-item density are
  component-internal and not expressible via `sidebar_style` without
  inventing new surface — left as documented deviation.
- `sidebar.border` dark = `#ebebeb` (source `oklch(0.9401 0 0)`) is a
  bright edge against the dark rail — ported faithfully, flagged for
  owner visual check.

## Font roles

| Role | Family | Weights vendored | Applied as |
|---|---|---|---|
| UI sans (body, labels, controls) | Hanken Grotesk | 400 / 500 / 600 / 700 | `font.family`, `font.size` = 14 (source body `0.875rem`) |
| Mono (code, telemetry, kbd) | Google Sans Code | 400 / 500 | `mono_font.family`, `mono_font.size` = 13 (source code `0.8125rem`) |
| Display / headings / wordmark | Xanh Mono | 400 regular + 400 italic | use-site styling (`font_family("Xanh Mono")` + `.italic()`); About-page `"settings-demo"` wordmark is the port's wordmark instance (source: `font-serif italic tracking-tight`) |

Vendoring note: `google/fonts` ships Hanken Grotesk and Google Sans Code
variable-only; the vendored files are static instances produced via
`fonttools varLib.instancer` (correct `OS/2.usWeightClass`, unique
PostScript names). Xanh Mono vendored as upstream statics. Registered in
`theme::install` via `cx.text_system().add_fonts` before `Theme::change`;
JSON family names match TTF name tables exactly.

## Control metrics

Source: 32px control height (`--control-height` = `ds-spacing-8`),
12px horizontal padding (`px-3`), 14px control font (`text-sm`),
6px control radius, 2px `ring-kumo-focus` focus ring.

Applied: `ring` = brand (`#124b68` / `#007595`) carries the focus-ring
color; radius/size targets are component-driven in gpui-kit
(`SettingField` sizes are component defaults) — the 32px/12px targets
are recorded as the design intent where components allow, not restyled.

## Known flags for owner review

- `sidebar.border` dark `#ebebeb` — bright divider on dark rail.
- Variable→static instancing — rendering verified by name/weight tables,
  not yet by glyphs in a live window (`make run` pending).
- Revert path: `git checkout -- assets src/theme.rs src/main.rs` removes
  the whole applied theme.
