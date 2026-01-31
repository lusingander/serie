use std::rc::Rc;

use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    app::AppContext,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    widget::commit_list::{CommitList, CommitListState, SearchState},
};

#[derive(Debug)]
pub struct ListView<'a> {
    commit_list_state: Option<CommitListState<'a>>,

    ctx: Rc<AppContext>,
    tx: Sender,
}

impl<'a> ListView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> ListView<'a> {
        ListView {
            commit_list_state: Some(commit_list_state),
            ctx,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, key: KeyEvent) {
        let event = event_with_count.event;
        let count = event_with_count.count;
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
        } else {
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
                UserEvent::UserCommandViewToggle(n) => {
                    self.tx.send(AppEvent::OpenUserCommand(n));
                }
                UserEvent::HelpToggle => {
                    self.tx.send(AppEvent::OpenHelp);
                }
                UserEvent::Cancel => {
                    self.as_mut_list_state().cancel_search();
                    self.clear_search_query();
                }
                UserEvent::Confirm => {
                    self.tx.send(AppEvent::OpenDetail);
                }
                UserEvent::RefListToggle => {
                    self.tx.send(AppEvent::OpenRefs);
                }
                _ => {}
            }
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
        let commit_list = CommitList::new(self.ctx.clone());
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
        if let SearchState::Searching { .. } = self.as_list_state().search_state() {
            let list_state = self.as_list_state();
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
