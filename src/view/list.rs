use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    color::ColorTheme,
    config::UiConfig,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::{CommitHash, Ref},
    widget::commit_list::{CommitList, CommitListState, FilterState, SearchState},
};

#[derive(Debug)]
pub struct ListView<'a> {
    commit_list_state: Option<CommitListState>,

    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    tx: Sender,
}

impl<'a> ListView<'a> {
    pub fn new(
        commit_list_state: CommitListState,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> ListView<'a> {
        ListView {
            commit_list_state: Some(commit_list_state),
            ui_config,
            color_theme,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, key: KeyEvent) {
        if self.commit_list_state.is_none() {
            return;
        }

        let event = event_with_count.event;
        let count = event_with_count.count;

        // Handle filter mode input
        if let FilterState::Filtering { .. } = self.as_list_state().filter_state() {
            match event {
                UserEvent::Confirm => {
                    self.as_mut_list_state().apply_filter();
                    self.clear_filter_query();
                }
                UserEvent::Cancel => {
                    self.as_mut_list_state().cancel_filter();
                    self.clear_filter_query();
                }
                UserEvent::IgnoreCaseToggle => {
                    self.as_mut_list_state().toggle_filter_ignore_case();
                    self.update_filter_query();
                }
                UserEvent::FuzzyToggle => {
                    self.as_mut_list_state().toggle_filter_fuzzy();
                    self.update_filter_query();
                }
                _ => {
                    self.as_mut_list_state().handle_filter_input(key);
                    self.update_filter_query();
                }
            }
            return;
        }

        // Handle search mode input
        if let SearchState::Searching { .. } = self.as_list_state().search_state() {
            match event {
                UserEvent::Confirm => {
                    self.as_mut_list_state().apply_search();
                    self.update_matched_message();
                }
                UserEvent::Cancel => {
                    self.as_mut_list_state().cancel_search();
                    self.clear_search_query();
                }
                UserEvent::IgnoreCaseToggle => {
                    self.as_mut_list_state().toggle_ignore_case();
                    self.update_search_query();
                }
                UserEvent::FuzzyToggle => {
                    self.as_mut_list_state().toggle_fuzzy();
                    self.update_search_query();
                }
                _ => {
                    self.as_mut_list_state().handle_search_input(key);
                    self.update_search_query();
                }
            }
            return;
        }

        // Normal mode
        match event {
            UserEvent::Quit => {
                self.tx.send(AppEvent::Quit);
            }
            UserEvent::NavigateDown | UserEvent::SelectDown => {
                for _ in 0..count {
                    self.as_mut_list_state().select_next();
                }
            }
            UserEvent::NavigateUp | UserEvent::SelectUp => {
                for _ in 0..count {
                    self.as_mut_list_state().select_prev();
                }
            }
            UserEvent::GoToParent => {
                for _ in 0..count {
                    self.as_mut_list_state().select_parent();
                }
            }
            UserEvent::GoToTop => {
                self.as_mut_list_state().select_first();
            }
            UserEvent::GoToBottom => {
                self.as_mut_list_state().select_last();
            }
            UserEvent::ScrollDown => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_down();
                }
            }
            UserEvent::ScrollUp => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_up();
                }
            }
            UserEvent::PageDown => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_down_page();
                }
            }
            UserEvent::PageUp => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_up_page();
                }
            }
            UserEvent::HalfPageDown => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_down_half();
                }
            }
            UserEvent::HalfPageUp => {
                for _ in 0..count {
                    self.as_mut_list_state().scroll_up_half();
                }
            }
            UserEvent::SelectTop => {
                self.as_mut_list_state().select_high();
            }
            UserEvent::SelectMiddle => {
                self.as_mut_list_state().select_middle();
            }
            UserEvent::SelectBottom => {
                self.as_mut_list_state().select_low();
            }
            UserEvent::ShortCopy => {
                self.copy_commit_short_hash();
            }
            UserEvent::FullCopy => {
                self.copy_commit_hash();
            }
            UserEvent::Search => {
                self.as_mut_list_state().start_search();
                self.update_search_query();
            }
            UserEvent::Filter => {
                self.as_mut_list_state().start_filter();
                self.update_filter_query();
            }
            UserEvent::UserCommandViewToggle(n) => {
                self.tx.send(AppEvent::OpenUserCommand(n));
            }
            UserEvent::HelpToggle => {
                self.tx.send(AppEvent::OpenHelp);
            }
            UserEvent::Cancel => {
                self.as_mut_list_state().cancel_search();
                self.as_mut_list_state().cancel_filter();
                self.clear_search_query();
            }
            UserEvent::Confirm => {
                self.tx.send(AppEvent::OpenDetail);
            }
            UserEvent::RefListToggle => {
                self.tx.send(AppEvent::OpenRefs);
            }
            UserEvent::CreateTag => {
                self.tx.send(AppEvent::OpenCreateTag);
            }
            UserEvent::DeleteTag => {
                self.tx.send(AppEvent::OpenDeleteTag);
            }
            UserEvent::Refresh => {
                self.tx.send(AppEvent::Refresh);
            }
            _ => {}
        }

        if let SearchState::Applied { .. } = self.as_list_state().search_state() {
            match event {
                UserEvent::GoToNext => {
                    self.as_mut_list_state().select_next_match();
                    self.update_matched_message();
                }
                UserEvent::GoToPrevious => {
                    self.as_mut_list_state().select_prev_match();
                    self.update_matched_message();
                }
                _ => {}
            }
            // Do not return here
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let Some(list_state) = self.commit_list_state.as_mut() else {
            return;
        };
        let commit_list = CommitList::new(&self.ui_config.list, self.color_theme);
        f.render_stateful_widget(commit_list, area, list_state);
    }
}

impl<'a> ListView<'a> {
    pub fn take_list_state(&mut self) -> Option<CommitListState> {
        self.commit_list_state.take()
    }

    pub fn add_ref_to_commit(&mut self, commit_hash: &CommitHash, new_ref: Ref) {
        if let Some(list_state) = self.commit_list_state.as_mut() {
            list_state.add_ref_to_commit(commit_hash, new_ref);
        }
    }

    pub fn remove_ref_from_commit(&mut self, commit_hash: &CommitHash, tag_name: &str) {
        if let Some(list_state) = self.commit_list_state.as_mut() {
            list_state.remove_ref_from_commit(commit_hash, tag_name);
        }
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState {
        self.commit_list_state.as_mut().expect("commit_list_state already taken")
    }

    fn as_list_state(&self) -> &CommitListState {
        self.commit_list_state.as_ref().expect("commit_list_state already taken")
    }

    fn update_search_query(&self) {
        let Some(list_state) = self.commit_list_state.as_ref() else {
            return;
        };
        if let SearchState::Searching { .. } = list_state.search_state() {
            if let Some(query) = list_state.search_query_string() {
                let cursor_pos = list_state.search_query_cursor_position();
                let transient_msg = list_state.transient_message_string();
                self.tx.send(AppEvent::UpdateStatusInput(
                    query,
                    Some(cursor_pos),
                    transient_msg,
                ));
            }
        }
    }

    fn clear_search_query(&self) {
        self.tx.send(AppEvent::ClearStatusLine);
    }

    fn update_filter_query(&self) {
        if let FilterState::Filtering { .. } = self.as_list_state().filter_state() {
            let list_state = self.as_list_state();
            if let Some(query) = list_state.filter_query_string() {
                let cursor_pos = list_state.filter_query_cursor_position();
                let transient_msg = list_state.filter_transient_message_string();
                self.tx.send(AppEvent::UpdateStatusInput(
                    query,
                    Some(cursor_pos),
                    transient_msg,
                ));
            }
        }
    }

    fn clear_filter_query(&self) {
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
