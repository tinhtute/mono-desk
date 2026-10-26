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

The accent swatch row and its `ColorPicker` produce **no** a11y node in a
headless frame at all (26 snapshots total, none under `accent`), so the
`.role(Role::Button)` + `.aria_label(...)` added to `accent_swatch` cannot
be confirmed by test. That needs a real `make run`. Do not read the
passing test suite as evidence that the swatches are announced.
