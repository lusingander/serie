# User Command

The User command feature allows you to execute custom external commands.
There are two types of user commands: `inline` and `silent`.

- `inline` (default)
  - Displays the output (stdout) of the command in a dedicated view within the TUI.
  - This allows you to do things like view commit diffs using your favorite tools.
- `silent`
  - Executes the command in the background without opening a view.
  - This is useful for operations that don't require checking output, such as deleting branches or adding tags.

To define a user command, you need to configure the following two settings:
- Keybinding definition. Specify the key to execute each user command.
  - Config: `keybind.user_command_{n}`
- Command definition. Specify the actual command you want to execute.
  - Config: `core.user_command.commands_{n}`

**Configuration example:**

```toml
[keybind]
user_command_1 = ["d"]
user_command_2 = ["shift-d"]
user_command_3 = ["b"]

[core.user_command]
# Inline command (default)
commands_1 = { "name" = "git diff", commands = ["git", "--no-pager", "diff", "--color=always", "{{first_parent_hash}}", "{{target_hash}}"] }
# Inline command with custom area size
commands_2 = { "name" = "xxx", commands = ["xxx", "{{first_parent_hash}}", "{{target_hash}}", "--width", "{{area_width}}", "--height", "{{area_height}}"] }
# Silent command with refresh
commands_3 = { "name" = "delete branch", type = "silent", commands = ["git", "branch", "-D", "{{branches}}"], refresh = true }
```

## Refresh

For `silent` commands, you can set `refresh = true` to automatically reload the repository and refresh the display (e.g., commit list) after the command is executed.
This is useful when the command modifies the repository state.

Note that `refresh = true` cannot be used with `inline` commands.

## Variables

The following variables can be used in command definitions.
They will be replaced with their respective values command is executed.

- `{{target_hash}}`
  - The hash of the selected commit.
  - example: `b0ce4cb9c798576af9b4accc9f26ddce5e72063d`
- `{{first_parent_hash}}`
  - The hash of the first parent of the selected commit.
  - example: `c103d9744df8ebf100773a11345f011152ec5581`
- `{{parent_hashes}}`
  - The hashes of all parents of the selected commit, separated by a space.
  - example: `c103d9744df8ebf100773a11345f011152ec5581 a1b2c3d4e5f67890123456789abcdef0123456789`
- `{{refs}}`
  - The names of all refs (branches, tags, stashes) pointing to the selected commit, separated by a space.
  - example: `master v1.0.0`
- `{{branches}}`
  - The names of all branches pointing to the selected commit, separated by a space.
  - example: `master feature-branch`
- `{{remote_branches}}`
  - The names of all remote branches pointing to the selected commit, separated by a space.
  - example: `origin/master origin/feature-branch`
- `{{tags}}`
  - The names of all tags pointing to the selected commit, separated by a space.
  - example: `v1.0.0 v1.0.1`
- `{{area_width}}`
  - Width of the user command display area (number of cells).
  - example: `80`
- `{{area_height}}`
  - Height of the user command display area (number of cells).
  - example: `30`

