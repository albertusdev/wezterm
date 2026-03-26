---
tags:
  - tab_bar
---
# `tab_bar_position = "Top"`

{{since('nightly')}}

Controls where the tab bar is rendered.

Possible values are:

* `"Top"`: render the tab bar at the top of the window. This is the default.
* `"Bottom"`: render the tab bar at the bottom of the window.
* `"Left"`: render the tab bar as a vertical rail on the left side of the window.
* `"Right"`: render the tab bar as a vertical rail on the right side of the window.

Example:

```lua
local wezterm = require 'wezterm'
local config = {}

config.tab_bar_position = 'Left'

return config
```

`tab_bar_at_bottom` remains supported for compatibility, but `tab_bar_position`
is the preferred configuration surface going forward.

Left and right tab bars are rendered using the fancy tab bar layout.
They also honor [tab_bar_width](tab_bar_width.md), and can be resized with the
mouse by dragging the rail edge in the GUI.
