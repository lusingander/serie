use arboard::Clipboard;

pub fn copy_to_clipboard(value: String) -> Result<(), String> {
    Clipboard::new()
        .and_then(|mut c| c.set_text(value))
        .map_err(|e| format!("Failed to copy to clipboard: {e:?}"))
}
