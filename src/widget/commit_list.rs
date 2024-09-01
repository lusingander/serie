use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget, Widget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    config::UiListConfig,
    git::{Commit, CommitHash, Head, Ref},
    graph::GraphImageManager,
};

const SELECTED_BACKGROUND_COLOR: Color = Color::DarkGray;
const SELECTED_FOREGROUND_COLOR: Color = Color::White;

const REF_PAREN_COLOR: Color = Color::Yellow;
const REF_BRANCH_COLOR: Color = Color::Green;
const REF_REMOTE_BRANCH_COLOR: Color = Color::Red;
const REF_TAG_COLOR: Color = Color::Yellow;
const REF_STASH_COLOR: Color = Color::Magenta;
const HEAD_COLOR: Color = Color::Cyan;

const SUBJECT_COLOR: Color = Color::Reset;
const NAME_COLOR: Color = Color::Cyan;
const HASH_COLOR: Color = Color::Yellow;
const DATE_COLOR: Color = Color::Magenta;
const MATCH_FOREGROUND_COLOR: Color = Color::Black;
const MATCH_BACKGROUND_COLOR: Color = Color::Yellow;

const ELLIPSIS: &str = "...";

#[derive(Debug)]
pub struct CommitInfo<'a> {
    commit: &'a Commit,
    refs: Vec<&'a Ref>,
    graph_color: Color,
}

impl<'a> CommitInfo<'a> {
    pub fn new(commit: &'a Commit, refs: Vec<&'a Ref>, graph_color: Color) -> Self {
        Self {
            commit,
            refs,
            graph_color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchState {
    Inactive,
    Searching {
        start_index: usize,
        match_index: usize,
    },
    Applied {
        start_index: usize,
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

#[derive(Debug, Default, Clone, Copy)]
struct SearchMatch {
    subject: Option<SearchMatchPosition>,
    author_name: Option<SearchMatchPosition>,
    commit_hash: Option<SearchMatchPosition>,
    match_index: usize, // 1-based
}

impl SearchMatch {
    fn matched(&self) -> bool {
        self.subject.is_some() || self.author_name.is_some() || self.commit_hash.is_some()
    }

    fn clear(&mut self) {
        self.subject = None;
        self.author_name = None;
        self.commit_hash = None;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct SearchMatchPosition {
    start: usize,
    end: usize,
}

impl SearchMatchPosition {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct CommitListState<'a> {
    commits: Vec<CommitInfo<'a>>,
    graph_image_manager: GraphImageManager,
    graph_cell_width: u16,
    head: &'a Head,

    ref_name_to_commit_index_map: HashMap<&'a str, usize>,

    search_state: SearchState,
    search_input: Input,
    search_matches: Vec<SearchMatch>,

    selected: usize,
    offset: usize,
    total: usize,
    height: usize,
}

impl<'a> CommitListState<'a> {
    pub fn new(
        commits: Vec<CommitInfo<'a>>,
        graph_image_manager: GraphImageManager,
        graph_cell_width: u16,
        head: &'a Head,
        ref_name_to_commit_index_map: HashMap<&'a str, usize>,
    ) -> CommitListState<'a> {
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
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < (self.total - 1).min(self.height - 1) {
            self.selected += 1;
        } else if self.selected + self.offset < self.total - 1 {
            self.offset += 1;
        }
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

    pub fn search_state(&self) -> SearchState {
        self.search_state
    }

    pub fn start_search(&mut self) {
        if let SearchState::Inactive | SearchState::Applied { .. } = self.search_state {
            self.search_state = SearchState::Searching {
                start_index: self.current_selected_index(),
                match_index: 0,
            };
            self.search_input.reset();
            self.clear_search_matches();
        }
    }

    pub fn handle_search_input(&mut self, key: KeyEvent) {
        if let SearchState::Searching { start_index, .. } = self.search_state {
            self.search_input.handle_event(&Event::Key(key));
            self.update_search_matches();
            self.select_current_or_next_match_index(start_index);
        }
    }

    pub fn apply_search(&mut self) {
        if let SearchState::Searching {
            start_index,
            match_index,
        } = self.search_state
        {
            if self.search_input.value().is_empty() {
                self.search_state = SearchState::Inactive;
            } else {
                let total_match = self.search_matches.iter().filter(|m| m.matched()).count();
                self.search_state = SearchState::Applied {
                    start_index,
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

    pub fn search_query_string(&self) -> Option<String> {
        if let SearchState::Searching { .. } = self.search_state {
            let query = self.search_input.value();
            Some(format!("/{}", query))
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
                let msg = format!("No matches found (query: \"{}\")", query);
                Some((msg, false))
            } else {
                let msg = format!(
                    "Match {} of {} (query: \"{}\")",
                    match_index, total_match, query
                );
                Some((msg, true))
            }
        } else {
            None
        }
    }

    pub fn search_query_cursor_position(&self) -> u16 {
        self.search_input.visual_cursor() as u16 + 1 // add 1 for "/"
    }

    fn update_search_matches(&mut self) {
        let query = self.search_input.value();
        let mut match_index = 1;
        for (i, commit_info) in self.commits.iter().enumerate() {
            let mut m = SearchMatch::default();
            if let Some(pos) = commit_info.commit.subject.find(query) {
                m.subject = Some(SearchMatchPosition::new(pos, pos + query.len()));
            }
            if let Some(pos) = commit_info.commit.author_name.find(query) {
                m.author_name = Some(SearchMatchPosition::new(pos, pos + query.len()));
            }
            if let Some(pos) = commit_info.commit.commit_hash.as_short_hash().find(query) {
                m.commit_hash = Some(SearchMatchPosition::new(pos, pos + query.len()));
            }
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

    fn encoded_image(&self, commit_info: &'a CommitInfo) -> &str {
        self.graph_image_manager
            .encoded_image(&commit_info.commit.commit_hash)
    }
}

pub struct CommitList<'a> {
    config: &'a UiListConfig,
}

impl<'a> CommitList<'a> {
    pub fn new(config: &'a UiListConfig) -> Self {
        Self { config }
    }
}

impl<'a> StatefulWidget for CommitList<'a> {
    type State = CommitListState<'a>;

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
        let graph_cell_width = state.graph_cell_width + 1; // right pad
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

                for w in 1..area.width {
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
                let mut spans = refs_spans(commit_info, state.head);
                let ref_spans_width: usize = spans.iter().map(|s| s.width()).sum();
                let max_width = max_width.saturating_sub(ref_spans_width);
                let commit = commit_info.commit;
                if max_width > ELLIPSIS.len() {
                    let truncate = console::measure_text_width(&commit.subject) > max_width;
                    let subject = if truncate {
                        console::truncate_str(&commit.subject, max_width, ELLIPSIS).to_string()
                    } else {
                        commit.subject.to_string()
                    };

                    let sub_spans = if let Some(SearchMatchPosition { start, end }) =
                        state.search_matches[state.offset + i].subject
                    {
                        if truncate {
                            let not_truncated_width = max_width - ELLIPSIS.len();
                            if not_truncated_width < start {
                                highlight_matched_text(
                                    subject,
                                    not_truncated_width,
                                    end,
                                    SUBJECT_COLOR,
                                )
                            } else if start <= not_truncated_width && not_truncated_width < end {
                                highlight_matched_text(subject, start, max_width, SUBJECT_COLOR)
                            } else {
                                highlight_matched_text(subject, start, end, SUBJECT_COLOR)
                            }
                        } else {
                            highlight_matched_text(subject, start, end, SUBJECT_COLOR)
                        }
                    } else {
                        vec![subject.into()]
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
                let spans = if let Some(SearchMatchPosition { start, end }) =
                    state.search_matches[state.offset + i].author_name
                {
                    if truncate {
                        let not_truncated_width = max_width - ELLIPSIS.len();
                        if not_truncated_width < start {
                            highlight_matched_text(name, not_truncated_width, end, NAME_COLOR)
                        } else if start <= not_truncated_width && not_truncated_width < end {
                            highlight_matched_text(name, start, max_width, NAME_COLOR)
                        } else {
                            highlight_matched_text(name, start, end, NAME_COLOR)
                        }
                    } else {
                        highlight_matched_text(name, start, end, NAME_COLOR)
                    }
                } else {
                    vec![name.fg(NAME_COLOR)]
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
                let spans = if let Some(SearchMatchPosition { start, end }) =
                    state.search_matches[state.offset + i].commit_hash
                {
                    highlight_matched_text(hash, start, end, HASH_COLOR)
                } else {
                    vec![hash.fg(HASH_COLOR)]
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
                self.to_commit_list_item(i, vec![date_str.fg(DATE_COLOR)], state)
            })
            .collect();
        Widget::render(List::new(items), area, buf);
    }

    fn rendering_commit_info_iter<'a>(
        &'a self,
        state: &'a CommitListState,
    ) -> impl Iterator<Item = (usize, &'a CommitInfo)> {
        state
            .commits
            .iter()
            .skip(state.offset)
            .take(state.height)
            .enumerate()
    }

    fn rendering_commit_iter<'a>(
        &'a self,
        state: &'a CommitListState,
    ) -> impl Iterator<Item = (usize, &'a Commit)> {
        self.rendering_commit_info_iter(state)
            .map(|(i, commit_info)| (i, commit_info.commit))
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
                .bg(SELECTED_BACKGROUND_COLOR)
                .fg(SELECTED_FOREGROUND_COLOR);
        }
        ListItem::new(line)
    }
}

fn refs_spans<'a>(commit_info: &'a CommitInfo, head: &'a Head) -> Vec<Span<'a>> {
    let refs = &commit_info.refs;

    if refs.len() == 1 {
        if let Ref::Stash { name, .. } = refs[0] {
            return vec![Span::raw(name).fg(REF_STASH_COLOR).bold(), Span::raw(" ")];
        }
    }

    let ref_spans: Vec<Span> = refs
        .iter()
        .filter_map(|r| match r {
            Ref::Branch { name, .. } => Some(Span::raw(name).fg(REF_BRANCH_COLOR).bold()),
            Ref::RemoteBranch { name, .. } => {
                Some(Span::raw(name).fg(REF_REMOTE_BRANCH_COLOR).bold())
            }
            Ref::Tag { name, .. } => Some(Span::raw(name).fg(REF_TAG_COLOR).bold()),
            Ref::Stash { .. } => None,
        })
        .collect();

    let mut spans = vec![Span::raw("(").fg(REF_PAREN_COLOR).bold()];

    if let Head::Detached { target } = head {
        if commit_info.commit.commit_hash == *target {
            spans.push(Span::raw("HEAD").fg(HEAD_COLOR).bold());
            if !ref_spans.is_empty() {
                spans.push(Span::raw(", ").fg(REF_PAREN_COLOR).bold());
            }
        }
    }

    for (i, s) in ref_spans.into_iter().enumerate() {
        if let Head::Branch { name } = head {
            if s.content == std::borrow::Cow::Borrowed(name) {
                spans.push(Span::raw("HEAD -> ").fg(HEAD_COLOR).bold());
            }
        }
        spans.push(s);
        if i < refs.len() - 1 {
            spans.push(Span::raw(", ").fg(REF_PAREN_COLOR).bold());
        }
    }

    spans.push(Span::raw(") ").fg(REF_PAREN_COLOR).bold());

    if spans.len() == 2 {
        spans.clear(); // contains only "(" and ")", so clear it
    }

    spans
}

fn highlight_matched_text(
    s: String,
    start: usize,
    end: usize,
    base_color: Color,
) -> Vec<Span<'static>> {
    let mut chars = s.chars();
    let head = chars.by_ref().take(start).collect::<String>();
    let matched = chars.by_ref().take(end - start).collect::<String>();
    let tail = chars.collect::<String>();
    vec![
        head.fg(base_color),
        matched
            .fg(MATCH_FOREGROUND_COLOR)
            .bg(MATCH_BACKGROUND_COLOR),
        tail.fg(base_color),
    ]
}
