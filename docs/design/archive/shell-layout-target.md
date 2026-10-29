# Shell layout target (owner-approved 2026-09-28)

> **Historical (pre-port gpui-kit stack).** This document records the
> approach that was dropped on 2026-10-28; the app now runs on zeron's
> GPUI stack. Current architecture: `docs/PROJECT.md`, module map:
> `docs/memory/zeron-stack-port.md`, plan:
> `docs/plans/active/zeron-stack-port.md`.

Expanded (240px sidebar):
```
┌──────────────────────────────────────────────────────┐
│ ● ● ●  ⊟                                             │  titlebar, no border
│                  ╭─────────────────────────────────╮ │
│  ◆ mono-desk     │ General                         │ │  main card: thin border,
│                  │─────────────────────────────────│ │  all 4 corners rounded,
│  [🔍 Search...]  │ Appearance                      │ │  lighter bg, equal inset
│                  │  Dark Mode            [ ●━ ]    │ │  on all 4 sides
│  ⚙ General     ˅ │                                 │ │
│  ╭────────────╮  │                                 │ │
│  │◐ Appearance│  │                                 │ │  active item: white raised pill
│  ╰────────────╯  │                                 │ │
│    ▶ Startup     │                                 │ │
│  🔔 Notific.   ˅ │                                 │ │
│  👤 Profile      │                                 │ │
│  ⓘ About       › │                                 │ │
│                  ╰─────────────────────────────────╯ │
└──────────────────────────────────────────────────────┘
  sidebar: no border, same bg as window
```

Collapsed (48px, via ⊟ or Cmd+B):
```
┌──────────────────────────────────────────────────────┐
│ ● ● ●  ⊟                                             │
│      ╭───────────────────────────────────────────────╮
│  ◆   │ General                                     │ │
│ ╭──╮ │                                             │ │
│ │⚙ │ │                                             │ │  page icons only (4),
│ ╰──╯ │                                             │ │  no children on the rail
│  🔔  │                                             │ │  search hidden
│  👤  │                                             │ │
│  ⓘ   │                                             │ │
│      ╰─────────────────────────────────────────────╯ │
└──────────────────────────────────────────────────────┘
```
Reference: zdesk screenshots (traffic lights + sidebar toggle in titlebar; borderless sidebar; only the content card has border/radius/lighter bg).
