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

use strum::{EnumMessage, IntoEnumIterator};

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
            UserEvent::HelpToggle | UserEvent::CloseOrCancel => {
                self.tx.send(AppEvent::ClearHelp); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseHelp);
            }
            UserEvent::NavigateDown => {
                self.scroll_down();
            }
            UserEvent::NavigateUp => {
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
fn build_lines(keybind: &KeyBind) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut event_key_maps = Vec::new();
    for user_event in UserEvent::iter() {
        let key_events: String = keybind.keys_for_event(&user_event).join(" ");
        event_key_maps.push((user_event.get_documentation(), key_events));
    }
    let mut key_lines = Vec::new();
    let mut value_lines = Vec::new();

    let key_title_lines = vec![Line::from("Help")
        .fg(BLOCK_TITLE_COLOR)
        .add_modifier(Modifier::BOLD)];
    let value_title_lines = vec![Line::from("")];
    let key_binding_lines: Vec<Line> = event_key_maps.clone()
        .into_iter()
        .map(|(_, keys)| {
            Line::from(Span::raw(keys)).fg(KEY_COLOR)
        })
        .collect();
    let value_binding_lines: Vec<Line> = event_key_maps
        .into_iter()
        .filter_map(|(user_event, _)| user_event.map(Line::from))
        .collect();

    key_lines.extend(key_title_lines);
    key_lines.extend(key_binding_lines);
    value_lines.extend(value_title_lines);
    value_lines.extend(value_binding_lines);

    (key_lines, value_lines)
}
