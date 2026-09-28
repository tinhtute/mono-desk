# Shell layout target (owner-approved 2026-09-28)

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
│ │⚙ │ │                                             │ │  group icons only (4),
│ ╰──╯ │                                             │ │  click = open first child
│  🔔  │                                             │ │  search hidden
│  👤  │                                             │ │
│  ⓘ   │                                             │ │
│      ╰─────────────────────────────────────────────╯ │
└──────────────────────────────────────────────────────┘
```
Reference: zdesk screenshots (traffic lights + sidebar toggle in titlebar; borderless sidebar; only the content card has border/radius/lighter bg).
