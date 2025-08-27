use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    color::ColorTheme,
    config::CoreConfig,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    keybind::KeyBind,
    protocol::ImageProtocol,
    view::View,
};

#[derive(Debug)]
pub struct HelpView<'a> {
    before: View<'a>,

    help_key_lines: Vec<Line<'static>>,
    help_value_lines: Vec<Line<'static>>,
    help_key_line_max_width: u16,

    offset: usize,
    height: usize,

    image_protocol: ImageProtocol,
    tx: Sender,
    clear: bool,
}

impl HelpView<'_> {
    pub fn new<'a>(
        before: View<'a>,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
        keybind: &'a KeyBind,
        core_config: &'a CoreConfig,
    ) -> HelpView<'a> {
        let (help_key_lines, help_value_lines) = build_lines(color_theme, keybind, core_config);
        let help_key_line_max_width = help_key_lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or_default() as u16;
        HelpView {
            before,
            help_key_lines,
            help_value_lines,
            help_key_line_max_width,
            offset: 0,
            height: 0,
            image_protocol,
            tx,
            clear: false,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, _: KeyEvent) {
        let event = event_with_count.event;
        let count = event_with_count.count;

        match event {
            UserEvent::Quit => {
                self.tx.send(AppEvent::Quit);
            }
            UserEvent::HelpToggle | UserEvent::Cancel | UserEvent::Close => {
                self.tx.send(AppEvent::ClearHelp); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseHelp);
            }
            UserEvent::NavigateDown => {
                for _ in 0..count {
                    self.scroll_down();
                }
            }
            UserEvent::NavigateUp => {
                for _ in 0..count {
                    self.scroll_up();
                }
            }
            UserEvent::PageDown => {
                for _ in 0..count {
                    self.scroll_page_down();
                }
            }
            UserEvent::PageUp => {
                for _ in 0..count {
                    self.scroll_page_up();
                }
            }
            UserEvent::HalfPageDown => {
                for _ in 0..count {
                    self.scroll_half_page_down();
                }
            }
            UserEvent::HalfPageUp => {
                for _ in 0..count {
                    self.scroll_half_page_up();
                }
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

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if self.clear {
            f.render_widget(Clear, area);
            return;
        }

        self.update_state(area);

        let [mut key_area, mut value_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);

        if key_area.width - 4 /* padding */ < self.help_key_line_max_width {
            [key_area, value_area] = Layout::horizontal([
                Constraint::Length(self.help_key_line_max_width + 4),
                Constraint::Min(0),
            ])
            .areas(area);
        }

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
        self.offset = self.offset.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    fn scroll_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height);
    }

    fn scroll_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height);
    }

    fn scroll_half_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height / 2);
    }

    fn scroll_half_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height / 2);
    }

    fn select_first(&mut self) {
        self.offset = 0;
    }

    fn select_last(&mut self) {
        self.offset = usize::MAX;
    }

    fn update_state(&mut self, area: Rect) {
        self.height = area.height as usize;
        self.offset = self.offset.min(self.help_key_lines.len() - 1)
    }
}

#[rustfmt::skip]
fn build_lines(
    color_theme: &ColorTheme,
    keybind: &KeyBind,
    core_config: &CoreConfig,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let user_command_view_toggle_helps = keybind
        .user_command_view_toggle_event_numbers()
        .into_iter()
        .flat_map(|n| {
            core_config
                .user_command
                .commands
                .get(&n.to_string())
                .map(|c| format!("Toggle user command {} - {}", n, c.name))
                .map(|desc| (vec![UserEvent::UserCommandViewToggle(n)], desc))
        })
        .collect::<Vec<_>>();

    let common_helps = vec![
        (vec![UserEvent::ForceQuit, UserEvent::Quit], "Quit app".into()),
        (vec![UserEvent::HelpToggle], "Open help".into()),
    ];
    let (common_key_lines, common_value_lines) = build_block_lines("Common:", common_helps, color_theme, keybind);

    let help_helps = vec![
        (vec![UserEvent::HelpToggle, UserEvent::Cancel, UserEvent::Close], "Close help".into()),
        (vec![UserEvent::NavigateDown], "Scroll down".into()),
        (vec![UserEvent::NavigateUp], "Scroll up".into()),
        (vec![UserEvent::PageDown], "Scroll page down".into()),
        (vec![UserEvent::PageUp], "Scroll page up".into()),
        (vec![UserEvent::HalfPageDown], "Scroll half page down".into()),
        (vec![UserEvent::HalfPageUp], "Scroll half page up".into()),
        (vec![UserEvent::GoToTop], "Go to top".into()),
        (vec![UserEvent::GoToBottom], "Go to bottom".into()),
    ];
    let (help_key_lines, help_value_lines) = build_block_lines("Help:", help_helps, color_theme, keybind);

    let mut list_helps = vec![
        (vec![UserEvent::NavigateDown], "Move down".into()),
        (vec![UserEvent::NavigateUp], "Move up".into()),
        (vec![UserEvent::GoToParent], "Go to parent".into()),
        (vec![UserEvent::GoToTop], "Go to top".into()),
        (vec![UserEvent::GoToBottom], "Go to bottom".into()),
        (vec![UserEvent::PageDown], "Scroll page down".into()),
        (vec![UserEvent::PageUp], "Scroll page up".into()),
        (vec![UserEvent::HalfPageDown], "Scroll half page down".into()),
        (vec![UserEvent::HalfPageUp], "Scroll half page up".into()),
        (vec![UserEvent::ScrollDown], "Scroll down".into()),
        (vec![UserEvent::ScrollUp], "Scroll up".into()),
        (vec![UserEvent::SelectTop], "Select top of the screen".into()),
        (vec![UserEvent::SelectMiddle], "Select middle of the screen".into()),
        (vec![UserEvent::SelectBottom], "Select bottom of the screen".into()),
        (vec![UserEvent::Confirm], "Show commit details".into()),
        (vec![UserEvent::RefListToggle], "Open refs list".into()),
        (vec![UserEvent::Search], "Start search".into()),
        (vec![UserEvent::Cancel], "Cancel search".into()),
        (vec![UserEvent::GoToNext], "Go to next search match".into()),
        (vec![UserEvent::GoToPrevious], "Go to previous search match".into()),
        (vec![UserEvent::IgnoreCaseToggle], "Toggle ignore case".into()),
        (vec![UserEvent::FuzzyToggle], "Toggle fuzzy match".into()),
        (vec![UserEvent::ShortCopy], "Copy commit short hash".into()),
        (vec![UserEvent::FullCopy], "Copy commit hash".into()),
    ];
    list_helps.extend(user_command_view_toggle_helps.clone());
    let (list_key_lines, list_value_lines) = build_block_lines("Commit List:", list_helps, color_theme, keybind);
    
    let mut detail_helps = vec![
        (vec![UserEvent::Cancel, UserEvent::Close], "Close commit details".into()),
        (vec![UserEvent::PageDown], "Scroll down".into()),
        (vec![UserEvent::PageUp], "Scroll up".into()),
        (vec![UserEvent::GoToTop], "Go to top".into()),
        (vec![UserEvent::GoToBottom], "Go to bottom".into()),
        (vec![UserEvent::ShortCopy], "Copy commit short hash".into()),
        (vec![UserEvent::FullCopy], "Copy commit hash".into()),
    ];
    detail_helps.extend(user_command_view_toggle_helps.clone());
    let (detail_key_lines, detail_value_lines) = build_block_lines("Commit Detail:", detail_helps, color_theme, keybind);

    let refs_helps = vec![
        (vec![UserEvent::Cancel, UserEvent::Close, UserEvent::RefListToggle], "Close refs list".into()),
        (vec![UserEvent::NavigateDown], "Move down".into()),
        (vec![UserEvent::NavigateUp], "Move up".into()),
        (vec![UserEvent::GoToTop], "Go to top".into()),
        (vec![UserEvent::GoToBottom], "Go to bottom".into()),
        (vec![UserEvent::NavigateRight], "Open node".into()),
        (vec![UserEvent::NavigateLeft], "Close node".into()),
        (vec![UserEvent::ShortCopy], "Copy ref name".into()),
    ];
    let (refs_key_lines, refs_value_lines) = build_block_lines("Refs List:", refs_helps, color_theme, keybind);
    
    let mut user_command_helps = vec![
        (vec![UserEvent::Cancel, UserEvent::Close], "Close user command".into()),
        (vec![UserEvent::PageDown], "Scroll down".into()),
        (vec![UserEvent::PageUp], "Scroll up".into()),
        (vec![UserEvent::GoToTop], "Go to top".into()),
        (vec![UserEvent::GoToBottom], "Go to bottom".into()),
    ];
    user_command_helps.extend(user_command_view_toggle_helps);
    let (user_command_key_lines, user_command_value_lines) = build_block_lines("User Command:", user_command_helps, color_theme, keybind);

    let key_lines = join_line_groups_with_empty(vec![
        common_key_lines,
        help_key_lines,
        list_key_lines,
        detail_key_lines,
        refs_key_lines,
        user_command_key_lines,
    ]);
    let value_lines = join_line_groups_with_empty(vec![
        common_value_lines,
        help_value_lines,
        list_value_lines,
        detail_value_lines,
        refs_value_lines,
        user_command_value_lines,
    ]);

    (key_lines, value_lines)
}

fn build_block_lines(
    title: &'static str,
    helps: Vec<(Vec<UserEvent>, String)>,
    color_theme: &ColorTheme,
    keybind: &KeyBind,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut key_lines = Vec::new();
    let mut value_lines = Vec::new();

    let key_title_lines = vec![Line::from(title)
        .fg(color_theme.help_block_title_fg)
        .add_modifier(Modifier::BOLD)];
    let value_title_lines = vec![Line::from("")];
    let key_binding_lines: Vec<Line> = helps
        .clone()
        .into_iter()
        .map(|(events, _)| {
            join_span_groups_with_space(
                events
                    .iter()
                    .flat_map(|event| keybind.keys_for_event(*event))
                    .map(|key| vec!["<".into(), key.fg(color_theme.help_key_fg), ">".into()])
                    .collect(),
            )
        })
        .collect();
    let value_binding_lines: Vec<Line> = helps
        .into_iter()
        .map(|(_, value)| Line::raw(value))
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
