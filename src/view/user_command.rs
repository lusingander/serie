use std::rc::Rc;

use ansi_to_tui::IntoText as _;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::Clear,
    Frame,
};

use crate::{
    app::AppContext,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    external::exec_user_command,
    git::{Commit, Repository},
    view::{ListRefreshViewContext, RefreshViewContext},
    widget::{
        commit_list::{CommitList, CommitListState},
        commit_user_command::{CommitUserCommand, CommitUserCommandState},
    },
};

#[derive(Debug)]
pub struct UserCommandView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_user_command_state: CommitUserCommandState,

    user_command_number: usize,
    user_command_output_lines: Vec<Line<'a>>,

    ctx: Rc<AppContext>,
    tx: Sender,
    clear: bool,
}

impl<'a> UserCommandView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        user_command_number: usize,
        view_area: Rect,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> UserCommandView<'a> {
        let user_command_output_lines =
            build_user_command_output_lines(&commit, user_command_number, view_area, ctx.clone())
                .unwrap_or_else(|err| {
                    tx.send(AppEvent::NotifyError(err));
                    vec![]
                });

        UserCommandView {
            commit_list_state: Some(commit_list_state),
            commit_user_command_state: CommitUserCommandState::default(),
            user_command_number,
            user_command_output_lines,
            ctx,
            tx,
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
            UserEvent::PageDown => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_page_down();
                }
            }
            UserEvent::PageUp => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_page_up();
                }
            }
            UserEvent::HalfPageDown => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_half_page_down();
                }
            }
            UserEvent::HalfPageUp => {
                for _ in 0..count {
                    self.commit_user_command_state.scroll_half_page_up();
                }
            }
            UserEvent::GoToTop => {
                self.commit_user_command_state.select_first();
            }
            UserEvent::GoToBottom => {
                self.commit_user_command_state.select_last();
            }
            UserEvent::SelectDown => {
                self.tx.send(AppEvent::SelectOlderCommit);
            }
            UserEvent::SelectUp => {
                self.tx.send(AppEvent::SelectNewerCommit);
            }
            UserEvent::GoToParent => {
                self.tx.send(AppEvent::SelectParentCommit);
            }
            UserEvent::HelpToggle => {
                self.tx.send(AppEvent::OpenHelp);
            }
            UserEvent::UserCommandViewToggle(n) => {
                if n == self.user_command_number {
                    self.close();
                } else {
                    // switch to another user command
                    self.tx.send(AppEvent::OpenUserCommand(n));
                }
            }
            UserEvent::Confirm => {
                self.tx.send(AppEvent::OpenDetail);
            }
            UserEvent::Cancel | UserEvent::Close => {
                self.close();
            }
            UserEvent::Refresh => {
                self.refresh();
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let user_command_height = (area.height - 1).min(self.ctx.ui_config.user_command.height);
        let [list_area, user_command_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(user_command_height)])
                .areas(area);

        let commit_list = CommitList::new(self.ctx.clone());
        f.render_stateful_widget(commit_list, list_area, self.as_mut_list_state());

        let commit_user_command =
            CommitUserCommand::new(&self.user_command_output_lines, self.ctx.clone());
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
            self.ctx.image_protocol.clear_line(y);
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

    fn as_list_state(&self) -> &CommitListState<'a> {
        self.commit_list_state.as_ref().unwrap()
    }

    pub fn select_older_commit(&mut self, repository: &Repository, view_area: Rect) {
        self.update_selected_commit(repository, view_area, |state| state.select_next());
    }

    pub fn select_newer_commit(&mut self, repository: &Repository, view_area: Rect) {
        self.update_selected_commit(repository, view_area, |state| state.select_prev());
    }

    pub fn select_parent_commit(&mut self, repository: &Repository, view_area: Rect) {
        self.update_selected_commit(repository, view_area, |state| state.select_parent());
    }

    fn update_selected_commit<F>(
        &mut self,
        repository: &Repository,
        view_area: Rect,
        update_commit_list_state: F,
    ) where
        F: FnOnce(&mut CommitListState<'a>),
    {
        let commit_list_state = self.as_mut_list_state();
        update_commit_list_state(commit_list_state);
        let selected = commit_list_state.selected_commit_hash().clone();
        let (commit, _) = repository.commit_detail(&selected);
        self.user_command_output_lines = build_user_command_output_lines(
            &commit,
            self.user_command_number,
            view_area,
            self.ctx.clone(),
        )
        .unwrap_or_else(|err| {
            self.tx.send(AppEvent::NotifyError(err));
            vec![]
        });

        self.commit_user_command_state.select_first();
    }

    pub fn clear(&mut self) {
        self.clear = true;
    }

    fn close(&self) {
        self.tx.send(AppEvent::ClearUserCommand); // hack: reset the rendering of the image area
        self.tx.send(AppEvent::CloseUserCommand);
    }

    fn refresh(&self) {
        let list_state = self.as_list_state();
        let commit_hash = list_state.selected_commit_hash().as_str().into();
        let (selected, _, height) = list_state.current_list_status();
        let list_context = ListRefreshViewContext {
            commit_hash,
            selected,
            height,
        };
        let context = RefreshViewContext::UserCommand {
            list_context,
            n: self.user_command_number,
        };
        self.tx.send(AppEvent::Refresh(context));
    }
}

fn build_user_command_output_lines<'a>(
    commit: &Commit,
    user_command_number: usize,
    view_area: Rect,
    ctx: Rc<AppContext>,
) -> Result<Vec<Line<'a>>, String> {
    let command = ctx
        .core_config
        .user_command
        .commands
        .get(&user_command_number.to_string())
        .ok_or_else(|| {
            format!(
                "No user command configured for number {}",
                user_command_number
            )
        })?
        .commands
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let target_hash = commit.commit_hash.as_str();
    let parent_hash = commit
        .parent_commit_hashes
        .first()
        .map(|c| c.as_str())
        .unwrap_or_default();

    let area_width = view_area.width - 4; // minus the left and right padding
    let area_height = (view_area.height - 1).min(ctx.ui_config.user_command.height) - 1; // minus the top border

    let tab_spaces = " ".repeat(ctx.core_config.user_command.tab_width as usize);
    exec_user_command(&command, target_hash, parent_hash, area_width, area_height)
        .and_then(|output| {
            output
                .replace('\t', &tab_spaces) // tab is not rendered correctly, so replace it
                .into_text()
                .map(|t| t.into_iter().collect())
                .map_err(|e| e.to_string())
        })
        .map_err(|err| format!("Failed to execute command: {}", err))
}
