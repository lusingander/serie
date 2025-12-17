use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
};

use crate::color::ColorTheme;

pub struct PendingOverlay<'a> {
    message: &'a str,
    color_theme: &'a ColorTheme,
}

impl<'a> PendingOverlay<'a> {
    pub fn new(message: &'a str, color_theme: &'a ColorTheme) -> Self {
        Self {
            message,
            color_theme,
        }
    }
}

impl Widget for PendingOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_width = 40u16.min(area.width.saturating_sub(4));
        let max_text_width = dialog_width.saturating_sub(4) as usize; // borders + padding

        // Wrap message into lines
        let message_lines: Vec<Line> = wrap_text(self.message, max_text_width)
            .into_iter()
            .map(|s| Line::from(Span::raw(s).add_modifier(Modifier::BOLD)))
            .collect();

        let dialog_height = (4 + message_lines.len() as u16).min(area.height.saturating_sub(2));

        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(
            area.x + dialog_x,
            area.y + dialog_y,
            dialog_width,
            dialog_height,
        );

        Clear.render(dialog_area, buf);

        let block = Block::default()
            .title(" Working... ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color_theme.divider_fg))
            .style(
                Style::default()
                    .bg(self.color_theme.bg)
                    .fg(self.color_theme.fg),
            )
            .padding(Padding::horizontal(1));

        let inner_area = block.inner(dialog_area);
        block.render(dialog_area, buf);

        let mut lines = vec![Line::raw("")];
        lines.extend(message_lines);
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("Esc").fg(self.color_theme.help_key_fg),
            Span::raw(" hide").fg(self.color_theme.fg),
        ]));

        Paragraph::new(lines)
            .centered()
            .render(inner_area, buf);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
