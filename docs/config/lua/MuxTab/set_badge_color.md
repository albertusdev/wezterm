# `tab:set_badge_color(COLOR)`

{{since('nightly')}}

Sets the badge background color for the tab.

`COLOR` may be any color string supported by WezTerm.

This is a convenience wrapper around the reserved metadata key
`"wezterm.badge_color"`.

```lua
tab:set_badge_color '#2b6cb0'
```
