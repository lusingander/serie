# Compatibility

## Supported terminal emulators

Serie supports three rendering modes:

- [Inline Images Protocol (iTerm2)](https://iterm2.com/documentation-images.html) — high-quality PNG images
- [Terminal graphics protocol (kitty)](https://sw.kovidgoyal.net/kitty/graphics-protocol/) — high-quality PNG images
  - Supports both the existing graphics protocol mode and [the Unicode placeholder](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders) mode.
- **ASCII / Unicode box-drawing fallback** — works in any terminal (gnome-terminal, alacritty, xterm, Windows Terminal, etc.). Used automatically when `auto` cannot identify a graphics-capable terminal, or by passing `--protocol ascii`.

The terminals on which each image protocol has been confirmed to work are listed below.

### Inline Images Protocol

| Terminal emulator                                                                   | Note                                                                                                                                         |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| [iTerm2](https://iterm2.com)                                                        | But slower than other terminals                                                                                                              |
| [WezTerm](https://wezfurlong.org/wezterm/)                                          |                                                                                                                                              |
| [Rio](https://rioterm.com)                                                          |                                                                                                                                              |
| [VSCode integrated terminal](https://code.visualstudio.com/docs/terminal/basics) \* | Requires the [`terminal.integrated.enableImages` setting](https://code.visualstudio.com/docs/terminal/advanced#_image-support) to be enabled |

\*Not only the VSCode integrated terminal, but any terminal emulator using [xterm.js](https://xtermjs.org) may basically work in the same way as long as [image display feature is enabled](https://github.com/xtermjs/xterm.js/tree/master/addons/addon-image).

### Terminal graphics protocol

| Terminal emulator                         | Unicode placeholder | Note |
| ----------------------------------------- | ------------------- | ---- |
| [kitty](https://sw.kovidgoyal.net/kitty/) | ○                  |      |
| [Ghostty](https://ghostty.org)            | ○                  |      |

Rendering using Unicode Placeholder is available by explicitly specifying `kitty-unicode` as `protocol` option or config.

### ASCII fallback

The `ascii` protocol renders graphs with Unicode box-drawing characters and works in any terminal — no image protocol required. Auto-detect falls back to this mode when none of the image protocols above can be identified.

It honors:

- `--graph-style rounded` (default) — uses `╭ ╮ ╰ ╯` for corners.
- `--graph-style angular` — uses `┌ ┐ └ ┘` for corners.
- `--graph-width single` — one character per branch column.
- `--graph-width double` — two characters per branch column, with `○` and `> / <` arrow markers on merge commits.

### Partially supported environments

- tmux is supported when using the kitty Unicode placeholder protocol (`kitty-unicode`) or the ASCII fallback (`ascii`). It is not supported with the plain `kitty` or `iterm` image protocols.
  - The kitty Unicode placeholder requires `set -g allow-passthrough on` in tmux.conf (version 3.2+). The ASCII fallback has no special requirements.

### Unsupported environments

- Sixel graphics is not supported (for image rendering). The `ascii` fallback still works.
- Other terminal multiplexers (screen, Zellij, etc.) are not supported for image rendering. The `ascii` fallback still works.
- Windows is not officially supported. Please refer to [the related issue](https://github.com/lusingander/serie/issues/147#issuecomment-4192875627).
