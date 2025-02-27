use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de::Deserializer, Deserialize};

use crate::event::UserEvent;

const DEFAULT_KEY_BIND: &str = include_str!("../assets/default-keybind.toml");

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyBind(HashMap<KeyEvent, UserEvent>);

impl Deref for KeyBind {
    type Target = HashMap<KeyEvent, UserEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyBind {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl KeyBind {
    pub fn new(custom_keybind_patch: Option<KeyBind>) -> Self {
        let mut keybind: KeyBind =
            toml::from_str(DEFAULT_KEY_BIND).expect("default key bind should be correct");

        if let Some(mut custom_keybind_patch) = custom_keybind_patch {
            for (key_event, user_event) in custom_keybind_patch.drain() {
                keybind.insert(key_event, user_event);
            }
        }

        keybind
    }

    pub fn keys_for_event(&self, user_event: UserEvent) -> Vec<String> {
        let mut key_events: Vec<KeyEvent> = self
            .iter()
            .filter(|(_, ue)| **ue == user_event)
            .map(|(ke, _)| *ke)
            .collect();
        key_events.sort_by(|a, b| a.partial_cmp(b).unwrap()); // At least when used for key bindings, it doesn't seem to be a problem...
        key_events.into_iter().map(key_event_to_string).collect()
    }
}

impl<'de> Deserialize<'de> for KeyBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<UserEvent, Vec<String>>::deserialize(deserializer)?;
        let mut key_map = HashMap::<KeyEvent, UserEvent>::new();
        for (user_event, key_events) in parsed_map {
            for key_event_str in key_events {
                let key_event = match parse_key_event(&key_event_str) {
                    Ok(e) => e,
                    Err(s) => {
                        let msg = format!("{key_event_str:?} is not a valid key event: {s:}");
                        return Err(serde::de::Error::custom(msg));
                    }
                };
                if let Some(conflict_user_event) = key_map.insert(key_event, user_event) {
                    let msg = format!(
                        "{:?} map to multiple events: {:?}, {:?}",
                        key_event, user_event, conflict_user_event
                    );
                    return Err(serde::de::Error::custom(msg));
                }
            }
        }

        Ok(KeyBind(key_map))
    }
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase().replace(' ', "");
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

fn key_event_to_string(key_event: KeyEvent) -> String {
    if let KeyCode::Char(c) = key_event.code {
        if key_event.modifiers == KeyModifiers::SHIFT {
            return c.to_ascii_uppercase().into();
        }
    }

    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "Backspace",
        KeyCode::Enter => "Enter",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "BackTab",
        KeyCode::Delete => "Delete",
        KeyCode::Insert => "Insert",
        KeyCode::F(n) => {
            char = format!("F{n}");
            &char
        }
        KeyCode::Char(' ') => "Space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "Esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("Ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("Shift");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("Alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn test_deserialize_keybind() {
        let toml = r#"
            navigate_up = ["k"]
            navigate_down = ["j", "down"]
            navigate_left = ["ctrl-h", "shift-h", "alt-h"]
            navigate_right = ["ctrl-shift-l", "alt-shift-ctrl-l"]
            quit = ["esc", "f12"]
        "#;

        let expected = KeyBind(
            [
                (
                    KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
                    UserEvent::NavigateUp,
                ),
                (
                    KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
                    UserEvent::NavigateDown,
                ),
                (
                    KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
                    UserEvent::NavigateDown,
                ),
                (
                    KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
                    UserEvent::NavigateLeft,
                ),
                (
                    KeyEvent::new(KeyCode::Char('h'), KeyModifiers::SHIFT),
                    UserEvent::NavigateLeft,
                ),
                (
                    KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT),
                    UserEvent::NavigateLeft,
                ),
                (
                    KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
                    UserEvent::NavigateRight,
                ),
                (
                    KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT),
                    UserEvent::NavigateRight,
                ),
                (
                    KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
                    UserEvent::Quit,
                ),
                (
                    KeyEvent::new(KeyCode::F(12), KeyModifiers::empty()),
                    UserEvent::Quit,
                ),
            ]
            .into_iter()
            .collect(),
        );

        let actual: KeyBind = toml::from_str(toml).unwrap();

        assert_eq!(actual, expected);
    }

    #[rustfmt::skip]
    #[test]
    fn test_key_event_to_string() {
        let key_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert_eq!(key_event_to_string(key_event), "k");

        let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert_eq!(key_event_to_string(key_event), "j");

        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert_eq!(key_event_to_string(key_event), "Down");

        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
        assert_eq!(key_event_to_string(key_event), "Ctrl-h");

        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::SHIFT);
        assert_eq!(key_event_to_string(key_event), "H");

        let key_event = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT);
        assert_eq!(key_event_to_string(key_event), "H");

        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
        assert_eq!(key_event_to_string(key_event), "Shift-Left");

        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT);
        assert_eq!(key_event_to_string(key_event), "Alt-h");

        let key_event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(key_event_to_string(key_event), "Ctrl-Shift-l");

        let key_event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT);
        assert_eq!(key_event_to_string(key_event), "Ctrl-Shift-Alt-l");

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert_eq!(key_event_to_string(key_event), "Esc");

        let key_event = KeyEvent::new(KeyCode::F(12), KeyModifiers::empty());
        assert_eq!(key_event_to_string(key_event), "F12");
    }
}
