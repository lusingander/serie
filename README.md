# Serie

[![Crate Status](https://img.shields.io/crates/v/serie.svg)](https://crates.io/crates/serie)
[![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=000&color=fff)](https://ratatui.rs)

A rich git commit graph in your terminal, like magic 📚

<img src="./img/demo.gif">

(This demo shows [Ratatui](https://github.com/ratatui/ratatui) repository!)

## About

Serie ([`/zéːriə/`](https://lusingander.github.io/serie/faq/index.html#how-do-i-pronounce-serie)) is a TUI application that uses the terminal emulators' image display protocol to render commit graphs like `git log --graph --all`.

### Why?

While some users prefer to use Git via CLI, they often rely on a GUI or feature-rich TUI to view commit logs. Others may find `git log --graph` sufficient.

Personally, I found the output from `git log --graph` difficult to read, even with additional options. Learning complex tools just to view logs seemed cumbersome.

### Goals

- Provide a rich `git log --graph` experience in the terminal.
- Offer commit graph-centric browsing of Git repositories.

### Non-Goals

- Implement a fully-featured Git client.
- Create a TUI application with a complex UI.
- Works in any terminal environment.

## Documentation

For detailed usage, configuration, and advanced features, see [the full documentation](https://lusingander.github.io/serie/).

## Requirements

- Git
- Supported terminal emulator
  - Refer to [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html) for details.

## Installation

If you're using Cargo:

```
$ cargo install --locked serie
```

For other download options, see [Installation](https://lusingander.github.io/serie/getting-started/installation.html).

## Usage

### Basic

Run `serie` in the directory where your git repository exists.

```
$ cd <your git repository>
$ serie
```

### Options

```
Serie - A rich git commit graph in your terminal, like magic 📚

Usage: serie [OPTIONS]

Options:
  -p, --protocol <TYPE>           Image protocol to render graph [default: auto] [possible values: auto, iterm, kitty]
  -o, --order <TYPE>              Commit ordering algorithm [default: chrono] [possible values: chrono, topo]
  -g, --graph-width <TYPE>        Commit graph image cell width [default: auto] [possible values: auto, double, single]
  -s, --graph-style <TYPE>        Commit graph image edge style [default: rounded] [possible values: rounded, angular]
  -i, --initial-selection <TYPE>  Initial selection of commit [default: latest] [possible values: latest, head]
      --preload                   Preload all graph images
  -h, --help                      Print help
  -V, --version                   Print version
```

For details on each option, see [Command Line Options](https://lusingander.github.io/serie/getting-started/command-line-options.html).

### Keybindings

You can see the keybindings by pressing the `?` key.

The [default key bindings](https://lusingander.github.io/serie/keybindings/index.html) can be overridden. See [Custom Keybindings](https://lusingander.github.io/serie/keybindings/custom-keybindings.html) for more information.

### Config

Config files are loaded in the following order of priority:

- `$SERIE_CONFIG_FILE`
  - If `$SERIE_CONFIG_FILE` is set but the file does not exist, an error occurs.
- `$XDG_CONFIG_HOME/serie/config.toml`
  - If `$XDG_CONFIG_HOME` is not set, `~/.config/` will be used instead.

If the config file does not exist, the default values will be used for all items.
If the config file exists but some items are not set, the default values will be used for those unset items.

For detailed information about the config file format, see [Config File Format](https://lusingander.github.io/serie/configurations/config-file-format.html).

### User command

The User command view allows you to display the output (stdout) of your custom external commands.
This allows you to do things like view commit diffs using your favorite tools.

For details on how to set commands, see [User Command](https://lusingander.github.io/serie/features/user-command.html).

## Compatibility

### Supported terminals

These image protocols are supported:

- [Inline Images Protocol (iTerm2)](https://iterm2.com/documentation-images.html)
- [Terminal graphics protocol (kitty)](https://sw.kovidgoyal.net/kitty/graphics-protocol/)

For more information, see [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html).

### Unsupported environments

- Sixel graphics is not supported.
- Terminal multiplexers (screen, tmux, Zellij, etc.) are not supported.

## Screenshots

<img src="./img/list.png" width=600>
<img src="./img/detail.png" width=600>
<img src="./img/refs.png" width=600>
<img src="./img/searching.png" width=600>
<img src="./img/applied.png" width=600>
<img src="./img/diff_git.png" width=600>
<img src="./img/diff_difft.png" width=600>

The following repositories are used as these examples:

- [ratatui/ratatui](https://github.com/ratatui/ratatui)
- [charmbracelet/vhs](https://github.com/charmbracelet/vhs)
- [lusingander/stu](https://github.com/lusingander/stu)

## Contributing

To get started with contributing, please review [CONTRIBUTING.md](CONTRIBUTING.md).

Contributions that do not follow these guidelines may not be accepted.

## License

MIT
