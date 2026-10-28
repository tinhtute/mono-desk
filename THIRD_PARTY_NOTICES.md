# Third-party notices

mono-desk vendors or links the following third-party sources. Files copied
into `vendor/zeron-ui/` keep this notice; the plan that introduced them is
`docs/plans/active/zeron-stack-port.md`.

## zeron — MIT

UI source ported into `vendor/zeron-ui/` (window chrome, sidebar/settings
layout, settings widgets, theme/motion/frost helpers, icon asset registry)
and the theme domain model in `vendor/zeron-theme/` come from zeron: https://github.com/zeronsh/zeron (pinned commit
`8b7afb3`, version v0.2.129).

```
MIT License

Copyright (c) 2026 Wing

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Solar Icons by 480 Design — CC BY 4.0

The icon SVGs under `vendor/zeron-ui/assets/icons/` are zeron's embedded
subset of the Solar Icons set (Linear weight), licensed CC BY 4.0
(https://creativecommons.org/licenses/by/4.0/). Attribution: "Solar Icons by
480 Design".

## gpui / gpui_platform — Apache-2.0

The UI foundation is zeron's GPUI fork, https://github.com/zeronsh/zui
(pinned rev `b564a344`), derived from Zed's gpui. `crates/gpui` and
`crates/gpui_platform` declare `license = "Apache-2.0"` (Copyright 2022-2025
Zed Industries, Inc.); they are consumed as git dependencies, not vendored.
