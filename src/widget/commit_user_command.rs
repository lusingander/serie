use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};

use crate::app::AppContext;

#[derive(Debug, Default)]
pub struct CommitUserCommandState {
    height: usize,
    offset: usize,
}

impl CommitUserCommandState {
    pub fn scroll_down(&mut self) {
        self.offset = self.offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub fn scroll_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height);
    }

    pub fn scroll_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height);
    }

    pub fn scroll_half_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height / 2);
    }

    pub fn scroll_half_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height / 2);
    }

    pub fn select_first(&mut self) {
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        self.offset = usize::MAX;
    }
}

pub struct CommitUserCommand<'a> {
    lines: &'a Vec<Line<'a>>,
    ctx: Rc<AppContext>,
}

impl<'a> CommitUserCommand<'a> {
    pub fn new(lines: &'a Vec<Line<'a>>, ctx: Rc<AppContext>) -> Self {
        Self { lines, ctx }
    }
}

impl StatefulWidget for CommitUserCommand<'_> {
    type State = CommitUserCommandState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content_area_height = area.height as usize - 1; // minus the top border
        self.update_state(state, self.lines.len(), content_area_height);

        self.render_user_command_lines(area, buf, state);
    }
}

impl CommitUserCommand<'_> {
    fn render_user_command_lines(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut CommitUserCommandState,
    ) {
        let lines = self
            .lines
            .iter()
            .skip(state.offset)
            .take(area.height as usize - 1)
            .cloned()
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.ctx.color_theme.fg))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .style(Style::default().fg(self.ctx.color_theme.divider_fg))
                    .padding(Padding::horizontal(2)),
            );
        paragraph.render(area, buf);
    }

    fn update_state(
        &self,
        state: &mut CommitUserCommandState,
        line_count: usize,
        area_height: usize,
    ) {
        state.height = area_height;
        state.offset = state.offset.min(line_count.saturating_sub(area_height));
    }
}
