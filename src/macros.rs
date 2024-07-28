#[macro_export]
macro_rules! key_code {
    ( $code:path ) => {
        ratatui::crossterm::event::KeyEvent { code: $code, .. }
    };
}

#[macro_export]
macro_rules! key_code_char {
    ( $c:ident ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr, Ctrl ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            modifiers: ratatui::crossterm::event::KeyModifiers::CONTROL,
            ..
        }
    };
}
