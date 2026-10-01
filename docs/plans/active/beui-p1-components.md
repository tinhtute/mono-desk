---
id: plan-beui-p1-components-20260930
intake_id: intake-20260930-beui-p1-components
status: active
lane: normal
created: 2026-09-30
updated: 2026-10-01
approach: planned
planning_status: planned
exact_next_action: "work phase `beui-p1-input-sync` (p0-2/p0-3 remainder: input aria_label + Role::Alert + message-row growth, the five view-level input tests)"
---

# beUI P1 components → mono-desk

Adopt the beUI P1 control set into the settings app on the merged
`quaqua/mdinput` baseline (`143e7b6` — the `origin/main` merge
`e494a92` into `77af8b5`; `origin/main` is an ancestor, so this branch
now carries `src/i18n.rs`, `src/persist.rs`, the `TITLEBAR_H = 36`
shell and the spring sidebar). Phase 0 fixes the input pilot's
synchronization defect and adds the view-level test backend; the
remaining phases adopt or compose each control on a named settings
surface, or record it dropped.

Sources: audit `~/quaqua/runs/quansu-beuiaudit.md` (corrections override
the research reports), research `~/quaqua/runs/bespace.md` and
`~/quaqua/runs/becomp.md` — authoring-time inputs, **not present on
this machine** (`~/quaqua/runs/` does not exist), so the contracts
below are the surviving record. Hard constraints (owner): **all current
colors stay; `font.size` 14 stays; `theme.shadow` stays `false`;
app-side composition is preferred over vendor patches.**

## Outcome

The settings app's P1 controls behave like the beUI reference within
GPUI's limits, with the input pilot accepted:

- the input field's validation stays consistent through user edits,
  page reset, external writes, and page remounts — proven by view-level
  tests, not just the window-free model tests;
- switch, checkbox (boolean + mixed), radio, select, tooltip, and
  notification are adopted or composed on named settings surfaces with
  explicit a11y/keyboard/reduced-motion acceptance;
- combobox, context-menu, and tabs are dropped or owner-gated rather
  than added as catalogue demos;
- colors, font size, shadow policy, and the established sidebar
  contract are unchanged.

## Requirements

1. **Phase 0 (mandatory, decision-free):** fix audit finding 1
   (reset/external write leaves stale validation) and add the
   view-level tests of finding 5; resolve the input-related audit
   items (aria label, message-row growth, config immutability).
2. **Per-control contracts** (below): target settings surface,
   retained/composed/dropped behavior, a11y/keyboard acceptance,
   reduced-motion behavior, effort — for switch, checkbox (incl.
   mixed), radio, select, combobox, tooltip, context-menu,
   notification; tabs only if a tabs surface is approved.
3. **Audit resolutions:** items 1–7 of “Required before the P1
   components plan” are resolved in this file (see below).
4. **Constraints:** colors unchanged; `font.size` 14; `shadow: false`;
   app-side composition first; any vendor patch must be additive,
   listed in the `Cargo.toml` comment and `docs/design/zdesk-port-spec.md`,
   and reversible.
5. **Gates:** per phase `make check && make test && make lint &&
   make fmt-check`; owner `make run` visual pass per phase; no GUI
   automation in worker gates (owner-only visual pass).

## Non-goals

- Any color, token, font-size, radius, or shadow change (separate
  tokens plan; audit “Required before the tokens plan” items are out
  of scope here).
- beUI motion parity: shared-layout (`layoutId`) glides, View
  Transitions, SVG goo filters, 3D perspective, canvas shaders,
  smooth scroll — no GPUI analog, recorded dropped per control.
- New app surfaces beyond the owner-gated candidates named below
  (no tabs/command palette/statusbar/context-menu/toast demos).
- Vendor patches beyond the additive minimum; no rewrite of vendored
  components.
- GUI launch, screenshots, or synthetic input on the owner's display.

## Control contracts

Legend: **Retained** = adopted from the vendored primitive as-is ·
**Composed** = app-side work on top of a primitive · **Dropped** =
beUI behavior with no GPUI analog or no surface. Effort: S ≤ 1 day,
M 2–4 days, L ≥ 1 week.

### switch — effort S

- **Surface:** General → Dark Mode (existing `SettingField::switch`);
  Notifications → Notifications Enabled (the existing raw `Switch`
  row — the merged baseline has no Auto Update row).
- **Retained:** spring thumb (`spring_move`), controlled `checked` +
  `on_click`, disabled, sizes, tooltip support.
- **Composed:** accessible name per row — convert the Dark Mode row to
  `SettingField::render` + `Switch` with `.accessibility_label(row
  title)` and an `.on_reset` handler (the notifications-enabled row is
  the pattern); add `.accessibility_label(...)` to that raw switch.
- **Dropped:** beUI press squish and disabled shake; layout-animated
  thumb (the GPUI spring is the substitute).
- **A11y/keyboard acceptance:** role `Switch`, toggled state, label =
  row title; Tab focus + Space/Enter toggles; disabled inert (no
  toggle, no focus); click toggles once and writes the global.
- **Reduced motion:** `spring` returns the terminal value under
  `App::reduce_motion` (`gpui-base/src/motion.rs:611`); test with
  `cx.set_reduce_motion(true)` — toggle still lands, no travel.

### checkbox (boolean) — effort S

- **Surface:** General → Startup → Launch at login (existing
  `SettingField::checkbox`).
- **Retained:** spring mark, controlled `checked`, disabled, sizes.
- **Composed:** accessible name — convert to `SettingField::render` +
  `Checkbox` with `.accessibility_label(row title)` + `.on_reset`.
- **Dropped:** beUI draw-on check path animation (GPUI fades the mark).
- **A11y/keyboard acceptance:** role `Checkbox`, toggled, label; Tab +
  Space/Enter; disabled inert.
- **Reduced motion:** `spring_control` honors `reduce_motion`; same
  test shape as switch.

### checkbox (mixed) — effort M — owner-gated surface

- **Surface:** none exists. Candidate: Notifications → new “Channels”
  group (Email / Desktop / Digest) with a parent “All channels”
  checkbox that shows mixed when 1..n−1 children are selected.
  Approve or drop (see Open decisions).
- **Retained:** `gpui_base::Checkbox` semantics — `CheckboxState::
  Indeterminate` → `Toggled::Mixed`, keyboard, focus, disabled
  (`gpui-base/src/checkbox.rs:17-38,106-115`). The vendored
  `gpui-component::Checkbox` is boolean-only and cannot carry mixed.
- **Composed:** tri-state logic (parent = all/none/mixed; child toggle
  updates parent; parent toggle sets all), the mixed mark (dash) in an
  app-owned indicator, a11y wiring.
- **Dropped:** beUI’s draw-on morph for the mixed mark.
- **A11y/keyboard acceptance:** `Toggled::Mixed` present in the a11y
  facts; activation from mixed → checked (base behavior); parent/child
  sync both directions; Tab + Space/Enter.
- **Reduced motion:** static mark (no new motion) or `spring_control`
  for the mark; honors `reduce_motion` either way.
- **Note:** without an approved surface this control is dropped, not
  built.

### radio — effort M — owner-gated surface

- **Surface:** none exists. The pre-merge candidate (Notifications →
  Digest, replacing a dropdown) is **stale**: the merged baseline has
  no Digest row and no `AppSettings.digest` field. Candidate now: a
  new Notifications → Digest group (Immediate / Daily / Weekly) as a
  radio group, which also adds the `digest` app state. Approve or drop
  (see Open decisions).
- **Retained:** `gpui_base::Radio`/`RadioGroup` semantics — group role
  + orientation (`gpui-base/src/radio_group.rs:57-70`), per-radio
  `Role::RadioButton`, `aria_toggled`/`aria_selected`,
  `aria_position_in_set`/`aria_size_of_set`, controlled selection,
  disabled.
- **Composed:** the dot indicator (the vendored `Radio` draws a check
  mark via `checkbox_check_icon`; beUI uses a dot) — compose app-side
  on `gpui_base::Radio` with an app-owned circle + dot, mirroring the
  input-field composition approach; arrow-key selection if approved
  (see Open decisions).
- **Dropped:** beUI’s gliding `layoutId` dot — no shared-layout
  projection in GPUI; a local dot spring is not continuity and is not
  claimed as such.
- **A11y/keyboard acceptance:** group role + orientation; each radio’s
  toggled/selected + position/size in set; Tab reaches the group;
  Space/Enter selects; arrow keys only if composed (decision).
- **Reduced motion:** local dot spring honors `reduce_motion`; no
  travel otherwise.
- **Fallback:** adopt the vendored `RadioGroup` as-is (S) if the owner
  prefers less code and accepts the check-mark deviation.

### select — effort M — owner-gated surface

- **Surface:** none exists. Candidate: General → Language (4 options)
  as a `Select` listbox replacing the `SettingField::dropdown` menu.
  Approve or drop (see Open decisions).
- **Retained:** listbox panel, keyboard (Up/Down/Enter/Escape via the
  `List` key context, `list/list.rs:29-33,667`), selected check mark,
  placement flip, menu width, empty state.
- **Composed:** `SelectState` + `SearchableVec<LanguageItem>` delegate
  (`searchable_list/vec.rs:76-108`) — `LanguageItem` is a **new**
  app-side item type (the merged baseline's Language dropdown uses
  `(SharedString, SharedString)` pairs) — `SettingField::render`
  wrapper with `.on_reset`, `Select::accessibility_label`,
  subscription to `SelectEvent::Confirm` writing `AppSettings.language`
  + `gpui_kit::component::set_locale(component_locale(&lang))` (the
  merged baseline's locale path; app strings come from
  `i18n::tr(key, &lang)` — there is no `rust_i18n`).
- **Dropped:** beUI panel pinch/unfold morph.
- **A11y/keyboard acceptance:** accessible label + value
  (`Select::accessibility_label`, `accessibility_value`); open via
  click; Up/Down navigate; Enter confirms; Escape cancels; selection
  writes the global and switches the locale.
- **Reduced motion:** no animation retained — nothing to gate.
- **Fallback:** keep the dropdown and drop Select.

### combobox — effort 0 (decision only)

- **Surface:** none — the app has no searchable list (Language has 4
  options; the merged baseline has no Digest row). Adding one would be
  a catalogue demo (audit item 3).
- **Disposition:** dropped from P1; revisit only if a searchable
  surface lands (e.g., a command palette or a long list).

### tooltip — effort S

- **Surface:** collapsed sidebar rail (48px) page items — upstream
  `SidebarMenuItem::collapsed_tooltip` already renders a managed
  tooltip (`Placement::Right`) for icon items
  (`vendor/gpui-component/src/sidebar/menu.rs:195-196,305-308`); the
  app’s pages carry icons, so this works today. The
  `zdesk-port-spec.md` deviation “Collapsed tooltips unavailable” is
  stale and gets corrected.
- **Retained:** managed tooltip system — 500 ms show delay, 300 ms
  grace (`gpui-base/src/tooltip.rs:13-14`), 150 ms slide/fade enter,
  200 ms same-row slide
  (`vendor/gpui-component/src/tooltip.rs:149,151,158-186`).
- **Composed:** nothing (verify + test only).
- **Dropped:** beUI collision-aware follow-cursor tooltip (the managed
  overlay owns placement).
- **A11y/keyboard acceptance:** hover-only (GPUI’s managed system has
  no keyboard trigger) — recorded limitation; acceptance = hover a
  collapsed page item, advance the clock past the show delay, tooltip
  content is present; leaving hides it.
- **Reduced motion:** the enter/slide transitions use `with_animation`,
  which honors `reduce_motion` (`gpui-pre/src/elements/animation.rs:301,407`).

### context-menu — effort S–M — owner-gated surface

- **Surface:** none exists. Candidates: (a) sidebar page right-click →
  “Reset page” — needs a small additive vendor patch exposing the
  existing `SidebarMenuItem::context_menu` through `Settings`
  (`sidebar/menu.rs:200-206,314-317`); (b) About → version chip
  right-click → “Copy version” — fully app-side; (c) drop. See Open
  decisions.
- **Retained:** `ContextMenuExt`/`PopupMenu` — right-click open,
  keyboard navigation, dismiss, `POPUP_PRIORITY`.
- **Composed:** menu content + action wiring.
- **Dropped:** beUI pointer-origin clip morph.
- **A11y/keyboard acceptance:** menu role + items; Up/Down/Enter/
  Escape; right-click opens at the pointer.
- **Reduced motion:** none (no animation retained).
- **Note:** the input’s own right-click menu is the vendored native
  menu (`input/input.rs:317-321`), not this component.

### notification — effort S

- **Surface:** Notifications → Send test notification (existing).
- **Retained:** per-placement stacks, `max_items`, autohide (5 s),
  close, action, click; enter animation.
- **Composed:** optional action button on the test notification to
  exercise the action path (action suppresses autohide); a11y gap
  recorded (no live-region/role in the vendored component).
- **Dropped:** beUI neutral/loading status morph, swipe-to-dismiss,
  view-all summary.
- **A11y/keyboard acceptance:** no live-region semantics exist in the
  vendored component — recorded limitation, not promised; acceptance =
  push → appears in `window.notifications()`; close dismisses;
  autohide after the timeout (advance the test clock).
- **Reduced motion:** enter animation uses `with_animation` → honors
  `reduce_motion`.

### tabs — effort S if approved — conditional

- **Surface:** none approved. Adopt `TabBar` (pill/underline/segmented
  + spring indicator) only if the owner names a tabs surface (e.g.,
  splitting a page into sub-sections); otherwise recorded dropped.
- **Retained (if approved):** `TabBar`/`Tab` variants, spring
  indicator, `aria_label` per tab.
- **Dropped:** beUI clip-masked label morph, overflow arrows.

## Audit resolutions (items 1–7)

1. **Finding 1 + test gap → Phase 0.** Validation is **immediate** on
   every value change (user edit, external write, reset, construction).
   Construction/remount seeds text **and** validation for presentation
   but does **not** bump the shake generation (no shake on mount).
   External writes/reset revalidate through one sync path; an external
   write into an error shakes (new error entry), a reset to a valid
   value does not. Source of truth: the app global owns the value,
   `FieldState` owns presentation; the sync path keeps them consistent.
   Page remount re-seeds from the stored value (no stale error, no lost
   success). Covered by the five view-level tests in Phase 0.
2. **Mapping contract.** The per-control contracts separate
   availability, retained behavior, composed behavior, and dropped
   behavior; no blanket “exists = parity”. Checkbox mixed is
   composition on `gpui_base::Checkbox`; radio’s gliding indicator is
   dropped (a local dot spring is not shared-layout continuity).
3. **P1 surfaces.** Named per control above; new surfaces (Channels,
   Digest radio, Language select, context-menu, tabs) are owner-gated;
   no catalogue demos. The sidebar contract and all colors stay.
4. **A11y/keyboard acceptance.** Per control above. The input gets a
   meaningful `aria_label` (not the placeholder fallback); the error
   message row gets `Role::Alert` when it carries an error; GPUI has no
   `aria_live`/`aria_describedby`, so announcement/association parity
   with beUI’s `role=alert` is **not** promised — recorded. Radio and
   mixed-checkbox semantics require the explicit proofs listed above.
5. **Config immutability + source of truth.** `InputFieldConfig` is
   documented as construction-time (initializer-only): the keyed state
   ignores later configs, so a mounted field does not react to a
   changed label/placeholder/validator. Acceptable today — locale
   changes happen on another page and page navigation remounts the
   field; recorded as a limitation. No new global refreshes: the
   username reset keeps its existing `cx.refresh_windows()`; the
   notification scope stays the window’s notification layer.
6. **Motion + reduced motion.** Retained: switch/checkbox springs,
   managed tooltips, generation-keyed shake. Dropped: shared-layout
   glides, panel morphs, press squish, draw-on morphs. The message row
   reserves a **minimum** line (`min_h`) and grows for wrapped/
   localized messages instead of the fixed `h_5`. Error re-entry,
   replay, and reduced motion are verified with the test backend.
7. **Gates + patch scope.** Per phase `make check && make test &&
   make lint && make fmt-check`; meaningful state/keyboard/a11y tests
   (below); app-side composition first; any vendor patch additive,
   documented in the `Cargo.toml` comment + `zdesk-port-spec.md`, and
   reversible. No GUI gate is claimed by workers; the owner does the
   visual pass.

## Approach and Risks

### Approach

Phase 0 first: the view-level test backend (`gpui-kit` `test-support`
dev-dependency + `#[gpui_kit::test]` harness) is the prerequisite for
every later acceptance test, and the input fix is the audit’s
acceptance gate for the pilot. Then existing-surface polish (Phase 1,
decision-free), then owner-gated new surfaces (Phase 2), then the
conditional controls (Phase 3).

Test mechanics (grounded in the vendored/test-support sources):

- `gpui-kit`’s `test-support` feature exposes `gpui_kit::test`
  (`TestWindowExt`: `find`/`try_find`/`within`/`render_frame`/`click`/
  `click_at`/`right_click`/`hover`/`scroll`/`drag`/`press`/`input`;
  `TestAppContextExt::wait_for`; `ElementSnapshot` with role/checked/
  indeterminate/selected/value/bounds/focused/disabled/label) and the
  `#[gpui_kit::test]` macro.
- `ElementSnapshot`/`find` only sees elements registered with
  `.test_support()`. The vendored `Switch`/`Checkbox`/`Radio`/`Input`
  are `RenderOnce` components wrapped in `ViewElement`
  (`PrepaintState = Option<AnyElement>`), so `TestSupportExt`
  (`PrepaintState = Option<Hitbox>`) cannot be applied to them
  app-side, and they do not register internally. Therefore:
  - interaction: app-side `debug_selector` on the row wrapper +
    `cx.debug_bounds(selector)` + `cx.simulate_click(center,
    Modifiers::default())` (the vendored test pattern);
  - keyboard: `window.focus_next(cx)` (or
    `InputState::focus_handle(cx).focus(window, cx)`) +
    `cx.simulate_event(KeyDownEvent/KeyUpEvent)`;
  - a11y facts: `write_a11y_info(&mut accesskit::Node)` on controls
    built by shared app-side builder functions (the vendored pattern,
    `gpui-base/src/switch.rs:671-726`);
  - state: assert the `AppSettings` global / `FieldState` directly.
- `debug_selector` is a no-op outside test builds
  (`gpui-pre/src/elements/div.rs:883-900`), so it can live in the
  render code unconditionally.

### Rejected alternatives

- **Vendor patches for a11y labels** (`BoolField` passing a label):
  rejected — `SettingField::render` + app-side controls is the
  existing pattern (the notifications-enabled row) and keeps the patch
  count flat.
- **Adopting the vendored `Radio` check mark**: rejected as the
  primary path — beUI’s dot is the target; recorded as the fallback.
- **A combobox/tabs/context-menu demo surface**: rejected — audit
  item 3 forbids catalogue demos.
- **Adding `.test_support()` to vendored controls** (3-line patch):
  rejected for now — `debug_selector` + `write_a11y_info` cover the
  acceptance without a patch; revisit only if a test genuinely needs
  live-tree a11y facts.

### Risks and mitigations

- **R1 — `SettingField::render` conversions lose typed reset
  integration.** Mitigation: `.on_reset(is_dirty, reset)` on the
  `SettingItem` (the notifications-enabled row is the working
  pattern); tests assert the reset path.
- **R2 — `test-support` feature unification rebuilds the GPUI stack
  for test builds.** One-time cost on the first `make test` after the
  dev-dependency lands; the binary keeps default features. If the
  rebuild is unacceptable, stop and report.
- **R3 — mixed checkbox diverges visually from the vendored
  checkbox.** Mitigation: reuse the vendored indicator sizes/tokens;
  owner visual pass.
- **R4 — radio arrow-key composition needs focus management.** If it
  cannot be made clean in one attempt, record arrow keys as dropped
  (deviation) and keep Tab + Space/Enter.
- **R5 — Select wiring is the largest single task.** Timebox; fall
  back to keeping the dropdown and dropping Select.
- **R6 — tooltip test timing.** The 500 ms show delay needs
  `cx.executor().advance_clock(...)`; if the overlay cannot be driven
  headlessly, assert the trigger wiring and record the visual as an
  owner check.
- **R7 — new surfaces add app state** (channels). Owner-gated; if
  declined, the control is dropped, not built.
- **R8 — `InputFieldConfig` initializer-only.** Documented; revisit
  only if a live-update need appears.

## Phases and Verification

Gate after every phase: `make check && make test && make lint &&
make fmt-check`, plus the owner `make run` visual pass. Baseline
(2026-09-30, this worktree at `143e7b6`): `make check` ✓ (31 s cold),
`make test` ✓ 38 passed, `make lint` ✓, `make fmt-check` ✓.

### Phase `beui-p1-input-sync` — story_id: story-beui-p1-input-sync-20260930

- **Status:** `planned`
- **Goal:** external writes/reset revalidate; view-level tests for the
  input field; input a11y + message-row growth; config immutability
  documented.
- **Depends on:** none.
- **Allowed surfaces:** `src/input_field.rs`, `src/main.rs` (username
  config + reset call site), `Cargo.toml` (dev-dependency),
  `docs/design/zdesk-port-spec.md` (input notes), `src/i18n.rs`
  (only if a new string is needed — prefer the existing row title;
  the merged baseline has no `locales/app.yml`).
- **Avoided:** `vendor/`, `assets/themes/`, `src/theme.rs`.

**Wave 1** — `p0-1` test backend:

- Add `[dev-dependencies] gpui-kit = { version = "0.6", features =
  ["test-support"] }` to `Cargo.toml` (feature unification: test builds
  only).
- Add a view-test module in `src/input_field.rs` (`#[cfg(test)]`) with
  a harness that: `cx.update(gpui_kit::init)`, mounts
  `InputFieldView::keyed(window, cx, "test-field", config)` inside a
  small `Render` view (handing the entity back to the test), and draws
  a frame. Config get/set use a local `Rc<RefCell<SharedString>>` for
  isolation.
- First smoke test: the initial frame mirrors the cell value; focusing
  the input (`InputState::focus_handle(cx).focus(window, cx)`) and
  `window.input("ab", cx)` writes the cell and sets the field status.
- **Output:** `make test` runs a real `InputFieldView` headlessly.
- **Check:** `make test` (smoke test passes), `make check`.

**Wave 2** — `p0-2` synchronization fix + a11y + message row:

- `FieldState`: add a construction-time seed that sets validation
  without bumping the shake generation (e.g. `with_validation`).
- `InputFieldView::new`: seed text + validation from
  `get_value`/`validate`.
- `InputFieldView`: one `sync_from_app(&mut self, window, cx)` used by
  `render` — when the app value differs from the input text,
  `set_value` + revalidate via `set_validation` (bumps the generation
  on error entry) + notify.
- `InputFieldConfig`: add `aria_label` (builder) → `Input::aria_label`;
  `main.rs` passes the row title. Document the config as
  construction-time.
- `render_message`: `min_h` instead of fixed `h_5`; set
  `Role::Alert` on the row when it carries an error.
- **Output:** reset/external writes update text **and** validation;
  the field has an accessible name; the message row can grow.
- **Check:** `make check`, `make test` (the existing 38 tests still
  pass).

**Wave 3** — `p0-3` behavioral tests (the audit’s five):

1. user edit → global + validation (type `ab` → global `ab`, status
   `Error`; type `ninja` → `Success`);
2. invalid edit → reset (write the default + `cx.refresh_windows()`,
   the reset handler’s effect) → text and status synchronized, no
   stale error;
3. external invalid write (global → invalid while mounted → render →
   `Error` + text updated);
4. initial/remount policy (construct with an invalid stored value →
   `Error` shown, generation 0, no shake; valid → `Success`);
5. repeat error + reduced motion (clear → re-enter error → generation
   bumps; `cx.set_reduce_motion(true)` → the shake renders at rest —
   assert via the field’s debug bounds).
- **Output:** the five behaviors are pinned by tests.
- **Check:** `make test`, `make lint`, `make fmt-check`.

**Phase checks:** full gate + owner visual pass (username field:
type invalid → error + shake; Reset All → text + success check; both
modes).
**Stop:** the test backend cannot drive the field headlessly after two
attempts → halt, report.

### Phase `beui-p1-existing-surfaces` — story_id: story-beui-p1-existing-surfaces-20260930

- **Status:** `planned`
- **Goal:** switch + checkbox rows get accessible names and acceptance
  tests; tooltip and notification verified on their existing surfaces;
  the stale tooltip deviation corrected.
- **Depends on:** `beui-p1-input-sync` (test backend).
- **Allowed surfaces:** `src/main.rs` (declare `#[cfg(test)] mod
  view_tests;`), `src/view_tests.rs` (new), `docs/design/zdesk-port-spec.md`.
- **Avoided:** `vendor/`, `assets/themes/`, `src/theme.rs`,
  `src/input_field.rs`.

**Wave 1** — `p1-1` switch + checkbox rows:

- Convert General → Dark Mode (`SettingField::switch`) and General →
  Startup → Launch at login (`SettingField::checkbox`) to
  `SettingField::render` with app-side `Switch`/`Checkbox` carrying
  `.accessibility_label(row title)` and `.on_reset(...)`; add
  `.accessibility_label(...)` to the one raw switch
  (notifications-enabled). Give each control a unique id and a
  `debug_selector` on its wrapper.
- Tests (`src/view_tests.rs`): render the app’s Settings pages with the
  relevant page selected (`Settings::default_selected_index`); click →
  global flips; Tab + Space/Enter → flips; a11y facts via
  `write_a11y_info` on the shared builder (role, toggled, label);
  reduced motion (`cx.set_reduce_motion(true)`) → toggle still lands.
- **Check:** `make test`, `make check`.

**Wave 2** — `p1-2` tooltip + notification acceptance:

- Tooltip: hover a collapsed-rail page item (set `ShellState.collapsed`
  + render), advance the clock past the show delay, assert the tooltip
  content is present; leaving hides it. Correct the
  `zdesk-port-spec.md` “Collapsed tooltips unavailable” deviation.
- Notification: mount the app window (with `Root` layers), push a
  notification, assert `window.notifications()`; close dismisses;
  autohide after the timeout (advance the clock). Record the a11y gap
  (no live region) in the spec.
- **Check:** `make test`, `make lint`, `make fmt-check`.

**Phase checks:** full gate + owner visual pass (switch/checkbox rows
in both modes; collapsed-rail tooltips; test notification).
**Stop:** the tooltip overlay cannot be driven headlessly → keep the
wiring test, record the visual as an owner check, continue.

### Phase `beui-p1-new-surfaces` — story_id: story-beui-p1-new-surfaces-20260930

- **Status:** `done` (2026-10-01, `quaqua/tieude-beuip23`) — owner
  decisions answered; all three approved controls composed and tested.
  Owner visual pass pending.
- **Goal:** the approved new-surface controls are composed app-side
  with their acceptance tests; declined controls are recorded dropped.
- **Depends on:** `beui-p1-input-sync` (test backend); owner decisions.
- **Allowed surfaces:** `src/main.rs`, `src/view_tests.rs`,
  `src/i18n.rs`, `docs/design/zdesk-port-spec.md`.
- **Avoided:** `vendor/`, `assets/themes/`, `src/theme.rs`.

**Wave 1** — `p2-1` radio (if approved): compose a new Notifications →
Digest radio group on `gpui_base::Radio`/`RadioGroup` with the
app-owned dot indicator; add the `AppSettings.digest` field (new
state — the merged baseline has none); tests for group semantics,
Tab + Space/Enter (+ arrow keys if approved), reduced motion.
Fallback: vendored `RadioGroup` as-is.

**Wave 2** — `p2-2` select (if approved): compose the Language
`SelectState` + `SearchableVec<LanguageItem>` field; subscribe to
`SelectEvent::Confirm`; tests for open/navigate/confirm/cancel + a11y
label/value. Fallback: keep the dropdown.

**Wave 3** — `p2-3` mixed checkbox (if approved): compose the Channels
group on `gpui_base::Checkbox`; tri-state logic; tests for
`Toggled::Mixed`, activation from mixed, parent/child sync.

**Phase checks:** full gate + owner visual pass per approved control.
**Stop:** a control cannot be composed cleanly after two attempts →
record it dropped with the reason, continue with the rest.

### Phase `beui-p1-conditional` — story_id: story-beui-p1-conditional-20260930

- **Status:** `done` (2026-10-01, `quaqua/tieude-beuip23`) — owner
  approved both surfaces (About “Copy version”, About tabs); both
  implemented and tested. Owner visual pass pending.
- **Goal:** the approved conditional controls are implemented with
  their acceptance tests, or recorded dropped.
- **Depends on:** owner decisions; `beui-p1-input-sync` (test backend).
- **Allowed surfaces:** `src/main.rs`, `src/view_tests.rs`,
  `vendor/patches/` + `vendor/gpui-component/src/setting/settings.rs`
  (only for the sidebar context-menu option, additive),
  `Cargo.toml` comment, `docs/design/zdesk-port-spec.md`.

**Wave 1** — `p3-1` context-menu (if approved): implement the chosen
surface (sidebar “Reset page” with the additive patch, or About “Copy
version” app-side); tests for open/select/dismiss.

**Wave 2** — `p3-2` tabs (if approved): adopt `TabBar` on the named
surface; tests for selection + a11y labels.

**Phase checks:** full gate + owner visual pass.
**Stop:** no surface approved → close the phase as dropped; no code.

## Open decisions (owner)

Phase 0 and Phase 1 need no decisions. Phases 2–3 start only after
these are answered:

1. **Mixed checkbox surface** — approve the Notifications “Channels”
   group (parent “All channels” + Email/Desktop/Digest) or drop mixed?
   *(Recommended: approve — it is the only genuine mixed surface.)*
2. **Radio surface** — approve a **new** Notifications → Digest radio
   group (the merged baseline has no Digest row; this adds the
   `digest` app state) or drop radio? *(Recommended: approve.)*
3. **Select surface** — approve Language as a `Select` listbox
   (replacing the dropdown menu) or drop Select? *(Recommended:
   approve; fallback keeps the dropdown.)*
4. **Context-menu surface** — (a) sidebar “Reset page” (needs a small
   additive vendor patch), (b) About “Copy version” (app-side), or
   (c) drop? *(Recommended: (b) if any; (c) is acceptable.)*
5. **Tabs surface** — name one or drop tabs. *(Recommended: drop —
   no surface exists.)*
6. **Combobox** — confirm dropped (no searchable surface).
7. **Validation policy** — confirm immediate validation everywhere,
   no shake on mount, external write into an error shakes, reset to a
   valid value does not.
8. **Radio keyboard/mark** — arrow-key selection composed or dropped?
   Dot composed app-side or the vendored check mark?

## Decisions

- 2026-09-30 — baseline locked: `quaqua/mdinput` merged into
  `quaqua/tieude-beuicomps` (`143e7b6`, the `origin/main` merge
  `e494a92` into `77af8b5`); audit corrections override the research
  reports.
- 2026-09-30 — hard constraints: colors unchanged; `font.size` 14;
  `shadow: false`; app-side composition preferred over vendor patches.
- 2026-09-30 — Phase 0 is the pilot acceptance gate: finding 1 fix +
  finding 5 view-level tests, decision-free.
- 2026-09-30 — new surfaces (Channels, Digest radio, Language select,
  context-menu, tabs) are owner-gated; combobox is dropped (no
  surface).
- 2026-10-01 — owner decisions (~/quaqua/runs/beui-decisions.md):
  (1) mixed checkbox approved on Notifications → Channels (All channels +
  Email/Desktop/Digest); (2) Digest radio group approved, adds `digest`
  state; (3) Language becomes a `Select` listbox, dropdown fallback kept;
  (4) context menu is About “Copy version”, app-side, no vendor patch;
  (5) tabs BUILT on the About page (Info / Licenses / Changelog);
  (6) combobox KEPT — worker-chosen surface: new General → Region →
  Time Zone, an in-menu-search `ComboboxState` over a curated IANA list
  plus a localized “System” sentinel (a real long-list surface the
  Language listbox does not need); (7) validation policy confirmed as
  planned (immediate; no shake on mount; external write into error
  shakes; reset to valid does not); (8) radio arrow-key selection +
  app-side dot mark approved.
- 2026-10-01 — worker decisions: Language moves to a new General →
  Region group alongside Time Zone (was Appearance); `digest`,
  `channel_*`, `timezone` persist via `persist.rs` (session-only was
  rejected — StoredSettings is a trivially-extensible key–value file,
  in allowed scope); tests mount `Settings` in a `SettingsHarness`
  (pages via `SettingsWindow::setting_pages`, now `&App`-scoped),
  navigate with `default_selected_index`; tab selection lives in
  `use_keyed_state("about-tab")` (self-observing).
- 2026-10-01 — combobox test root cause (worker): the vendored
  `ComboboxState` decided “selection changed” by comparing
  `IndexPath`s, but a filtered list reuses row indices — clicking the
  filtered row that lands on the selected row's index mutated the
  selection without emitting `Change`/`Confirm` or closing. Fixed by
  comparing item **values** (the identity `selection_changes` /
  `is_item_checked` already use) in both the live deferred confirm
  closure and `handle_item_select`; recorded as the third beUI P1
  vendor patch (`combobox-value-change.patch`) — app-side cannot
  reach the commit decision, so the patch is required, not optional.

## Validation

Per phase: `make check && make test && make lint && make fmt-check`,
plus the owner `make run` visual pass. Baseline at `143e7b6`
(2026-09-30): all four green (`make test`: 38 passed). View-level
tests run headless via `#[gpui_kit::test]`; no GUI automation is
claimed. Worker gates never launch the app.

## Progress

- 2026-09-30 — plan authored from the audit + research reports on the
  `77af8b5` baseline; baseline gates green (`make check` 5.9 s,
  `make test` 21 passed, `make lint`, `make fmt-check`). Phases 0–1
  are decision-free; Phases 2–3 wait on the owner decisions above.
- 2026-09-30 — integrated `origin/main` via `origin/quaqua/mdinput`
  (`143e7b6`, conflict-free — this branch only touches this plan file)
  and re-grounded every path/API/line/test-count/token reference on
  the merged tree; gates re-run green (`make check` 31 s cold,
  `make test` 38 passed).
- 2026-10-01 — `beui-p1-new-surfaces` + `beui-p1-conditional`
  implemented on `quaqua/sude-beuip23`: Channels tri-state checkboxes
  (composed on `gpui_base::Checkbox`, spring mark, focus ring,
  indented children), Digest radio (composed `gpui_base::Radio`, dot
  mark, arrow-key + roving-tab-stop group semantics), Language as
  `Select` listbox under new General → Region group, Time Zone as
  searchable `Combobox` (the kept-combobox surface), About context
  menu (Copy version → clipboard) and About `TabBar`
  (Info/Licenses/Changelog panels). `persist.rs` gains
  `channel_*`/`digest`/`timezone`; `i18n.rs` gains all new keys in
  en/vi/zh-CN/ja; `view_tests.rs` adds headless `#[gpui_kit::test]`
  coverage.
- 2026-10-01 — `beui-p1-new-surfaces` + `beui-p1-conditional`
  finished on `quaqua/tieude-beuip23` (the `quaqua/sude-beuip23` WIP
  fast-forwarded): the combobox search/commit test is green (root
  cause + `combobox-value-change.patch` above); added acceptance
  coverage for the gated Digest group
  (`digest_disable_with_notifications`), Space/Enter activation and
  reduced motion for the composed Channels/Digest controls, Select
  Escape-cancel, and context-menu Escape-dismiss; the patch table in
  `docs/design/zdesk-port-spec.md` now lists all thirteen patches.
  Gates: `cargo test` 54 passed, `cargo clippy --all-targets -- -D
  warnings` clean, `cargo fmt --check` clean. Owner visual pass still
  pending (no GUI automation by workers).

## Current State

Phases `beui-p1-new-surfaces` and `beui-p1-conditional` are
implemented and validated on `quaqua/tieude-beuip23` (worker gates
green; owner visual pass pending). Phase 0 (`beui-p1-input-sync`)
remains the next executable phase — the revalidation fix landed on
`main` (`01f12a9`) and the view-test backend exists, but the input
`aria_label`, the error row's `Role::Alert`, the growing message row,
and the five view-level input tests are still open; Phase 1
(existing-surface switch/checkbox labels, tooltip/notification
acceptance, the stale collapsed-tooltip deviation) is planned and
decision-free. The `zdesk-ui-port` plan remains active pending its
owner visual pass.
