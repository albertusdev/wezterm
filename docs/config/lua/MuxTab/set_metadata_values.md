# `tab:set_metadata_values(METADATA)`

Replaces the complete metadata map for the tab.

`METADATA` must be a Lua table mapping string keys to string values.

```lua
tab:set_metadata_values {
  ["agent_hud.title"] = "Review",
  ["agent_hud.urgency"] = "high",
}
```
