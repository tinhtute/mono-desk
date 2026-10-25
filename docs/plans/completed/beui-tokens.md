---
id: plan-beui-tokens-20260930
intake_id: intake-20260930-beui-tokens
status: completed
lane: normal
created: 2026-09-30
updated: 2026-10-25
approach: planned
planning_status: planned
exact_next_action: "none — initiative closed 2026-10-25"
---

# beUI token layer (spacing / typography / radius / overlay shadow)

Decide, document, and pin the app's non-color token layer on the
`143e7b6` baseline, and add the app-owned elevation snapshot the audit
requires: resolved from the live theme with
`Theme::resolve_semantic_config` (never the apply API), consumed by the
app-owned raised surface, with `theme.shadow` staying `false`. Colors,
`font.size` 14, and the vendored patch set are frozen.

## Outcome

- `assets/themes/mono.json` is unchanged: both modes keep `font.size` 14,
  `mono_font.size` 12, `radius` 8, `radius.lg` 16, `shadow` false, and
  every color byte-identical to `143e7b6`.
- `docs/design/zdesk-port-spec.md` states the card's real radius
  (`.rounded_lg()` = 0.5rem = **7px** at rem 14 — a rem helper, not
  `theme.radius_lg`) and records the token decisions: the rem coupling,
  the radius roles, the lg/xl collision, and shadow ownership.
- `src/tokens.rs` holds `UiTokens` — the app's elevation scale
  (`sm`/`md`/`lg`, 10% black ink, Tailwind radii halved) resolved from the
  live theme via `Theme::resolve_semantic_config`;
  `theme::apply_overrides` re-resolves it after every theme rebuild; the
  active sidebar pill consumes `sm`; `theme.shadow` stays `false`.
- Tests pin: both modes' palettes (digest), the config values, the derived
  Radius/Spacing/Typography tokens, the effective rem values, the
  default/reset agreement, and the resolve behavior (both modes, no
  mutation, no color leakage).
- Gates: `make check`, `make test`, `make lint`, `make fmt-check` green;
  owner `make run` visual pass for the pill shadow and density.

Success when every pin fails on an unintended change and the pill renders
the `sm` elevation in both modes with `theme.shadow == false`.

## Authority and Requirements

**Baseline:** `143e7b6` (`quaqua/mdinput` with `origin/main` merged in,
merged into `quaqua/tieude-beuitokens`; owner-approved). Both this plan
and the components plan share it. Its colors are identical to `9fb40ec`;
the only non-color delta is `font.size` 13→14, `radius` 6→8, `radius.lg`
10→16.

Sources:

- `~/quaqua/runs/bespace.md`, `~/quaqua/runs/becomp.md` — beUI research.
- `~/quaqua/runs/quansu-beuiaudit.md` — audit; **its corrections override
  the reports**, and this plan resolves its "Required before the tokens
  plan" items 1–6.
- `docs/design/zdesk-port-spec.md` — applied port state.
- Vendored `gpui-component` 0.6.1 (`theme/{mod,schema}.rs`, `styled.rs`,
  `sidebar/menu.rs`, `setting/settings.rs`), `gpui-base` 0.6.1
  (`theme_tokens.rs`), `gpui-pre` 0.3.5 (`style.rs`, `styled.rs`).

Owner constraints (locked): all current colors; `font.size` 14 stays;
`theme.shadow` stays `false`; prefer app-side composition over vendor
patches.

Requirements:

1. **Baseline and font authority (audit 1–2).** Baseline `143e7b6`;
   `font.size` 14 retained in both modes; no 16px rem base. The rem
   coupling is documented and pinned: `Root::render` sets the window rem
   to `theme.font_size`, so every rem helper renders at 14/16 of its
   Tailwind nominal and the Font Size setting (8–32) scales rem-derived
   layout — accepted behavior, not a defect.
2. **Radius roles (audit 3).** `radius` 8 = controls (inputs, sidebar
   items, buttons, tooltip); `radius.lg` 16 = large surfaces (dialogs,
   notifications). The card's radius is decided (O2) and the spec's 16px
   claim corrected. The lg/xl collision is decided (O3) and pinned.
   Legacy px tokens versus rem helpers are specified per use site.
3. **Overlay shadow (audit 4).** App-owned resolved values via
   `Theme::resolve_semantic_config`; `apply_semantic_config*` is never
   called; `theme.shadow` stays `false`; ink is 10% black (the vendored
   scale); the switch-thumb exception is decided; the app's overlays are
   named.
4. **Snapshot use and reapply (audit 5).** The snapshot carries only
   non-color values; its consumers are listed; it is re-resolved in
   `theme::apply_overrides` — the existing reapply point after every
   `Theme::change` and every font-size change/reset — so a mode switch
   cannot restore stale colors and the token path cannot overwrite the
   user's font preference.
5. **Executable acceptance (audit 6).** Both modes' colors unchanged;
   agreed font/radius/rem values; default/reset agreement;
   theme-switch/font-preference behavior.
6. **No vendor patches, no new runtime dependencies, no color changes.**

## Non-goals

- Any color change (the palette is frozen; the digest pin enforces it).
- Any font change (14 stays; no 16px; no family change).
- `theme.shadow = true` (the legacy flag gates surface-control shadows —
  not beUI, and not this app).
- Spacing/typography snapshots: the vendored components never read
  `.spacing`/`.typography`, and the app uses rem helpers plus the legacy
  fields, so a snapshot would have no consumer.
- Component work (input-field validation fix, checkbox/radio/select
  ports, motion) — that is the components plan.
- Vendor patches (switch-thumb shadow, dialog shadow hook, pill shadow
  hook).
- New app surfaces or overlays.
- `docs/design/mono-port-spec.md` (the superseded kumo-era spec) is not
  updated here.

## Approach and Risks

### Decisions (audit items 1–6)

**D1 — Baseline.** `143e7b6`; both plans share it. Colors identical to
`9fb40ec`; non-color delta `font.size` 14, `radius` 8, `radius.lg` 16,
`shadow` false.

**D2 — Font authority.** 14px retained (owner constraint). The rem
coupling is accepted and documented: at rem 14, `gap_2` = 7px,
`text_sm` = 12.25px, `rounded_lg` = 7px, `h_8` = 28px; the Font Size
setting scales those. No 16px. Pinned by test.

**D3 — Radius roles.**

| Role | Token | Value | Consumers |
|---|---|---|---|
| control | `theme.radius` | 8px | inputs, sidebar items, buttons, tooltip (`tooltip.rs:120`) |
| large surface | `theme.radius_lg` | 16px | dialogs (`dialog/dialog.rs:580`), notifications (`notification.rs:425`) |
| pill | `rounded_full()` helper (app) / `theme.radius_full()` (vendored) | 9999 | status chip (`src/status_chip.rs:249`), vendored badges |
| card | `.rounded_lg()` rem helper | 7px @ rem 14 | the content card (`src/main.rs`) |

Derived `RadiusTokens` = none 0 · sm 4 · md 8 · lg 16 · **xl 16** · full
9999. The card decision is O2; the collision decision is O3.

**D4 — Overlay shadow ownership.**

- Mechanism: `Theme::resolve_semantic_config(&SemanticThemeConfig)` —
  non-mutating. `apply_semantic_config`/`_str` are never called: they set
  `theme.shadow = true` whenever a tier is non-empty and write legacy
  color fields.
- Values: the vendored elevation scale (shadcn/Tailwind, blur halved for
  GPUI's sigma, 10% black ink — `SURFACE_SHADOW_INK`): `sm` =
  (0,1,1.5,0)+(0,1,1,−1); `md` = (0,4,3,−1)+(0,2,2,−2); `lg` =
  (0,10,7.5,−3)+(0,4,3,−4).
- Overlays: the app's overlays are the vendored Dialog, Sheet,
  Notification, Popover and Tooltip; they paint their own hardcoded
  elevations (the same scale) and are not restyled. The app-owned raised
  surface is the active sidebar pill → `sm` (O1).
- Switch-thumb exception: beUI gives the switch thumb `shadow-md`; the
  vendored `Switch` has none. **Not adopted** — controls stay flat, and a
  thumb shadow would need a vendor patch.
- Ink: 10% black in both modes, matching the vendored overlays (the
  popover *ring* follows the foreground; the shadow ink does not).

**D5 — Snapshot use and reapply.** `UiTokens { shadow: ShadowTokens }` —
non-color, mode-independent; stored as a `Global`; resolved from the live
theme; re-resolved in `theme::apply_overrides` (reached via
`apply_font_size`, so it runs after every `Theme::change`, after every
font-size change, and at install — the preference itself is persisted in
`src/persist.rs`). Consumers: the active pill (`sm`). No
spacing/typography/radius snapshot: those are live projections
(`radius_tokens()`, `typography_tokens()`) or defaults
(`spacing_tokens()`), and nothing in the app or the vendored components
reads a stored copy.

**D6 — Executable acceptance.** See `## Validation` and the phase checks.

### Rejected alternatives

- `apply_semantic_config_str` plus a stored snapshot (the research's
  proposal): flips `theme.shadow` and writes legacy colors — the audit's
  finding 2.
- `font.size` 16 for full Tailwind parity: rejected by the owner; it
  would grow every rem dimension by 16/14 ≈ 1.143.
- A spacing/typography snapshot: no consumer (the vendored components
  read only `.colors` and `.radius`).
- An explicit `radius.xl` = 24: nothing in the app reads `radius.xl`.
- Vendor patches for the switch thumb / dialog shadow / pill shadow hook:
  the vendored overlays already carry the same elevations, and the pill
  is expressible app-side (`StyleRefinement` carries `box_shadow` and
  `refine_style` applies it).
- A test-backend suite for the token layer: the snapshot is
  mode-independent and non-color, so the observable behavior is covered
  window-free; the test backend is the components plan's requirement.

### Risks and mitigations

- **Palette digest pin** — an intentional color change must update the
  digest; the test prints the canonical JSON on mismatch, and the phase
  verify diffs `mono.json` against `143e7b6`.
- **Pill shadow clipping or reading as grime** — the elevation is the
  vendored `raised_shadow` scale (tight falloff); the item's inner flex
  clips only x, the sidebar clips at its bounds. Owner visual pass; if it
  reads wrong, O1 falls back to dropping the pill shadow (and the
  elevation phase with it).
- **`theme.shadow` flip** — the resolve path is non-mutating, and the
  tests assert `theme.shadow == false` and unchanged colors after
  resolve.
- **Snapshot read before install** — `install` → `apply_overrides` →
  `UiTokens::sync` runs before any window renders; the pill reads the
  global in render only.
- **lg/xl collision** — no app consumer; pinned so a future radius change
  is caught; revisit if `Attachment` (the only `radius.xl` reader) lands.
- **Card radius (O2)** — the plan keeps the code as-is (7px) unless the
  owner decides otherwise; the doc fix is independent of the decision.
- **`md`/`lg` tiers have no app consumer today** — they are the
  framework's `ShadowTokens` type (the scale the vendored overlays use);
  no speculative consumers are added.

## Phases and Verification

Order: `beui-token-baseline` → `beui-elevation`. Phase 1 pins the values
phase 2 derives from; phase 2 adds the snapshot and its consumer.

### Phase `beui-token-baseline` — story_id: story-beui-token-baseline-20260930

- **Status:** `done` (closed on the baseline gate; final independent `full` review 2026-10-25 re-checked its pins)
- **Goal:** the token layer's agreed values are documented and pinned; the
  spec's card-radius error is corrected; no behavior change.
- **Depends on:** none.
- **Allowed surfaces:** `docs/design/zdesk-port-spec.md`,
  `src/theme.rs` (tests only).
- **Avoided:** `assets/themes/mono.json`, `src/main.rs`, `vendor/`.
- **Traces to:** requirements 1, 2, 5.

**Wave 1** (parallel — disjoint files):

1. `bt-1` — `docs/design/zdesk-port-spec.md`:
   - Fix the Content card row: `.rounded_lg()` is the GPUI style helper
     (`rems(0.5)`), **7px at rem 14** — not `theme.radius_lg`/16px.
   - Refresh the Shell metrics table's `src/main.rs` refs, which the
     `origin/main` merge shifted: card block `src/main.rs:680-688`,
     `active_item_style` 670-676, `sidebar_style` 663-668,
     `traffic_light_position` 809.
   - Extend the existing "beUI pilot (2026-09-30)" paragraph (it already
     records the retune and the 14/16 rem coupling) with a "beUI token
     layer" section: the radius role table (D3) including the derived
     `RadiusTokens` and the lg/xl collision; the shadow policy (D4):
     `theme.shadow` false, the app-owned elevation scale, the vendored
     overlays owning theirs, the switch-thumb exception not adopted, 10%
     black ink.
   - Check: `make fmt-check`; the doc's claims match the code
     (`src/main.rs` card, `src/theme.rs`, `assets/themes/mono.json`).
2. `bt-2` — `src/theme.rs` tests (build the theme like the existing
   `mono_theme(mode)` helper in `src/input_field.rs` tests:
   `Theme::default()` + `apply_config(&Rc::new(config))`):
   - `theme_token_values_are_pinned`: both modes' config values
     (`font.size` 14, `mono_font.size` 12, `radius` 8, `radius.lg` 16,
     `shadow` false); the derived tokens: `RadiusTokens` 0/4/8/16/16/9999,
     `SpacingTokens` 2/4/8/12/16/24/32, `TypographyTokens` xs 12/16 ·
     sm 14/20 · md 14/24 · lg 18/28 · xl 20/28 · mono_md 12/20, and
     `ShadowTokens` empty.
   - `rem_derived_values_match_the_approved_base`: with
     `write_overrides(&mut theme, 14.0)`, `theme.font_size == px(14.)`
     and `rems(0.5).to_pixels(theme.font_size) == px(7.)` (card),
     `rems(0.875)` = 12.25px (`text_sm`), `rems(2.)` = 28px (`h_8`).
   - `default_and_reset_agree_with_the_config`: `AppSettings::default()
     .font_size` is exactly 14, equals both modes' `font.size`, and sits
     inside `FONT_SIZE_MIN..=FONT_SIZE_MAX` (the reset closures capture
     the same default).
   - `mono_palettes_are_pinned`: canonicalize each mode's `colors` object
     (collect into a `BTreeMap` and serialize compactly) and pin its
     FNV-1a 64 digest per mode; print the canonical JSON on mismatch.
   - Check: `make test`; `make lint`; `make fmt-check`.

**Phase checks:** `make check && make test && make lint && make fmt-check`;
`git diff 143e7b6 -- assets/themes/mono.json` is empty (no color or token
change); the spec's card row reads 7px.

**Stop:** a pinned value disagrees with the config (the pin and the
config must be reconciled before phase 2) → halt, report.

### Phase `beui-elevation` — story_id: story-beui-elevation-20260930

- **Status:** `done` (2026-10-25 — closed on the independent `full` verdict below; owner visual accepted 2026-10-25)
- **Goal:** the app's elevation scale is resolved (non-mutating) into an
  app-owned snapshot and consumed by the app-owned raised surface;
  `theme.shadow` stays false.
- **Depends on:** `beui-token-baseline` (the pins guard the values the
  snapshot derives from).
- **Allowed surfaces:** `src/tokens.rs` (new), `src/theme.rs` (the
  `apply_overrides` hook), `src/main.rs` (`mod tokens;` + the pill),
  `docs/design/zdesk-port-spec.md`.
- **Avoided:** `assets/themes/mono.json`, `vendor/`, `Cargo.toml`.
- **Traces to:** requirements 3, 4, 5.

**Wave 1** (parallel — disjoint files; `bt-4` codes against the `bt-3`
contract below):

Contract (fixed before the wave):

```rust
// src/tokens.rs
pub struct UiTokens { pub shadow: ShadowTokens }
impl Global for UiTokens {}
impl UiTokens {
    pub fn resolve(theme: &Theme) -> Self; // non-mutating
    pub fn sync(cx: &mut App);             // set_global(resolve(Theme::global(cx)))
}
```

1. `bt-3` — `src/tokens.rs` (new) + `src/theme.rs`:
   - `elevation_config() -> SemanticThemeConfig` with only `shadow.sm/md/lg`
     set (D4 values, `hsla(0., 0., 0., 0.10)` ink, `box_shadow`).
   - `UiTokens::resolve` = `theme.resolve_semantic_config(&config()).shadow`
     (store only the shadow; never colors).
   - `UiTokens::sync` = `cx.set_global(UiTokens::resolve(Theme::global(cx)))`.
   - `theme::apply_overrides` gains `UiTokens::sync(cx);` after
     `write_overrides` + `Theme::sync_base`.
   - Tests: `elevation_config_carries_no_colors` (every
     `SemanticColorConfig` field is `None`);
     `resolve_matches_the_vendored_elevation_scale` (both modes → the D4
     values); `resolve_is_mode_independent_and_non_mutating` (light and
     dark snapshots are equal; `theme.shadow == false` and the theme's
     colors are unchanged after resolve);
     `font_overrides_leave_the_snapshot_unchanged`
     (`write_overrides(&mut theme, 16.0)` → `font_size` 16, snapshot
     equal; back to 14).
   - Check: `make test`; `make lint`; `make fmt-check`.
2. `bt-4` — `src/main.rs`: `mod tokens;` and the active pill's
   `active_item_style` gains
   `.shadow(cx.global::<UiTokens>().shadow.sm.clone())` (O1;
   `StyleRefinement` carries `box_shadow` and `refine_style` applies it).
   - Check: `make check` (compiles against the contract); `make lint`.
3. `bt-5` — `docs/design/zdesk-port-spec.md`: the shadow policy section
   (D4) — the app-owned scale, the vendored overlays, the switch-thumb
   exception, the ink, and the pill consumer.
   - Check: `make fmt-check`; the doc matches `src/tokens.rs`.

**Phase checks:** `make check && make test && make lint && make fmt-check`;
owner `make run` visual pass: the pill's `sm` elevation in both modes, no
clipping, no grime; density, long/localized strings, and focus visibility
unchanged (no sizes or colors move); `theme.shadow` false.

**Stop:** the pill shadow cannot be expressed app-side (clipped or
overridden by the built-in active style) after two attempts → drop `bt-4`
and report; the elevation snapshot then has no consumer and the phase is
re-planned.

## Progress

- 2026-09-30 — plan authored from the beUI research reports plus the
  `quansu-beuiaudit` corrections, on the `77af8b5` baseline (ff-merged
  `quaqua/mdinput`). Resolves the audit's "Required before the tokens
  plan" items 1–6 as D1–D6; open decisions O1–O3 recorded below.
- 2026-09-30 — baseline advanced to `143e7b6` (`origin/main` `e494a92`
  merged into `quaqua/mdinput`, then into this branch). Re-checked every
  ref against the merged tree: tokens stay 14/8/16 (`origin/main` never
  touched `mono.json`), the Font Size range is now 8–32, the pill moved
  to `src/status_chip.rs:249`, the spec's `src/main.rs` refs shifted, and
  the font preference now lives in `src/persist.rs`.
- 2026-10-01 — phase `beui-token-baseline` implemented (wave bt-1 ∥
  bt-2): the spec's card radius corrected to the 7px rem helper, the
  Shell metrics refs refreshed, the beUI token-layer section added
  (radius roles + shadow policy); `src/theme.rs` pins the config values,
  the derived Radius/Spacing/Typography tokens, the rem-derived values,
  the default/reset agreement, and both palettes' FNV-1a 64 digests.
  Gates green: `make check && make test && make lint && make fmt-check`;
  `git diff 143e7b6 -- assets/themes/mono.json` empty.
- 2026-10-01 — phase `beui-elevation` implemented (wave bt-3 ∥ bt-4 ∥
  bt-5): `src/tokens.rs` resolves the app-owned `sm`/`md`/`lg` scale
  (10% black ink) via `Theme::resolve_semantic_config`;
  `theme::apply_overrides` re-resolves it after every rebuild;
  the active pill carries `sm`; the spec's shadow policy section records
  the implementation. Gates green: `make check && make test && make lint
  && make fmt-check`. Owner `make run` visual pass (pill elevation in
  both modes, no clipping/grime) still pending.

## Decisions

- 2026-09-30 — baseline `143e7b6` (owner-approved merge); colors frozen;
  `font.size` 14 retained; `theme.shadow` false; app-side composition
  over vendor patches (owner constraints).
- 2026-10-01 — **O1 decided (owner): adopt `sm` on the active sidebar
  pill** (`~/quaqua/runs/beui-decisions.md`). Implemented in
  `src/tokens.rs` + `src/main.rs` (`active_item_style`); closes the
  recorded zdesk `shadow-raised` deviation and gives the resolved scale
  its consumer.
- 2026-10-01 — **O2 decided (owner): keep the 7px rem card radius** (no
  code change). `.rounded_lg()` (`rems(0.5)`) stays; the spec's 16px
  claim is corrected.
- 2026-10-01 — **O3 decided (owner): accept the lg/xl collision** and pin
  the derived values (`theme::tests::theme_token_values_are_pinned`).

## Validation

Per phase gate: `make check && make test && make lint && make fmt-check`.
Phase 1 additionally: `git diff 143e7b6 -- assets/themes/mono.json` empty.
Phase 2 additionally: owner `make run` visual pass (pill shadow, both
modes, no clipping). GUI-verifiable items are recorded as owner checks in
`## Progress`; no GUI is launched by the agent.

- 2026-10-25T14:38:53Z — phase `beui-elevation` — verdict: APPROVE_WITH_REQUESTS — mode: full
  - `make check` — clean, `Finished dev profile in 40.56s`, 0 warnings
  - `make lint` — clean, `cargo clippy --all-targets -- -D warnings`, finished 13.41s
  - `cargo test` — ok. 86 passed; 0 failed; 0 ignored; 0 filtered out (5.72s)
  - `make fmt-check` — clean (exit 0)
  - scope: on target — HEAD `061401e`, tree clean; `assets/themes/mono.json` last touched by `2f75822`, i.e. before this plan's baseline, so the frozen palette was never re-edited by it
  - requirements: R1 met (`mono-light`/`mono-dark` both carry `font.size` 14, `radius` 8, `radius.lg` 16 — read from the embedded JSON; rem coupling pinned by `theme_token_values_are_pinned`) | R2 met (radius roles + the corrected card radius recorded in `docs/design/zdesk-port-spec.md`; the real card helper is `.rounded_lg()` = 7px at rem 14) | R3 met (`theme.resolve_semantic_config` only, `src/tokens.rs:36`; no `apply_semantic_config*` call exists anywhere in `src/`, so `theme.shadow` cannot flip — both modes still carry `shadow: false`) | R4 met (`UiTokens::sync` is the tail of `apply_overrides`, `src/theme.rs:163`, so a rebuild or font-size change cannot leave the scale stale; the scale is mode-independent by construction, `src/tokens.rs:20,55`) | R5 met (`mono_palettes_are_pinned` recomputes a canonical FNV-1a digest over every colour, `src/theme.rs:596-637`) | R6 met (no vendor patch, no new dependency — `Cargo.toml` dependency list unchanged since the baseline)
  - rollback_point: `7121a8c`
  - requests: 1. the palette digest test would pass vacuously if the embedded `themes` array were ever empty (`src/theme.rs:624` iterates the array; an empty set asserts nothing). 2. `clamp_font_size` is NaN-transparent — verified empirically that `f64::NAN.clamp(8.0, 32.0)` returns NaN — so the clamp alone would poison `px()`; it is currently safe only because `normalize_font_size` (`src/main.rs:2142-2144`) and `from_stored` (`src/main.rs:294-298`) both gate on `is_finite()` first. Worth a defensive `is_finite` guard inside `write_overrides`.
  - not_independently_verified: the owner `make run` visual pass for the pill's `sm` elevation (both modes, no clipping, no grime). Headless session. The reviewers confirmed the scale resolves and is re-resolved correctly, but could not observe the rendered shadow.
  - judge: independent (read-only audits 2026-10-25 by separate reviewer agents: `src/theme.rs` + `src/tokens.rs` → digest is a real pin, `resolve` is genuinely non-mutating and colour-free by borrow rules, `sync` cannot go stale; `src/main.rs` → every font-size/accent/appearance path funnels through `apply_overrides`)
  - judge_model: DeepSeek V4.1 Flash Max
  - enforcement: local-only (no pre-commit hook installed; `scripts/install-git-hooks.sh` absent)
  - failure_ledger: absent (`docs/evals/failures.md` does not exist)

## Current State

- active_phase: none
- lifecycle_status: completed (2026-10-25)
- branch: `main` (`061401e`); the work merged via `quaqua/tieude-beuitok` and is on main
- completed: both phases (`beui-token-baseline`, `beui-elevation`); gates green (86 tests); final independent `full` verdict APPROVE_WITH_REQUESTS
- blockers: none
- open_items: two carried, non-blocking — the palette digest asserts nothing if the embedded theme array were emptied (`src/theme.rs:624`), and `clamp_font_size` is NaN-transparent so the safety comes from the callers' `is_finite()` gates (`src/main.rs:2142-2144`, `:294-298`) rather than from the clamp itself
- next action: none — initiative closed. The sibling `zdesk-ui-port` plan closed the same day; `beui-p1-components` and `ely-port` remain.
