# Config File Format

## Example

```toml
[core.option]
protocol = "auto"
order = "chrono"
graph_width = "auto"
graph_style = "rounded"
initial_selection = "latest"

[core.search]
ignore_case = false
fuzzy = false

[core.user_command]
commands_1 = { name = "git diff", commands = ["git", "--no-pager", "diff", "--color=always", "{{first_parent_hash}}", "{{target_hash}}"]}
tab_width = 4

[core.external]
clipboard = "Auto"

[ui.common]
cursor_type = "Native"

[ui.list]
columns = ["graph", "marker", "subject", "name", "hash", "date"]
subject_min_width = 20
date_format = "%Y-%m-%d"
date_width = 10
date_local = true
name_width = 20

[ui.detail]
height = 20
date_format = "%Y-%m-%d %H:%M:%S %z"
date_local = true

[ui.user_command]
height = 20

[ui.refs]
width = 26

[graph.color]
branches = [
  "#E06C76",
  "#98C379",
  "#E5C07B",
  "#61AFEF",
  "#C678DD",
  "#56B6C2",
]
edge = "#00000000"
background = "#00000000"

[color]
fg = "reset"
bg = "reset"
list_selected_fg = "white"
list_selected_bg = "dark-gray"
list_ref_paren_fg = "yellow"
list_ref_branch_fg = "green"
list_ref_remote_branch_fg = "red"
list_ref_tag_fg = "yellow"
list_ref_stash_fg = "magenta"
list_head_fg = "cyan"
list_subject_fg = "reset"
list_name_fg = "cyan"
list_hash_fg = "yellow"
list_date_fg = "magenta"
list_match_fg = "black"
list_match_bg = "yellow"
detail_label_fg = "reset"
detail_name_fg = "reset"
detail_date_fg = "reset"
detail_email_fg = "blue"
detail_hash_fg = "reset"
detail_ref_branch_fg = "green"
detail_ref_remote_branch_fg = "red"
detail_ref_tag_fg = "yellow"
detail_file_change_add_fg = "green"
detail_file_change_modify_fg = "yellow"
detail_file_change_delete_fg = "red"
detail_file_change_move_fg = "magenta"
ref_selected_fg = "white"
ref_selected_bg = "dark-gray"
help_block_title_fg = "green"
help_key_fg = "yellow"
virtual_cursor_fg = "reset"
status_input_fg = "reset"
status_input_transient_fg = "dark-gray"
status_info_fg = "cyan"
status_success_fg = "green"
status_warn_fg = "yellow"
status_error_fg = "red"
divider_fg = "dark-gray"

[keybind]
# See the separate Custom Keybindings section for details.
# ...
```

## Configuration Options

### `core.option.protocol`

The protocol type for rendering images of commit graphs.

- type: `string` (enum)
- default: `auto`
- possible values:
  - `auto`
  - `iterm`
  - `kitty`

The value specified in the command line argument takes precedence.

### `core.option.order`

The commit ordering algorithm.

- type: `string` (enum)
- default: `chrono`
- possible values:
  - `chrono`
  - `topo`

The value specified in the command line argument takes precedence.

### `core.option.graph_width`

The character width that a graph image unit cell occupies.

- type: `string` (enum)
- default: `auto`
- possible values:
  - `auto`
  - `double`
  - `single`

The value specified in the command line argument takes precedence.

### `core.option.graph_style`

The commit graph image edge style.

- type: `string` (enum)
- default: `rounded`
- possible values:
  - `rounded`
  - `angular`

The value specified in the command line argument takes precedence.

### `core.option.initial_selection`

The initial selection of commit when starting the application.

- type: `string` (enum)
- default: `latest`
- possible values:
  - `latest`
  - `head`

The value specified in the command line argument takes precedence.

### `core.search.ignore_case`

Whether to enable ignore case by default.

- type: `boolean`
- default: `false`

### `core.search.fuzzy`

Whether to enable fuzzy matching by default.

- type: `boolean`
- default: `false`

### `core.user_command.commands_{n}`

The command definition for generating the content displayed in the user command view.

Multiple commands can be specified in the format `commands_{n}`.
For details about user command, see the separate [User command](../features/user-command.md) section.

- type: `object`
- fields:
  - `name`: `string` - The name of the user command.
  - `commands`: `array of strings` - The command and its arguments.
- examples:
    - `commands_1 = { name = "git diff", commands = ["git", "--no-pager", "diff", "--color=always", "{{first_parent_hash}}", "{{target_hash}}"]}`
  
### `core.user_command.tab_width`

The number of spaces to replace tabs in the user command output.

- type: `u16`
- default: `4`

### `core.external.clipboard`

The clipboard command to use for copy operations.

- type: `object` (enum)
- default: `Auto`
- possible values:
  - `Auto`: Use the default clipboard library
  - `{ Custom = { commands = ["..."] } }`: Use a custom command that receives text via stdin
    - `commands`: `array of strings` - The command and its arguments.
- examples:
    - `clipboard = "Auto"`
    - `clipboard = { Custom = { commands = ["wl-copy"] } }`
    - `clipboard = { Custom = { commands = ["xclip", "-selection", "clipboard"] } }`

### `ui.common.cursor_type`

The type of a cursor to display in the input.

- type: `object` (enum)
- default: `Native`
- possible values:
  - `Native`: Use the terminal native cursor.
  - `{ Virtual = "|" }`: Use a virtual cursor with the specified string.
    - value: `string` - The string to display as the virtual cursor.

### `ui.list.columns`

The order and visibility of columns in the commit list.

- type: `array of strings` (enum)
- default: `["graph", "marker", "subject", "name", "hash", "date"]`
- possible values:
  - `graph`
  - `marker`
  - `subject`
  - `name`
  - `hash`
  - `date`

### `ui.list.subject_min_width`

The minimum width of a subject in the commit list.

- type: `u16`
- default: `20`

### `ui.list.date_format`

The date format of a author date in the commit list.

- type: `string`
- default: `"%Y-%m-%d"`

The format must be specified in strftime format.
https://docs.rs/chrono/latest/chrono/format/strftime/index.html

### `ui.list.date_width`

The width of a author date in the commit list.

- type: `u16`
- default: `10`

### `ui.list.date_local`

Whether to show a author date in the commit list in local timezone.

- type: `boolean`
- default: `true`

### `ui.list.name_width`

The width of a author name in the commit list.

- type: `u16`
- default: `20`

### `ui.detail.height`

The height of a commit detail area.

- type: `u16`
- default: `20`

### `ui.detail.date_format`

The date format of a author/committer date in the commit detail.

- type: `string`
- default: `"%Y-%m-%d %H:%M:%S %z"`

The format must be specified in strftime format.
https://docs.rs/chrono/latest/chrono/format/strftime/index.html

### `ui.detail.date_local`

Whether to show a author/committer date in the commit list in local timezone.

- type: `boolean`
- default: `true`

### `ui.user_command.height`

The height of a user command area.

- type: `u16`
- default: `20`

### `ui.refs.width`

The width of a refs list area.

- type: `u16`
- default: `26`

### `graph.color.branches`

Array of colors used for the commit graph.

- type: `array of strings`
- default:
  - `"#E06C76"`
  - `"#98C379"`
  - `"#E5C07B"`
  - `"#61AFEF"`
  - `"#C678DD"`
  - `"#56B6C2"`

Colors should be specified in the format `#RRGGBB` or `#RRGGBBAA`.

### `graph.color.edge`

Color of the edge surrounding the commit circles in the graph.

- type: `string`
- default: `"#00000000"`

Colors should be specified in the format `#RRGGBB` or `#RRGGBBAA`.

### `graph.color.background`

Background color of the commit graph.

- type: `string`
- default: `"#00000000"`

Colors should be specified in the format `#RRGGBB` or `#RRGGBBAA`.

### `color`

The colors of each element of the application.

Note: Graph colors are specified with `[graph.color]`.

- type: `string`
- default: see the example above

Colors should be specified in one of the following formats:

- ANSI color name
  - `"red"`, `"bright-blue"`, `"light-red"`, `"reset"`, ...
- 8-bit color (256-color) index values
  - `"34"`, `"128"`, `"255"`, ...
- 24-bit true color hex codes
  - `"#abcdef"`, ...

### `keybind`

Key bindings for various actions in the application.

See the separate [Custom Keybindings](../keybindings/custom-keybindings.md) section for details.
