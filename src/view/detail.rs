use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::{
    config::UiConfig,
    event::{AppEvent, Sender, UserEvent},
    git::{Commit, FileChange, Ref},
    protocol::ImageProtocol,
    widget::{
        commit_detail::{CommitDetail, CommitDetailState},
        commit_list::{CommitList, CommitListState},
    },
};

#[derive(Debug)]
pub struct DetailView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_detail_state: CommitDetailState,

    commit: Commit,
    changes: Vec<FileChange>,
    refs: Vec<Ref>,

    ui_config: &'a UiConfig,
    image_protocol: ImageProtocol,
    tx: Sender,
    clear: bool,
}

impl<'a> DetailView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        commit: Commit,
        changes: Vec<FileChange>,
        refs: Vec<Ref>,
        ui_config: &'a UiConfig,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> DetailView<'a> {
        DetailView {
            commit_list_state: Some(commit_list_state),
            commit_detail_state: CommitDetailState::default(),
            commit,
            changes,
            refs,
            ui_config,
            image_protocol,
            tx,
            clear: false,
        }
    }

    pub fn handle_event(&mut self, event: &UserEvent, _: KeyEvent) {
        match event {
            UserEvent::NavigateDown => {
                self.commit_detail_state.scroll_down();
            }
            UserEvent::NavigateUp => {
                self.commit_detail_state.scroll_up();
            }
            UserEvent::GoToTop => {
                self.commit_detail_state.select_first();
            }
            UserEvent::GoToBottom => {
                self.commit_detail_state.select_last();
            }
            UserEvent::ShortCopy => {
                self.copy_commit_short_hash();
            }
            UserEvent::FullCopy => {
                self.copy_commit_hash();
            }
            UserEvent::HelpToggle => {
                self.tx.send(AppEvent::OpenHelp);
            }
            UserEvent::Cancel | UserEvent::Close => {
                self.tx.send(AppEvent::ClearDetail); // hack: reset the rendering of the image area
                self.tx.send(AppEvent::CloseDetail);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let detail_height = (area.height - 1).min(self.ui_config.detail.height);
        let [list_area, detail_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(detail_height)]).areas(area);

        let commit_list = CommitList::new(&self.ui_config.list);
        f.render_stateful_widget(commit_list, list_area, self.as_mut_list_state());

        if self.clear {
            f.render_widget(Clear, detail_area);
            return;
        }

        let commit_detail = CommitDetail::new(
            &self.commit,
            &self.changes,
            &self.refs,
            &self.ui_config.detail,
        );
        f.render_stateful_widget(commit_detail, detail_area, &mut self.commit_detail_state);

        // clear the image area if needed
        for y in detail_area.top()..detail_area.bottom() {
            self.image_protocol.clear_line(y);
        }
    }
}

impl<'a> DetailView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        self.clear = true;
    }

    fn copy_commit_short_hash(&self) {
        let selected = &self.commit.commit_hash;
        self.copy_to_clipboard("Commit SHA (short)".into(), selected.as_short_hash());
    }

    fn copy_commit_hash(&self) {
        let selected = &self.commit.commit_hash;
        self.copy_to_clipboard("Commit SHA".into(), selected.as_str().into());
    }

    fn copy_to_clipboard(&self, name: String, value: String) {
        self.tx.send(AppEvent::CopyToClipboard { name, value });
    }
}
