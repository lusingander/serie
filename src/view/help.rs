use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    event::{AppEvent, Sender, UserEvent},
    keybind::KeyBind,
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
    pub fn new<'a>(
        before: View<'a>,
        image_protocol: ImageProtocol,
        tx: Sender,
        keybind: &'a KeyBind,
    ) -> HelpView<'a> {
        let (help_key_lines, help_value_lines) = build_lines(keybind);
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

    pub fn handle_event(&mut self, event: &UserEvent, _: KeyEvent) {
        match event {
            UserEvent::Quit => {
                self.tx.send(AppEvent::Quit);
            }
            UserEvent::HelpToggle | UserEvent::Cancel | UserEvent::Close => {
                self.tx.send(AppEvent::ClearHelp); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseHelp);
            }
            UserEvent::NavigateDown => {
                self.scroll_down();
            }
            UserEvent::NavigateUp => {
                self.scroll_up();
            }
            UserEvent::GoToTop => {
                self.select_first();
            }
            UserEvent::GoToBottom => {
                self.select_last();
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

    fn select_first(&mut self) {
        self.offset = 0;
    }

    fn select_last(&mut self) {
        self.offset = self.help_key_lines.len() - 1;
    }
}

#[rustfmt::skip]
fn build_lines(keybind: &KeyBind) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let (common_key_lines, common_value_lines) = build_block_lines(
        "Common:",
        &[
            (&[UserEvent::ForceQuit, UserEvent::Quit], "Quit app"),
            (&[UserEvent::HelpToggle], "Open help"),
        ],
        keybind,
    );
    let (help_key_lines, help_value_lines) = build_block_lines(
        "Help:",
        &[
            (&[UserEvent::HelpToggle, UserEvent::Cancel, UserEvent::Close], "Close help"),
            (&[UserEvent::NavigateDown], "Scroll down"),
            (&[UserEvent::NavigateUp], "Scroll up"),
            (&[UserEvent::GoToTop], "Go to top"),
            (&[UserEvent::GoToBottom], "Go to bottom"),
        ],
        keybind,
    );
    let (list_key_lines, list_value_lines) = build_block_lines(
        "Commit List:",
        &[
            (&[UserEvent::NavigateDown], "Move down"),
            (&[UserEvent::NavigateUp], "Move up"),
            (&[UserEvent::GoToParent], "Go to parent"),
            (&[UserEvent::GoToTop], "Go to top"),
            (&[UserEvent::GoToBottom], "Go to bottom"),
            (&[UserEvent::PageDown], "Scroll page down"),
            (&[UserEvent::PageUp], "Scroll page up"),
            (&[UserEvent::HalfPageDown], "Scroll half page down"),
            (&[UserEvent::HalfPageUp], "Scroll half page up"),
            (&[UserEvent::ScrollDown], "Scroll down"),
            (&[UserEvent::ScrollUp], "Scroll up"),
            (&[UserEvent::SelectTop], "Select top of the screen"),
            (&[UserEvent::SelectMiddle], "Select middle of the screen"),
            (&[UserEvent::SelectBottom], "Select bottom of the screen"),
            (&[UserEvent::Confirm], "Show commit details"),
            (&[UserEvent::RefListToggle], "Open refs list"),
            (&[UserEvent::Search], "Start search"),
            (&[UserEvent::Cancel], "Cancel search"),
            (&[UserEvent::GoToNext], "Go to next search match"),
            (&[UserEvent::GoToPrevious], "Go to previous search match"),
            (&[UserEvent::ShortCopy], "Copy commit short hash"),
            (&[UserEvent::FullCopy], "Copy commit hash"),
        ],
        keybind,
    );
    let (detail_key_lines, detail_value_lines) = build_block_lines(
        "Commit Detail:",
        &[
            (&[UserEvent::Cancel, UserEvent::Close], "Close commit details"),
            (&[UserEvent::PageDown], "Scroll down"),
            (&[UserEvent::PageUp], "Scroll up"),
            (&[UserEvent::GoToTop], "Go to top"),
            (&[UserEvent::GoToBottom], "Go to bottom"),
            (&[UserEvent::ShortCopy], "Copy commit short hash"),
            (&[UserEvent::FullCopy], "Copy commit hash"),
        ],
        keybind,
    );
    let (refs_key_lines, refs_value_lines) = build_block_lines(
        "Refs List:",
        &[
            (&[UserEvent::Cancel, UserEvent::Close, UserEvent::RefListToggle], "Close refs list"),
            (&[UserEvent::NavigateDown], "Move down"),
            (&[UserEvent::NavigateUp], "Move up"),
            (&[UserEvent::GoToTop], "Go to top"),
            (&[UserEvent::GoToBottom], "Go to bottom"),
            (&[UserEvent::NavigateRight], "Open node"),
            (&[UserEvent::NavigateLeft], "Close node"),
            (&[UserEvent::ShortCopy], "Copy ref name"),
        ],
        keybind,
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
    helps: &[(&[UserEvent], &'static str)],
    keybind: &KeyBind,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut key_lines = Vec::new();
    let mut value_lines = Vec::new();

    let key_title_lines = vec![Line::from(title)
        .fg(BLOCK_TITLE_COLOR)
        .add_modifier(Modifier::BOLD)];
    let value_title_lines = vec![Line::from("")];
    let key_binding_lines: Vec<Line> = helps
        .iter()
        .map(|(events, _)| {
            join_span_groups_with_space(
                events
                    .iter()
                    .flat_map(|event| keybind.keys_for_event(event))
                    .map(|key| vec!["<".into(), key.fg(KEY_COLOR), ">".into()])
                    .collect(),
            )
        })
        .collect();
    let value_binding_lines: Vec<Line> =
        helps.iter().map(|(_, value)| Line::from(*value)).collect();

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
