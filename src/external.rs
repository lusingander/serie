use std::process::Command;

use arboard::Clipboard;

const USER_COMMAND_TARGET_HASH_MARKER: &str = "{{target_hash}}";
const USER_COMMAND_PARENT_HASH_MARKER: &str = "{{parent_hash}}";

pub fn copy_to_clipboard(value: String) -> Result<(), String> {
    Clipboard::new()
        .and_then(|mut c| c.set_text(value))
        .map_err(|e| format!("Failed to copy to clipboard: {e:?}"))
}

pub fn exec_user_command(
    command: &[&str],
    target_hash: &str,
    parent_hash: &str,
) -> Result<String, String> {
    let command = command
        .iter()
        .map(|s| {
            s.replace(USER_COMMAND_TARGET_HASH_MARKER, target_hash)
                .replace(USER_COMMAND_PARENT_HASH_MARKER, parent_hash)
        })
        .collect::<Vec<_>>();

    let output = Command::new(&command[0])
        .args(&command[1..])
        .output()
        .map_err(|e| format!("Failed to execute command: {e:?}"))?;

    if !output.status.success() {
        let msg = format!(
            "Command exited with non-zero status: {}, stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(msg);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into())
}
