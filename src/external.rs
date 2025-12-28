use std::{cell::RefCell, process::Command};

use arboard::Clipboard;

const USER_COMMAND_TARGET_HASH_MARKER: &str = "{{target_hash}}";
const USER_COMMAND_FIRST_PARENT_HASH_MARKER: &str = "{{first_parent_hash}}";
const USER_COMMAND_AREA_WIDTH_MARKER: &str = "{{area_width}}";
const USER_COMMAND_AREA_HEIGHT_MARKER: &str = "{{area_height}}";

thread_local! {
    static CLIPBOARD: RefCell<Option<Clipboard>> = const { RefCell::new(None) };
}

// arboard may use X11 via XWayland on Wayland sessions, causing silent clipboard failures
pub fn copy_to_clipboard(value: String) -> Result<(), String> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        copy_to_clipboard_wayland(value)
    } else {
        copy_to_clipboard_arboard(value)
    }
}

fn copy_to_clipboard_wayland(value: String) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run wl-copy: {e}"))?;

    child
        .stdin
        .take()
        .expect("stdin should be available")
        .write_all(value.as_bytes())
        .map_err(|e| format!("Failed to write to wl-copy: {e}"))?;

    child.wait().map_err(|e| format!("wl-copy failed: {e}"))?;

    Ok(())
}

fn copy_to_clipboard_arboard(value: String) -> Result<(), String> {
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

pub fn exec_user_command(
    command: &[&str],
    target_hash: &str,
    first_parent_hash: &str,
    area_width: u16,
    area_height: u16,
) -> Result<String, String> {
    let command = command
        .iter()
        .map(|s| {
            s.replace(USER_COMMAND_TARGET_HASH_MARKER, target_hash)
                .replace(USER_COMMAND_FIRST_PARENT_HASH_MARKER, first_parent_hash)
                .replace(USER_COMMAND_AREA_WIDTH_MARKER, &area_width.to_string())
                .replace(USER_COMMAND_AREA_HEIGHT_MARKER, &area_height.to_string())
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
