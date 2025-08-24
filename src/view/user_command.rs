use ansi_to_tui::IntoText as _;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::Clear,
    Frame,
};

use crate::{
    color::ColorTheme,
    config::UiConfig,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    external::exec_user_command,
    git::Commit,
    protocol::ImageProtocol,
    widget::{
        commit_list::{CommitList, CommitListState},
        commit_user_command::{CommitUserCommand, CommitUserCommandState},
    },
};

#[derive(Debug)]
pub enum UserCommandViewBeforeView {
    List,
    Detail,
}

#[derive(Debug)]
pub struct UserCommandView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_user_command_state: CommitUserCommandState,

    user_command_output_lines: Vec<Line<'a>>,

    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    image_protocol: ImageProtocol,
    tx: Sender,
    before_view: UserCommandViewBeforeView,
    clear: bool,
}

impl<'a> UserCommandView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
        before_view: UserCommandViewBeforeView,
    ) -> UserCommandView<'a> {
        let user_command_output_lines = if commit.parent_commit_hashes.is_empty() {
            vec![]
        } else {
            // fixme
            let command = &[
                "git",
                "--no-pager",
                "diff",
                "--color=always",
                "{{parent_hash}}",
                "{{target_hash}}",
            ];
            let cmd_output = exec_user_command(
                command,
                commit.commit_hash.as_str(),
                commit.parent_commit_hashes[0].as_str(),
            );
            match cmd_output {
                Ok(output) => output.into_text().unwrap().into_iter().collect(),
                Err(err) => {
                    let msg = format!("Failed to execute command: {}", err);
                    tx.send(AppEvent::NotifyError(msg));
                    vec![]
                }
            }
        };

        UserCommandView {
            commit_list_state: Some(commit_list_state),
            commit_user_command_state: CommitUserCommandState::default(),
            user_command_output_lines,
            ui_config,
            color_theme,
            image_protocol,
            tx,
            before_view,
            clear: false,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, _: KeyEvent) {
        let event = event_with_count.event;
        let count = event_with_count.count;

        match event {
            UserEvent::NavigateDown => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_down();
                }
            }
            UserEvent::NavigateUp => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_up();
                }
            }
            UserEvent::GoToTop => {
                self.commit_user_command_state.select_first();
            }
            UserEvent::GoToBottom => {
                self.commit_user_command_state.select_last();
            }
            UserEvent::HelpToggle => {
                self.tx.send(AppEvent::OpenHelp);
            }
            UserEvent::Cancel | UserEvent::Close | UserEvent::UserCommandViewToggle(_) => {
                self.tx.send(AppEvent::ClearUserCommand); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseUserCommand);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let user_command_height = (area.height - 1).min(self.ui_config.user_command.height);
        let [list_area, user_command_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(user_command_height)])
                .areas(area);

        let commit_list = CommitList::new(&self.ui_config.list, self.color_theme);
        f.render_stateful_widget(commit_list, list_area, self.as_mut_list_state());

        let commit_user_command =
            CommitUserCommand::new(&self.user_command_output_lines, self.color_theme);
        f.render_stateful_widget(
            commit_user_command,
            user_command_area,
            &mut self.commit_user_command_state,
        );

        if self.clear {
            f.render_widget(Clear, user_command_area);
            return;
        }

        // clear the image area if needed
        for y in user_command_area.top()..user_command_area.bottom() {
            self.image_protocol.clear_line(y);
        }
    }
}

impl<'a> UserCommandView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        self.clear = true;
    }

    pub fn before_view_is_list(&self) -> bool {
        matches!(self.before_view, UserCommandViewBeforeView::List)
    }
}
