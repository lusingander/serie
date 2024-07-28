use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    event::{AppEvent, Sender},
    key_code, key_code_char,
    protocol::ImageProtocol,
    view::View,
};

const BLOCK_TITLE_COLOR: Color = Color::Green;
const KEY_COLOR: Color = Color::Yellow;

#[derive(Debug)]
pub struct HelpView<'a> {
    before: View<'a>,

    help_key_lines: Vec<Line<'static>>,
    help_value_lines: Vec<Line<'static>>,

    offset: usize,

    image_protocol: ImageProtocol,
    tx: Sender,
    clear: bool,
}

impl HelpView<'_> {
    pub fn new(before: View, image_protocol: ImageProtocol, tx: Sender) -> HelpView {
        let (help_key_lines, help_value_lines) = build_lines();
        HelpView {
            before,
            help_key_lines,
            help_value_lines,
            offset: 0,
            image_protocol,
            tx,
            clear: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key {
            key_code_char!('q') => {
                self.tx.send(AppEvent::Quit);
            }
            key_code_char!('?') | key_code!(KeyCode::Esc) | key_code!(KeyCode::Backspace) => {
                self.tx.send(AppEvent::ClearHelp); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseHelp);
            }
            key_code_char!('j') | key_code!(KeyCode::Down) => {
                self.scroll_down();
            }
            key_code_char!('k') | key_code!(KeyCode::Up) => {
                self.scroll_up();
            }
            _ => {}
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.clear {
            f.render_widget(Clear, area);
            return;
        }

        let [key_area, value_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);

        let key_lines: Vec<Line> = self
            .help_key_lines
            .iter()
            .skip(self.offset)
            .take(area.height as usize)
            .cloned()
            .collect();
        let value_lines: Vec<Line> = self
            .help_value_lines
            .iter()
            .skip(self.offset)
            .take(area.height as usize)
            .cloned()
            .collect();

        let key_paragraph = Paragraph::new(key_lines)
            .block(Block::default().padding(Padding::new(3, 1, 0, 0)))
            .right_aligned();
        let value_paragraph = Paragraph::new(value_lines)
            .block(Block::default().padding(Padding::new(1, 3, 0, 0)))
            .left_aligned();

        f.render_widget(key_paragraph, key_area);
        f.render_widget(value_paragraph, value_area);

        // clear the image area if needed
        for y in area.top()..area.bottom() {
            self.image_protocol.clear_line(y);
        }
    }
}

impl<'a> HelpView<'a> {
    pub fn take_before_view(&mut self) -> View<'a> {
        std::mem::take(&mut self.before)
    }

    pub fn clear(&mut self) {
        self.clear = true;
    }

    fn scroll_down(&mut self) {
        if self.offset < self.help_key_lines.len() - 1 {
            self.offset += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }
}

#[rustfmt::skip]
fn build_lines() -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let (common_key_lines, common_value_lines) = build_block_lines(
        "Common:",
        &[
            (&["Ctrl-c", "q"], "Quit app"),
            (&["?"], "Open help"),
        ]
    );
    let (help_key_lines, help_value_lines) = build_block_lines(
        "Help:",
        &[
            (&["Esc", "Backspace", "?"], "Close help"),
            (&["Down", "j"], "Scroll down"),
            (&["Up", "k"], "Scroll up"),
        ]
    );
    let (list_key_lines, list_value_lines) = build_block_lines(
        "Commit List:",
        &[
            (&["Down", "j"], "Move down"),
            (&["Up", "k"], "Move up"),
            (&["g"], "Go to top"),
            (&["G"], "Go to bottom"),
            (&["Ctrl-f"], "Scroll page down"),
            (&["Ctrl-b"], "Scroll page up"),
            (&["Ctrl-d"], "Scroll half page down"),
            (&["Ctrl-u"], "Scroll half page up"),
            (&["H"], "Select top of the screen"),
            (&["M"], "Select middle of the screen"),
            (&["L"], "Select bottom of the screen"),
            (&["Enter"], "Show commit details"),
            (&["Tab"], "Open refs list"),
            (&["/"], "Start search"),
            (&["Esc"], "Cancel search"),
            (&["n"], "Go to next search match"),
            (&["N"], "Go to previous search match"),
            (&["c"], "Copy commit short hash"),
            (&["C"], "Copy commit hash"),
        ]
    );
    let (detail_key_lines, detail_value_lines) = build_block_lines(
        "Commit Detail:",
        &[
            (&["Esc", "Backspace"], "Close commit details"),
            (&["Down", "j"], "Scroll down"),
            (&["Up", "k"], "Scroll up"),
            (&["c"], "Copy commit short hash"),
            (&["C"], "Copy commit hash"),
        ]
    );
    let (refs_key_lines, refs_value_lines) = build_block_lines(
        "Refs List:",
        &[
            (&["Esc", "Backspace", "Tab"], "Close refs list"),
            (&["Down", "j"], "Move down"),
            (&["Up", "k"], "Move up"),
            (&["g"], "Go to top"),
            (&["G"], "Go to bottom"),
            (&["Right", "l"], "Open node"),
            (&["Left", "h"], "Close node"),
            (&["c"], "Copy ref name"),
        ]
    );
    
    let key_lines = join_line_groups_with_empty(vec![
        common_key_lines,
        help_key_lines,
        list_key_lines,
        detail_key_lines,
        refs_key_lines,
    ]);
    let value_lines = join_line_groups_with_empty(vec![
        common_value_lines,
        help_value_lines,
        list_value_lines,
        detail_value_lines,
        refs_value_lines,
    ]);

    (key_lines, value_lines)
}

fn build_block_lines(
    title: &'static str,
    keybindings: &[(&[&'static str], &'static str)],
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut key_lines = Vec::new();
    let mut value_lines = Vec::new();

    let key_title_lines = vec![Line::from(title)
        .fg(BLOCK_TITLE_COLOR)
        .add_modifier(Modifier::BOLD)];
    let value_title_lines = vec![Line::from("")];
    let key_binding_lines: Vec<Line> = keybindings
        .iter()
        .map(|(keys, _)| {
            join_span_groups_with_space(
                keys.iter()
                    .map(|key| vec!["<".into(), key.fg(KEY_COLOR), ">".into()])
                    .collect(),
            )
        })
        .collect();
    let value_binding_lines: Vec<Line> = keybindings
        .iter()
        .map(|(_, value)| Line::from(*value))
        .collect();

    key_lines.extend(key_title_lines);
    key_lines.extend(key_binding_lines);
    value_lines.extend(value_title_lines);
    value_lines.extend(value_binding_lines);

    (key_lines, value_lines)
}

fn join_line_groups_with_empty(line_groups: Vec<Vec<Line<'static>>>) -> Vec<Line<'static>> {
    let mut result = Vec::new();
    let n = line_groups.len();
    for (i, lines) in line_groups.into_iter().enumerate() {
        result.extend(lines);
        if i < n - 1 {
            result.push(Line::raw(""));
        }
    }
    result
}

fn join_span_groups_with_space(span_groups: Vec<Vec<Span<'static>>>) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    let n = span_groups.len();
    for (i, ss) in span_groups.into_iter().enumerate() {
        spans.extend(ss);
        if i < n - 1 {
            spans.push(Span::raw(" "));
        }
    }
    Line::from(spans)
}
