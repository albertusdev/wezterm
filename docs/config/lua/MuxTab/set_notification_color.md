# `tab:set_notification_color(COLOR)`

{{since('nightly')}}

Sets the notification background color for the tab.

`COLOR` may be any color string supported by WezTerm.

This is a convenience wrapper around the reserved metadata key
`"wezterm.notification_color"`.

```lua
tab:set_notification_color '#d14d41'
```
