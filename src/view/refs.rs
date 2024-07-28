use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{
    config::Config,
    event::{AppEvent, Sender},
    git::Ref,
    key_code, key_code_char,
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

    config: &'a Config,
    tx: Sender,
}

impl<'a> RefsView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        refs: Vec<Ref>,
        config: &'a Config,
        tx: Sender,
    ) -> RefsView<'a> {
        RefsView {
            commit_list_state: Some(commit_list_state),
            ref_list_state: RefListState::new(),
            refs,
            config,
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key {
            key_code_char!('q') => {
                self.tx.send(AppEvent::Quit);
            }
            key_code!(KeyCode::Esc) | key_code!(KeyCode::Backspace) | key_code!(KeyCode::Tab) => {
                self.tx.send(AppEvent::CloseRefs);
            }
            key_code_char!('j') | key_code!(KeyCode::Down) => {
                self.ref_list_state.select_next();
                self.update_commit_list_selected();
            }
            key_code_char!('k') | key_code!(KeyCode::Up) => {
                self.ref_list_state.select_prev();
                self.update_commit_list_selected();
            }
            key_code_char!('g') => {
                self.ref_list_state.select_first();
                self.update_commit_list_selected();
            }
            key_code_char!('G') => {
                self.ref_list_state.select_last();
                self.update_commit_list_selected();
            }
            key_code_char!('l') | key_code!(KeyCode::Right) => {
                self.ref_list_state.open_node();
                self.update_commit_list_selected();
            }
            key_code_char!('h') | key_code!(KeyCode::Left) => {
                self.ref_list_state.close_node();
                self.update_commit_list_selected();
            }
            key_code_char!('c') => {
                self.copy_ref_name();
            }
            key_code_char!('?') => {
                self.tx.send(AppEvent::OpenHelp);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let [list_area, refs_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(26)]).areas(area);

        let commit_list = CommitList::new(&self.config.ui.list);
        f.render_stateful_widget(commit_list, list_area, self.as_mut_list_state());

        let ref_list = RefList::new(&self.refs);
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
}
