# `tab:get_badge()`

{{since('nightly')}}

Returns the badge text assigned to the tab, if any.

This is a convenience wrapper around the reserved metadata key
`"wezterm.badge"`.

```lua
local badge = tab:get_badge()
if badge then
  wezterm.log_info('badge=' .. badge)
end
```
