# GPUI pitfalls, distilled from Ely's AGENTS.md

Ely's `AGENTS.md` rules section is a bug log: each line is a trap that was hit,
diagnosed, and fixed. These are the **GPUI-general** ones — they apply to any
GPUI app, including mono-desk on gpui-kit. Read this before debugging an
interaction bug; most of these look like "the framework is broken" until you
know the rule.

Vendor-specific harness notes (macOS capture scripting, `NSEvent` posting,
`NSColorSampler`) are omitted — see `/tmp/ely-repo/AGENTS.md` if ever needed.

---

## Hover

- **`hover()` needs an element id.** Without one, `hover` applies at *paint*,
  after the text already took its color, and a mouse move alone redraws nothing.
  `.id(..).hover(..)` is what recolors text.
- After a key press, paint drops hover and layout keeps it until the mouse
  moves. A hover that sets *both* text and background then shows only the text.
- A hover that must survive typing under a resting pointer sets
  `HoverListenerMode::InputModalityIndependent` — toast stacks, carousels,
  tooltips, hover cards, charts, scroll activity all need this.
- A hover preview the keyboard also moves keeps the default on its items; the
  box that clears it on leave listens independently so a real exit still
  reports, and ignores a leave that follows a key (`last_input_was_keyboard`).
- What shows on hover should rest `invisible()` and turn `visible()` through
  `group_hover`, not be conditionally built.

## Focus

- **The focus ring is a 1px focus-colored border, not a shadow.** gpui paints
  shadows *under* the element, so a shadow ring floods transparent elements.
  Focusable elements keep a 1px border that is transparent at rest.
- Buttons stay out of focus on click (`prevent_default` on mouse down); Tab
  still reaches them.
- A tracked `FocusHandle` is a Tab stop **only** when built with
  `.tab_stop(true)`. `tab_index` on the div reaches only handles gpui makes
  itself.
- An overlay takes focus on open and **returns it on every close path**; while
  open it re-takes focus whenever the focused element leaves the tree
  (`on_focus_lost`). A root focus scope takes focus when the focused element
  leaves — as a toast's or alert's button does.

## Draw order and overlays

- **gpui paints deferred draws by priority alone, not by nesting.** A tooltip
  (priority 0) deferred *inside* a fullscreen dialog (priority 1) draws
  **under** it.
- The fix is a `raise` helper: at the top level it defers; inside something
  already raised it lays out on its own and draws last *there*. So a list opened
  in a dialog correctly lies over the dialog.
- A raise takes its owner's id and draws under it, so its keyed state and focus
  hold while other raises come and go.
- **`occlude()` blocks the pointer for everything painted before it — its own
  ancestors too.** Put it on the outermost box that should stop the pointer, or
  the ancestors' hover never fires.

## Drag

- gpui starts a drag on the first move past its threshold; only moves *after*
  that reach `on_drag_move`. A component that anchors on its first drag move
  sets its anchor there.
- A drag that pans, turns or slides must go through gpui's drag — its moves
  reach the owner inside *or* outside its box. `on_mouse_move` stops at the
  box's edge.
- gpui sends a drag's moves to **every** listener of its type, so each payload
  must carry its owner's `EntityId` and handlers must check it.

## Motion and animation

- **An animation's id passes to every descendant, and a new id rebuilds their
  focus, drags and hover.** So an effect set off by a change (flash, shake, a
  bar fading at rest) should read elapsed time through a `since_change` helper
  rather than restarting an animation id.
- A **repeating** animation (spinner, breath, sweep) must *hold still* under
  reduced motion rather than shorten — shortening would spin it at 1 ms a turn.
- gpui hands an easing's value to the whole animator **unclamped**, so a spring
  belongs *inside* the animator and only on what may overshoot. Spring the
  travel of a switch thumb, not its colors.

## State and redraw

- **A state change made outside render must call `cx.notify()`**, or nothing
  redraws.
- A notify made *while drawing* (e.g. in a canvas's prepaint) marks the view but
  schedules no frame — pair it with `window.request_animation_frame()`.

## Text input

- Every text field should be a single `TextInput`-style entity, with its keys
  under a dedicated key context bound in `init`. Wrappers that need Up, Down,
  Enter or Backspace take them first with `capture_action`.
- Bounds read from the *last* layout: text changed this frame clamps to the
  laid-out end until the next paint.

## Floating lists

- Float below the anchor, or above when only above has room. gpui's own switch
  keeps the anchor point, so a naive flip covers the trigger.
- An option list should be at least its trigger's width and at most the window
  width less a margin each side — wider, gpui pins it left and the rest runs off.

## SVG and icons

- **`svg()` paints only with its own `text_color`.** Always set one.

## Composition rules worth copying

- **One component, one home.** Duplicates point to it.
- **Fail fast.** No silent fallbacks; log state changes.
- **Colors, control and icon sizes, text sizes, radii, shadows come from
  `cx.theme()`. No raw `px()` literals inside components.**
- Gaps and padding use gpui's **rem scale** helpers (`gap_2`, `px_3`) — that
  scale *is* the spacing system.
- Borders use gpui's **fixed-pixel** helpers (`border_1` is 1px). Hairlines do
  not scale.
- Files stay at or under ~500 lines; comments are one short line and rare.
- Share behaviour in a named helper rather than repeating it: one `listing`
  helper for every trigger-that-opens-a-list, one `Palette` for every command
  palette, one `menu` module for every menu — the call sites then only build
  rows.

---

## Applying this to mono-desk

These rules are gpui-level, so they hold on gpui-kit 0.6.1 (`gpui-pre` 0.3.5)
too. Two caveats:

- gpui-kit's vendored `gpui-component` may already encode some of these — check
  `vendor/gpui-component/src/` before hand-rolling a fix.
- Where a rule names a gpui API (`HoverListenerMode`, `occlude`, `capture_action`,
  `on_focus_lost`), confirm the same API exists at 0.3.5 before relying on it.
