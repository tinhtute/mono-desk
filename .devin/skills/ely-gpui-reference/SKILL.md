---
name: ely-gpui-reference
description: Consult the Ely GPUI component library — its live WASM gallery, its source, the settings-chapter API, and the GPUI pitfalls its AGENTS.md documents. Use when researching GPUI component patterns, when asked "how does Ely do X", when a settings/forms/theme control needs a reference implementation, or when porting an Ely pattern onto mono-desk's gpui-kit stack. Also use before inventing a new settings control, to check whether Ely already solved it.
allowed-tools:
  - read
  - grep
  - glob
  - exec
  - webfetch
---

# Ely GPUI Component — reference

A large, macOS-first GPUI component library by Zachary Zhang. Not a dependency of
this project; a **reference implementation** to consult.

## Where it lives

| | |
|---|---|
| Live gallery | https://ely-gpui.zacharyzhang.com/components |
| Source | https://github.com/ZacharyZhang-NY/Ely-GPUI-Components |
| License | MIT OR Apache-2.0 |
| Scale | 1272 components / 622 stories / 43 chapters, ~197k lines of Rust, 840 files, one crate `ely-gpui-component` |

Local clone (throwaway, not a project dependency):

```sh
git clone --depth 1 https://github.com/ZacharyZhang-NY/Ely-GPUI-Components.git /tmp/ely-repo
```

## Do not try to add it as a dependency

**Correction (2026-10-02).** An earlier version of this file claimed
`gpui_platform` is unpublished and that this alone rules Ely out. That was
wrong — it checked the wrong crate name. The facts:

- The package behind `gpui_platform` is **`gpui-pre-platform`**, and it **is**
  published on crates.io (max **0.3.7**), as are `gpui-pre` and `gpui-pre-web`.
- mono-desk already resolves `gpui-pre` **0.3.5** and `gpui-pre-platform`
  **0.3.5** — `gpui-kit` 0.6.1 declares `gpui = { package = "gpui-pre" }` and
  `gpui_platform = { package = "gpui-pre-platform" }`. Both are renamed
  crates.io deps, not git deps.
- Ely pins `gpui` and `gpui_platform` to Zed's git repo at rev `1a28cff` — the
  code `gpui-pre` 0.3.7 republishes. Caret ranges would unify everything on
  0.3.7, so **the gpui pin is not a hard blocker.**

The real reasons it is still the wrong move:

- Ely is **one crate** (`publish = false`, ~197k lines). You cannot cherry-pick
  `src/settings/` without forking the whole thing and its macOS-only deps
  (`cocoa`, `objc`, `wry`, `alacritty_terminal`, `raw-window-handle`).
- Platform fit: README says "Tested on macOS only". `src/` has **51**
  `cfg(target_os = "macos")` sites against **1** Linux branch. mono-desk targets
  Linux.
- Adopting it means a git dependency whose transitive `gpui`/`gpui_platform`
  must be redirected onto the crates.io republish with `[patch]` — fragile, and
  it would put a second, differently-pinned gpui in the tree if it ever fails
  to unify.

**Conclusion: port the pattern, never the code** — now on practicality and
platform grounds, not impossibility. Reimplement against gpui-kit's API. Every
Ely component is `RenderOnce` + builder methods + `Rc<dyn Fn>` callbacks; the
shape translates, the bodies do not.

## How to consult it

### Live gallery

Routes are `/components/<chapter-slug>/<story-slug>/`. The whole gallery is a
GPUI app compiled to WebAssembly in an iframe — **it takes ~20 s to boot**. Wait
before screenshotting or you capture "Starting…".

```sh
export AGENT_BROWSER_SESSION="$(agent-browser session id --scope worktree --prefix ely)"
agent-browser open "https://ely-gpui.zacharyzhang.com/components/settings/"
sleep 25 && agent-browser screenshot /tmp/ely.png
```

Chapters most relevant to a settings app: `settings`, `forms`, `theme`,
`interaction`, `overlays`, `navigation`, `lists`, `feedback`.

### Source

`examples/gallery/stories.json` is the machine-readable inventory — every
chapter, every story, in drawing order. Query it instead of guessing:

```sh
cd /tmp/ely-repo/examples/gallery && python3 -c "
import json
for p in json.load(open('stories.json')):
    if p['slug'] == 'settings':
        for s in p['stories']: print(s['title'], '|', s['slug'])
"
```

## Reference files

- `references/settings-chapter.md` — the Settings chapter API in full, plus the
  gap analysis against what mono-desk already has. **Read this first when the
  task is about settings UI.**
- `references/gpui-pitfalls.md` — distilled from Ely's 56 KB `AGENTS.md`. GPUI
  behaviours that cost real time to rediscover (focus rings, hover, deferred
  draw order, animation ids, occlusion). **Read this before debugging any GPUI
  interaction bug** — it is the highest-value artifact in the repo.

## Why this repo is worth consulting at all

Beyond component inventory, three things are hard to get elsewhere:

1. **A complete settings surface.** Most GPUI projects stop at a switch and a
   dropdown. Ely ships theme/accent/font-size/density controls, a synonym-aware
   settings search, a changed-from-default reset panel, storage usage,
   import/export, feature flags, proxy, privacy, and a shortcuts list.
2. **Documented GPUI failure modes.** Its `AGENTS.md` rules section reads like a
   bug log — each line is a trap that was hit and fixed.
3. **A tested search-matching contract.** `settings/search.rs` has unit tests
   pinning the semantics (every query word must match; name + section + synonyms
   are one haystack).

## Rules when porting a pattern

- Reimplement on gpui-kit primitives (`SettingField`, `SettingItem`,
  `SettingGroup`, `SettingPage`). Do not add `gpui` as a direct dependency.
- Respect this repo's own conventions from `AGENTS.md`: state in a `Global`,
  `.default_value(...)` on every `SettingField` so page reset works, theme colors
  via `cx.theme()`, Tailwind-flavoured chained methods.
- Ely's spacing/border conventions match gpui's helpers already used here
  (`gap_2`, `px_3`, `border_1`) — keep them.
- A ported control needs a real effect and persistence, or it is decoration.
  See `docs/plans/completed/real-settings.md` for the bar this project holds.
