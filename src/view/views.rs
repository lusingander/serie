use std::rc::Rc;

use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    app::AppContext,
    event::{Sender, UserEventWithCount},
    git::{Commit, FileChange, Ref},
    view::{
        detail::DetailView, help::HelpView, list::ListView, refs::RefsView,
        user_command::UserCommandView,
    },
    widget::commit_list::CommitListState,
};

#[derive(Debug, Clone, Copy)]
pub enum RefreshViewContext {
    List,
    Detail,
    UserCommand { n: usize },
    Refs,
}

#[derive(Debug, Default)]
pub enum View<'a> {
    #[default]
    Default, // dummy variant to make #[default] work
    List(Box<ListView<'a>>),
    Detail(Box<DetailView<'a>>),
    UserCommand(Box<UserCommandView<'a>>),
    Refs(Box<RefsView<'a>>),
    Help(Box<HelpView<'a>>),
}

impl<'a> View<'a> {
    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, key_event: KeyEvent) {
        match self {
            View::Default => {}
            View::List(view) => view.handle_event(event_with_count, key_event),
            View::Detail(view) => view.handle_event(event_with_count, key_event),
            View::UserCommand(view) => view.handle_event(event_with_count, key_event),
            View::Refs(view) => view.handle_event(event_with_count, key_event),
            View::Help(view) => view.handle_event(event_with_count, key_event),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            View::Default => {}
            View::List(view) => view.render(f, area),
            View::Detail(view) => view.render(f, area),
            View::UserCommand(view) => view.render(f, area),
            View::Refs(view) => view.render(f, area),
            View::Help(view) => view.render(f, area),
        }
    }

    pub fn of_list(
        commit_list_state: CommitListState<'a>,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        View::List(Box::new(ListView::new(commit_list_state, ctx, tx)))
    }

    pub fn of_detail(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        changes: Vec<FileChange>,
        refs: Vec<Ref>,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        View::Detail(Box::new(DetailView::new(
            commit_list_state,
            commit,
            changes,
            refs,
            ctx,
            tx,
        )))
    }

    pub fn of_user_command(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        user_command_number: usize,
        view_area: Rect,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        View::UserCommand(Box::new(UserCommandView::new(
            commit_list_state,
            commit,
            user_command_number,
            view_area,
            ctx,
            tx,
        )))
    }

    pub fn of_refs(
        commit_list_state: CommitListState<'a>,
        refs: Vec<Ref>,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        View::Refs(Box::new(RefsView::new(commit_list_state, refs, ctx, tx)))
    }

    pub fn of_help(before: View<'a>, ctx: Rc<AppContext>, tx: Sender) -> Self {
        View::Help(Box::new(HelpView::new(before, ctx, tx)))
    }
}
