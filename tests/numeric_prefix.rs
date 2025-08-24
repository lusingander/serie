use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn is_valid_numeric_prefix_char(c: char, current_prefix: &str) -> bool {
    c.is_ascii_digit() && (c != '0' || !current_prefix.is_empty())
}

fn simulate_numeric_input(inputs: &[char]) -> String {
    let mut prefix = String::new();

    for &c in inputs {
        if is_valid_numeric_prefix_char(c, &prefix) {
            prefix.push(c);
        } else {
            break;
        }
    }

    prefix
}

#[test]
fn test_numeric_prefix_single_digit() {
    assert_eq!(simulate_numeric_input(&['5']), "5");
    assert_eq!(simulate_numeric_input(&['1']), "1");
    assert_eq!(simulate_numeric_input(&['9']), "9");
}

#[test]
fn test_numeric_prefix_multiple_digits() {
    assert_eq!(simulate_numeric_input(&['1', '2', '3']), "123");
    assert_eq!(simulate_numeric_input(&['4', '2']), "42");
    assert_eq!(simulate_numeric_input(&['9', '9', '9']), "999");
}

#[test]
fn test_numeric_prefix_zero_rules() {
    // Zero should not be accepted as the first character
    assert_eq!(simulate_numeric_input(&['0']), "");

    // But zero should be accepted after other digits
    assert_eq!(simulate_numeric_input(&['1', '0']), "10");
    assert_eq!(simulate_numeric_input(&['2', '0', '5']), "205");
}

#[test]
fn test_numeric_prefix_stops_at_non_digit() {
    assert_eq!(simulate_numeric_input(&['5', 'j']), "5");
    assert_eq!(simulate_numeric_input(&['1', '2', 'k']), "12");
    assert_eq!(simulate_numeric_input(&['3', ' ']), "3");
}

#[test]
fn test_numeric_prefix_empty_for_non_numeric_start() {
    assert_eq!(simulate_numeric_input(&['j']), "");
    assert_eq!(simulate_numeric_input(&['k']), "");
    assert_eq!(simulate_numeric_input(&[' ']), "");
}

#[test]
fn test_numeric_prefix_parsing() {
    assert_eq!("5".parse::<usize>().unwrap_or(1), 5);
    assert_eq!("42".parse::<usize>().unwrap_or(1), 42);
    assert_eq!("999".parse::<usize>().unwrap_or(1), 999);
    assert_eq!("".parse::<usize>().unwrap_or(1), 1);
    assert_eq!("abc".parse::<usize>().unwrap_or(1), 1);
}

#[test]
fn test_key_event_digit_detection() {
    let digit_events = [
        KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('5'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('9'), KeyModifiers::empty()),
    ];

    for event in digit_events {
        if let KeyCode::Char(c) = event.code {
            assert!(c.is_ascii_digit());
        }
    }

    let non_digit_events = [
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
    ];

    for event in non_digit_events {
        if let KeyCode::Char(c) = event.code {
            assert!(!c.is_ascii_digit())
        }
    }
}
