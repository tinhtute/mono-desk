---
id: plan-kumo-design-port-20260918
intake_id: intake-20260918-kumo-design-port
status: completed
lane: normal
created: 2026-09-18
updated: 2026-09-22
approach: planned
planning_status: planned
exact_next_action: none — initiative closed; plan archived under docs/plans/completed/
---

# Mono design port

Reskin `mono-desk`'s existing `Settings` window with the `kumo-desk`
design language: ported palette (light + dark), vendored font stack, a
written port spec inside this repo, and app-owned rail chrome (brand
block, search, grouped item list, footer theme toggle) inside that same
window. No second window and no app-level shell.

## Outcome

`mono-desk` runs the ported palette in both modes, renders Hanken
Grotesk / Xanh Mono italic / Google Sans Code from embedded fonts, and
owns its settings rail chrome — brand block, search, grouped page list,
theme toggle — on a vendored, minimally patched `gpui-component`; the
mapping and the applied rail state are recorded in
`docs/design/mono-port-spec.md`. Success when:

- `assets/themes/mono.json` exists, deserializes as a gpui `ThemeSet`,
  and is applied at startup (verified by `cargo test` + code path).
- The three font families are embedded via `include_bytes!` and
  registered; `font.family`/`mono_font.family` resolve to them.
- `docs/design/mono-port-spec.md` records every mapped token **and** the
  applied rail composition (brand block, search, item list, footer
  toggle, vendored-patch record).
- The rail renders brand mark + Xanh Mono italic wordmark above the
  search input, keeps the page/group item list, and ends in a theme
  toggle that flips light/dark in place.
- `vendor/gpui-component/` differs from upstream 0.6.1 only by
  `vendor/patches/sidebar-slots.patch` (two additive `Settings` slots).
- `cargo test`, `make check`, `make lint` pass on the patched dep graph;
  `make fmt` applied. Owner runs `make run` for final visual sign-off
  (GUI-only check).

*Refinement 2026-09-18: rail chrome (brand block + footer toggle) moved
into scope; it was a documented deviation of the first pass.*

## Authority and Requirements

Sources (read-only, current on-disk state). The kumo-desk checkout is
host-local: `$HOME/Lab/kumo-desk` (Linux `/home/tinhpt/…`, macOS
`/Users/tinhtute/…`); absolute paths below name the Linux session.

- `/home/tinhpt/Lab/kumo-desk/DESIGN.md` — rules + token frontmatter.
- `/home/tinhpt/Lab/kumo-desk/specs/tokens/token-reference.md`,
  `specs/foundations/{color,typography}.md` — canonical values.
- `/home/tinhpt/Lab/kumo-desk/src/app.css` `@theme` — resolved
  light/dark values.
- `/home/tinhpt/Lab/kumo-desk/src/shell/AppShell.tsx`,
  `src/components/ViewShell.tsx` — component-pattern reference.
- `gpui-component 0.6.1` `theme/{schema,registry,mod,theme_color}.rs`
  (cargo registry) — target API surface.
- `mono-desk/AGENTS.md` — repo conventions, remain in force.
- `$HOME/Lab/kumo-desk/public/brand-mark.svg` — brand mark, 1254×1254,
  gradient stops `#0789B2` / `#0BA8C8` @55% / `#18C6E1` (2417 bytes).
- `gpui-component 0.6.1` `sidebar/{mod,menu,group,header,footer}.rs` and
  `setting/{settings,page,group,item,fields/*}.rs` — the surface the
  vendored patch extends, and the `pub(crate)` / `pub(super)` boundaries
  (`SettingPage::render`, `SettingItem::render_item`, `BoolField`,
  `DropdownField`, …) that rule out a custom shell without reimplementing
  settings content.

Requirements:

1. **Palette.** `assets/themes/mono.json` defines `mono-light` and
   `mono-dark` `ThemeConfig`s using exactly the resolved mapping table
   in *Approach and Risks* (source token → hex → `ThemeConfigColors`
   key). `"radius": 6`, `"radius.lg": 10`.
2. **Fonts.** `assets/fonts/` vendors Xanh Mono (regular+italic),
   Hanken Grotesk (400–700), Google Sans Code (400, 500) as TTF/OTF
   (OFL). Registered via `cx.text_system().add_fonts` before theme
   application; `"font.family": "Hanken Grotesk"`,
   `"mono_font.family": "Google Sans Code"` in both configs; About-page
   wordmark renders Xanh Mono italic.
3. **Port spec.** `docs/design/mono-port-spec.md` records the token
   mapping (source value + resolved hex + target key, both modes),
   radii, shadow policy (flat surfaces; overlays excepted), sidebar
   list-item metrics, and font roles.
4. **Sidebar styling.** `Settings` nav uses `sidebar_width(px(240))`
   plus `sidebar_style` refinement if needed for the source list-item
   look (active = `sidebar.accent.*`).
5. **Scope boundary.** Single `Settings` window retained — no header
   bar, app sidebar, ⌘K palette, or multi-view navigation.
6. **Conventions unchanged.** `AGENTS.md` rules stay authoritative
   (`cx.theme()` colors, `SettingField` `.default_value`, `Root`
   overlay layers). Source named-rules inform values only; they are
   not adopted as mono-desk policy.
7. **Gates.** `cargo test`, `make check`, `make lint` green; `make fmt`
   applied. `make run` is owner-performed (GUI), not a CI gate.
8. **Rail slots (vendored patch).** `vendor/gpui-component/` holds
   gpui-component 0.6.1 with exactly two additive builder methods on
   `Settings`: `sidebar_header(impl IntoElement + 'static)` — content
   rendered above the search `Input` inside the rail header — and
   `sidebar_footer(impl IntoElement + 'static)` — content passed to the
   existing `Sidebar::footer` slot. `Cargo.toml` gains
   `[patch.crates-io] gpui-component = { path = "vendor/gpui-component" }`.
   `vendor/patches/sidebar-slots.patch` reproduces the delta against
   upstream 0.6.1; no other upstream behavior changes.
9. **Rail composition.** Rail behavior is preserved (search filtering,
   page/group selection, nested sub-items for pages with more than one
   titled group, `sidebar.accent.*` active look, 240px width) and the
   rail now renders, top to bottom: brand block, search input, item list,
   footer toggle.
10. **Brand block.** `assets/brand/brand-mark.svg` is the kumo-desk mark
    copied verbatim (gradient preserved), rendered 32px square next to the
    wordmark `settings-demo` in Xanh Mono italic — source `h-8 w-8` +
    `font-serif text-xl italic tracking-tight`.
11. **Asset source.** `main()` registers one app-level `AssetSource` that
    serves the brand path and delegates every other path to
    `gpui_kit::assets::Assets` (so `IconName` icons keep resolving); the
    mark renders via `svg()` with no second `with_assets` call.
12. **Theme toggle.** The rail footer holds a theme toggle that flips
    `ThemeMode`, shows the current mode (sun/moon), and repaints the
    window in the same click — `Theme::change(mode, Some(window), cx)`,
    since the existing `None` form refreshes no window.
13. **Spec sync.** `docs/design/mono-port-spec.md` Sidebar metrics section
    moves from "not ported" to the applied state: brand block, footer
    toggle, vendored-patch record, and the gaps that remain.
14. **No regression.** The three checked phases keep their behavior
    (dark-mode setting item, page resets, About wordmark) and all gates
    stay green on the patched dep graph.

## Non-goals

- Product port: no Tasks/AI Chat/Kanban or any multi-view surface.
- App chrome outside the settings rail: no 40px header bar, no app-level
  collapsible sidebar (`Settings` keeps `collapsible(false)`), no ⌘B
  collapse, no command palette, no window controls, no second window.
- Upstreaming the two slots to `longbridge/gpui-kit` — deferred; the
  vendored patch is the delivery vehicle, and dropping `vendor/` is a
  follow-up once a release carries the hooks.
- Reworking `Settings` internals beyond the two additive slots: item
  density, inactive-item 70% opacity, and 10px uppercase group labels
  stay documented deviations.
- Source rule adoption (R2 token rule, 60% floor, CLS rules) as
  enforced mono-desk policy.
- Full 8-level typography hierarchy (families + heading italic only).
- Persistence, i18n, backend, Tauri parity.
- Any change inside the kumo-desk checkout (`$HOME/Lab/kumo-desk`,
  read-only source).

## Approach and Risks

**Approach.** One declarative theme file + one small installer:

- `assets/themes/mono.json` — gpui `ThemeSet` (`name`, `themes`) with
  `mono-light` / `mono-dark` `ThemeConfig`s; colors as resolved hex in
  `ThemeConfigColors` serde keys.
- `src/theme.rs` `install(cx)` — called in `main()` after
  `gpui_kit::init(cx)`:
  `ThemeRegistry::global_mut(cx).load_themes_from_str(include_str!())`
  → assign `Theme::global_mut(cx).light_theme` / `.dark_theme` from
  `themes()` → `add_fonts` → `Theme::change(mode, None, cx)`.
  Verified: `load_themes_from_str` only fills `themes()`;
  `default_*_theme` stays built-in, so direct field assignment is
  required. `Theme::change` then drives both modes — the existing
  dark-mode `SettingField::switch` keeps working untouched.
- Unit test in `src/theme.rs` (`#[cfg(test)]`) — deserializes the
  embedded JSON into `ThemeSet`, asserts both theme names, required
  color keys `Some`, and font family values.

**Resolved token mapping** (authoritative for `mono.json`; oklch→hex
precomputed, cross-checked against the source token table's own literal
anchors `#1f1e1d`, `#f7f9fa`, `#141414`):

| Source token | Light | Dark | ThemeConfigColors key(s) |
|---|---|---|---|
| brand | `#124b68` | `#007595` | `primary.background`, `ring`, `link`, `caret`, `progress.bar.background` |
| brand hover | `#0d3a52` | `#3689a4` | `primary.hover.background`, `link.hover` |
| base canvas | `#ffffff` | `#262624` | `background`, `popover.background`, `group_box.background`, `tab_bar.background`, `list.background`, `tab.active.background` |
| recessed | `#f7f9fa` | `#1b1b19` | `muted.background`, `skeleton.background`, `secondary.background`, `switch.background` |
| line | `#e5e5e5` | `#3e3e38` | `border`, `input.border`, `drag.border` |
| text | `#18181b` | `#f5f5f5` | `foreground`, `popover.foreground`, `secondary.foreground`, `tab.foreground` |
| muted text (60% blend) | `#747476` | `#a2a2a1` | `muted.foreground`, `description_list.label.foreground` |
| sidebar bg | `#ffffff` | `#1f1e1d` | `sidebar.background` |
| sidebar fg | `#141414` | `#c3c0b6` | `sidebar.foreground` |
| sidebar accent | `#e5e5e5` | `#0f0f0e` | `sidebar.accent.background`, `accent.background`, `list.hover.background` |
| sidebar accent fg | `#124b68` | `#c3c0b6` | `sidebar.accent.foreground`, `accent.foreground`, `link.active` |
| sidebar border | `#e5e5e5` | `#ebebeb` | `sidebar.border` |
| sidebar primary | `#124b68` | `#343434` | `sidebar.primary.background` |
| sidebar primary fg | `#ffffff` | `#fbfbfb` | `sidebar.primary.foreground`, `primary.foreground` |
| selection (20% brand) | `#d0dbe1` | `#1e363b` | `selection.background`, `list.active.background` |
| scrollbar thumb (18% text) | `#d5d5d6` | `#4b4b4a` | `scrollbar.thumb.background`, `scrollbar.thumb.hover.background` |
| danger (red) | `#e95048` | `#e95048` | `danger.background`, `danger.foreground` |
| warning (amber) | `#e5a14b` | `#e5a14b` | `warning.background`, `warning.foreground` |
| success (green) | `#3fb785` | `#3fb785` | `success.background`, `success.foreground` |
| info (cyan) | `#52b3d1` | `#52b3d1` | `info.background`, `info.foreground` |
| status hues | — | — | `chart_1..5` = amber, green, cyan, purple `#8d6be7`, red |

Fonts: `font.family` = `Hanken Grotesk`; `mono_font.family` =
`Google Sans Code`; `font.size` = 14, `mono_font.size` = 13; wordmark /
headings styled `Xanh Mono` italic at use sites.

**Constraints.** Rust 2024, single crate, `gpui-kit = "0.6"`; headless
gates are `cargo test` / `make check` / `make lint`; `make run` needs
the owner's GUI session. Source repo stays strictly read-only.

**Dependencies.** Network fetch for font TTFs (google/fonts repo or
Google Fonts CSS API — OFL). `serde_json` may need `cargo add` for the
test (currently transitive-only).

**Risks and mitigations.**

- Hex conversion drift → values above are precomputed and anchored;
  any change must update table + spec doc together.
- Hanken Grotesk may ship variable-only → fallback: Google Fonts CSS
  API static instances; else variable TTF at 400 + record limitation.
  Stop after two failed sources per family; ship remaining, report.
- `sidebar.border` dark `#ebebeb` looks inverted vs typical dark UIs —
  ported faithfully; flag for owner visual check.
- Theme name strings must match TTF family names exactly → test
  asserts JSON values; spec doc lists them.
- Revert = `git checkout` of `assets/`, `src/theme.rs`, installer call.

**Rejected alternatives.**

- Mutating `ThemeColor` fields in Rust — scatters ~40 constants, loses
  file↔spec review symmetry.
- `watch_dir` + runtime themes dir — `include_str!` embeds instead.
- `light-dark()` semantics — meaningless in GPUI; two configs is the
  native equivalent.

**Approach — rail chrome (requirements 8–14).** Vendor upstream, add two
slots, compose in the app:

- `vendor/gpui-component/` is the registry crate tree for 0.6.1 copied
  verbatim (minus `.cargo-ok`); the only source edit is
  `src/setting/settings.rs` gaining `sidebar_header` / `sidebar_footer`
  as `Option<AnyElement>` — the same shape as `Sidebar::header`/`footer`,
  which already `take()` their slot from a `mut self` render. The header
  slot renders above the search `Input` by wrapping the existing header
  child in `v_flex().w_full().gap_2()`; the footer is handed to
  `Sidebar::footer`. `Cargo.toml` adds
  `[patch.crates-io] gpui-component = { path = "vendor/gpui-component" }`
  (`gpui-kit` requires `0.6.1`, so the patch resolves).
- `vendor/patches/sidebar-slots.patch` carries that delta against the
  registry copy, so an upgrade = re-copy upstream + re-apply one patch.
- Why not a custom shell: `SettingPage::render`, `SettingGroup::render`
  and `SettingItem::render_item` are `pub(super)`, and the field
  renderers (`BoolField`, `NumberField`, `StringField`, `DropdownField`,
  `ElementField`) are `pub(crate)` — an app-side `Sidebar` shell would
  reimplement page/group/item rendering, search filtering, reset and
  dirty state. Two additive slots keep all of it and are upstreamable.

**Rejected alternatives (rail chrome).**

- Git fork + `[patch] git = …` — needs a fork you own, and network on a
  fresh clone.
- Machine-local path patch (`~/Lab/gpui-component`) — breaks `make check`
  on any other machine.
- Absolute-positioned brand block over the rail via `header_style`
  padding — fragile z-order and hit-testing; sits over the search box.
- Upstream-first — deferred; the vendored patch ships now (Non-goals).

**Rail risks and mitigations.**

- Vendored tree drifts from upstream → `diff -r` audit in `vp-1`; the
  patch file is the single record of the delta.
- `[patch.crates-io]` fails to resolve → checked in `vp-3` via
  `cargo tree -i gpui-component`; on failure stop and escalate, never
  hand-edit `Cargo.lock`.
- Dep cache: the patch rebuilds `gpui-component`, `gpui-kit` and the app
  — not the ~400-crate tree; `RUSTFLAGS` stays untouched (Makefile).
- Toggle does not repaint → `Theme::change(mode, Some(window), cx)`; the
  `None` form refreshes no window (verified in `theme/mod.rs`).
- Brand mark renders blank → 1254×1254 gradient SVG drawn at 32px; the
  `AppAssets` test asserts the bytes resolve, owner `make run` confirms
  the render.
- Icon regression from the new `AssetSource` → the same test asserts an
  `IconName` path still resolves through the delegate.
- Repo weight: ~3.4 MB vendored, owner-approved (`brainstorm`
  2026-09-18); `vendor/` is not gitignored (verified).
- Rollback: `git checkout -- Cargo.toml Cargo.lock src` +
  `git clean -fd vendor` restores the pre-patch graph.

## Phases and Verification

Order: `theme-colors` → `mono-fonts` → `patterns-spec` → `vendor-patch` →
`rail-chrome` → `spec-sync`.
Font keys enter the JSON only in `mono-fonts` (a named-but-missing
family is used as-is and could render wrong before fonts register).

### Phase `theme-colors` — story_id: story-kumo-theme-colors-20260918

- **Status:** `done`
- **Goal:** `mono-light`/`mono-dark` active; dark-mode toggle intact.
- **Depends on:** none.
- **Allowed surfaces:** `assets/themes/mono.json` (new), `src/theme.rs`
  (new), `src/main.rs` (one `theme::install(cx)` line), `Cargo.toml`
  (serde_json only if needed).
- **Avoided surfaces:** view/page code, `docs/design/`.

**Wave 1** (independent tasks):

1. `tc-1` — Write `assets/themes/mono.json` per the mapping table
   (required keys all present; optional extras allowed).
   - Output: valid ThemeSet JSON, two themes.
   - Check: `python3 -m json.tool assets/themes/mono.json > /dev/null`.
2. `tc-2` — `src/theme.rs` `pub fn install(cx: &mut App)` as specced
   above; wire into `main()` after `gpui_kit::init(cx)`.
   - Output: compiles; toggle path unchanged.
   - Check: `make check`.
3. `tc-3` — `#[cfg(test)]` test: embedded JSON → `ThemeSet`; asserts
   `mono-light`/`mono-dark` present and `background`, `foreground`,
   `border`, `primary.background`, `sidebar.background`,
   `sidebar.accent.background`, `danger.background`, `ring` are `Some`
   in both. `cargo add serde_json` only if unreachable as transitive.
   - Output: `cargo test` green headless.
   - Check: `cargo test`.

**Phase checks:** `cargo test` + `make check` + `make lint` green;
`make fmt` applied.
**Stop:** ThemeSet fails deserialization twice → halt, report schema
mismatch (per WORKFLOW `escalate_when`).

### Phase `mono-fonts` — story_id: story-kumo-fonts-20260918

- **Status:** `done`
- **Goal:** three font families embedded + active; wordmark italic.
- **Depends on:** `theme-colors`.
- **Allowed surfaces:** `assets/fonts/**` (new), `src/theme.rs`
  (add_fonts block), `assets/themes/mono.json` (font keys only),
  `src/main.rs` (wordmark styling only).
- **Avoided surfaces:** `docs/`, settings field definitions.

**Wave 1:**

1. `mf-1` — Fetch TTFs → `assets/fonts/`: `XanhMono-Regular.ttf`,
   `XanhMono-Italic.ttf`, `HankenGrotesk-{Regular,Medium,SemiBold,Bold}.ttf`,
   `GoogleSansCode-{Regular,Medium}.ttf` (OFL; static instances
   preferred, variable acceptable for Hanken — record which).
   - Output: 8 files present.
   - Check: `file assets/fonts/*.ttf` → TrueType/OpenType each.
2. `mf-2` — In `install(cx)` before `Theme::change`:
   `cx.text_system().add_fonts(vec![Cow::Borrowed(&include_bytes!("../assets/fonts/<f>")[..]), ...])`;
   add `font.family`/`mono_font.family`/`font.size`/`mono_font.size`
   to both configs.
   - Output: compiles — `include_bytes!` fails on missing file.
   - Check: `make check`.
3. `mf-3` — Extend `tc-3` test: assert `font.family`/`mono_font.family`
   `Some` in both configs. Style the About-page `"settings-demo"`
   wordmark Xanh Mono italic via gpui text styling.
   - Output: `cargo test` green.
   - Check: `cargo test`; `make lint`.

**Phase checks:** `file` output per font; `cargo test` + `make check` +
`make lint` green. Owner `make run` recommended — highest visual-drift
risk lives here.
**Stop:** a family unobtainable as TTF/OTF after two source attempts →
ship without it, record in spec doc, report.

### Phase `patterns-spec` — story_id: story-kumo-patterns-20260918

- **Status:** `done`
- **Goal:** spec doc committed; sidebar metrics applied.
- **Depends on:** `theme-colors`, `mono-fonts` (documents applied
  state).
- **Allowed surfaces:** `docs/design/mono-port-spec.md` (new),
  `src/main.rs` (`Settings::sidebar_width`/`sidebar_style` only).
- **Avoided surfaces:** theme JSON (frozen), fonts.

**Wave 1:**

1. `ps-1` — Write `docs/design/mono-port-spec.md`: the full mapping
   table (source value + hex + key, both modes), radii, shadow policy,
   sidebar metrics (240px, active → `sidebar.accent.*`), font roles,
   control metrics (32px h / 12px px as target where components allow).
   - Output: doc committed.
   - Check: every `ThemeConfigColors` key used in `mono.json` appears
     in the table (grep-driven).
2. `ps-2` — `Settings::new(...).sidebar_width(px(240.))`; add
   `sidebar_style` refinement only if the active-item look needs it;
   keep `sidebar_size_range` defaults.
   - Output: compiles.
   - Check: `make check`; `make lint`.
3. `ps-3` — Final gate: `make fmt`, `make check`, `make lint` green;
   `git status --short` shows only intended files.
   - Output: clean tree scoped to plan.
   - Check: `make check && make lint`; `git status --short`.

**Phase checks:** spec doc complete vs JSON keys; all gates green.
Owner `make run` is the initiative release check.
**Stop:** if `sidebar_style` cannot express the source list-item look,
record the gap in the spec doc — do not invent new styling surface.

### Phase `vendor-patch` — story_id: story-kumo-vendor-patch-20260918

- **Status:** `done`
- **Goal:** `gpui-component` resolves to `vendor/gpui-component/`
  (upstream 0.6.1 + exactly the two additive `Settings` slots); the delta
  is reproducible from `vendor/patches/sidebar-slots.patch`; the app
  builds and tests pass on the patched graph.
- **Depends on:** none.
- **Allowed surfaces:** `vendor/gpui-component/**` (new),
  `vendor/patches/sidebar-slots.patch` (new), `Cargo.toml`, `Cargo.lock`.
- **Avoided surfaces:** `src/**`, `assets/**`, `docs/**`.

**Wave 1:**

1. `vp-1` — Copy the upstream tree verbatim:
   `rsync -a --exclude .cargo-ok ~/.cargo/registry/src/index.crates.io-*/gpui-component-0.6.1/
   vendor/gpui-component/`.
   - Output: `vendor/gpui-component/` identical to upstream.
   - Check: `diff -r --brief -x .cargo-ok ~/.cargo/registry/src/index.crates.io-*/gpui-component-0.6.1
     vendor/gpui-component` → no output.

**Wave 2:**

2. `vp-2` — `Cargo.toml`: add `[patch.crates-io]` with
   `gpui-component = { path = "vendor/gpui-component" }`.
   - Output: dependency resolves to the vendored path.
   - Check: `cargo tree -i gpui-component` → `gpui-component v0.6.1
     (…/vendor/gpui-component)`; `make check`.
3. `vp-3` — In `vendor/gpui-component/src/setting/settings.rs` add the
   two slots: fields `sidebar_header: Option<AnyElement>` /
   `sidebar_footer: Option<AnyElement>`; builders
   `pub fn sidebar_header(mut self, header: impl IntoElement) -> Self`
   and `pub fn sidebar_footer(mut self, footer: impl IntoElement) -> Self`
   storing `into_any_element()`; `fn render(mut self, …)` and
   `render_sidebar(&mut self, …)`; the existing header child wrapped in
   `v_flex().w_full().gap_2().when_some(self.sidebar_header.take(), …)`;
   `.when_some(self.sidebar_footer.take(), |this, footer|
   this.footer(footer))` on the `Sidebar`. Imports: add `v_flex` to the
   `crate::{…}` list and `AnyElement` to the `gpui::{…}` list. No other
   edits.
   - Output: compiles; behavior unchanged while both slots are unset.
   - Check: `make check`; `cargo test`.

**Wave 3:**

4. `vp-4` — Write `vendor/patches/sidebar-slots.patch`: `diff -u` of the
   registry `src/setting/settings.rs` against the vendored file, headers
   rewritten to `a/src/…` / `b/src/…`.
   - Output: one patch file holding the whole delta.
   - Check: on a pristine copy (`cp -R ~/.cargo/registry/src/index.crates.io-*/gpui-component-0.6.1
     /tmp/gpui-check`), `git apply --check -p1
     vendor/patches/sidebar-slots.patch` → clean; then apply it and
     `diff -r --brief -x .cargo-ok /tmp/gpui-check vendor/gpui-component`
     → no output.

**Phase checks:** `cargo test`, `make check`, `make lint` green on the
patched graph; `make fmt` applied; `git status --short` scoped to
`vendor/`, `Cargo.{toml,lock}`.
**Stop:** `[patch.crates-io]` fails to resolve, or the vendored crate
fails to build after two attempts → halt and report (WORKFLOW
`escalate_when`); never hand-edit `Cargo.lock`.

### Phase `rail-chrome` — story_id: story-kumo-rail-chrome-20260918

- **Status:** `done`
- **Goal:** the rail opens with the kumo brand mark + Xanh Mono italic
  wordmark above the search input, and ends with a working theme toggle;
  icons keep resolving.
- **Depends on:** `vendor-patch`.
- **Allowed surfaces:** `assets/brand/brand-mark.svg` (new),
  `src/assets.rs` (new), `src/main.rs`.
- **Avoided surfaces:** `vendor/**` (frozen after `vp-4`),
  `assets/themes/**`, `assets/fonts/**`, `src/theme.rs`, `docs/**`.

**Wave 1:**

1. `rc-1` — Copy `$HOME/Lab/kumo-desk/public/brand-mark.svg` to
   `assets/brand/brand-mark.svg` verbatim; add `src/assets.rs` with
   `pub struct AppAssets;` implementing `gpui::AssetSource` — `load`
   returns the `include_bytes!` brand bytes for `"brand/brand-mark.svg"`
   and otherwise delegates to `gpui_kit::assets::Assets`; `list`
   delegates and adds the brand path. `main()` switches to
   `.with_assets(AppAssets)`. `#[cfg(test)]` test: `load` is `Some` for
   the brand path, and
   `gpui_kit::assets::IconNamed::path(IconName::Search)` still resolves
   through the delegate.
   - Output: brand asset embedded; icon paths unchanged.
   - Check: `cargo test`; `make check`.
2. `rc-2` — `SettingsWindow::render`: `.sidebar_header(rail_brand(cx))`
   — `svg().path("brand/brand-mark.svg").size(px(32.))` beside
   `div().font_family("Xanh Mono").italic().child("settings-demo")` in
   `h_flex().gap_2().items_center()` — and `.sidebar_footer(rail_footer())`
   — `Button::new("theme-toggle")` showing `IconName::Sun` /
   `IconName::Moon` per `cx.theme().mode.is_dark()`, `on_click` calling
   `Theme::change(next, Some(window), cx)`. `svg` joins the `gpui_kit`
   import list.
   - Output: rail renders brand → search → list → toggle.
   - Check: `make check`; `make lint`. (Both icons are in the default
     set — `icons/sun.svg`, `icons/moon.svg` in `default-icons.txt`.)

**Wave 2:**

3. `rc-3` — Verification pass: `make fmt`, `make check`, `make lint`,
   `cargo test`; `git status --short` scoped to `assets/brand/`,
   `src/assets.rs`, `src/main.rs`; owner `make run` checklist — mark
   renders at 32px with its gradient, wordmark italic, search still
   filters, page/group items still select, toggle flips the palette and
   repaints, sidebar icons still draw.
   - Output: green gates + owner visual verdict.
   - Check: `make check && make lint && cargo test`;
     `git status --short`.

**Phase checks:** all gates green; owner `make run` (highest visual-drift
risk since `mono-fonts`).
**Stop:** the mark renders blank after two attempts → fall back to
`Icon::new(IconName::GalleryVerticalEnd)` (the app's existing About mark)
and record the deviation for `spec-sync`; do not patch the vendored crate
further.

### Phase `spec-sync` — story_id: story-kumo-spec-sync-20260918

- **Status:** `done`
- **Goal:** the spec doc states the applied rail; the initiative closes
  on green gates.
- **Depends on:** `rail-chrome`.
- **Allowed surfaces:** `docs/design/mono-port-spec.md`.
- **Avoided surfaces:** `src/**`, `vendor/**`, `assets/**`.

**Wave 1:**

1. `ss-1` — Rewrite the Sidebar metrics section: replace "Not ported —
   sidebar brand header" and "Future paths if chrome is ever in scope"
   with the applied state (brand block metrics, footer toggle, the
   vendored-patch record — crate, version, patched file, patch path);
   keep the remaining gaps (item density, inactive 70% opacity, 10px
   uppercase labels); drop the brand-header entry from "Known flags for
   owner review", keep the dark `sidebar.border` flag.
   - Output: doc matches the shipped rail.
   - Check: every claim is grep-verifiable —
     `grep -n "sidebar_header\|sidebar_footer" src/main.rs`,
     `grep -n "patch.crates-io" -A 2 Cargo.toml`,
     `ls assets/brand/brand-mark.svg` — all three succeed.
2. `ss-2` — Final gate: `make fmt`, `make check`, `make lint`,
   `cargo test`; `git status --short` scoped to the initiative;
   `docs/PROJECT.md` already names the vendored patch (verify only).
   - Output: clean tree scoped to plan.
   - Check: `make check && make lint && cargo test`;
     `git status --short`.

**Phase checks:** doc claims trace to code; all gates green. Owner
`make run` is the initiative release check.
**Stop:** any claim that cannot be verified headless → record it as "not
independently verified" instead of asserting it.

## Progress

- 2026-09-18 16:01 +07 — phase `theme-colors`, wave 1, task `phase-start`,
  task_status=in-progress — run anchor; executing tc-1/tc-2/tc-3 per plan.
- 2026-09-18 16:01 +07 — phase `theme-colors`, wave 1, task `tc-1`,
  task_status=DONE — wrote `assets/themes/mono.json` (mono-light/mono-dark,
  55 color keys each, radius 6/10, shadow false); `python3 -m json.tool
  assets/themes/mono.json > /dev/null` pass; full-table diff: 0 mismatches,
  0 extra keys.
- 2026-09-18 16:01 +07 — phase `theme-colors`, wave 1, task `tc-2`,
  task_status=DONE — `src/theme.rs` `install(cx)`: `load_themes_from_str` →
  assign `light_theme`/`dark_theme` → `Theme::change(mode)`; wired
  `mod theme;` + `theme::install(cx)` after `gpui_kit::init(cx)` in
  `src/main.rs`; `make check` pass.
- 2026-09-18 16:01 +07 — phase `theme-colors`, wave 1, task `tc-3`,
  task_status=DONE — `#[cfg(test)]` test deserializes embedded JSON into
  `ThemeSet`, asserts both theme names + 8 required keys `Some` in both;
  `serde_json` added as dev-dependency (transitive-only before);
  `cargo test` pass (1 test).
- 2026-09-18 16:01 +07 — phase `theme-colors`, wave 1 summary — 3/3 tasks
  DONE; phase checks green: `cargo test`, `make check`, `make lint` pass;
  `make fmt` applied. Proceeding to in-session gate.
- 2026-09-18 16:34 +07 — phase `mono-fonts`, wave 1, task `phase-start`,
  task_status=in-progress — run anchor; executing mf-1/mf-2/mf-3 per plan.
- 2026-09-18 16:34 +07 — phase `mono-fonts`, wave 1, task `mf-1`,
  task_status=DONE — 8 TTFs in `assets/fonts/`: XanhMono-{Regular,Italic}
  upstream statics; HankenGrotesk-{Regular,Medium,SemiBold,Bold} +
  GoogleSansCode-{Regular,Medium} instanced from google/fonts variable
  TTFs (`fonttools varLib.instancer --update-name-table`); correct
  OS/2.usWeightClass + unique PS names verified. `file assets/fonts/*.ttf`
  → TrueType Font data, all 8.
- 2026-09-18 16:34 +07 — phase `mono-fonts`, wave 1, task `mf-2`,
  task_status=DONE — `cx.text_system().add_fonts(...)` (8 `include_bytes!`)
  inserted in `install(cx)` before `Theme::change`;
  `font.family`/`mono_font.family`/`font.size`/`mono_font.size` added to
  both configs. `make check` pass.
- 2026-09-18 16:34 +07 — phase `mono-fonts`, wave 1, task `mf-3`,
  task_status=DONE — test extended asserting `font_family`/
  `mono_font_family` `Some` in both configs; About wordmark
  `"settings-demo"` wrapped in `div().font_family("Xanh Mono").italic()`.
  `cargo test` + `make lint` pass.
- 2026-09-18 16:34 +07 — phase `mono-fonts`, wave 1 summary — 3/3 tasks
  DONE; phase checks green: `file` (8/8 TrueType), `cargo test`,
  `make check`, `make lint` pass; `make fmt` applied. Proceeding to
  in-session gate.
- 2026-09-18 16:45 +07 — phase `patterns-spec`, wave 1, task
  `phase-start`, task_status=in-progress — run anchor; executing
  ps-1/ps-2/ps-3 per plan.
- 2026-09-18 16:45 +07 — phase `patterns-spec`, wave 1, task `ps-1`,
  task_status=DONE — wrote `docs/design/mono-port-spec.md`: full mapping
  table (source value + hex + key, both modes), radii, shadow policy,
  sidebar metrics + recorded gaps, font roles, control metrics, owner
  flags. Check: all 55 `ThemeConfigColors` keys from `mono.json` grep
  present in doc.
- 2026-09-18 16:45 +07 — phase `patterns-spec`, wave 1, task `ps-2`,
  task_status=DONE — `Settings::sidebar_width(px(240.))` applied;
  `sidebar_style` not added (active-item look is native
  `SidebarMenuItem`: font_medium + `sidebar.accent.*`; gaps recorded in
  spec doc per Stop clause); `sidebar_size_range` defaults kept.
  `make check` + `make lint` pass.
- 2026-09-18 16:45 +07 — phase `patterns-spec`, wave 1, task `ps-3`,
  task_status=DONE — `make fmt`, `make check`, `make lint`, `cargo test`
  green; `cargo fmt --check` stable; `git status --short` shows only
  intended files (pre-existing `docs/PROJECT.md` mod excluded).
- 2026-09-18 16:45 +07 — phase `patterns-spec`, wave 1 summary — 3/3
  tasks DONE; all gates green. Proceeding to in-session gate (final
  phase → stays in-progress on clean verdict, routes to `check full`).

- 2026-09-19 06:50 +07 — phase `vendor-patch`, wave 1, task `phase-start`,
  task_status=in-progress — run anchor; executing vp-1/vp-2/vp-3/vp-4 per
  plan; surfaces: `vendor/**`, `Cargo.{toml,lock}`.
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 1, task `vp-1`,
  task_status=DONE — `rsync -a --exclude .cargo-ok` of the registry crate
  → `vendor/gpui-component/` (248 files, 3.4 MB); check
  `diff -r --brief -x .cargo-ok <registry>/gpui-component-0.6.1
  vendor/gpui-component` → no output (byte-identical to upstream).
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 1 summary — 1/1 task
  DONE; proceeding to wave 2.
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 2, task `vp-2`,
  task_status=DONE — `Cargo.toml` gained `[patch.crates-io]`
  `gpui-component = { path = "vendor/gpui-component" }` + pointer comment;
  `cargo tree -i gpui-component` → `gpui-component v0.6.1
  (/Users/tinhtute/Lab/mono-desk/vendor/gpui-component)` ← `gpui-kit
  v0.6.1` ← `mono-desk v0.1.0`; `make check` pass (21.4s, only
  gpui-component + downstream recompiled).
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 2, task `vp-3`,
  task_status=DONE — vendored `src/setting/settings.rs`: `sidebar_header`
  / `sidebar_footer` as `Option<AnyElement>` (fields, `new()`, two
  builders), `render(mut self)` + `render_sidebar(&mut self)`, header
  child wrapped in `v_flex().w_full().gap_2()` above the search `Input`,
  `Sidebar::footer` pass-through; imports extended (`v_flex`,
  `AnyElement`). No other edits. `rustfmt --check --edition 2024
  vendor/gpui-component/src/setting/settings.rs` → exit 0 (delta is
  formatting-neutral); `make check` pass; `cargo test` pass (1 test).
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 2 summary — 2/2 tasks
  DONE; proceeding to wave 3.
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 3, task `vp-4`,
  task_status=DONE — wrote `vendor/patches/sidebar-slots.patch` (101
  lines, 39 added / 6 removed, 4 hunks, headers `a/src/…` / `b/src/…`);
  on a pristine `mktemp -d` copy: `git apply --check -p1` → clean, then
  applied and `diff -r --brief -x .cargo-ok` vs `vendor/gpui-component`
  → no output (patch reproduces the tree exactly).
- 2026-09-19 06:52 +07 — phase `vendor-patch`, wave 3 summary — 1/1 task
  DONE; all gates green: `cargo test`, `make check`, `make lint`,
  `cargo fmt --check`; proceeding to in-session gate.
- 2026-09-19 06:52 +07 — phase `vendor-patch`, tree note — untracked
  `.agents/` and `CLAUDE.md` appeared in the worktree at 06:49 (session
  tooling, not this phase); left untouched, out of scope.
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 1, task `phase-start`,
  task_status=in-progress — run anchor recorded with the wave flush
  (execution began immediately after the `vendor-patch` gate, 06:53);
  surfaces: `assets/brand/`, `src/assets.rs`, `src/main.rs`.
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 1, task `rc-1`,
  task_status=DONE — `assets/brand/brand-mark.svg` copied verbatim
  (`diff` clean; sha256 `c0518c5a…e8dd` on both sides); `src/assets.rs`
  (55 lines) `AppAssets` serves `brand/brand-mark.svg` from
  `include_bytes!` and delegates every other path to
  `gpui_kit::assets::Assets`; `main()` switched to
  `.with_assets(assets::AppAssets)`; test
  `assets::tests::brand_mark_loads_and_icons_still_delegate` asserts the
  `<svg` bytes and `IconName::Search` path equality with the delegate.
  `cargo test` pass (2 tests).
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 1, task `rc-2`,
  task_status=DONE — `rail_brand()` (`svg().path("brand/brand-mark.svg")
  .size(px(32.))` + Xanh Mono italic `text_xl` wordmark) and
  `rail_footer(cx)` (`Button::new("rail-theme-toggle")`, `primary()`,
  `w_full()`, Sun/Moon per mode, labels Light/Dark mode, `on_click` →
  `Theme::change(next, Some(window), cx)`) wired through
  `.sidebar_header(..)` / `.sidebar_footer(..)`; +40/-3 in `src/main.rs`.
  One failure: `ButtonVariants` not in scope → added to the
  `button::{Button, ButtonVariants}` import; re-check green.
  `make check`, `make lint`, `cargo fmt --check` pass.
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 1 summary — 2/2 tasks
  DONE; proceeding to wave 2.
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 2, task `rc-3`,
  task_status=DONE — `make fmt`, `make check`, `make lint`, `cargo test`
  (2 tests), `cargo fmt --check`, `make build` all pass;
  `git status --short` scoped to `assets/brand/`, `src/assets.rs`,
  `src/main.rs` (+ `vendor/`, `Cargo.{toml,lock}` from `vendor-patch`);
  owner `make run` checklist handed over (mark 32px with gradient,
  italic wordmark, search filters, page/group items select, toggle
  repaints, sidebar icons draw).
- 2026-09-19 07:11 +07 — phase `rail-chrome`, wave 2 summary — 1/1 task
  DONE; all gates green; proceeding to in-session gate.
- 2026-09-19 07:12 +07 — phase `spec-sync`, wave 1, task `phase-start`,
  task_status=in-progress — run anchor; surfaces:
  `docs/design/mono-port-spec.md`.
- 2026-09-19 07:12 +07 — phase `spec-sync`, wave 1, task `ss-1`,
  task_status=DONE — Sidebar metrics section rewritten: "Not ported —
  sidebar brand header" and "Future paths if chrome is ever in scope"
  replaced by the applied rail chrome (brand block metrics, `AppAssets`
  asset path, footer toggle with the `Some(window)` repaint note, the
  vendored-patch record with the patch path, rejected alternatives);
  remaining gaps kept; "Known flags for owner review" drops the
  brand-header entry, adds the unverified-rail note, and the revert path
  now covers the rail. Checks: `grep -n "sidebar_header\|sidebar_footer"
  src/main.rs` → 346/347; `grep -n "patch.crates-io" -A 2 Cargo.toml` →
  14/15; `ls -l assets/brand/brand-mark.svg` → 2417 bytes;
  `grep -n "patch.crates-io\|vendor/gpui-component" docs/PROJECT.md` →
  hit.
- 2026-09-19 07:12 +07 — phase `spec-sync`, wave 1, task `ss-2`,
  task_status=DONE — final gate: `make fmt`, `make check`, `make lint`,
  `cargo test` (2 tests), `cargo fmt --check` pass; `git status --short`
  scoped to the initiative (`vendor/`, `assets/brand/`, `src/assets.rs`,
  `src/main.rs`, `Cargo.{toml,lock}`, `docs/design/`, plan); `.agents/`,
  `CLAUDE.md` untracked and unrelated; `docs/PROJECT.md` already names
  the vendored patch (verified).
- 2026-09-19 07:12 +07 — phase `spec-sync`, wave 1 summary — 2/2 tasks
  DONE; all gates green; final phase → stays `in-progress` on the clean
  verdict and routes to an independent `check full` (complete review
  deferred there).
- 2026-09-19 07:26 +07 — phase `vendor-patch`, task `vp-4`, log
  correction — the vp-4 entry above says "4 hunks"; the patch has 7
  hunks (101 lines). Content unchanged and re-verified line-by-line by
  the independent reviewer in `check full`; the count was a transcription
  error in this log only.
- 2026-09-22 19:47 +07 — initiative `mono-design-port`, owner sign-off —
  owner accepted the GUI-only proof gap ("tui thấy ok rồi"): the rail
  visual check (brand mark + italic wordmark, search, item list, footer
  theme toggle) is discharged by owner verdict; no `make run` was
  executed in this session. `handoff` proceeds to final closure.

## Decisions

- 2026-09-18 — phase `theme-colors`, task `tc-3`: `serde_json` added under
  `[dev-dependencies]` rather than `[dependencies]` (`cargo add` default).
  Rationale: only the `#[cfg(test)]` test uses it; release dep tree stays
  unchanged. Affected: `Cargo.toml`.
- 2026-09-18 — phase `mono-fonts`, task `mf-1`: Hanken Grotesk and Google
  Sans Code ship variable-only in `google/fonts`; produced the required
  static TTFs via `fonttools varLib.instancer wght=<n>
  --update-name-table` (weights 400/500/600/700 and 400/500
  respectively). Xanh Mono used upstream statics as-is. Rationale: plan
  names static files; instancing yields real TTFs with correct
  `OS/2.usWeightClass` + unique PostScript names (family name preserved
  for text-system lookup). Spec doc (`ps-1`) must record the variable →
  static instancing. Tooling: `fonttools` in throwaway `/tmp/fontenv`
  venv, not a repo dependency.
- 2026-09-18 — phase `theme-colors`, task `tc-1`: set `"shadow": false` in
  both configs (optional extra beyond the required mapping table).
  Rationale: requirement 3 documents "flat surfaces; overlays excepted";
  verified `theme().shadow` gates only surface controls (e.g.
  `button.rs`) while dialog/popover/notification shadows are hardcoded,
  so `false` is the faithful port. Spec doc (`ps-1`) must record it.
- 2026-09-19 — phase `rail-chrome`: the phase ran without the immediate
  `in-progress` write; its status, Current State, and `phase-start`
  Progress entry landed at 07:11 together with the wave record.
  Rationale: execution was continuous from the `vendor-patch` gate, so
  the plan file lagged one phase-start behind reality until the flush —
  a concurrent session would have read a stale `planned` status. No
  scope or evidence impact; `vendor-patch` used the immediate write and
  `spec-sync` does too. Affected: `rail-chrome` (bookkeeping only).
absorb: none

## Validation

- 2026-09-18 16:01 +07 — phase `theme-colors` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard)
  - `python3 -m json.tool assets/themes/mono.json > /dev/null` → pass
    (valid ThemeSet JSON; mono-light/mono-dark, 55 color keys each)
  - `cargo test` → pass (1 test:
    `theme::tests::embedded_theme_set_has_required_colors`)
  - `make check` → pass (`cargo check`, 0 errors)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass (after `make fmt`)
  - not_independently_verified: rendered theme and dark-mode toggle in a
    live window — `make run` is the owner-performed GUI check
  - receipt: context_sources: plan phase block + gpui-component 0.6.1
    theme/{schema,registry,mod}.rs / policy: work-full step 11 +
    check.md durable gate / judge: same-session / judge_model: SWE-2 Max /
    retries: 0 / rollback_point: `git checkout -- assets src/theme.rs
    src/main.rs Cargo.toml Cargo.lock` / failure_ledger: absent /
    enforcement: local-only (no pre-commit hook installed)
- 2026-09-18 16:34 +07 — phase `mono-fonts` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard)
  - `file assets/fonts/*.ttf` → pass (8/8 TrueType Font data; XanhMono
    upstream statics, Hanken/GoogleSansCode instanced statics at correct
    OS/2 weights with unique PS names)
  - `cargo test` → pass (1 test:
    `theme::tests::embedded_theme_set_has_required_colors`, now incl.
    font.family/mono_font.family assertions)
  - `make check` → pass (`cargo check`; `include_bytes!` paths resolve)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass (after `make fmt`)
  - `python3 -m json.tool assets/themes/mono.json > /dev/null` → pass
  - not_independently_verified: glyph rendering of Hanken weights, Xanh
    Mono italic wordmark, and Google Sans Code in a live window —
    `make run` is the owner-performed GUI check (plan flags this phase
    as highest visual-drift risk)
  - receipt: context_sources: plan phase block + gpui-pre-0.3.5
    text_system.rs + gpui-component-0.6.1 theme sources / policy:
    work-full step 11 + check.md durable gate / judge: same-session /
    judge_model: SWE-2 Max / retries: 0 / rollback_point:
    `git checkout -- assets src/theme.rs src/main.rs` /
    failure_ledger: absent / enforcement: local-only (no pre-commit
    hook installed)
- 2026-09-18 16:45 +07 — phase `patterns-spec` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard)
  - `cargo test` → pass (1 test:
    `theme::tests::embedded_theme_set_has_required_colors`)
  - `make check` → pass (`cargo check`)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass (after `make fmt`)
  - `git status --short` → scoped: assets/, src/theme.rs, src/main.rs,
    docs/design/, Cargo.{toml,lock} + plan docs (pre-existing
    docs/PROJECT.md mod excluded)
  - spec completeness: all 55 `ThemeConfigColors` keys in `mono.json`
    grep present in `docs/design/mono-port-spec.md`
  - not_independently_verified: 240px rail width, Xanh Mono italic
    wordmark, and vendored-font rendering in a live window —
    `make run` is the owner-performed release check
  - receipt: context_sources: plan phase block + kumo-desk DESIGN.md /
    app.css / AppShell.tsx + gpui-component-0.6.1 setting+sidebar
    sources / policy: work-full step 11 + check.md durable gate /
    judge: same-session / judge_model: SWE-2 Max / retries: 0 /
    rollback_point: `git checkout -- docs/design src/main.rs` /
    failure_ledger: absent / enforcement: local-only (no pre-commit
    hook installed)
- 2026-09-18 16:45 +07 — phase `patterns-spec` — verdict: APPROVED
  (mode: full; judge: independent; judge_model: SWE-2 Max; scope:
  on target; depth: standard)
  - `python3 -m json.tool assets/themes/mono.json > /dev/null` → pass
  - `file assets/fonts/*.ttf` → pass (8/8 TrueType Font data)
  - `cargo test` → pass (1 test:
    `theme::tests::embedded_theme_set_has_required_colors`)
  - `make check` → pass (`cargo check`)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass
  - `git status --short` → scoped (assets/, src/theme.rs, src/main.rs,
    docs/design/, Cargo.{toml,lock} + plan docs; pre-existing
    docs/PROJECT.md excluded)
  - spec completeness: 55/55 `ThemeConfigColors` keys grep present in
    `docs/design/mono-port-spec.md`
  - font integrity: per-face `OS/2.usWeightClass` 400/500/600/700 +
    400/500, `XanhMono-Italic` fsSelection italic bit set, unique
    PostScript names, JSON family names match TTF name tables
  - complete review: security clean (embedded assets, dev-dep only, no
    unsafe/secrets/network surface); performance OK (~0.5 MB fonts,
    one-time startup cost); architecture verified (install order,
    observer interplay, untouched toggle); code quality OK
  - findings: none blocking; `docs/PROJECT.md` mod predates initiative
    (flagged); documented spec gaps are owner-visible by design
  - not_independently_verified: rendered output — 240px rail, italic
    wordmark, font weights, dark-mode palette — `make run` is the
    owner-performed release check; `judge: independent` declared per
    full-mode contract though reviewer shares model lineage with author
  - receipt: context_sources: plan (all phases) + mono.json + theme.rs +
    kumo-desk DESIGN.md/app.css/AppShell.tsx + gpui-component-0.6.1
    theme/setting/sidebar + fonttools name-table audit / policy:
    check.md full / judge: independent / judge_model: SWE-2 Max /
    retries: 0 / rollback_point: `git checkout -- assets src
    docs/design Cargo.toml Cargo.lock` / failure_ledger: absent /
    enforcement: local-only (no pre-commit hook installed)

- 2026-09-19 06:53 +07 — phase `vendor-patch` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard)
  - `cargo test` → pass (1 test:
    `theme::tests::embedded_theme_set_has_required_colors`, compiled
    against the vendored crate)
  - `make check` → pass (`cargo check`; 21.4s on the first patched run,
    only `gpui-component` + downstream recompiled)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `make build` → pass (`cargo build`; binary links against the patched
    crate, no GUI session needed)
  - `cargo fmt --check` → pass
  - `rustfmt --check --edition 2024 vendor/gpui-component/src/setting/settings.rs` → pass (exit 0; delta is formatting-neutral)
  - `cargo tree -i gpui-component` → `gpui-component v0.6.1
    (/Users/tinhtute/Lab/mono-desk/vendor/gpui-component)` under
    `gpui-kit v0.6.1` under `mono-desk v0.1.0`
  - `patch -p1 -R --dry-run -d vendor/gpui-component < vendor/patches/sidebar-slots.patch` → pass (exit 0; the patch is exactly the delta)
  - `git status --short` → scoped: `vendor/` (new), `Cargo.{toml,lock}`;
    untracked `.agents/`, `CLAUDE.md` appeared 06:49 from session tooling,
    not this phase — left untouched
  - proof gaps: no unit test exercises the two slots yet (first real use
    is `rail-chrome` rc-1/rc-2); the vendored crate is reviewed only
    through its 101-line patch file; no GUI run (owner `make run`)
  - not_independently_verified: slot rendering and rail composition in a
    live window
  - receipt: context_sources: plan phase block (`vendor-patch`) +
    registry `gpui-component 0.6.1` source + `sidebar/mod.rs`
    header/footer slot pattern / policy: work-full step 11 + check.md
    durable gate / judge: same-session / judge_model: SWE-2 Max /
    retries: 0 / rollback_point: `git checkout -- Cargo.toml Cargo.lock`
    + `git clean -fd vendor` / failure_ledger: absent / enforcement:
    local-only (no pre-commit hook installed)

- 2026-09-19 07:11 +07 — phase `rail-chrome` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard)
  - `cargo test` → pass (2 tests:
    `theme::tests::embedded_theme_set_has_required_colors`,
    `assets::tests::brand_mark_loads_and_icons_still_delegate`)
  - `make check` → pass (`cargo check`)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `make build` → pass (`cargo build`)
  - `cargo fmt --check` → pass
  - `shasum -a 256 assets/brand/brand-mark.svg` → `c0518c5a550288dc3b89173d0cf3ea26cc90673bd8827cb77e538eaf56bae8dd`, identical to the kumo-desk source file
  - `git status --short` → scoped: `assets/brand/`, `src/assets.rs`,
    `src/main.rs` (+ `vendor/`, `Cargo.{toml,lock}` from `vendor-patch`);
    untracked `.agents/`, `CLAUDE.md` are not this initiative
  - proof gaps: no headless test covers `svg()` path resolution at render
    time, brand-block layout, or the toggle's repaint; requirement 12's
    repaint path is verified by reading `Theme::change` (window refresh)
    plus the owner GUI check
  - not_independently_verified: rendered rail — mark gradient at 32px,
    italic wordmark, toggle repaint, icon drawing (owner `make run`)
  - receipt: context_sources: plan phase block (`rail-chrome`) +
    gpui-base `styled.rs` (`h_flex`, `text_xl`) + gpui-component
    `button/button.rs` (`ButtonVariants`) + kumo-desk `AppShell.tsx`
    sidebar footer / policy: work-full step 11 + check.md durable gate /
    judge: same-session / judge_model: SWE-2 Max / retries: 1
    (`ButtonVariants` import) / rollback_point:
    `git checkout -- src/main.rs` + `git clean -fd assets/brand
    src/assets.rs` / failure_ledger: absent / enforcement: local-only
    (no pre-commit hook installed)

- 2026-09-19 07:12 +07 — phase `spec-sync` — verdict: APPROVED
  (judge: same-session; judge_model: SWE-2 Max; scope: on target;
  depth: standard; final phase → left `in-progress`, routed to an
  independent `check full` for the complete review)
  - `cargo test` → pass (2 tests:
    `theme::tests::embedded_theme_set_has_required_colors`,
    `assets::tests::brand_mark_loads_and_icons_still_delegate`)
  - `make check` → pass (`cargo check`)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass
  - `grep -n "sidebar_header\|sidebar_footer" src/main.rs` → `346: .sidebar_header(rail_brand())`, `347: .sidebar_footer(rail_footer(cx))`
  - `grep -n "patch.crates-io" -A 2 Cargo.toml` → `14: [patch.crates-io]`, `15: gpui-component = { path = "vendor/gpui-component" }`
  - `ls -l assets/brand/brand-mark.svg` → 2417 bytes, copied verbatim
  - `grep -n "patch.crates-io\|vendor/gpui-component" docs/PROJECT.md` → hit (architecture answer names the patch)
  - `git status --short` → scoped to the initiative: `vendor/`,
    `assets/brand/`, `src/assets.rs`, `src/main.rs`, `Cargo.{toml,lock}`,
    `docs/design/`, plan; `.agents/`, `CLAUDE.md` untracked, unrelated
  - proof gaps: `check.md` step 5's complete Security / Performance /
    Architecture / Code-Quality review is deferred to `check full` — the
    initiative's required independent gate; rail rendering remains
    GUI-only
  - not_independently_verified: rendered rail, and the initiative diff as
    a whole
  - receipt: context_sources: plan phase block (`spec-sync`) +
    `docs/design/mono-port-spec.md` + `src/main.rs` + `src/assets.rs` +
    `Cargo.toml` / policy: work-full step 11 + check.md durable gate
    (steps 1-4, 6-11) / judge: same-session / judge_model: SWE-2 Max /
    retries: 0 / rollback_point: `git checkout -- docs/design
    docs/PROJECT.md` / failure_ledger: absent / enforcement: local-only
    (no pre-commit hook installed)

- 2026-09-19 07:26 +07 — phase `spec-sync` (initiative `mono-design-port`, mode: full) — verdict: APPROVED
  (judge: independent; judge_model: DeepSeek V4.1 Flash Max — read-only
  subagent reviewer, separate context, no author access; scope: on
  target; depth: standard)
  - `cargo test` → pass (2 tests:
    `theme::tests::embedded_theme_set_has_required_colors`,
    `assets::tests::brand_mark_loads_and_icons_still_delegate`)
  - `make check` → pass (`cargo check`)
  - `make lint` → pass (`cargo clippy --all-targets -- -D warnings`)
  - `cargo fmt --check` → pass
  - `make build` → pass (`cargo build`)
  - `shasum -a 256 src/main.rs src/assets.rs Cargo.toml vendor/patches/sidebar-slots.patch` → `60351907…`, `e295f812…`, `a28d4ff1…`, `eb9ff45e…` (pins the reviewed revision)
  - `git status --short` → scoped to the initiative (`vendor/`,
    `assets/brand/`, `src/assets.rs`, `src/main.rs`, `Cargo.{toml,lock}`,
    `docs/design/`, `docs/PROJECT.md`, plan); `.agents/`, `CLAUDE.md`
    untracked, unrelated
  - independent review (read-only subagent, separate context): APPROVED —
    vendored delta verified line-by-line against the pristine upstream
    crate (`src/setting/settings.rs` identical except the patch's 7 hunks;
    file inventories match minus `.cargo-ok`; `sidebar_header`/
    `sidebar_footer` appear nowhere else); `.take()` slot lifecycle
    confirmed safe under `RenderOnce` (fresh `Settings` each pass,
    `render_sidebar` called once, `container_query` never touches the
    slots); security clean (no `unsafe`, secrets, or path traversal;
    noted that `[patch.crates-io]` compiles the vendored crate from the
    tree — inventory + marker greps clean); performance negligible;
    spec-vs-code table 12/12 PASS
  - findings: none blocking; two informational nits accepted without a
    code change (a post-review edit would invalidate the review): (1)
    `AppAssets::list` prefix-matches, so an exact-path `list` can return
    the brand path twice — harmless, `list` serves directory enumeration;
    (2) requirement 12 says "shows the current mode" while the button
    shows the target mode (Sun + "Light mode" when dark) — matches the
    spec doc and conventional toggle UX
  - proof gaps: the review covered the diff, not a byte-level `diff -r` of
    all ~250 vendored files (inventory + `settings.rs` + marker greps);
    rail rendering is GUI-only
  - not_independently_verified: rendered rail — mark gradient at 32px,
    italic wordmark, search filtering, item selection, toggle repaint,
    icon drawing (owner `make run`); full byte-level vendored-tree diff
  - receipt: context_sources: plan (requirements 8–14 + `vendor-patch` /
    `rail-chrome` / `spec-sync` blocks) + `src/{main,assets}.rs` +
    `Cargo.toml` + `vendor/patches/sidebar-slots.patch` + pristine
    upstream crate + `docs/design/mono-port-spec.md` + `docs/PROJECT.md` /
    policy: check.md full (steps 1-11) + check-validation.md / judge:
    independent / judge_model: DeepSeek V4.1 Flash Max (subagent, separate
    context) / retries: 0 / rollback_point: `git checkout -- Cargo.toml
    Cargo.lock src docs` + `git clean -fd vendor assets/brand` /
    failure_ledger: absent / enforcement: local-only (no pre-commit hook
    installed)

## Current State

- active phase: none; lifecycle_status: done — all six phases `done`
- anchors: `check full` APPROVED (judge: independent — read-only subagent
  reviewer, separate context) 2026-09-19 07:26 +07; owner accepted the
  GUI-only proof gap 2026-09-22 19:47 +07; gates green (`cargo test` 2
  tests, `make check`, `make lint`, `cargo fmt --check`, `make build`)
- completed work: `mono-light`/`mono-dark` theme set + installer; three
  vendored font families; port spec; vendored gpui-component 0.6.1 with
  `sidebar-slots` + `sidebar-cursor` patches; rail chrome (brand block,
  search, item list, footer theme toggle); spec synced to applied state
- blockers: none
- open items: none — `.agents/` remains untracked and unrelated to this
  initiative
- next action: none — initiative closed; plan moved to
  `docs/plans/completed/mono-design-port.md`
