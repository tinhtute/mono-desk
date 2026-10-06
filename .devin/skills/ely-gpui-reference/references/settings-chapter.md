# Ely Settings chapter — API and gap analysis

Source: `/tmp/ely-repo/src/settings/` (clone per SKILL.md).
Live: https://ely-gpui.zacharyzhang.com/components/settings/

Ely's settings surface is split across three chapters:

- **35 Settings** — the layout and the preference controls (`src/settings/`)
- **39 Theme** — palette/type/motion editing (`src/theme/`, `settings/editor.rs`)
- **06 Forms** — the input primitives those controls are built from

Every component follows the same shape: `RenderOnce` + builder methods, a
`Rc<dyn Fn(Value, &mut Window, &mut App)>` callback, and a `panic!` when a
required callback is missing (fail-fast is a house rule there).

---

## Layout primitives

### `SettingsLayout`

A settings page: sections down the side, the open one beside them. In a narrow
window the list moves above the page.

```rust
SettingsLayout::new(id, [(key, name, IconName)], selected)
    .page(element)
    .on_select(|key: &SharedString, window, cx| { /* ... */ })
```

`selected` is asserted to exist among `sections`. Selection state is the
**owner's** — the component is stateless and just reports.

### `SettingsSection`

```rust
SettingsSection::new("Appearance")
    .description("How the app looks.")
    .row(SettingsRow::new("Theme").description("...").control(control))
```

### `SettingsRow`

`new(title)` / `.description(text)` / `.control(element)`. The control sits at
the end of the line, or drops below the text when the line is too narrow.

---

## Preference controls

### `SettingsSearch` — the most portable idea here

```rust
SettingsSearch::new(id, entries, &text_input_entity)
    .on_pick(|key: &SharedString, window, cx| { /* jump to setting */ })
```

`SettingEntry { key, section, name, words: Vec<SharedString> }` — `words` are
synonyms ("dark", "light" for a Theme setting).

`matching(entries, query)` semantics, pinned by unit tests:

- query is split on whitespace, lowercased
- each entry's haystack is `name + " " + section + " " + words`, lowercased
- **every** term must be a substring of the haystack (AND, not OR)
- empty query → no results (the list is hidden, not "show everything")
- no match → a quiet `fg_subtle` line: `No setting answers to "<query>".`

Results render as a `SelectableList`, one row per hit, with the **section name as
the row's description**. Enter or double-press activates and the owner jumps.
The search field is an `Entity<TextInput>` owned by the caller, so the query
survives re-render.

### `ThemeSelector`

```rust
ThemeSelector::new(id, appearance).on_change(|a: Appearance, window, cx| {})
```

`Appearance::{Light, Dark, System}`, rendered as three segments with
Sun / Moon / Monitor icons. `Appearance::mode(system: Mode) -> Mode` resolves
System against the OS mode — **this is the piece a bare dark-mode switch lacks.**

### `AccentColorPicker`

```rust
AccentColorPicker::new(id, color, [(name, Hsla)]).on_change(|c: Hsla, window, cx| {})
```

Named presets as a `ColorPalette` (one row, columns = preset count) beside a
`color_well` for an arbitrary color.

### `FontSizeControl`

```rust
FontSizeControl::new(id, size, default, range).on_change(|px: f32, window, cx| {})
```

Asserts `range.0 < range.1` and that both `size` and `default` lie inside it.
Renders: slider (step 1.0) + `"{n} px"` label + a reset `IconButton` that is
**disabled when already at default** + a live preview line drawn at the chosen
size ("The quick brown fox jumps over the lazy dog.").

### `DensitySelector`

`Density::{Compact, Standard, Comfortable}` as three segments.
Ely backs this with a real token scale (`theme::Density`), so gaps and control
heights actually change — not just a label.

### `KeyboardShortcutsList`

```rust
KeyboardShortcutsList::new(shortcuts, &search_entity)
```

`Shortcut { group, name, keys }` where keys is written like `"cmd-shift-p"`.
Grouped, searchable over name **and** keys, keys drawn as `Kbd` caps. Editing
bindings is a separate component (`editor::KeybindingsEditor`) — the list is
read-only.

### `ProxySettings`

```rust
ProxySettings::new(id, proxy).on_apply(|p: Proxy, window, cx| {})
```

`Proxy::{Off, System, Manual { host, port, bypass }}`. Port validated `1..=65535`,
bypass split at commas. **Apply is explicit** (not on-change) and waits for a
valid host and port. A new `Proxy` handed down from the owner refills the form.

### `PrivacySettings`

`Privacy` struct of switches; `new(id, privacy).on_change(...)`.

### `NotificationSettings`

`Notices { pairs: Vec<(kind, channel)>, hold: bool }` — a **kind × channel
matrix**, one row per notification kind, a named box per channel. In a narrow
window the channel boxes fold under the kind name. Plus a switch that holds
every notice back. This is a much richer model than a single on/off.

### `StartupSettings`

`Startup { open_at_login, restore_windows, check_updates, start_page }` plus a
`pages` list for the start-page picker.

### `StorageSettings`

`Store { key, name, bytes, clearable }`. A bar showing each store's share of the
total, then a row per store with its size and a **Clear that asks twice** before
calling back.

### `ResetToDefault`

```rust
ResetToDefault::new(id, changed)
    .on_reset(|key: &SharedString, window, cx| {})
    .on_reset_all(|window, cx| {})
```

`Changed { key, name, current, default }` — both values as display strings. Lists
what differs from defaults with a way back for each; **Reset all asks twice**;
with nothing changed it says so rather than rendering an empty box.

### `ImportExportSettings`

`new(id)` + `.on_export(fn)` + `.on_import(fn(PathBuf, ...))`. Export button; a
JSON file can be **dropped or picked**.

### `FeatureFlags` and `DeveloperModeToggle`

`Flag { key, name, description, stage: Stage, on }` where `Stage` is how far
along a feature is. Searchable over name and description, toggle per flag.

`DeveloperModeToggle::new(id, on).on_change(...)` — a switch carrying a warning
about what turning it on reveals.

---

## Theme chapter (39) — related

`ThemeEditor` (edit the palette live), `ThemeImporter` (import a VS Code theme —
`read_vscode_theme`, `VsCodeTheme`), `SyntaxThemePicker`, and provider-style
controls: `DensityProvider`, `RadiusProvider`, `FontProvider`,
`HighContrastMode`, `ReducedMotionProvider`.

---

## Gap analysis — Ely settings vs mono-desk today

mono-desk today: `Settings` (gpui-component, vendored) with two pages —
*General* (dark-mode switch, language dropdown, font-size number input,
launch-at-login checkbox) and *Notifications* (one switch + status chip, a test
button). Persistence in `src/persist.rs`, theme in `src/theme.rs`.

`gpui-component` gives `SettingField::{checkbox, dropdown, input, number_input,
render, scrollable_dropdown, switch, element}`, plus `ColorPicker`, `Slider`,
and `Kbd`. It has **no** segmented control and **no** density scale.

| Capability | Ely | mono-desk | Nature of the gap |
|---|---|---|---|
| Light/Dark/**System** selector | `ThemeSelector` | dark-mode switch only | new control + OS-mode follow |
| Accent color picker | `AccentColorPicker` | none | `ColorPicker` already vendored — **wiring**, not capability |
| Font size: slider + preview + reset | `FontSizeControl` | `number_input` | new control |
| Density (compact/standard/comfortable) | `DensitySelector` + token scale | none | needs a **token scale**, not just a control |
| Settings search w/ synonyms + jump | `SettingsSearch` + tested `matching` | built-in field, no synonym/jump API | new behaviour over existing search |
| Changed-from-default reset panel | `ResetToDefault` | per-page `resettable(true)` only | new page/overlay |
| Keyboard shortcuts list | `KeyboardShortcutsList` | none | new page; `Kbd` already vendored |
| Import / export settings | `ImportExportSettings` | internal write only | new I/O surface + file pick/drop |
| Storage usage + clear | `StorageSettings` | none | new page |
| Feature flags + dev mode | `FeatureFlags`, `DeveloperModeToggle` | none | new page |
| Proxy settings | `ProxySettings` | none | new page, needs a network consumer to be real |
| Privacy + notification matrix | `PrivacySettings`, `NotificationSettings` | single switch | richer model |
| Startup: restore windows / updates / start page | `StartupSettings` | `launch_at_login` only | wider model |
| Theme editor / VS Code import | `ThemeEditor`, `ThemeImporter` | static `mono.json` | largest lift |
| Segmented control primitive | `SegmentedControl` | absent | **prerequisite** for the two selector controls |

### Reading the table

Two clusters, different cost:

1. **Controls over state that already exists** — theme selector, accent picker,
   font-size slider, density. Cheap; the state or the primitive is already
   there. Density is the exception: it needs a real token scale or it is
   decoration.
2. **New product surfaces** — shortcuts, storage, import/export, flags, proxy,
   privacy, startup. Each needs a consumer, or it violates this repo's
   "every control produces a real effect" bar set in
   `docs/plans/completed/real-settings.md`.

The **search** item is the highest value-per-line: it is small, fully specified
by tests upstream, and it makes a growing settings tree navigable — which is
exactly the problem mono-desk's sidebar has been reshaping around.
