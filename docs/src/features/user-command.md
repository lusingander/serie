# User Command

The User command view allows you to display the output (stdout) of your custom external commands.
This allows you to do things like view commit diffs using your favorite tools.

To define a user command, you need to configure the following two settings:
- Keybinding definition. Specify the key to display each user command.
  - Config: `keybind.user_command_{n}`
- Command definition. Specify the actual command you want to execute.
  - Config: `core.user_command.commands_{n}`

**Configuration example:**

```toml
[keybind]
user_command_1 = ["d"]
user_command_2 = ["shift-d"]

[core.user_command]
commands_1 = { "name" = "git diff", commands = ["git", "--no-pager", "diff", "--color=always", "{{first_parent_hash}}", "{{target_hash}}"] }
commands_2 = { "name" = "xxx", commands = ["xxx", "{{first_parent_hash}}", "{{target_hash}}", "--width", "{{area_width}}", "--height", "{{area_height}}"] }
```

## Variables

The following variables can be used in command definitions.
They will be replaced with their respective values command is executed.

- `{{target_hash}}`
  - The hash of the selected commit.
  - example: `b0ce4cb9c798576af9b4accc9f26ddce5e72063d`
- `{{first_parent_hash}}`
  - The hash of the first parent of the selected commit.
  - example: `c103d9744df8ebf100773a11345f011152ec5581`
- `{{area_width}}`
  - Width of the user command display area (number of cells).
  - example: `80`
- `{{area_height}}`
  - Height of the user command display area (number of cells).
  - example: `30`

