---
id: plan-kumo-design-port-20260918
intake_id: intake-20260918-kumo-design-port
status: active
lane: normal
created: 2026-09-18
updated: 2026-09-18
approach: planned
planning_status: planned
exact_next_action: work full (first phase: theme-colors)
---

# Mono design port

Reskin `mono-desk`'s existing `Settings` window with the `kumo-desk`
design language: ported palette (light + dark), vendored font stack, and
a written port spec inside this repo. No new app chrome.

## Outcome

`mono-desk` runs the ported palette in both modes and renders Hanken
Grotesk / Xanh Mono italic / Google Sans Code from embedded fonts, with
the mapping recorded in `docs/design/mono-port-spec.md`. Success when:

- `assets/themes/mono.json` exists, deserializes as a gpui `ThemeSet`,
  and is applied at startup (verified by `cargo test` + code path).
- The three font families are embedded via `include_bytes!` and
  registered; `font.family`/`mono_font.family` resolve to them.
- `docs/design/mono-port-spec.md` records every mapped token.
- `cargo test`, `make check`, `make lint` pass; `make fmt` applied.
  Owner runs `make run` for final visual sign-off (GUI-only check).

## Authority and Requirements

Sources (read-only, current on-disk state):

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

## Non-goals

- Product port: no Tasks/AI Chat/Kanban or any multi-view surface.
- App chrome: no 40px header, app-level collapsible sidebar, command
  palette, window controls.
- Source rule adoption (R2 token rule, 60% floor, CLS rules) as
  enforced mono-desk policy.
- Full 8-level typography hierarchy (families + heading italic only).
- Persistence, i18n, backend, Tauri parity.
- Any change inside `/home/tinhpt/Lab/kumo-desk` (read-only source).

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

## Phases and Verification

Order: `theme-colors` → `mono-fonts` → `patterns-spec`.
Font keys enter the JSON only in `mono-fonts` (a named-but-missing
family is used as-is and could render wrong before fonts register).

### Phase `theme-colors` — story_id: story-kumo-theme-colors-20260918

- **Status:** `checked`
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

- **Status:** `checked`
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

- **Status:** `checked`
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

## Current State

- active phase: `patterns-spec`; lifecycle_status: checked
- anchors: `check full` APPROVED (independent, SWE-2 Max) 2026-09-18;
  all three phases gated checked; Validation entries recorded
- blockers: none
- open items: owner `make run` for visual sign-off (initiative release
  check); rendered output not independently verified headless;
  `docs/PROJECT.md` mod predates initiative — separate before commit
- next action: closing `handoff` (owns `done` transition) or `git`
  commit of the scoped initiative diff
