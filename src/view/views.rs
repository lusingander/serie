use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    config::UiConfig,
    event::{Sender, UserEvent},
    git::{Commit, FileChange, Ref},
    keybind::KeyBind,
    protocol::ImageProtocol,
    view::{detail::DetailView, help::HelpView, list::ListView, refs::RefsView},
    widget::commit_list::CommitListState,
};

#[derive(Debug, Default)]
pub enum View<'a> {
    #[default]
    Default, // dummy variant to make #[default] work
    List(Box<ListView<'a>>),
    Detail(Box<DetailView<'a>>),
    Refs(Box<RefsView<'a>>),
    Help(Box<HelpView<'a>>),
}

impl<'a> View<'a> {
    pub fn handle_event(&mut self, user_event: UserEvent, key_event: KeyEvent) {
        match self {
            View::Default => {}
            View::List(view) => view.handle_event(user_event, key_event),
            View::Detail(view) => view.handle_event(user_event, key_event),
            View::Refs(view) => view.handle_event(user_event, key_event),
            View::Help(view) => view.handle_event(user_event, key_event),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            View::Default => {}
            View::List(view) => view.render(f, area),
            View::Detail(view) => view.render(f, area),
            View::Refs(view) => view.render(f, area),
            View::Help(view) => view.render(f, area),
        }
    }

    pub fn of_list(
        commit_list_state: CommitListState<'a>,
        ui_config: &'a UiConfig,
        tx: Sender,
    ) -> Self {
        View::List(Box::new(ListView::new(commit_list_state, ui_config, tx)))
    }

    pub fn of_detail(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        changes: Vec<FileChange>,
        refs: Vec<Ref>,
        ui_config: &'a UiConfig,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> Self {
        View::Detail(Box::new(DetailView::new(
            commit_list_state,
            commit,
            changes,
            refs,
            ui_config,
            image_protocol,
            tx,
        )))
    }

    pub fn of_refs(
        commit_list_state: CommitListState<'a>,
        refs: Vec<Ref>,
        ui_config: &'a UiConfig,
        tx: Sender,
    ) -> Self {
        View::Refs(Box::new(RefsView::new(
            commit_list_state,
            refs,
            ui_config,
            tx,
        )))
    }

    pub fn of_help(
        before: View<'a>,
        image_protocol: ImageProtocol,
        tx: Sender,
        keybind: &'a KeyBind,
    ) -> Self {
        View::Help(Box::new(HelpView::new(before, image_protocol, tx, keybind)))
    }
}
