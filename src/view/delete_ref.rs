use std::{path::PathBuf, rc::Rc, thread};

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
    git::{
        delete_branch, delete_branch_force, delete_remote_branch, delete_remote_tag, delete_tag,
        Ref, RefType,
    },
    widget::{
        commit_list::{CommitList, CommitListState},
        ref_list::RefListState,
    },
};

#[derive(Debug)]
pub struct DeleteRefView<'a> {
    commit_list_state: Option<CommitListState>,
    ref_list_state: RefListState,
    refs: Vec<Rc<Ref>>,
    repo_path: PathBuf,

    ref_name: String,
    ref_type: RefType,
    delete_from_remote: bool,
    force_delete: bool,

    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    tx: Sender,
}

impl<'a> DeleteRefView<'a> {
    pub fn new(
        commit_list_state: CommitListState,
        ref_list_state: RefListState,
        refs: Vec<Rc<Ref>>,
        repo_path: PathBuf,
        ref_name: String,
        ref_type: RefType,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> DeleteRefView<'a> {
        DeleteRefView {
            commit_list_state: Some(commit_list_state),
            ref_list_state,
            refs,
            repo_path,
            ref_name,
            ref_type,
            delete_from_remote: ref_type == RefType::RemoteBranch,
            force_delete: false,
            ui_config,
            color_theme,
            tx,
        }
    }

    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, _key: KeyEvent) {
        let event = event_with_count.event;

        match event {
            UserEvent::Cancel => {
                self.tx.send(AppEvent::CloseDeleteRef);
            }
            UserEvent::Confirm => {
                self.delete_ref();
            }
            UserEvent::NavigateRight | UserEvent::NavigateLeft | UserEvent::NavigateDown => {
                match self.ref_type {
                    RefType::Tag => {
                        self.delete_from_remote = !self.delete_from_remote;
                    }
                    RefType::Branch => {
                        self.force_delete = !self.force_delete;
                    }
                    RefType::RemoteBranch => {}
                }
            }
            _ => {}
        }
    }

    fn delete_ref(&mut self) {
        let ref_name = self.ref_name.clone();
        let ref_type = self.ref_type;
        let repo_path = self.repo_path.clone();
        let delete_from_remote = self.delete_from_remote;
        let force_delete = self.force_delete;
        let tx = self.tx.clone();

        let pending_msg = match ref_type {
            RefType::Tag => {
                if delete_from_remote {
                    format!("Deleting tag '{}' from local and remote...", ref_name)
                } else {
                    format!("Deleting tag '{}'...", ref_name)
                }
            }
            RefType::Branch => {
                if force_delete {
                    format!("Force deleting branch '{}'...", ref_name)
                } else {
                    format!("Deleting branch '{}'...", ref_name)
                }
            }
            RefType::RemoteBranch => {
                format!("Deleting remote branch '{}'...", ref_name)
            }
        };

        self.tx.send(AppEvent::ShowPendingOverlay {
            message: pending_msg,
        });
        self.tx.send(AppEvent::CloseDeleteRef);

        thread::spawn(move || {
            let result = match ref_type {
                RefType::Tag => {
                    if let Err(e) = delete_tag(&repo_path, &ref_name) {
                        Err(e)
                    } else if delete_from_remote {
                        delete_remote_tag(&repo_path, &ref_name).map_err(|e| {
                            format!("Local tag deleted, but failed to delete from remote: {}", e)
                        })
                    } else {
                        Ok(())
                    }
                }
                RefType::Branch => {
                    if force_delete {
                        delete_branch_force(&repo_path, &ref_name)
                    } else {
                        delete_branch(&repo_path, &ref_name)
                    }
                }
                RefType::RemoteBranch => delete_remote_branch(&repo_path, &ref_name),
            };

            match result {
                Ok(()) => {
                    let msg = match ref_type {
                        RefType::Tag => {
                            if delete_from_remote {
                                format!("Tag '{}' deleted from local and remote", ref_name)
                            } else {
                                format!("Tag '{}' deleted locally", ref_name)
                            }
                        }
                        RefType::Branch => {
                            format!("Branch '{}' deleted", ref_name)
                        }
                        RefType::RemoteBranch => {
                            format!("Remote branch '{}' deleted", ref_name)
                        }
                    };
                    tx.send(AppEvent::RemoveRefFromList {
                        ref_name: ref_name.clone(),
                    });
                    tx.send(AppEvent::NotifySuccess(msg));
                    tx.send(AppEvent::HidePendingOverlay);
                }
                Err(e) => {
                    tx.send(AppEvent::HidePendingOverlay);
                    tx.send(AppEvent::NotifyError(e));
                }
            }
        });
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let Some(list_state) = self.commit_list_state.as_mut() else {
            return;
        };

        let graph_width = list_state.graph_area_cell_width() + 1;
        let refs_width = (area.width.saturating_sub(graph_width)).min(self.ui_config.refs.width);

        let [list_area, refs_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(refs_width)]).areas(area);

        let commit_list = CommitList::new(&self.ui_config.list, self.color_theme);
        f.render_stateful_widget(commit_list, list_area, list_state);

        let ref_list = crate::widget::ref_list::RefList::new(&self.refs, self.color_theme);
        f.render_stateful_widget(ref_list, refs_area, &mut self.ref_list_state);

        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let dialog_height = 6u16.min(area.height.saturating_sub(2));

        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(
            area.x + dialog_x,
            area.y + dialog_y,
            dialog_width,
            dialog_height,
        );

        f.render_widget(Clear, dialog_area);

        let title = match self.ref_type {
            RefType::Tag => " Delete Tag ",
            RefType::Branch => " Delete Branch ",
            RefType::RemoteBranch => " Delete Remote Branch ",
        };

        let block = Block::default()
            .title(title)
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

        let [name_area, checkbox_area, hint_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .areas(inner_area);

        let name_line = Line::from(vec![Span::raw(&self.ref_name)
            .fg(self.color_theme.fg)
            .add_modifier(Modifier::BOLD)]);
        f.render_widget(Paragraph::new(name_line), name_area);

        let checkbox_line = match self.ref_type {
            RefType::Tag => {
                let checkbox = if self.delete_from_remote {
                    "[x]"
                } else {
                    "[ ]"
                };
                Line::from(vec![
                    Span::styled(checkbox, Style::default().fg(self.color_theme.fg)),
                    Span::raw(" Delete from origin").fg(self.color_theme.fg),
                ])
            }
            RefType::Branch => {
                let checkbox = if self.force_delete { "[x]" } else { "[ ]" };
                Line::from(vec![
                    Span::styled(checkbox, Style::default().fg(self.color_theme.fg)),
                    Span::raw(" Force delete (-D)").fg(self.color_theme.fg),
                ])
            }
            RefType::RemoteBranch => Line::from(vec![Span::raw("").fg(self.color_theme.fg)]),
        };
        f.render_widget(Paragraph::new(checkbox_line), checkbox_area);

        let hint_line = Line::from(vec![
            Span::raw("Enter").fg(self.color_theme.help_key_fg),
            Span::raw(" delete  ").fg(self.color_theme.fg),
            Span::raw("Esc").fg(self.color_theme.help_key_fg),
            Span::raw(" cancel  ").fg(self.color_theme.fg),
            Span::raw("←→").fg(self.color_theme.help_key_fg),
            Span::raw(" toggle").fg(self.color_theme.fg),
        ]);
        f.render_widget(Paragraph::new(hint_line).centered(), hint_area);
    }
}

impl<'a> DeleteRefView<'a> {
    pub fn take_list_state(&mut self) -> Option<CommitListState> {
        self.commit_list_state.take()
    }

    pub fn take_ref_list_state(&mut self) -> RefListState {
        std::mem::take(&mut self.ref_list_state)
    }

    pub fn take_refs(&mut self) -> Vec<Rc<Ref>> {
        std::mem::take(&mut self.refs)
    }

    pub fn remove_ref(&mut self, ref_name: &str) {
        if let Some(target) = self.refs.iter().find(|r| r.name() == ref_name).map(|r| r.target().clone()) {
            if let Some(list_state) = self.commit_list_state.as_mut() {
                list_state.remove_ref_from_commit(&target, ref_name);
            }
        }
        self.refs.retain(|r| r.name() != ref_name);
        self.ref_list_state.adjust_selection_after_delete();
    }
}
