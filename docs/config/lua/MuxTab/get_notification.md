# `tab:get_notification()`

{{since('nightly')}}

Returns the notification text assigned to the tab, if any.

This is a convenience wrapper around the reserved metadata key
`"wezterm.notification"`.

```lua
local notification = tab:get_notification()
if notification then
  wezterm.log_info('notification=' .. notification)
end
```
