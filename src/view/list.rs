use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    Frame,
};

use crate::{
    config::Config,
    event::{AppEvent, Sender},
    key_code, key_code_char,
    widget::commit_list::{CommitList, CommitListState, SearchState},
};

#[derive(Debug)]
pub struct ListView<'a> {
    commit_list_state: Option<CommitListState<'a>>,

    config: &'a Config,
    tx: Sender,
}

impl<'a> ListView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        config: &'a Config,
        tx: Sender,
    ) -> ListView<'a> {
        ListView {
            commit_list_state: Some(commit_list_state),
            config,
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let SearchState::Searching { .. } = self.as_list_state().search_state() {
            match key {
                key_code!(KeyCode::Enter) => {
                    self.as_mut_list_state().apply_search();
                    self.update_matched_message();
                }
                key_code!(KeyCode::Esc) => {
                    self.as_mut_list_state().cancel_search();
                    self.clear_search_query();
                }
                _ => {
                    self.as_mut_list_state().handle_search_input(key);
                    self.update_search_query();
                }
            }
            return;
        }

        if let SearchState::Applied { .. } = self.as_list_state().search_state() {
            match key {
                key_code_char!('n') => {
                    self.as_mut_list_state().select_next_match();
                    self.update_matched_message();
                }
                key_code_char!('N') => {
                    self.as_mut_list_state().select_prev_match();
                    self.update_matched_message();
                }
                _ => {}
            }
            // Do not return here
        }

        match key {
            key_code_char!('q') => {
                self.tx.send(AppEvent::Quit);
            }
            key_code_char!('j') | key_code!(KeyCode::Down) => {
                self.as_mut_list_state().select_next();
            }
            key_code_char!('k') | key_code!(KeyCode::Up) => {
                self.as_mut_list_state().select_prev();
            }
            key_code_char!('g') => {
                self.as_mut_list_state().select_first();
            }
            key_code_char!('G') => {
                self.as_mut_list_state().select_last();
            }
            key_code_char!('e', Ctrl) => {
                self.as_mut_list_state().scroll_down();
            }
            key_code_char!('y', Ctrl) => {
                self.as_mut_list_state().scroll_up();
            }
            key_code_char!('f', Ctrl) => {
                self.as_mut_list_state().scroll_down_page();
            }
            key_code_char!('b', Ctrl) => {
                self.as_mut_list_state().scroll_up_page();
            }
            key_code_char!('d', Ctrl) => {
                self.as_mut_list_state().scroll_down_half();
            }
            key_code_char!('u', Ctrl) => {
                self.as_mut_list_state().scroll_up_half();
            }
            key_code_char!('H') => {
                self.as_mut_list_state().select_high();
            }
            key_code_char!('M') => {
                self.as_mut_list_state().select_middle();
            }
            key_code_char!('L') => {
                self.as_mut_list_state().select_low();
            }
            key_code_char!('c') => {
                self.copy_commit_short_hash();
            }
            key_code_char!('C') => {
                self.copy_commit_hash();
            }
            key_code_char!('/') => {
                self.as_mut_list_state().start_search();
                self.update_search_query();
            }
            key_code_char!('?') => {
                self.tx.send(AppEvent::OpenHelp);
            }
            key_code!(KeyCode::Esc) => {
                self.as_mut_list_state().cancel_search();
                self.clear_search_query();
            }
            key_code!(KeyCode::Enter) => {
                self.tx.send(AppEvent::OpenDetail);
            }
            key_code!(KeyCode::Tab) => {
                self.tx.send(AppEvent::OpenRefs);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let commit_list = CommitList::new(&self.config.ui.list);
        f.render_stateful_widget(commit_list, area, self.as_mut_list_state());
    }
}

impl<'a> ListView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }

    fn as_list_state(&self) -> &CommitListState<'a> {
        self.commit_list_state.as_ref().unwrap()
    }

    fn update_search_query(&self) {
        let list_state = self.as_list_state();
        if let Some(query) = list_state.search_query_string() {
            let cursor_pos = list_state.search_query_cursor_position();
            self.tx
                .send(AppEvent::UpdateStatusInput(query, Some(cursor_pos)));
        }
    }

    fn clear_search_query(&self) {
        self.tx.send(AppEvent::ClearStatusLine);
    }

    fn update_matched_message(&self) {
        if let Some((msg, matched)) = self.as_list_state().matched_query_string() {
            if matched {
                self.tx.send(AppEvent::NotifyInfo(msg));
            } else {
                self.tx.send(AppEvent::NotifyWarn(msg));
            }
        } else {
            self.tx.send(AppEvent::ClearStatusLine);
        }
    }

    fn copy_commit_short_hash(&self) {
        let selected = self.as_list_state().selected_commit_hash();
        self.copy_to_clipboard("Commit SHA (short)".into(), selected.as_short_hash());
    }

    fn copy_commit_hash(&self) {
        let selected = self.as_list_state().selected_commit_hash();
        self.copy_to_clipboard("Commit SHA".into(), selected.as_str().into());
    }

    fn copy_to_clipboard(&self, name: String, value: String) {
        self.tx.send(AppEvent::CopyToClipboard { name, value });
    }
}
