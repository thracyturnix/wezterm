---
tags:
  - exit_behavior
---
# `window_close_confirmation`

Whether to display a confirmation prompt when the window is closed by the
windowing environment, either because the user closed it with the window
decorations, or instructed their window manager to close it.

Set this to `"NeverPrompt"` if you don't like confirming closing
windows every time.

```lua
config.window_close_confirmation = 'AlwaysPrompt'
```

See also
[skip_close_confirmation_for_processes_named](../config/skip_close_confirmation_for_processes_named.md).

The default `CTRL-SHIFT-W` and `CMD-w` key assignments also respect
`"NeverPrompt"` when closing the final tab in a window. Closing a tab while
other tabs remain still uses the normal tab-close confirmation behavior. When
the final tab belongs to a multiplexer session, the shortcut closes the window
without destroying that session. You can override the tab-close behavior as
shown in the
[CloseCurrentTab](../keyassignment/CloseCurrentTab.md) documentation.
