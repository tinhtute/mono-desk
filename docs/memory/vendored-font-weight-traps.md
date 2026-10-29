---
id: mem-vendored-font-weight-traps-20261026
created: 2026-10-26
---

> **Historical (pre-port gpui-kit stack), kept for context** — the code it
> describes is on branch `wip/gpui-kit-shell`; the app now runs on zeron's
> GPUI stack (see `docs/memory/zeron-stack-port.md`).

# Swapping the UI sans: three traps that all fail silently

Done when replacing Inclusive Sans with Momo Trust Sans
(`assets/themes/mono.json` `font.family`, registered in
`theme::install`). Each trap below produces an app that runs, builds,
and passes type-checking while rendering the wrong font — none of them
raise an error.

## 1. A variable font renders every weight at its default

Upstream Momo Trust Sans ships as one variable file,
`MomoTrustSans[wght].ttf` (`wght` 200–800) — smaller (162 KB) and
tidier than four statics. It is **unusable here**: GPUI has no
variation-axis support. A grep across `gpui-pre`, `zed-font-kit` and
`cosmic_text` finds no `set_variation` / `wght` plumbing, and CoreText
reports `CTWeightTrait = 0.0` for the file. Anything asking for 700 gets
the 400 instance, and GPUI never synthesizes bold, so bold text silently
renders as regular.

Vendor **static instances** instead (this is also why
`XANH_MONO_SEMIBOLD_ITALIC` exists — see the comment in `src/theme.rs`).

## 2. The two font backends resolve families by different name IDs

macOS and Linux do not share a text stack:

| Platform | Stack | Family lookup |
|---|---|---|
| macOS | `zed-font-kit` → CoreText | CoreText's family name |
| Linux | `cosmic_text` → `fontdb` | `TYPOGRAPHIC_FAMILY` (name 16), falling back to `FAMILY` (name 1) only when 16 is absent |

`fontdb`'s `parse_names` collects name 16 **exclusively** when it is
present, so a face that puts its per-weight name in name 16
(`"Foo Medium"`) becomes invisible under family `"Foo"`. Google Fonts
marks non-RIBBI weights correctly (name 16 = base family, name 17 =
`Medium`), but this is worth checking on every re-vendor:

```python
from fontTools.ttLib import TTFont
for w in (400, 500, 600, 700):
    n = TTFont(f"Foo-{w}.ttf")["name"]
    print(w, n.getDebugName(16), "|", n.getDebugName(17))
```

Trap inside the trap: `print()` of two names puts them **space
separated on one line**, which reads exactly like one long family name.
I misread `"Momo Trust Sans" + " " + "Medium"` as a single string
`"Momo Trust Sans Medium"` and concluded the 500/600 faces were
unreachable. Dump one value per line, or inspect every `name` record
(`platformID`, `langID`) before drawing a conclusion.

`embedded_ui_faces_share_one_family_and_distinct_weights` now pins
this: every vendored face must resolve to the theme family under
`fontdb`'s rule, with a distinct `OS/2` weight.

## 3. `ttf-parser` is not the macOS render path

The dev-dependency comment claimed the face tests load bytes "exactly
as the app does" via `fontdb` → `ttf-parser`. True on Linux; on macOS
`fontdb` is only pulled in by `usvg` for SVG, and text renders through
CoreText. The tests are still meaningful — both backends read the same
`name` and `OS/2` tables — but the claim was wrong and is now worded
accurately in `Cargo.toml` and `src/theme.rs`.

## Verifying a swap without a screenshot

Screen Recording permission was unavailable, and the AX tree exposes no
attributed string (`AXValue` came back un-attributed, so `NSFont` could
not be read off running labels). What did work:

- A **CoreText probe** (`CGFont` → `CTFontCreateWithGraphicsFont` →
  `CTFontCopyFamilyName` / `CTFontCopyTraits`) reports the family and
  weight trait the macOS backend will actually see.
- Glyph coverage compared against every non-ASCII character in the
  source's string literals: Momo Trust Sans and Inclusive Sans miss the
  **same 387** CJK/Japanese characters (handled by the fallback stack)
  and both cover all 52 Vietnamese characters the UI uses — so the swap
  introduces no coverage regression.
- Both new tests were confirmed to **go red** when fed a wrong-family
  font / stale theme JSON, rather than only passing.
