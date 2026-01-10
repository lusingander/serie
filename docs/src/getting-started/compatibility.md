# Compatibility

## Supported terminal emulators

These image protocols are supported:

- [Inline Images Protocol (iTerm2)](https://iterm2.com/documentation-images.html)
- [Terminal graphics protocol (kitty)](https://sw.kovidgoyal.net/kitty/graphics-protocol/)

The terminals on which each has been confirmed to work are listed below.

### Inline Images Protocol

| Terminal emulator                                                                   | Note                                                                                                                                         |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| [iTerm2](https://iterm2.com)                                                        | But slower than other terminals                                                                                                              |
| [WezTerm](https://wezfurlong.org/wezterm/)                                          |                                                                                                                                              |
| [VSCode integrated terminal](https://code.visualstudio.com/docs/terminal/basics) \* | Requires the [`terminal.integrated.enableImages` setting](https://code.visualstudio.com/docs/terminal/advanced#_image-support) to be enabled |

\*Not only the VSCode integrated terminal, but any terminal emulator using [xterm.js](https://xtermjs.org) may basically work in the same way as long as [image display feature is enabled](https://github.com/xtermjs/xterm.js/tree/master/addons/addon-image).

### Terminal graphics protocol

| Terminal emulator                         | Note |
| ----------------------------------------- | ---- |
| [kitty](https://sw.kovidgoyal.net/kitty/) |      |
| [Ghostty](https://ghostty.org)            |      |

## Unsupported environments

- Sixel graphics is not supported.
- Terminal multiplexers (screen, tmux, Zellij, etc.) are not supported.
