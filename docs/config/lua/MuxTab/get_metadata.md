# `tab:get_metadata()`

Returns a table holding the metadata that has been assigned to this tab.

Tab metadata is a string-to-string map that lives for the lifetime of the mux tab.

```lua
local meta = tab:get_metadata()
wezterm.log_info(meta["agent_hud.session_id"] or "")
```
