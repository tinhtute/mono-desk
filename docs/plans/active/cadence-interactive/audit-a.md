# Cadence UI audit — `src/cadence/blocks.rs` + `src/cadence/controls.rs`

Read-only audit. No files changed.

**Caveat on the reference:** the four renders are not on this machine
(`~/quaqua/runs/ref-cadence/` does not exist; `~/quaqua` is an unrelated
agent-home dir). Judgments about reference geometry therefore lean on the
style notes in `src/cadence/mod.rs:5-34` and standard Lucide/Tailwind
readings; findings based purely on code semantics (not ref-diffing) are
marked **[definite]**, ref-fidelity calls are marked **[ref]**.

**Gate:** `make check` → PASS (`Finished dev profile in 0.79s`; the only
warning is the pre-existing `block v0.1.6` future-incompat note, unrelated).

**Setup facts used below:**
- `h_flex()` = row + `items_center`; `v_flex()` = column + default stretch
  (gpui-base `styled.rs:38-46`, and its doc warns a taller column child of a
  row is *centered* and overflows both edges).
- Taffy 0.13 resolves `position: absolute` against the **direct parent**
  regardless of its own `position` — so a missing `.relative()` is harmless,
  and `pie_avatar` children do anchor to the disc.
- Theme values: light — `accent` #e3e3e3, `border` #e9e9e9, `popover`
  #ffffff, `foreground`/`button.foreground` both #2e2e2e; dark — `accent`
  #373737, `border` #262626, `popover`/`accordion` #212121,
  `foreground`/`button.foreground` both #ebebeb (`assets/themes/mono.json`).
- `gpui_kit::component::IconName` is the **101-icon legacy subset**
  (`gpui-kit-assets/default-icons.txt`), not the full Lucide catalog
  (`gpui_kit_assets::IconName`, ~2800 variants). All variants used in-tree
  exist in the subset — nothing is *missing*, but see F10.

---

## High

### F1. `pie_avatar` draws a vertical stripe, not a pie wedge — broken "logo" [definite]
`blocks.rs:544-553` — the "wedge" is `div().absolute().left_0().top_0().bottom_0().w(relative(frac))`,
i.e. a solid rectangle clipped by `rounded_full().overflow_hidden()` on the
disc. At `frac < ~0.35` the clip rounds the stripe's right edge so it reads
as a squashed pill, not a radial sector. These are the supplier/project
"avatars" (showcase.rs `pie_avatar(...)` calls at ~:639 and ~:1261); the
squashed-pill look is a plausible source of the reported "misaligned logos".
**Fix:** render a real sector — `Icon::default().data(...)` or an inline
`svg()` child with an `<path>` wedge (`M cx cy L cx 0 A r r 0 f 1 … Z`)
computed from `frac`. GPUI `div` has no conic-gradient, so an SVG child is
the only faithful option. Cheap approximation if SVG is off the table:
a 3-4px `rounded_full` *ring* (border-only div) reads as the same family of
mark without pretending to be a pie.

### F2. CardHeader trailing slot is top-aligned while the icon tile is centered — diagonal icon/badge drift [definite]
`blocks.rs:87-93` — outer `h_flex().items_start()` pins `trailing`
(MonoPill, legend, close button) to the top of the header row, while the
inner `h_flex()` (line 93) centers the 36px `icon_tile` against the
title+subtitle column (~44-48px tall). Result: the pill's top edge sits at
the row top but the tile's top edge is ~4-6px lower — pill and icon read
misaligned, exactly the class of defect reported. Also mis-centers the
28px bordered close button against the tile.
**Fix:** drop `.items_start()` on the outer row (h_flex already centers) —
the left group and trailing slot then share one center line. If a
top-anchored close button is ever wanted, do it per-child with
`.self_start()`, not globally.

### F3. Active-tab underline never overlaps the hairline → doubled 1px+2px rule; wrapped rows lose the baseline entirely [definite]
`controls.rs:77-78,119-125` — the 2px accent bar (`h(px(2.)).w_full()
.mt_1()`, always rendered, tinted only when active) ends exactly where the
row's 1px `border_b_1` begins. Content box and border box are disjoint, so
the inactive bar is invisible but the hairline paints *below* the accent
bar: the intended "2px accent replaces the 1px hairline" reads as a thin
double line, and `rounded_full` on the bar makes the hairline peek past its
ends. Separately, `.flex_wrap()` (line 77, deliberate for Font-Size ≥ ~20px
per the comment) applies `border_b_1` to the container's *border box* — the
hairline only sits under the last wrapped line, so wrapped tabs lose their
underline reference and the accent bar floats mid-card.
**Fix:** give every tab its own baseline — in the tab's `v_flex`, paint the
bar `bg(border)` always and `bg(Accent::solid)` + `h(px(2.)).mb(px(-1.))`
when active, and remove the container `border_b_1` (or keep it as the
unwrapped fast-path only). That makes wrapped rows look correct too.

### F4. Segmented control has squared corners; the reference is a pill [ref]
`controls.rs:191,209` — container `rounded(px(10.))`, active segment
`rounded(px(8.))`. The reference's segmented control ("Your own · To review
2 · To pay 3") is a fully-rounded capsule — consistent with the module's own
doc: "pill segmented control" (controls.rs:153) — and with MonoPill's
`rounded_full`. 10px radius on a ~36px-tall track reads as a rounded
rectangle, visibly squarer than the mock.
**Fix:** `rounded_full()` on the container and each segment.

### F5. Fixed `px()` control heights vs rem-scaled text — layout drift under the Font Size setting [definite]
`blocks.rs:223` (FilterChip `h(px(34.))`), `controls.rs:208` (segment item
`h(px(30.))`), plus fixed boxes at `blocks.rs:34` (icon tile 36px), :457
(checkbox 18px), :509-510 (toggle 40×22). Text sizes are `rem`-based
(`text_sm` = 0.875rem etc., gpui-pre styled.rs:545+) and `theme.font_size`
is user-adjustable — a fact the codebase already honors, cf. the `flex_wrap`
comment at controls.rs:73-76. At large font sizes the fixed-height chip and
segment clip or crowd their labels, and the fixed-size icons/checkboxes end
up proportionally misaligned beside the grown text.
**Fix:** size text-bearing controls in rems — e.g. FilterChip
`h(rems(34./14.))`/`h_9()`-equivalent, segment `h(rems(30./14.))`, or drop
fixed heights in favor of `py_1p5`/`py_2` so padding scales the box with the
label. Icon-only geometry (checkbox, toggle, tile) can stay `px` but should
be checked at max font size.

---

## Medium

### F6. `stat_row` divider is a fixed 36px stub floating mid-column [definite]
`blocks.rs:336-340` — `w(px(1.)).self_center().h(px(36.))` between stats.
The stat block beside it is ~64px (14px label + ~22px `text_xl` value +
14px sub, plus `gap_1`s), so the hairline covers only the middle — visibly
shorter than the reference's near-full-height separators and top/bottom
"ragged" vs the stat text.
**Fix:** `.self_stretch().h_full()` (cross-axis stretch on the row child) —
or `h(px(52.))` if a partial-height rule is intended.

### F7. FilterChip values aren't mono — breaks the "every numeral is monospace" rule [definite vs module policy]
`blocks.rs:242` — `div().text_sm().child(self.text)` renders the value in
the UI grotesk. The module contract (mod.rs:19-22) puts *every numeral* in
the mono family, and the chip's values are date ranges / counts
("Sep 1 – Sep 30", "Sep 2026" at showcase.rs:207,808). The muted prefix
(blocks.rs:236-242) correctly stays proportional — only the value should
switch.
**Fix:** `.child(mono(self.text, cx).text_sm())`.

### F8. `btn_primary` hover/active states are invisible [definite]
`blocks.rs:563-564` — `ButtonCustomVariant.hover/active` take a *background*
color (vendored `button.rs:121-131`), but they're fed
`cx.theme().button_foreground` — which equals `foreground` (#2e2e2e light /
#ebebeb dark), identical to the base `.color(foreground)`. Result: black /
near-black button whose hover and pressed states change nothing — controls
look dead; the reference's buttons lighten on hover.
**Fix:** `.hover(cx.theme().foreground.alpha(0.82))` /
`.active(cx.theme().foreground.alpha(0.75))` (or `secondary_foreground`-lite).
Rename is not needed — just pass a background-role color.

### F9. Toggle thumb nearly invisible in dark mode [definite]
`blocks.rs:526-530` — off-state thumb is `bg(cx.theme().popover)` (#212121
dark) on a `border`-colored track (#262626): ~4% lightness delta, white-on-
dark shadow can't save it; the thumb reads as a faint smudge. Light mode is
fine (white on #e9e9e9 is also low-contrast but borderline-acceptable).
**Fix:** thumb `bg(cx.theme().accent_foreground)` (#ebebeb dark / #124b68
light — hmm, light would go dark-blue; better: `bg(if mode.is_dark() {
accent_foreground } else { popover })`, or simply a fixed `rgb(0xffffff)`
thumb on both modes since the off track is dark enough in both).

---

## Low

### F10. Icon props typed to the 101-icon legacy `IconName` → forced substitutions [definite mechanism; ref-visible substitutions]
`blocks.rs:33` (`icon_tile(icon: IconName)`), :58 (`CardHeader.icon`), :179
(`FilterChip.icon`) all take `gpui_kit::component::IconName` — the
101-entry subset. The full `gpui_kit_assets::IconName` catalog exists and is
already re-exported by the same crate, but these signatures can't accept it.
Consequences visible in showcase: `IconName::Map` stands in for
plane/receipt/car glyphs (showcase.rs:774,780,786 — flights, client dinner,
mileage), `LayoutDashboard` is reused for three unrelated cards
(showcase.rs:31,670,973 — page icon, "Owed to suppliers", "By category"),
`FileText` serves both "Purchase orders" and "Expense claims"
(showcase.rs:176,797). Generic duplicates are why "logos" look wrong —
`Map` on a flights row is a substitution, not a faithful glyph.
**Fix:** genericize the props — `icon: impl IconNamed` (the trait both enums
implement; `Icon::new` already takes `impl Into<Icon>` and `From<T: IconNamed>`
exists, vendored icon.rs:155-159). Then use real glyphs at the call sites
(`Plane`, `CarFront`, `Receipt`, `Wallet`, `HandCoins`… all in the full
catalog). Call-site icon swaps themselves are outside this audit's scope.

### F11. Tab count baseline/size drift + doubled gap before the underline [low-definite]
`controls.rs:92,95,99,108-111,123` — `gap_1()` *and* `.mt_1()` stack to 8px
between label row and the 2px bar (the reference reads ~4px / `gap_1` alone
plus the border-adjacent bar). The count is `text_sm` against a `text_base`
label, center-aligned via the inner `h_flex` — so the smaller count's visual
center sits slightly below the label's; combining smaller size + centering
reads as a subscript rather than a peer.
**Fix:** drop `.mt_1()` (keep `gap_1`); render counts `text_sm` but
`items_baseline()` on the label row — or match the reference exactly with
count `text_sm` baseline-aligned.

### F12. EmptyState illustration double-insets vs the card padding [definite]
`blocks.rs:614-617` (`px_4`/`pb_4` on the column) + `showcase.rs:1071`
(`.mx_4()` on the illustration) → the well is inset 32px each side while
title/body sit at 16px. The reference's recessed illustration spans the same
inner width as the text (its own ~12px well radius, not extra inset).
**Fix:** remove `.mx_4()` at the call site (or the `px_4` on the
illustration child); keep a single 16px inset.

### F13. MonoPill reads large vs reference tags [ref]
`blocks.rs:163-168` — `px_2` (8px sides) + `py(px(2.))` + `text_sm` (14px
mono → ~20px tall). Reference pills ("2 need you", "Pending") are ~18-20px
tall with ~11-12px text. Slightly oversized next to the hairline chips.
**Fix:** `text_xs` + keep `py(px(2.))` → ~18px; optionally `px_2p5` for the
solid "Today" marker.

### F14. Two icon-tile specs — header tiles 36/10/18, claim rows hand-rolled 32/8/16 [definite]
`blocks.rs:33-43` vs `showcase.rs:910-916` — the claim-row tile duplicates
`icon_tile` with different numbers (32px box, 8px radius, 16px icon, and no
`flex_shrink_0`). Divergent "logo" boxes in the same canvas; also the row
tile can shrink under width pressure since it lacks `flex_shrink_0`.
**Fix:** parametrize `icon_tile` — `icon_tile(icon, cx)` stays 36px; add
`icon_tile_sized(icon, px(32.), px(16.), px(8.), cx)` (or a builder) and use
it in the claim row.

### F15. FilterChip's element `id` lands on the tiny × button, not the chip [low-definite]
`blocks.rs:222-263` — the chip container takes no `.id()`; `self.id` is
assigned to the remove-× div (line 253). Any caller expecting the id on the
chip (hit-testing, inspector, future chip-level click) gets a 16px button.
Also the × (12px icon in 2px padding = 16px target) is smaller than the
14px leading icon/caret — visually weaker trailing glyph than the mock's.
**Fix:** put `.id(self.id)` on the outer chip and give the remove control a
derived id (`{id}-remove`), or make the whole trailing area clickable;
bump the × to `p_1p5`/`Size(px(14.))` for balance.

### F16. Header padding asymmetry crowds the hairline [ref]
`blocks.rs:91` — `pt_4` (16px) over `pb_2` (8px). The reference header reads
as 16px-ish all around before the tab row's border; 8px under the subtitle
makes the subtitle text sit closer to the hairline than the title does to
the card top.
**Fix:** `pb_3` (12px) or `pb_4`; verify against the tab border.

---

## Cross-checks that came back clean (no action)
- `icon_tile`, `CadenceCheckbox`, `close_button` centering: plain `div()`
  defaults to display:flex in GPUI/Taffy, so `items_center/justify_center`
  work as written.
- `pie_avatar` missing `.relative()`: harmless — Taffy anchors absolute
  children to the direct parent (verified in taffy-0.13.0 source).
- `Icon` inherits `text_color` from the ancestor text style — all tinted
  icons (chip, tile, close, caret) color correctly.
- Every `IconName` variant referenced in showcase + blocks + controls
  exists in the 101-icon subset — nothing unresolved at build time.
- `Meter` absolute fill, `legend_dot/swatch` (8px, items_center, correct
  square-vs-round), checkbox `Icon::Check` 12px in 18px box — all sound.
