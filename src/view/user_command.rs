use std::rc::Rc;

use ansi_to_tui::IntoText as _;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    text::Line,
    Frame,
};

use crate::{
    app::AppContext,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::{Commit, Ref, Repository},
    view::{ListRefreshViewContext, RefreshViewContext, UserCommandRefreshViewContext},
    widget::{
        commit_list::{CommitList, CommitListState},
        commit_user_command::{CommitUserCommand, CommitUserCommandState},
    },
};

type ExecCommandFn = fn(&Commit, &[Ref], usize, Rect, &AppContext) -> Result<String, String>;

#[derive(Debug)]
pub struct UserCommandView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_user_command_state: CommitUserCommandState,

    user_command_number: usize,
    user_command_output_lines: Vec<Line<'a>>,

    ctx: Rc<AppContext>,
    tx: Sender,
}

impl<'a> UserCommandView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        command_output: String,
        user_command_number: usize,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> UserCommandView<'a> {
        let user_command_output_lines =
            build_user_command_output_lines(command_output, ctx.clone()).unwrap_or_else(|err| {
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
            UserEvent::UserCommand(n) => {
                if n == self.user_command_number {
                    self.tx.send(AppEvent::CloseUserCommand);
                } else {
                    // switch to another user command
                    self.tx.send(AppEvent::OpenUserCommand(n));
                }
            }
            UserEvent::Confirm => {
                self.tx.send(AppEvent::OpenDetail);
            }
            UserEvent::Cancel | UserEvent::Close => {
                self.tx.send(AppEvent::CloseUserCommand);
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
    }

    pub fn update_layout(&mut self, area: Rect) {
        let user_command_height = (area.height - 1).min(self.ctx.ui_config.user_command.height);
        let [list_area, _user_command_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(user_command_height)])
                .areas(area);
        self.as_mut_list_state()
            .update_height(list_area.height as usize);
    }

    pub fn prepare_graph_uploads(&mut self) {
        self.as_mut_list_state().ensure_visible_graph_uploaded();
    }
}

impl<'a> UserCommandView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }

    pub fn as_list_state(&self) -> &CommitListState<'a> {
        self.commit_list_state.as_ref().unwrap()
    }

    pub fn drain_pending_graph_uploads(&mut self) -> Vec<String> {
        self.as_mut_list_state().drain_pending_graph_uploads()
    }

    pub fn graph_image_ids_sorted(&self) -> Vec<u32> {
        self.as_list_state().graph_image_ids_sorted()
    }

    pub fn select_older_commit(
        &mut self,
        repository: &Repository,
        view_area: Rect,
        exec_command: ExecCommandFn,
    ) {
        self.update_selected_commit(repository, view_area, exec_command, |state| {
            state.select_next()
        });
    }

    pub fn select_newer_commit(
        &mut self,
        repository: &Repository,
        view_area: Rect,
        exec_command: ExecCommandFn,
    ) {
        self.update_selected_commit(repository, view_area, exec_command, |state| {
            state.select_prev()
        });
    }

    pub fn select_parent_commit(
        &mut self,
        repository: &Repository,
        view_area: Rect,
        exec_command: ExecCommandFn,
    ) {
        self.update_selected_commit(repository, view_area, exec_command, |state| {
            state.select_parent()
        });
    }

    fn update_selected_commit<F>(
        &mut self,
        repository: &Repository,
        view_area: Rect,
        exec_command: ExecCommandFn,
        update_commit_list_state: F,
    ) where
        F: FnOnce(&mut CommitListState<'a>),
    {
        let commit_list_state = self.as_mut_list_state();
        update_commit_list_state(commit_list_state);

        let selected = commit_list_state.selected_commit_hash().clone();
        let (commit, _) = repository.commit_detail(&selected);
        let refs: Vec<Ref> = repository.refs(&selected).into_iter().cloned().collect();
        self.user_command_output_lines = exec_command(
            &commit,
            &refs,
            self.user_command_number,
            view_area,
            &self.ctx,
        )
        .and_then(|output| build_user_command_output_lines(output, self.ctx.clone()))
        .unwrap_or_else(|err| {
            self.tx.send(AppEvent::NotifyError(err));
            vec![]
        });

        self.commit_user_command_state.select_first();
    }

    pub fn refresh(&self) {
        let list_state = self.as_list_state();
        let list_context = ListRefreshViewContext::from(list_state);
        let user_command_context = UserCommandRefreshViewContext {
            n: self.user_command_number,
        };
        let context = RefreshViewContext::UserCommand {
            list_context,
            user_command_context,
        };
        self.tx.send(AppEvent::Refresh(context));
    }
}

fn build_user_command_output_lines<'a>(
    command_output: String,
    ctx: Rc<AppContext>,
) -> Result<Vec<Line<'a>>, String> {
    let tab_spaces = " ".repeat(ctx.core_config.user_command.tab_width as usize);
    command_output
        .replace('\t', &tab_spaces) // tab is not rendered correctly, so replace it
        .into_text()
        .map(|t| t.into_iter().collect())
        .map_err(|e| e.to_string())
}
