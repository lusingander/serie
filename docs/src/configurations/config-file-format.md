# Config File Format

The values set in this example are the default values.

```toml
[core.option]
# The protocol type for rendering images of commit graphs.
# The value specified in the command line argument takes precedence.
# type: enum (possible values: "auto", "iterm", "kitty")
protocol = "auto"
# The commit ordering algorithm.
# The value specified in the command line argument takes precedence.
# type: enum (possible values: "chrono", "topo")
order = "chrono"
# The character width that a graph image unit cell occupies.
# The value specified in the command line argument takes precedence.
# type: enum (possible values: "auto", "double", "single")
graph_width = "auto"
# The commit graph image edge style.
# The value specified in the command line argument takes precedence.
# type: enum (possible values: "rounded", "angular")
graph_style = "rounded"
# The initial selection of commit when starting the application.
# The value specified in the command line argument takes precedence.
# type: enum (possible values: "latest", "head")
initial_selection = "latest"

[core.search]
# Whether to enable ignore case by default.
# type: boolean
ignore_case = false
# Whether to enable fuzzy matching by default.
# type: boolean
fuzzy = false

[core.user_command]
# The command definition for generating the content displayed in the user command view.
# Multiple commands can be specified in the format commands_{n}.
# For details about user command, see the separate User command section.
# type: object
commands_1 = { name = "git diff", commands = ["git", "--no-pager", "diff", "--color=always", "{{first_parent_hash}}", "{{target_hash}}"]}
# The number of spaces to replace tabs in the user command output.
# type: u16
tab_width = 4

[core.external]
# Configuration for external commands used by the application.
# The clipboard command to use for copy operations.
# - "Auto": Use the default clipboard library
# - { Custom = { commands = ["..."] } }: Use a custom command that receives text via stdin
# type: enum
# Examples:
#   clipboard = "Auto"
#   clipboard = { Custom = { commands = ["wl-copy"] } }
#   clipboard = { Custom = { commands = ["xclip", "-selection", "clipboard"] } }
clipboard = "Auto"

[ui.common]
# The type of a cursor to display in the input.
# If `cursor_type = "Native"` is set, the terminal native cursor is used.
# If `cursor_type = { "Virtual" = "|" }` is set, a virtual cursor with the specified string will be used.
# type: enum
cursor_type = "Native"

[ui.list]
# The minimum width of a subject in the commit list.
# type: u16
subject_min_width = 20
# The date format of a author date in the commit list.
# The format must be specified in strftime format.
# https://docs.rs/chrono/latest/chrono/format/strftime/index.html
# type: string
date_format = "%Y-%m-%d"
# The width of a author date in the commit list.
# type: u16
date_width = 10
# Whether to show a author date in the commit list in local timezone.
# type: boolean
date_local = true
# The width of a author name in the commit list.
# type: u16
name_width = 20

[ui.detail]
# The height of a commit detail area.
# type: u16
height = 20
# The date format of a author/committer date in the commit detail.
# The format must be specified in strftime format.
# https://docs.rs/chrono/latest/chrono/format/strftime/index.html
# type: string
date_format = "%Y-%m-%d %H:%M:%S %z"
# Whether to show a author/committer date in the commit list in local timezone.
# type: boolean
date_local = true

[ui.user_command]
# The height of a user command area.
# type: u16
height = 20

[ui.refs]
# The width of a refs list area.
# type: u16
width = 26

[graph.color]
# Colors should be specified in the format #RRGGBB or #RRGGBBAA.

# Array of colors used for the commit graph.
# type: array of strings
branches = [
  "#E06C76",
  "#98C379",
  "#E5C07B",
  "#61AFEF",
  "#C678DD",
  "#56B6C2",
]
# Color of the edge surrounding the commit circles in the graph.
# type: string
edge = "#00000000"
# Background color of the commit graph.
# type: string
background = "#00000000"

[color]
# The colors of each element of the application.
# Note: Graph colors are specified with [graph.color].
#
# Colors should be specified in one of the following formats:
# - ANSI color name
#   - "red", "bright-blue", "light-red", "reset", ...
# - 8-bit color (256-color) index values
#   - "34", "128", "255", ...
# - 24-bit true color hex codes
#   - "#abcdef", ...
# type: string
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
# See default-keybind.toml for a specific example configuration.
# ...
```
