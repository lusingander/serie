# Custom Keybindings

You can set your own custom key bindings.

Custom key bindings can be applied by writing them in the `[keybind]` section of [the config file](../configurations/config-file-format.md).

The default key binding settings are described in [`./assets/default-keybind.toml`](https://github.com/lusingander/serie/blob/master/assets/default-keybind.toml).
You can set key bindings for each action in the same format.

- It is possible to set multiple key bindings for one action.
- If you do not set key bindings for an action, the default key bindings will be assigned.
- You can disable an action by setting `[]` as the key bindings.

## Key Formats

You can use the following formats to define key bindings.

### Modifier Keys

- `ctrl-`
- `alt-`
- `shift-`

Modifiers can be combined, for example: `ctrl-shift-a`.

### Special Keys

| Key | Description |
| --- | --- |
| `esc` | Escape |
| `enter` | Enter |
| `left` | Left arrow |
| `right` | Right arrow |
| `up` | Up arrow |
| `down` | Down arrow |
| `home` | Home |
| `end` | End |
| `pageup` | Page Up |
| `pagedown` | Page Down |
| `backtab` | Back Tab (Shift + Tab) |
| `backspace` | Backspace |
| `delete` | Delete |
| `insert` | Insert |
| `f1` - `f12` | Function keys |
| `space` | Space |
| `hyphen`, `minus` | Hyphen (-) |
| `tab` | Tab |

### Character Keys

Any single character not listed above (e.g., `a`, `b`, `1`, `!`) can be used as a key.
