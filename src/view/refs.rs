use std::rc::Rc;

use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{
    app::AppContext,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::Ref,
    view::RefreshViewContext,
    widget::{
        commit_list::{CommitList, CommitListState},
        ref_list::{RefList, RefListState},
    },
};

#[derive(Debug)]
pub struct RefsView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    ref_list_state: RefListState,

    refs: Vec<Ref>,

    ctx: Rc<AppContext>,
    tx: Sender,
}

impl<'a> RefsView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        refs: Vec<Ref>,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> RefsView<'a> {
        RefsView {
            commit_list_state: Some(commit_list_state),
            ref_list_state: RefListState::new(),
            refs,
            ctx,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, _: KeyEvent) {
        let event = event_with_count.event;
        let count = event_with_count.count;

        match event {
            UserEvent::Quit => {
                self.tx.send(AppEvent::Quit);
            }
            UserEvent::Cancel | UserEvent::Close | UserEvent::RefListToggle => {
                self.tx.send(AppEvent::CloseRefs);
            }
            UserEvent::NavigateDown | UserEvent::SelectDown => {
                for _ in 0..count {
                    self.ref_list_state.select_next();
                }
                self.update_commit_list_selected();
            }
            UserEvent::NavigateUp | UserEvent::SelectUp => {
                for _ in 0..count {
                    self.ref_list_state.select_prev();
                }
                self.update_commit_list_selected();
            }
            UserEvent::GoToTop => {
                self.ref_list_state.select_first();
                self.update_commit_list_selected();
            }
            UserEvent::GoToBottom => {
                self.ref_list_state.select_last();
                self.update_commit_list_selected();
            }
            UserEvent::NavigateRight => {
                self.ref_list_state.open_node();
                self.update_commit_list_selected();
            }
            UserEvent::NavigateLeft => {
                self.ref_list_state.close_node();
                self.update_commit_list_selected();
            }
            UserEvent::ShortCopy | UserEvent::FullCopy => {
                self.copy_ref_name();
            }
            UserEvent::HelpToggle => {
                self.tx.send(AppEvent::OpenHelp);
            }
            UserEvent::Refresh => {
                self.refresh();
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let graph_width = self.as_list_state().graph_area_cell_width() + 1; // graph area + marker
        let refs_width =
            (area.width.saturating_sub(graph_width)).min(self.ctx.ui_config.refs.width);

        let [list_area, refs_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(refs_width)]).areas(area);

        let commit_list = CommitList::new(self.ctx.clone());
        f.render_stateful_widget(commit_list, list_area, self.as_mut_list_state());

        let ref_list = RefList::new(&self.refs, self.ctx.clone());
        f.render_stateful_widget(ref_list, refs_area, &mut self.ref_list_state);
    }
}

impl<'a> RefsView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }

    fn as_list_state(&self) -> &CommitListState<'a> {
        self.commit_list_state.as_ref().unwrap()
    }

    fn update_commit_list_selected(&mut self) {
        if let Some(selected) = self.ref_list_state.selected_ref_name() {
            self.as_mut_list_state().select_ref(&selected)
        }
    }

    fn copy_ref_name(&self) {
        if let Some(selected) = self.ref_list_state.selected_branch() {
            self.copy_to_clipboard("Branch Name".into(), selected);
        } else if let Some(selected) = self.ref_list_state.selected_tag() {
            self.copy_to_clipboard("Tag Name".into(), selected);
        }
    }

    fn copy_to_clipboard(&self, name: String, value: String) {
        self.tx.send(AppEvent::CopyToClipboard { name, value });
    }

    fn refresh(&self) {
        let context = RefreshViewContext::Refs;
        self.tx.send(AppEvent::Refresh(context));
    }
}
