use std::{path::PathBuf, thread};

use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    color::ColorTheme,
    config::UiConfig,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::{delete_remote_tag, delete_tag, CommitHash, Ref},
    widget::commit_list::{CommitList, CommitListState},
};

#[derive(Debug)]
pub struct DeleteTagView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    commit_hash: CommitHash,
    repo_path: PathBuf,

    tags: Vec<String>,
    selected_index: usize,
    delete_from_remote: bool,

    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    tx: Sender,
}

impl<'a> DeleteTagView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        commit_hash: CommitHash,
        tags: Vec<Ref>,
        repo_path: PathBuf,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> DeleteTagView<'a> {
        let mut tag_names: Vec<String> = tags
            .into_iter()
            .filter_map(|r| match r {
                Ref::Tag { name, .. } => Some(name),
                _ => None,
            })
            .collect();

        tag_names.sort_by(|a, b| compare_semver(a, b));

        DeleteTagView {
            commit_list_state: Some(commit_list_state),
            commit_hash,
            repo_path,
            tags: tag_names,
            selected_index: 0,
            delete_from_remote: true,
            ui_config,
            color_theme,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, _key: KeyEvent) {
        let event = event_with_count.event;

        match event {
            UserEvent::Cancel => {
                self.tx.send(AppEvent::CloseDeleteTag);
            }
            UserEvent::Confirm => {
                self.delete_selected();
            }
            UserEvent::NavigateDown | UserEvent::SelectDown => {
                if self.selected_index < self.tags.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            UserEvent::NavigateUp | UserEvent::SelectUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            UserEvent::NavigateRight | UserEvent::NavigateLeft => {
                self.delete_from_remote = !self.delete_from_remote;
            }
            _ => {}
        }
    }

    fn delete_selected(&mut self) {
        if self.tags.is_empty() {
            return;
        }

        let tag_name = self.tags[self.selected_index].clone();

        // Prepare data for background thread
        let repo_path = self.repo_path.clone();
        let commit_hash = self.commit_hash.clone();
        let delete_from_remote = self.delete_from_remote;
        let tx = self.tx.clone();

        // Show pending overlay and close dialog
        let pending_msg = if delete_from_remote {
            format!("Deleting tag '{}' from local and remote...", tag_name)
        } else {
            format!("Deleting tag '{}'...", tag_name)
        };
        self.tx
            .send(AppEvent::ShowPendingOverlay { message: pending_msg });
        self.tx.send(AppEvent::CloseDeleteTag);

        // Run git commands in background
        thread::spawn(move || {
            if let Err(e) = delete_tag(&repo_path, &tag_name) {
                tx.send(AppEvent::HidePendingOverlay);
                tx.send(AppEvent::NotifyError(e));
                return;
            }

            if delete_from_remote {
                if let Err(e) = delete_remote_tag(&repo_path, &tag_name) {
                    tx.send(AppEvent::HidePendingOverlay);
                    tx.send(AppEvent::NotifyError(format!(
                        "Local tag deleted, but failed to delete from remote: {}",
                        e
                    )));
                    // Still remove tag from UI since local deletion succeeded
                    tx.send(AppEvent::RemoveTagFromCommit {
                        commit_hash,
                        tag_name,
                    });
                    return;
                }
            }

            // Success
            tx.send(AppEvent::RemoveTagFromCommit {
                commit_hash,
                tag_name: tag_name.clone(),
            });

            let msg = if delete_from_remote {
                format!("Tag '{}' deleted from local and remote", tag_name)
            } else {
                format!("Tag '{}' deleted locally", tag_name)
            };
            tx.send(AppEvent::NotifySuccess(msg));
            tx.send(AppEvent::HidePendingOverlay);
        });
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let commit_list = CommitList::new(&self.ui_config.list, self.color_theme);
        f.render_stateful_widget(commit_list, area, self.as_mut_list_state());

        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let list_height = (self.tags.len() as u16).min(8);
        let dialog_height = (6 + list_height).min(area.height.saturating_sub(2));

        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(
            area.x + dialog_x,
            area.y + dialog_y,
            dialog_width,
            dialog_height,
        );

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Delete Tag ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color_theme.divider_fg))
            .style(
                Style::default()
                    .bg(self.color_theme.bg)
                    .fg(self.color_theme.fg),
            )
            .padding(Padding::horizontal(1));

        let inner_area = block.inner(dialog_area);
        f.render_widget(block, dialog_area);

        let [commit_area, list_area, checkbox_area, hint_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(list_height),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .areas(inner_area);

        let commit_line = Line::from(vec![
            Span::raw("Commit: ").fg(self.color_theme.fg),
            Span::raw(self.commit_hash.as_short_hash()).fg(self.color_theme.list_hash_fg),
        ]);
        f.render_widget(Paragraph::new(commit_line), commit_area);

        let tag_lines: Vec<Line> = self
            .tags
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                let is_selected = i == self.selected_index;
                let prefix = if is_selected { "> " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .bg(self.color_theme.list_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(format!("{}{}", prefix, tag), style))
            })
            .collect();

        if tag_lines.is_empty() {
            f.render_widget(
                Paragraph::new(Line::from("No tags on this commit".fg(self.color_theme.fg))),
                list_area,
            );
        } else {
            f.render_widget(Paragraph::new(tag_lines), list_area);
        }

        let checkbox = if self.delete_from_remote {
            "[x]"
        } else {
            "[ ]"
        };
        let checkbox_style = Style::default().fg(self.color_theme.fg);
        let checkbox_line = Line::from(vec![
            Span::styled(checkbox, checkbox_style),
            Span::raw(" Delete from origin").fg(self.color_theme.fg),
        ]);
        f.render_widget(Paragraph::new(checkbox_line), checkbox_area);

        let hint_line = Line::from(vec![
            Span::raw("Enter").fg(self.color_theme.help_key_fg),
            Span::raw(" delete  ").fg(self.color_theme.fg),
            Span::raw("Esc").fg(self.color_theme.help_key_fg),
            Span::raw(" close  ").fg(self.color_theme.fg),
            Span::raw("↑↓").fg(self.color_theme.help_key_fg),
            Span::raw(" select  ").fg(self.color_theme.fg),
            Span::raw("←→").fg(self.color_theme.help_key_fg),
            Span::raw(" toggle").fg(self.color_theme.fg),
        ]);
        f.render_widget(Paragraph::new(hint_line).centered(), hint_area);
    }

    fn as_mut_list_state(&mut self) -> &mut CommitListState<'a> {
        self.commit_list_state.as_mut().unwrap()
    }
}

impl<'a> DeleteTagView<'a> {
    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    pub fn remove_ref_from_commit(&mut self, commit_hash: &CommitHash, tag_name: &str) {
        self.as_mut_list_state()
            .remove_ref_from_commit(commit_hash, tag_name);
    }
}

fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |s: &str| -> Option<(u64, u64, u64, String)> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() >= 3 {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            let patch_str = parts[2];
            let (patch_num, suffix) =
                if let Some(idx) = patch_str.find(|c: char| !c.is_ascii_digit()) {
                    let (num, suf) = patch_str.split_at(idx);
                    (num.parse::<u64>().ok()?, suf.to_string())
                } else {
                    (patch_str.parse::<u64>().ok()?, String::new())
                };
            Some((major, minor, patch_num, suffix))
        } else {
            None
        }
    };

    match (parse_version(a), parse_version(b)) {
        (Some(va), Some(vb)) => va.cmp(&vb),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.cmp(b),
    }
}
