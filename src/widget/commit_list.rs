use std::{collections::HashMap, rc::Rc};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use laurier::highlight::highlight_matched_text;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget, Widget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    color::ColorTheme,
    config::UiListConfig,
    git::{Commit, CommitHash, Head, Ref},
    graph::GraphImageManager,
};

static FUZZY_MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().respect_case());

const ELLIPSIS: &str = "...";

#[derive(Debug)]
pub struct CommitInfo {
    commit: Rc<Commit>,
    refs: Vec<Rc<Ref>>,
    graph_color: Color,
}

impl CommitInfo {
    pub fn new(commit: Rc<Commit>, refs: Vec<Rc<Ref>>, graph_color: Color) -> Self {
        Self {
            commit,
            refs,
            graph_color,
        }
    }

    pub fn commit_hash(&self) -> &CommitHash {
        &self.commit.commit_hash
    }

    fn add_ref(&mut self, r: Rc<Ref>) {
        self.refs.push(r);
        self.refs.sort_by(|a, b| a.cmp(b));
    }

    fn remove_ref(&mut self, name: &str) {
        self.refs.retain(|r| r.name() != name);
    }

    fn refs_to_vec(&self) -> Vec<Rc<Ref>> {
        self.refs.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchState {
    Inactive,
    Searching {
        start_index: usize,
        match_index: usize,
        ignore_case: bool,
        fuzzy: bool,
        transient_message: TransientMessage,
    },
    Applied {
        match_index: usize,
        total_match: usize,
    },
}

impl SearchState {
    fn update_match_index(&mut self, index: usize) {
        match self {
            SearchState::Searching { match_index, .. } => *match_index = index,
            SearchState::Applied { match_index, .. } => *match_index = index,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransientMessage {
    None,
    IgnoreCaseOff,
    IgnoreCaseOn,
    FuzzyOff,
    FuzzyOn,
}

#[derive(Debug, Default, Clone)]
struct SearchMatch {
    refs: HashMap<String, SearchMatchPosition>,
    subject: Option<SearchMatchPosition>,
    author_name: Option<SearchMatchPosition>,
    commit_hash: Option<SearchMatchPosition>,
    match_index: usize, // 1-based
}

impl SearchMatch {
    fn new<'a>(
        c: &Commit,
        refs: impl Iterator<Item = &'a Ref>,
        q: &str,
        ignore_case: bool,
        fuzzy: bool,
    ) -> Self {
        let matcher = SearchMatcher::new(q, ignore_case, fuzzy);
        let refs = refs
            .filter(|r| !matches!(r, Ref::Stash { .. }))
            .filter_map(|r| {
                matcher
                    .matched_position(r.name())
                    .map(|pos| (r.name().into(), pos))
            })
            .collect();
        let subject = matcher.matched_position(&c.subject);
        let author_name = matcher.matched_position(&c.author_name);
        let commit_hash = matcher.matched_position(&c.commit_hash.as_short_hash());
        Self {
            refs,
            subject,
            author_name,
            commit_hash,
            match_index: 0,
        }
    }

    fn matched(&self) -> bool {
        !self.refs.is_empty()
            || self.subject.is_some()
            || self.author_name.is_some()
            || self.commit_hash.is_some()
    }

    fn clear(&mut self) {
        self.refs.clear();
        self.subject = None;
        self.author_name = None;
        self.commit_hash = None;
    }
}

#[derive(Debug, Default, Clone)]
struct SearchMatchPosition {
    matched_indices: Vec<usize>,
}

impl SearchMatchPosition {
    fn new(matched_indices: Vec<usize>) -> Self {
        Self { matched_indices }
    }
}

struct SearchMatcher {
    query: String,
    ignore_case: bool,
    fuzzy: bool,
}

impl SearchMatcher {
    fn new(query: &str, ignore_case: bool, fuzzy: bool) -> Self {
        let query = if ignore_case {
            query.to_lowercase()
        } else {
            query.into()
        };
        Self {
            query,
            ignore_case,
            fuzzy,
        }
    }

    fn matched_position(&self, s: &str) -> Option<SearchMatchPosition> {
        if self.fuzzy {
            let result = if self.ignore_case {
                FUZZY_MATCHER.fuzzy_indices(&s.to_lowercase(), &self.query)
            } else {
                FUZZY_MATCHER.fuzzy_indices(s, &self.query)
            };
            result
                .map(|(_, indices)| indices)
                .map(SearchMatchPosition::new)
        } else {
            let result = if self.ignore_case {
                s.to_lowercase().find(&self.query)
            } else {
                s.find(&self.query)
            };
            result
                .map(|p| (p..(p + self.query.len())).collect())
                .map(SearchMatchPosition::new)
        }
    }
}

#[derive(Debug)]
pub struct CommitListState {
    commits: Vec<CommitInfo>,
    graph_image_manager: GraphImageManager,
    graph_cell_width: u16,
    head: Head,

    ref_name_to_commit_index_map: HashMap<String, usize>,

    search_state: SearchState,
    search_input: Input,
    search_matches: Vec<SearchMatch>,

    selected: usize,
    offset: usize,
    total: usize,
    height: usize,

    default_ignore_case: bool,
    default_fuzzy: bool,
}

impl CommitListState {
    pub fn new(
        commits: Vec<CommitInfo>,
        graph_image_manager: GraphImageManager,
        graph_cell_width: u16,
        head: Head,
        ref_name_to_commit_index_map: HashMap<String, usize>,
        default_ignore_case: bool,
        default_fuzzy: bool,
    ) -> CommitListState {
        let total = commits.len();
        CommitListState {
            commits,
            graph_image_manager,
            graph_cell_width,
            head,
            ref_name_to_commit_index_map,
            search_state: SearchState::Inactive,
            search_input: Input::default(),
            search_matches: vec![SearchMatch::default(); total],
            selected: 0,
            offset: 0,
            total,
            height: 0,
            default_ignore_case,
            default_fuzzy,
        }
    }

    pub fn graph_area_cell_width(&self) -> u16 {
        self.graph_cell_width + 1 // right pad
    }

    pub fn add_ref_to_commit(&mut self, commit_hash: &CommitHash, new_ref: Ref) {
        for (index, commit_info) in self.commits.iter_mut().enumerate() {
            if commit_info.commit_hash() == commit_hash {
                self.ref_name_to_commit_index_map
                    .insert(new_ref.name().to_string(), index);
                commit_info.add_ref(Rc::new(new_ref));
                break;
            }
        }
    }

    pub fn remove_ref_from_commit(&mut self, commit_hash: &CommitHash, tag_name: &str) {
        for commit_info in self.commits.iter_mut() {
            if commit_info.commit_hash() == commit_hash {
                self.ref_name_to_commit_index_map.remove(tag_name);
                commit_info.remove_ref(tag_name);
                break;
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < (self.total - 1).min(self.height - 1) {
            self.selected += 1;
        } else if self.selected + self.offset < self.total - 1 {
            self.offset += 1;
        }
    }

    pub fn select_parent(&mut self) {
        if let Some(target_commit) = self.selected_commit_parent_hash().cloned() {
            while target_commit.as_str() != self.selected_commit_hash().as_str() {
                self.select_next();
            }
        }
    }

    pub fn selected_commit_parent_hash(&self) -> Option<&CommitHash> {
        self.commits[self.current_selected_index()]
            .commit
            .parent_commit_hashes
            .first()
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else if self.offset > 0 {
            self.offset -= 1;
        }
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = (self.height - 1).min(self.total - 1);
        if self.height < self.total {
            self.offset = self.total - self.height;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.offset + self.height < self.total {
            self.offset += 1;
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
            if self.selected < self.height - 1 {
                self.selected += 1;
            }
        }
    }

    pub fn scroll_down_page(&mut self) {
        self.scroll_down_height(self.height);
    }

    pub fn scroll_up_page(&mut self) {
        self.scroll_up_height(self.height);
    }

    pub fn scroll_down_half(&mut self) {
        self.scroll_down_height(self.height / 2);
    }

    pub fn scroll_up_half(&mut self) {
        self.scroll_up_height(self.height / 2);
    }

    fn scroll_down_height(&mut self, scroll_height: usize) {
        if self.offset + self.height + scroll_height < self.total {
            self.offset += scroll_height;
        } else {
            let old_offset = self.offset;
            let size = self.height.min(self.total);
            self.offset = self.total - size;
            self.selected += scroll_height - (self.offset - old_offset);
            if self.selected >= size {
                self.selected = size - 1;
            }
        }
    }

    fn scroll_up_height(&mut self, scroll_height: usize) {
        if self.offset > scroll_height {
            self.offset -= scroll_height;
        } else {
            let old_offset = self.offset;
            self.offset = 0;
            self.selected = self
                .selected
                .saturating_sub(scroll_height - (old_offset - self.offset));
        }
    }

    pub fn select_high(&mut self) {
        self.selected = 0;
    }

    pub fn select_middle(&mut self) {
        if self.total > self.height {
            self.selected = self.height / 2;
        } else {
            self.selected = self.total / 2;
        }
    }

    pub fn select_low(&mut self) {
        if self.total > self.height {
            self.selected = self.height - 1;
        } else {
            self.selected = self.total - 1;
        }
    }

    fn select_index(&mut self, index: usize) {
        if index < self.total {
            if self.total > self.height {
                self.selected = 0;
                self.offset = index;
            } else {
                self.selected = index;
            }
        }
    }

    pub fn select_next_match(&mut self) {
        self.select_next_match_index(self.current_selected_index());
    }

    pub fn select_prev_match(&mut self) {
        self.select_prev_match_index(self.current_selected_index());
    }

    pub fn selected_commit_hash(&self) -> &CommitHash {
        &self.commits[self.current_selected_index()]
            .commit
            .commit_hash
    }

    pub fn selected_commit_refs(&self) -> Vec<Rc<Ref>> {
        self.commits[self.current_selected_index()].refs_to_vec()
    }

    fn current_selected_index(&self) -> usize {
        self.offset + self.selected
    }

    pub fn select_ref(&mut self, ref_name: &str) {
        if let Some(&index) = self.ref_name_to_commit_index_map.get(ref_name) {
            if self.total > self.height {
                self.selected = 0;
                self.offset = index;
            } else {
                self.selected = index;
            }
        }
    }

    pub fn select_commit_hash(&mut self, commit_hash: &CommitHash) {
        for (i, commit_info) in self.commits.iter().enumerate() {
            if commit_info.commit.commit_hash == *commit_hash {
                if self.total > self.height {
                    self.selected = 0;
                    self.offset = i;
                } else {
                    self.selected = i;
                }
                break;
            }
        }
    }

    pub fn search_state(&self) -> SearchState {
        self.search_state
    }

    pub fn start_search(&mut self) {
        if let SearchState::Inactive | SearchState::Applied { .. } = self.search_state {
            self.search_state = SearchState::Searching {
                start_index: self.current_selected_index(),
                match_index: 0,
                ignore_case: self.default_ignore_case,
                fuzzy: self.default_fuzzy,
                transient_message: TransientMessage::None,
            };
            self.search_input.reset();
            self.clear_search_matches();
        }
    }

    pub fn handle_search_input(&mut self, key: KeyEvent) {
        if let SearchState::Searching {
            transient_message, ..
        } = &mut self.search_state
        {
            *transient_message = TransientMessage::None;
        }

        if let SearchState::Searching {
            start_index,
            ignore_case,
            fuzzy,
            ..
        } = self.search_state
        {
            self.search_input.handle_event(&Event::Key(key));
            self.update_search_matches(ignore_case, fuzzy);
            self.select_current_or_next_match_index(start_index);
        }
    }

    pub fn apply_search(&mut self) {
        if let SearchState::Searching { match_index, .. } = self.search_state {
            if self.search_input.value().is_empty() {
                self.search_state = SearchState::Inactive;
            } else {
                let total_match = self.search_matches.iter().filter(|m| m.matched()).count();
                self.search_state = SearchState::Applied {
                    match_index,
                    total_match,
                };
            }
        }
    }

    pub fn cancel_search(&mut self) {
        if let SearchState::Searching { .. } | SearchState::Applied { .. } = self.search_state {
            self.search_state = SearchState::Inactive;
            self.search_input.reset();
            self.clear_search_matches();
        }
    }

    pub fn toggle_ignore_case(&mut self) {
        if let SearchState::Searching {
            ignore_case,
            transient_message,
            ..
        } = &mut self.search_state
        {
            *ignore_case = !*ignore_case;
            *transient_message = if *ignore_case {
                TransientMessage::IgnoreCaseOn
            } else {
                TransientMessage::IgnoreCaseOff
            };
        }

        if let SearchState::Searching {
            start_index,
            ignore_case,
            fuzzy,
            ..
        } = self.search_state
        {
            self.update_search_matches(ignore_case, fuzzy);
            self.select_current_or_next_match_index(start_index);
        }
    }

    pub fn toggle_fuzzy(&mut self) {
        if let SearchState::Searching {
            fuzzy,
            transient_message,
            ..
        } = &mut self.search_state
        {
            *fuzzy = !*fuzzy;
            *transient_message = if *fuzzy {
                TransientMessage::FuzzyOn
            } else {
                TransientMessage::FuzzyOff
            };
        }

        if let SearchState::Searching {
            start_index,
            ignore_case,
            fuzzy,
            ..
        } = self.search_state
        {
            self.update_search_matches(ignore_case, fuzzy);
            self.select_current_or_next_match_index(start_index);
        }
    }

    pub fn search_query_string(&self) -> Option<String> {
        if let SearchState::Searching { .. } = self.search_state {
            let query = self.search_input.value();
            Some(format!("/{query}"))
        } else {
            None
        }
    }

    pub fn matched_query_string(&self) -> Option<(String, bool)> {
        if let SearchState::Applied {
            match_index,
            total_match,
            ..
        } = self.search_state
        {
            let query = self.search_input.value();
            if total_match == 0 {
                let msg = format!("No matches found (query: \"{query}\")");
                Some((msg, false))
            } else {
                let msg = format!("Match {match_index} of {total_match} (query: \"{query}\")");
                Some((msg, true))
            }
        } else {
            None
        }
    }

    pub fn search_query_cursor_position(&self) -> u16 {
        self.search_input.visual_cursor() as u16 + 1 // add 1 for "/"
    }

    pub fn transient_message_string(&self) -> Option<String> {
        if let SearchState::Searching {
            transient_message, ..
        } = self.search_state
        {
            match transient_message {
                TransientMessage::None => None,
                TransientMessage::IgnoreCaseOn => Some("Ignore case: ON ".to_string()),
                TransientMessage::IgnoreCaseOff => Some("Ignore case: OFF".to_string()),
                TransientMessage::FuzzyOn => Some("Fuzzy match: ON ".to_string()),
                TransientMessage::FuzzyOff => Some("Fuzzy match: OFF".to_string()),
            }
        } else {
            None
        }
    }

    fn update_search_matches(&mut self, ignore_case: bool, fuzzy: bool) {
        let q = self.search_input.value();
        let mut match_index = 1;
        for (i, commit_info) in self.commits.iter().enumerate() {
            let mut m = SearchMatch::new(
                &commit_info.commit,
                commit_info.refs.iter().map(|r| r.as_ref()),
                q,
                ignore_case,
                fuzzy,
            );
            if m.matched() {
                m.match_index = match_index;
                match_index += 1;
            }
            self.search_matches[i] = m;
        }
    }

    fn clear_search_matches(&mut self) {
        self.search_matches.iter_mut().for_each(|m| m.clear());
    }

    fn select_current_or_next_match_index(&mut self, current_index: usize) {
        if self.search_matches[current_index].matched() {
            self.select_index(current_index);
            self.search_state
                .update_match_index(self.search_matches[current_index].match_index);
        } else {
            self.select_next_match_index(current_index)
        }
    }

    fn select_next_match_index(&mut self, current_index: usize) {
        let mut i = (current_index + 1) % self.total;
        while i != current_index {
            if self.search_matches[i].matched() {
                self.select_index(i);
                self.search_state
                    .update_match_index(self.search_matches[i].match_index);
                return;
            }
            if i == self.total - 1 {
                i = 0;
            } else {
                i += 1;
            }
        }
    }

    fn select_prev_match_index(&mut self, current_index: usize) {
        let mut i = (current_index + self.total - 1) % self.total;
        while i != current_index {
            if self.search_matches[i].matched() {
                self.select_index(i);
                self.search_state
                    .update_match_index(self.search_matches[i].match_index);
                return;
            }
            if i == 0 {
                i = self.total - 1;
            } else {
                i -= 1;
            }
        }
    }

    fn encoded_image(&self, commit_info: &CommitInfo) -> &str {
        self.graph_image_manager
            .encoded_image(&commit_info.commit.commit_hash)
    }
}

pub struct CommitList<'a> {
    config: &'a UiListConfig,
    color_theme: &'a ColorTheme,
}

impl<'a> CommitList<'a> {
    pub fn new(config: &'a UiListConfig, color_theme: &'a ColorTheme) -> Self {
        Self {
            config,
            color_theme,
        }
    }
}

impl<'a> StatefulWidget for CommitList<'a> {
    type State = CommitListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.update_state(area, state);

        let (
            graph_cell_width,
            marker_cell_width,
            name_cell_width,
            hash_cell_width,
            date_cell_width,
        ) = self.calc_cell_widths(
            state,
            area.width,
            self.config.subject_min_width,
            self.config.name_width,
            self.config.date_width,
        );

        let chunks = Layout::horizontal([
            Constraint::Length(graph_cell_width),
            Constraint::Length(marker_cell_width),
            Constraint::Min(0), // subject
            Constraint::Length(name_cell_width),
            Constraint::Length(hash_cell_width),
            Constraint::Length(date_cell_width),
        ])
        .split(area);

        self.render_graph(buf, chunks[0], state);
        self.render_marker(buf, chunks[1], state);
        self.render_subject(buf, chunks[2], state);
        self.render_name(buf, chunks[3], state);
        self.render_hash(buf, chunks[4], state);
        self.render_date(buf, chunks[5], state);
    }
}

impl CommitList<'_> {
    fn update_state(&self, area: Rect, state: &mut CommitListState) {
        state.height = area.height as usize;

        if state.total > state.height && state.total - state.height < state.offset {
            let diff = state.offset - (state.total - state.height);
            state.selected += diff;
            state.offset -= diff;
        }
        if state.selected >= state.height {
            let diff = state.selected - state.height + 1;
            state.selected -= diff;
            state.offset += diff;
        }

        state
            .commits
            .iter()
            .skip(state.offset)
            .take(state.height)
            .for_each(|commit_info| {
                state
                    .graph_image_manager
                    .load_encoded_image(&commit_info.commit.commit_hash);
            });
    }

    fn calc_cell_widths(
        &self,
        state: &CommitListState,
        width: u16,
        subject_min_width: u16,
        name_width: u16,
        date_width: u16,
    ) -> (u16, u16, u16, u16, u16) {
        let pad = 2;
        let graph_cell_width = state.graph_area_cell_width();
        let marker_cell_width = 1;
        let mut name_cell_width = name_width + pad;
        let mut hash_cell_width = 7 + pad;
        let mut date_cell_width = date_width + pad;

        let mut total_width = graph_cell_width
            + marker_cell_width
            + hash_cell_width
            + name_cell_width
            + date_cell_width
            + subject_min_width;

        if total_width > width {
            total_width = total_width.saturating_sub(name_cell_width);
            name_cell_width = 0;
        }
        if total_width > width {
            total_width = total_width.saturating_sub(date_cell_width);
            date_cell_width = 0;
        }
        if total_width > width {
            hash_cell_width = 0;
        }

        (
            graph_cell_width,
            marker_cell_width,
            name_cell_width,
            hash_cell_width,
            date_cell_width,
        )
    }

    fn render_graph(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        self.rendering_commit_info_iter(state)
            .for_each(|(i, commit_info)| {
                buf[(area.left(), area.top() + i as u16)]
                    .set_symbol(state.encoded_image(commit_info));

                // width - 1 for right pad
                for w in 1..area.width - 1 {
                    buf[(area.left() + w, area.top() + i as u16)].set_skip(true);
                }
            });
    }

    fn render_marker(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        let items: Vec<ListItem> = self
            .rendering_commit_info_iter(state)
            .map(|(_, commit_info)| ListItem::new("│".fg(commit_info.graph_color)))
            .collect();
        Widget::render(List::new(items), area, buf)
    }

    fn render_subject(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        let max_width = (area.width as usize).saturating_sub(2);
        if area.is_empty() || max_width == 0 {
            return;
        }
        let items: Vec<ListItem> = self
            .rendering_commit_info_iter(state)
            .map(|(i, commit_info)| {
                let mut spans = refs_spans(
                    commit_info,
                    &state.head,
                    &state.search_matches[state.offset + i].refs,
                    self.color_theme,
                );
                let ref_spans_width: usize = spans.iter().map(|s| s.width()).sum();
                let max_width = max_width.saturating_sub(ref_spans_width);
                let commit = &commit_info.commit;
                if max_width > ELLIPSIS.len() {
                    let truncate = console::measure_text_width(&commit.subject) > max_width;
                    let subject = if truncate {
                        console::truncate_str(&commit.subject, max_width, ELLIPSIS).to_string()
                    } else {
                        commit.subject.to_string()
                    };

                    let sub_spans =
                        if let Some(pos) = state.search_matches[state.offset + i].subject.clone() {
                            highlighted_spans(
                                subject.into(),
                                pos,
                                self.color_theme.list_subject_fg,
                                Modifier::empty(),
                                self.color_theme,
                                truncate,
                            )
                        } else {
                            vec![subject.fg(self.color_theme.list_subject_fg)]
                        };

                    spans.extend(sub_spans)
                }
                self.to_commit_list_item(i, spans, state)
            })
            .collect();
        Widget::render(List::new(items), area, buf);
    }

    fn render_name(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        let max_width = (area.width as usize).saturating_sub(2);
        if area.is_empty() || max_width == 0 {
            return;
        }
        let items: Vec<ListItem> = self
            .rendering_commit_iter(state)
            .map(|(i, commit)| {
                let truncate = console::measure_text_width(&commit.author_name) > max_width;
                let name = if truncate {
                    console::truncate_str(&commit.author_name, max_width, ELLIPSIS).to_string()
                } else {
                    commit.author_name.to_string()
                };
                let spans =
                    if let Some(pos) = state.search_matches[state.offset + i].author_name.clone() {
                        highlighted_spans(
                            name.into(),
                            pos,
                            self.color_theme.list_name_fg,
                            Modifier::empty(),
                            self.color_theme,
                            truncate,
                        )
                    } else {
                        vec![name.fg(self.color_theme.list_name_fg)]
                    };
                self.to_commit_list_item(i, spans, state)
            })
            .collect();
        Widget::render(List::new(items), area, buf);
    }

    fn render_hash(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        if area.is_empty() {
            return;
        }
        let items: Vec<ListItem> = self
            .rendering_commit_iter(state)
            .map(|(i, commit)| {
                let hash = commit.commit_hash.as_short_hash();
                let spans =
                    if let Some(pos) = state.search_matches[state.offset + i].commit_hash.clone() {
                        highlighted_spans(
                            hash.into(),
                            pos,
                            self.color_theme.list_hash_fg,
                            Modifier::empty(),
                            self.color_theme,
                            false,
                        )
                    } else {
                        vec![hash.fg(self.color_theme.list_hash_fg)]
                    };
                self.to_commit_list_item(i, spans, state)
            })
            .collect();
        Widget::render(List::new(items), area, buf);
    }

    fn render_date(&self, buf: &mut Buffer, area: Rect, state: &CommitListState) {
        if area.is_empty() {
            return;
        }
        let items: Vec<ListItem> = self
            .rendering_commit_iter(state)
            .map(|(i, commit)| {
                let date = &commit.author_date;
                let date_str = if self.config.date_local {
                    let local = date.with_timezone(&chrono::Local);
                    local.format(&self.config.date_format).to_string()
                } else {
                    date.format(&self.config.date_format).to_string()
                };
                self.to_commit_list_item(i, vec![date_str.fg(self.color_theme.list_date_fg)], state)
            })
            .collect();
        Widget::render(List::new(items), area, buf);
    }

    fn rendering_commit_info_iter<'b>(
        &'b self,
        state: &'b CommitListState,
    ) -> impl Iterator<Item = (usize, &'b CommitInfo)> {
        state
            .commits
            .iter()
            .skip(state.offset)
            .take(state.height)
            .enumerate()
    }

    fn rendering_commit_iter<'b>(
        &'b self,
        state: &'b CommitListState,
    ) -> impl Iterator<Item = (usize, &'b Commit)> {
        self.rendering_commit_info_iter(state)
            .map(|(i, commit_info)| (i, commit_info.commit.as_ref()))
    }

    fn to_commit_list_item<'a, 'b>(
        &'b self,
        i: usize,
        spans: Vec<Span<'a>>,
        state: &'b CommitListState,
    ) -> ListItem<'a> {
        let mut spans = spans;
        spans.insert(0, Span::raw(" "));
        spans.push(Span::raw(" "));
        let mut line = Line::from(spans);
        if i == state.selected {
            line = line
                .bg(self.color_theme.list_selected_bg)
                .fg(self.color_theme.list_selected_fg);
        }
        ListItem::new(line)
    }
}

fn refs_spans<'a>(
    commit_info: &'a CommitInfo,
    head: &'a Head,
    refs_matches: &'a HashMap<String, SearchMatchPosition>,
    color_theme: &'a ColorTheme,
) -> Vec<Span<'a>> {
    let refs = &commit_info.refs;

    if refs.len() == 1 {
        if let Ref::Stash { name, .. } = refs[0].as_ref() {
            return vec![
                Span::raw(name.clone())
                    .fg(color_theme.list_ref_stash_fg)
                    .bold(),
                Span::raw(" "),
            ];
        }
    }

    let ref_spans: Vec<(Vec<Span>, &String)> = refs
        .iter()
        .filter_map(|r| match r.as_ref() {
            Ref::Branch { name, .. } => {
                let fg = color_theme.list_ref_branch_fg;
                Some((name, fg))
            }
            Ref::RemoteBranch { name, .. } => {
                let fg = color_theme.list_ref_remote_branch_fg;
                Some((name, fg))
            }
            Ref::Tag { name, .. } => {
                let fg = color_theme.list_ref_tag_fg;
                Some((name, fg))
            }
            Ref::Stash { .. } => None,
        })
        .map(|(name, fg)| {
            let spans = refs_matches
                .get(name)
                .map(|pos| {
                    highlighted_spans(
                        name.into(),
                        pos.clone(),
                        fg,
                        Modifier::BOLD,
                        color_theme,
                        false,
                    )
                })
                .unwrap_or_else(|| vec![Span::raw(name).fg(fg).bold()]);
            (spans, name)
        })
        .collect();

    let mut spans = vec![Span::raw("(").fg(color_theme.list_ref_paren_fg).bold()];

    if let Head::Detached { target } = head {
        if commit_info.commit.commit_hash == *target {
            spans.push(Span::raw("HEAD").fg(color_theme.list_head_fg).bold());
            if !ref_spans.is_empty() {
                spans.push(Span::raw(", ").fg(color_theme.list_ref_paren_fg).bold());
            }
        }
    }

    let refs_len = refs.len();
    for (i, ss) in ref_spans.into_iter().enumerate() {
        let (ref_spans, ref_name) = ss;
        if let Head::Branch { name } = head {
            if ref_name == name {
                spans.push(Span::raw("HEAD -> ").fg(color_theme.list_head_fg).bold());
            }
        }
        spans.extend(ref_spans);
        if i < refs_len - 1 {
            spans.push(Span::raw(", ").fg(color_theme.list_ref_paren_fg).bold());
        }
    }

    spans.push(Span::raw(") ").fg(color_theme.list_ref_paren_fg).bold());

    if spans.len() == 2 {
        spans.clear(); // contains only "(" and ")", so clear it
    }

    spans
}

fn highlighted_spans(
    s: Span<'_>,
    pos: SearchMatchPosition,
    base_fg: Color,
    base_modifier: Modifier,
    color_theme: &ColorTheme,
    truncate: bool,
) -> Vec<Span<'static>> {
    let mut hm = highlight_matched_text(vec![s])
        .matched_indices(pos.matched_indices)
        .not_matched_style(Style::default().fg(base_fg).add_modifier(base_modifier))
        .matched_style(
            Style::default()
                .fg(color_theme.list_match_fg)
                .bg(color_theme.list_match_bg)
                .add_modifier(base_modifier),
        );
    if truncate {
        hm = hm.ellipsis(ELLIPSIS);
    }
    hm.into_spans()
}
