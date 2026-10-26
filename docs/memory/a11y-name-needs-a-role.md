---
id: mem-a11y-name-needs-a-role-20261026
created: 2026-10-26
---
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

## Still unverified

Two controls could not be named, and are now marked rather than faked:

- **Accent swatch row / `ColorPicker`** produce no a11y node at all in a
  headless frame (26 snapshots total, none under `accent`), so the
  `.role(Role::Button)` + `.aria_label(...)` on `accent_swatch` cannot be
  confirmed by test. Needs a real `make run`.
- **Timezone `Combobox`** is announced with no name. `Combobox` exposes no
  `accessibility_label` at any layer — unlike `Select`
  (`vendor/gpui-component/src/select.rs:642`), it never passes one to
  `BaseSelect` in `render`. A `div` wrapper with `aria_label` (and even with
  `.role(Role::ComboBox)`) still did not appear in the tree, so it was
  removed rather than left as decoration that looks like it works. Fixing
  this properly means a vendored patch following the `select.rs:642` pattern,
  or switching the field to `Select` — which loses search, a product change.

