---
tags:
  - tab_bar
---
# `tab_bar_width`

{{since('nightly')}}

Specifies the width of a left or right tab bar rail.

This setting is only used when [tab_bar_position](tab_bar_position.md) is set to
`"Left"` or `"Right"`.

If omitted, WezTerm derives a default width from
[tab_max_width](tab_max_width.md).

Example:

```lua
local wezterm = require 'wezterm'
local config = {}

config.tab_bar_position = 'Left'
config.tab_bar_width = '18cell'

return config
```

When the tab bar is vertical, the rail can also be resized interactively by
dragging its edge in the GUI. That live resize is applied as a per-window config
override.
