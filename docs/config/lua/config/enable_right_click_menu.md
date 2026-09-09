# `enable_right_click_menu = true`

When enabled, right-clicking a terminal pane opens a compact action menu for
copying, pasting, selecting all retained text, and splitting the pane.

The menu is enabled by default. Set this option to `false` to restore normal
right-click mouse bindings and terminal mouse reporting:

```lua
config.enable_right_click_menu = false
```

The menu uses the same Ubuntu 10-point UI font as the reference by default. It can be adjusted with
`right_click_menu_font_size`.

Its colors can be adjusted with `right_click_menu_colors`.
