---
tags:
  - tab_bar
---
# `tab_max_width`

Specifies the maximum width that a tab can have in the
tab bar when using retro tab mode.  It is ignored when
using fancy tab mode.

Defaults to 16 glyphs in width.

When using a vertical tab bar and [tab_bar_width](tab_bar_width.md) is not set,
WezTerm derives the default rail width from `tab_max_width`.

```lua
config.tab_max_width = 16
```
