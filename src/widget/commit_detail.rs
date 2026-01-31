use std::rc::Rc;

use chrono::{DateTime, FixedOffset};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::AppContext,
    git::{Commit, FileChange, Ref},
};

#[derive(Debug, Default)]
pub struct CommitDetailState {
    height: usize,
    offset: usize,
}

impl CommitDetailState {
    pub fn scroll_down(&mut self) {
        self.offset = self.offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub fn scroll_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height);
    }

    pub fn scroll_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height);
    }

    pub fn scroll_half_page_down(&mut self) {
        self.offset = self.offset.saturating_add(self.height / 2);
    }

    pub fn scroll_half_page_up(&mut self) {
        self.offset = self.offset.saturating_sub(self.height / 2);
    }

    pub fn select_first(&mut self) {
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        self.offset = usize::MAX;
    }
}

pub struct CommitDetail<'a> {
    commit: &'a Commit,
    changes: &'a Vec<FileChange>,
    refs: &'a Vec<Ref>,
    ctx: Rc<AppContext>,
}

impl<'a> CommitDetail<'a> {
    pub fn new(
        commit: &'a Commit,
        changes: &'a Vec<FileChange>,
        refs: &'a Vec<Ref>,
        ctx: Rc<AppContext>,
    ) -> Self {
        Self {
            commit,
            changes,
            refs,
            ctx,
        }
    }
}

impl StatefulWidget for CommitDetail<'_> {
    type State = CommitDetailState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [labels_area, value_area] =
            Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).areas(area);

        let (mut label_lines, mut value_lines) = self.contents(area);

        let content_area_height = area.height as usize - 1; // minus the top border
        self.update_state(state, value_lines.len(), content_area_height);

        label_lines = label_lines.into_iter().skip(state.offset).collect();
        value_lines = value_lines.into_iter().skip(state.offset).collect();

        self.render_labels_paragraph(label_lines, labels_area, buf);
        self.render_value_paragraph(value_lines, value_area, buf);
    }
}

impl CommitDetail<'_> {
    fn render_labels_paragraph(&self, lines: Vec<Line>, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.ctx.color_theme.fg))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .style(Style::default().fg(self.ctx.color_theme.divider_fg))
                    .padding(Padding::left(2)),
            );
        paragraph.render(area, buf);
    }

    fn render_value_paragraph(&self, lines: Vec<Line>, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.ctx.color_theme.fg))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .style(Style::default().fg(self.ctx.color_theme.divider_fg))
                    .padding(Padding::new(1, 2, 0, 0)),
            );
        paragraph.render(area, buf);
    }

    fn contents(&self, area: Rect) -> (Vec<Line<'_>>, Vec<Line<'_>>) {
        let mut label_lines: Vec<Line> = Vec::new();
        let mut value_lines: Vec<Line> = Vec::new();

        label_lines.push(Line::from("   Author: ").fg(self.ctx.color_theme.detail_label_fg));
        label_lines.push(self.empty_line());
        value_lines.extend(self.author_lines());

        if is_author_committer_different(self.commit) {
            label_lines.push(Line::from("Committer: ").fg(self.ctx.color_theme.detail_label_fg));
            label_lines.push(self.empty_line());
            value_lines.extend(self.committer_lines());
        }

        label_lines.push(Line::from("      SHA: ").fg(self.ctx.color_theme.detail_label_fg));
        value_lines.push(self.sha_line());

        if has_parent(self.commit) {
            label_lines.push(Line::from("  Parents: ").fg(self.ctx.color_theme.detail_label_fg));
            value_lines.push(self.parents_line());
        }

        if has_refs(self.refs) {
            label_lines.push(Line::from("     Refs: ").fg(self.ctx.color_theme.detail_label_fg));
            value_lines.push(self.refs_line());
        }

        value_lines.push(self.divider_line(area.width as usize));
        value_lines.extend(self.commit_message_lines());

        value_lines.push(self.divider_line(area.width as usize));
        value_lines.extend(self.changes_lines());

        (label_lines, value_lines)
    }

    fn author_lines(&self) -> Vec<Line<'_>> {
        self.author_committer_lines(
            &self.commit.author_name,
            &self.commit.author_email,
            &self.commit.author_date,
        )
    }

    fn committer_lines(&self) -> Vec<Line<'_>> {
        self.author_committer_lines(
            &self.commit.committer_name,
            &self.commit.committer_email,
            &self.commit.committer_date,
        )
    }

    fn author_committer_lines<'a>(
        &'a self,
        name: &'a str,
        email: &'a str,
        date: &'a DateTime<FixedOffset>,
    ) -> Vec<Line<'a>> {
        let date_str = if self.ctx.ui_config.detail.date_local {
            let local = date.with_timezone(&chrono::Local);
            local
                .format(&self.ctx.ui_config.detail.date_format)
                .to_string()
        } else {
            date.format(&self.ctx.ui_config.detail.date_format)
                .to_string()
        };
        vec![
            Line::from(vec![
                name.fg(self.ctx.color_theme.detail_name_fg),
                " <".into(),
                email.fg(self.ctx.color_theme.detail_email_fg),
                "> ".into(),
            ]),
            Line::from(date_str.fg(self.ctx.color_theme.detail_date_fg)),
        ]
    }

    fn sha_line(&self) -> Line<'_> {
        Line::from(
            self.commit
                .commit_hash
                .as_str()
                .fg(self.ctx.color_theme.detail_hash_fg),
        )
    }

    fn parents_line(&self) -> Line<'_> {
        let mut spans = Vec::new();
        let parents = &self.commit.parent_commit_hashes;
        for (i, hash) in parents
            .iter()
            .map(|hash| hash.as_short_hash().fg(self.ctx.color_theme.detail_hash_fg))
            .enumerate()
        {
            spans.push(hash);
            if i < parents.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }
        Line::from(spans)
    }

    fn refs_line(&self) -> Line<'_> {
        let ref_spans = self.refs.iter().filter_map(|r| match r {
            Ref::Branch { name, .. } => Some(
                Span::raw(name)
                    .fg(self.ctx.color_theme.detail_ref_branch_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Ref::RemoteBranch { name, .. } => Some(
                Span::raw(name)
                    .fg(self.ctx.color_theme.detail_ref_remote_branch_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Ref::Tag { name, .. } => Some(
                Span::raw(name)
                    .fg(self.ctx.color_theme.detail_ref_tag_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Ref::Stash { .. } => None,
        });

        let mut spans = Vec::new();
        for (i, ref_span) in ref_spans.enumerate() {
            spans.push(ref_span);
            if i < self.refs.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }
        Line::from(spans)
    }

    fn commit_message_lines(&self) -> Vec<Line<'_>> {
        let subject_line = Line::from(self.commit.subject.as_str().bold());

        let mut lines = vec![subject_line];

        if self.commit.body.is_empty() {
            return lines;
        }

        let body_lines = self.commit.body.lines().map(Line::raw);

        lines.push(self.empty_line());
        lines.extend(body_lines);

        lines
    }

    fn changes_lines(&self) -> Vec<Line<'_>> {
        self.changes
            .iter()
            .map(|c| match c {
                FileChange::Add { path } => Line::from(vec![
                    "A".fg(self.ctx.color_theme.detail_file_change_add_fg),
                    " ".into(),
                    path.into(),
                ]),
                FileChange::Modify { path } => Line::from(vec![
                    "M".fg(self.ctx.color_theme.detail_file_change_modify_fg),
                    " ".into(),
                    path.into(),
                ]),
                FileChange::Delete { path } => Line::from(vec![
                    "D".fg(self.ctx.color_theme.detail_file_change_delete_fg),
                    " ".into(),
                    path.into(),
                ]),
                FileChange::Move { from, to } => Line::from(vec![
                    "R".fg(self.ctx.color_theme.detail_file_change_move_fg),
                    " ".into(),
                    from.into(),
                    " -> ".into(),
                    to.into(),
                ]),
            })
            .collect()
    }

    fn empty_line(&self) -> Line<'_> {
        Line::raw("")
    }

    fn divider_line(&self, width: usize) -> Line<'_> {
        Line::from("─".repeat(width).fg(self.ctx.color_theme.divider_fg))
    }

    fn update_state(&self, state: &mut CommitDetailState, line_count: usize, area_height: usize) {
        state.height = area_height;
        state.offset = state.offset.min(line_count.saturating_sub(area_height));
    }
}

fn is_author_committer_different(commit: &Commit) -> bool {
    commit.author_name != commit.committer_name
        || commit.author_email != commit.committer_email
        || commit.author_date != commit.committer_date
}

fn has_parent(commit: &Commit) -> bool {
    !commit.parent_commit_hashes.is_empty()
}

fn has_refs(refs: &[Ref]) -> bool {
    refs.iter().any(|r| {
        matches!(
            r,
            Ref::Branch { .. } | Ref::RemoteBranch { .. } | Ref::Tag { .. }
        )
    })
}
