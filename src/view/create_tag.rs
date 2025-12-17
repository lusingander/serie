use std::path::PathBuf;

use ratatui::{
    crossterm::event::{Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    color::ColorTheme,
    config::UiConfig,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::{create_tag, push_tag, CommitHash, Ref},
    widget::commit_list::{CommitList, CommitListState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedField {
    TagName,
    Message,
    PushCheckbox,
}

#[derive(Debug)]
pub struct CreateTagView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_hash: CommitHash,
    repo_path: PathBuf,

    tag_name_input: Input,
    tag_message_input: Input,
    push_to_remote: bool,
    focused_field: FocusedField,

    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    tx: Sender,
}

impl<'a> CreateTagView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        commit_hash: CommitHash,
        repo_path: PathBuf,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> CreateTagView<'a> {
        CreateTagView {
            commit_list_state: Some(commit_list_state),
            commit_hash,
            repo_path,
            tag_name_input: Input::default(),
            tag_message_input: Input::default(),
            push_to_remote: true,
            focused_field: FocusedField::TagName,
            ui_config,
            color_theme,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, key: KeyEvent) {
        use ratatui::crossterm::event::KeyCode;

        // Handle Tab for focus switching (before UserEvent processing)
        if key.code == KeyCode::Tab {
            self.focus_next();
            return;
        }
        if key.code == KeyCode::BackTab {
            self.focus_prev();
            return;
        }

        // Handle Backspace for input (don't close dialog)
        if key.code == KeyCode::Backspace {
            self.handle_input(key);
            return;
        }

        let event = event_with_count.event;

        match event {
            UserEvent::Cancel => {
                self.tx.send(AppEvent::CloseCreateTag);
            }
            UserEvent::Confirm => {
                self.submit();
            }
            UserEvent::NavigateDown => {
                self.focus_next();
            }
            UserEvent::NavigateUp => {
                self.focus_prev();
            }
            UserEvent::NavigateRight | UserEvent::NavigateLeft => {
                if self.focused_field == FocusedField::PushCheckbox {
                    self.push_to_remote = !self.push_to_remote;
                } else {
                    self.handle_input(key);
                }
            }
            _ => {
                self.handle_input(key);
            }
        }
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match self.focused_field {
            FocusedField::TagName => {
                self.tag_name_input.handle_event(&Event::Key(key));
            }
            FocusedField::Message => {
                self.tag_message_input.handle_event(&Event::Key(key));
            }
            FocusedField::PushCheckbox => {
                if key.code == ratatui::crossterm::event::KeyCode::Char(' ') {
                    self.push_to_remote = !self.push_to_remote;
                }
            }
        }
    }

    fn focus_next(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::TagName => FocusedField::Message,
            FocusedField::Message => FocusedField::PushCheckbox,
            FocusedField::PushCheckbox => FocusedField::TagName,
        };
    }

    fn focus_prev(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::TagName => FocusedField::PushCheckbox,
            FocusedField::Message => FocusedField::TagName,
            FocusedField::PushCheckbox => FocusedField::Message,
        };
    }

    fn submit(&mut self) {
        let tag_name = self.tag_name_input.value().trim();
        if tag_name.is_empty() {
            self.tx
                .send(AppEvent::NotifyError("Tag name cannot be empty".into()));
            return;
        }

        let message = self.tag_message_input.value().trim();
        let message = if message.is_empty() {
            None
        } else {
            Some(message)
        };

        if let Err(e) = create_tag(&self.repo_path, tag_name, &self.commit_hash, message) {
            self.tx.send(AppEvent::NotifyError(e));
            return;
        }

        if self.push_to_remote {
            if let Err(e) = push_tag(&self.repo_path, tag_name) {
                self.tx.send(AppEvent::NotifyError(e));
                return;
            }
        }

        // Update UI with new tag
        self.tx.send(AppEvent::AddTagToCommit {
            commit_hash: self.commit_hash.clone(),
            tag_name: tag_name.to_string(),
        });

        let msg = if self.push_to_remote {
            format!("Tag '{}' created and pushed to origin", tag_name)
        } else {
            format!("Tag '{}' created", tag_name)
        };
        self.tx.send(AppEvent::NotifySuccess(msg));
        self.tx.send(AppEvent::CloseCreateTag);
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Render commit list in background
        let commit_list = CommitList::new(&self.ui_config.list, self.color_theme);
        f.render_stateful_widget(commit_list, area, self.as_mut_list_state());

        // Dialog dimensions
        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let dialog_height = 10u16.min(area.height.saturating_sub(2));

        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(
            area.x + dialog_x,
            area.y + dialog_y,
            dialog_width,
            dialog_height,
        );

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Create Tag ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color_theme.divider_fg))
            .style(Style::default().bg(self.color_theme.bg).fg(self.color_theme.fg))
            .padding(Padding::horizontal(1));

        let inner_area = block.inner(dialog_area);
        f.render_widget(block, dialog_area);

        let [commit_area, tag_name_area, message_area, push_area, hint_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(inner_area);

        // Commit hash
        let commit_line = Line::from(vec![
            Span::raw("Commit: ").fg(self.color_theme.fg),
            Span::raw(self.commit_hash.as_short_hash()).fg(self.color_theme.list_hash_fg),
        ]);
        f.render_widget(Paragraph::new(commit_line), commit_area);

        // Tag name input
        let tag_input_area = self.render_input_field(f, tag_name_area, "Tag name:", self.tag_name_input.value(), FocusedField::TagName);

        // Message input
        let msg_input_area = self.render_input_field(f, message_area, "Message:", self.tag_message_input.value(), FocusedField::Message);

        // Push checkbox
        let checkbox = if self.push_to_remote { "[x]" } else { "[ ]" };
        let checkbox_style = if self.focused_field == FocusedField::PushCheckbox {
            Style::default().add_modifier(Modifier::BOLD).fg(self.color_theme.status_success_fg)
        } else {
            Style::default().fg(self.color_theme.fg)
        };
        let push_line = Line::from(vec![
            Span::styled(checkbox, checkbox_style),
            Span::raw(" Push to origin").fg(self.color_theme.fg),
        ]);
        f.render_widget(Paragraph::new(push_line), push_area);

        // Hints
        let hint_line = Line::from(vec![
            Span::raw("Enter").fg(self.color_theme.help_key_fg),
            Span::raw(" submit  ").fg(self.color_theme.fg),
            Span::raw("Esc").fg(self.color_theme.help_key_fg),
            Span::raw(" cancel  ").fg(self.color_theme.fg),
            Span::raw("Tab/↑↓").fg(self.color_theme.help_key_fg),
            Span::raw(" nav").fg(self.color_theme.fg),
        ]);
        f.render_widget(Paragraph::new(hint_line).centered(), hint_area);

        // Cursor positioning
        if self.focused_field == FocusedField::TagName {
            let cursor_x = tag_input_area.x + 1 + self.tag_name_input.visual_cursor() as u16;
            f.set_cursor_position((cursor_x.min(tag_input_area.right().saturating_sub(1)), tag_input_area.y));
        } else if self.focused_field == FocusedField::Message {
            let cursor_x = msg_input_area.x + 1 + self.tag_message_input.visual_cursor() as u16;
            f.set_cursor_position((cursor_x.min(msg_input_area.right().saturating_sub(1)), msg_input_area.y));
        }
    }

    fn render_input_field(&self, f: &mut Frame, area: Rect, label: &str, value: &str, field: FocusedField) -> Rect {
        let is_focused = self.focused_field == field;
        let label_style = if is_focused {
            Style::default().add_modifier(Modifier::BOLD).fg(self.color_theme.status_success_fg)
        } else {
            Style::default().fg(self.color_theme.fg)
        };

        let [label_area, input_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        f.render_widget(
            Paragraph::new(Line::from(Span::styled(label, label_style))),
            label_area,
        );

        let input_style = if is_focused {
            Style::default().bg(self.color_theme.list_selected_bg)
        } else {
            Style::default()
        };

        let max_width = input_area.width.saturating_sub(2) as usize;
        let display_value = if value.len() > max_width {
            &value[value.len() - max_width..]
        } else {
            value
        };

        f.render_widget(
            Paragraph::new(Line::from(Span::raw(format!(" {}", display_value))))
                .style(input_style),
            input_area,
        );

        input_area
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }
}

impl<'a> CreateTagView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    pub fn add_ref_to_commit(&mut self, commit_hash: &CommitHash, new_ref: Ref) {
        self.as_mut_list_state().add_ref_to_commit(commit_hash, new_ref);
    }
}
