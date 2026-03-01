use std::{cell::RefCell, process::Command};

use arboard::Clipboard;

use crate::config::ClipboardConfig;

const USER_COMMAND_TARGET_HASH_MARKER: &str = "{{target_hash}}";
const USER_COMMAND_FIRST_PARENT_HASH_MARKER: &str = "{{first_parent_hash}}";
const USER_COMMAND_PARENT_HASHES_MARKER: &str = "{{parent_hashes}}";
const USER_COMMAND_REFS_MARKER: &str = "{{refs}}";
const USER_COMMAND_BRANCHES_MARKER: &str = "{{branches}}";
const USER_COMMAND_REMOTE_BRANCHES_MARKER: &str = "{{remote_branches}}";
const USER_COMMAND_TAGS_MARKER: &str = "{{tags}}";
const USER_COMMAND_AREA_WIDTH_MARKER: &str = "{{area_width}}";
const USER_COMMAND_AREA_HEIGHT_MARKER: &str = "{{area_height}}";

thread_local! {
    static CLIPBOARD: RefCell<Option<Clipboard>> = const { RefCell::new(None) };
}

pub fn copy_to_clipboard(value: String, config: &ClipboardConfig) -> Result<(), String> {
    match config {
        ClipboardConfig::Auto => copy_to_clipboard_auto(value),
        ClipboardConfig::Custom { commands } => copy_to_clipboard_custom(value, commands),
    }
}

fn copy_to_clipboard_custom(value: String, commands: &[String]) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;

    if commands.is_empty() {
        return Err("No clipboard command specified".to_string());
    }

    let mut child = Command::new(&commands[0])
        .args(&commands[1..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {e}", commands[0]))?;

    child
        .stdin
        .take()
        .expect("stdin should be available")
        .write_all(value.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {e}", commands[0]))?;

    child
        .wait()
        .map_err(|e| format!("{} failed: {e}", commands[0]))?;

    Ok(())
}

fn copy_to_clipboard_auto(value: String) -> Result<(), String> {
    CLIPBOARD.with_borrow_mut(|clipboard| {
        if clipboard.is_none() {
            *clipboard = Clipboard::new()
                .map(Some)
                .map_err(|e| format!("Failed to create clipboard: {e:?}"))?;
        }

        clipboard
            .as_mut()
            .expect("The clipboard should have been initialized above")
            .set_text(value)
            .map_err(|e| format!("Failed to copy to clipboard: {e:?}"))
    })
}

pub struct ExternalCommandParameters {
    pub command: Vec<String>,
    pub target_hash: String,
    pub parent_hashes: Vec<String>,
    pub all_refs: Vec<String>,
    pub branches: Vec<String>,
    pub remote_branches: Vec<String>,
    pub tags: Vec<String>,
    pub area_width: u16,
    pub area_height: u16,
}

pub fn exec_user_command(params: ExternalCommandParameters) -> Result<String, String> {
    let sep = " ";
    let first_parent_hash = params.parent_hashes.first().cloned().unwrap_or_default();
    let command = params
        .command
        .iter()
        .map(|s| {
            s.replace(USER_COMMAND_TARGET_HASH_MARKER, &params.target_hash)
                .replace(USER_COMMAND_FIRST_PARENT_HASH_MARKER, &first_parent_hash)
                .replace(
                    USER_COMMAND_PARENT_HASHES_MARKER,
                    &params.parent_hashes.join(sep),
                )
                .replace(USER_COMMAND_REFS_MARKER, &params.all_refs.join(sep))
                .replace(USER_COMMAND_BRANCHES_MARKER, &params.branches.join(sep))
                .replace(
                    USER_COMMAND_REMOTE_BRANCHES_MARKER,
                    &params.remote_branches.join(sep),
                )
                .replace(USER_COMMAND_TAGS_MARKER, &params.tags.join(sep))
                .replace(
                    USER_COMMAND_AREA_WIDTH_MARKER,
                    &params.area_width.to_string(),
                )
                .replace(
                    USER_COMMAND_AREA_HEIGHT_MARKER,
                    &params.area_height.to_string(),
                )
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
