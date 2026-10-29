---
id: mem-a11y-name-needs-a-role-20261026
created: 2026-10-26
---

> **Historical (pre-port gpui-kit stack), kept for context** — the code it
> describes is on branch `wip/gpui-kit-shell`; the app now runs on zeron's
> GPUI stack (see `docs/memory/zeron-stack-port.md`).

# A11y names need a role, and `Button` has its own label API

Two traps hit while fixing the "controls with no accessible name"
findings in mono-desk (gpui-kit 0.6.1, vendored gpui-component).

## 1. `aria_label` on a bare `div` announces nothing

A `div` with no accessibility role is **filtered out of the a11y tree**,
so the label never reaches a screen reader. The first attempt at naming
the sidebar collapse toggle wrapped the button in

```rust
div().id("...").aria_label("Toggle sidebar").child(Button::new(...))
```

which compiles, and `window.find()` cannot even see it (the wrapper is
not a leaf). Dumping the tree showed the truth — the wrapper was absent
and the snapshot that *did* exist was the inner `Button` with
`LABEL=None`:

```text
PATH=[... Name("sidebar-collapse-toggle-wrap"), ... Name("sidebar-collapse-toggle")]
LABEL=None
```

after the fix, with no wrapper at all:

```text
PATH=[... Name("sidebar-collapse-toggle")]
LABEL=Some("Toggle sidebar") ROLE=Some(Button)
```

→ Rule: an a11y name must land on an element that has a role. Either put
it on the real control, or add `.role(...)` to the container. Verify by
reading `snapshots()`, never by "it compiled".

## 2. `Button` does have `accessibility_label`

An earlier review claimed the vendored `Button` exposed no label API and
that a wrapper was therefore required. That was wrong:
`vendor/gpui-component/src/button/button.rs:356` provides
`accessibility_label`, documented for exactly this case — "an icon-only
button has no label to read". `Switch` has the same method
(`vendor/gpui-component/src/switch.rs:66`).

→ Rule: grep the vendored control for `accessibility_label` before
concluding it cannot be named.

## 3. How to test a name

`window.find(id)` only sees leaf elements and panics when the id is not a
registered leaf. For a container or a non-leaf, filter
`gpui_kit::base::test_support::snapshots(window)` by path and read
`.label()` / `.role()`. The working assertion is
`view_tests::sidebar_collapse_toggle_is_named`.

## Verified on the running app (macOS AX tree)

`make run`, then dump the real tree with `AXUIElementCreateApplication` —
this is the only way to know what a screen reader actually gets. Findings:

- **Accent swatches are named**: `AXButton title=teal|blue|green|violet|
  amber|red`. The `.role(Role::Button)` is what put them in the tree; the
  headless test suite could never have shown this (the swatch row produces
  no a11y node in a test frame at all).
- **Selection is `AXHelp`, not `AXSelected`.** `aria_selected` is written by
  gpui as `AccessKit::Node::selected`, but macOS exposes no `AXSelected` on a
  plain `AXButton`; `Role::RadioButton`, which would, drops the swatch out of
  the tree entirely (verified — the swatches vanished). `aria_description`
  survives and lands in `AXHelp`: the selected swatch reads
  `AXHelp=Selected`. So selection is announced by description, not by state.
- **The sidebar toggle is named**: `AXButton title=Toggle sidebar`.
- **The Font Size slider is NOT named**: `AXSlider` with
  `AXValue=14 AXMinValue=8 AXMaxValue=32` and no title. The vendored
  `Slider` sets value/min/max/orientation but never a name and exposes no
  label API. Wrapping it in `div().aria_label(...)` was tried on the running
  app and did **not** work — a role-less div is filtered out, so the name was
  still absent. Recorded as a TODO at the slider.
- **The Timezone combobox is NOT named**: `AXPopUpButton` with no title.
  `Combobox` exposes no `accessibility_label` at any layer — unlike `Select`
  (`vendor/gpui-component/src/select.rs:642`) it never passes one to
  `BaseSelect`. A `div` wrapper with `aria_label` *and* `.role(Role::ComboBox)`
  also failed to appear. Fixed only by a vendored patch, or by switching the
  field to `Select` (loses search — a product change).

Note that the AX tree is unstable while the app is not frontmost: reads
return zero windows when another app has focus. `app.unhide()` +
`activate` + set `AXFrontmost` makes the tree readable.


