use std::{collections::HashMap, rc::Rc};

use ratatui::{
    backend::Backend,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};

use crate::{
    color::{ColorTheme, GraphColorSet},
    config::{CoreConfig, CursorType, UiConfig},
    event::{AppEvent, Receiver, Sender, UserEvent, UserEventWithCount},
    external::copy_to_clipboard,
    git::{CommitHash, Head, Ref, RefType, Repository},
    graph::{calc_graph, CellWidthType, Graph, GraphImageManager},
    keybind::KeyBind,
    protocol::ImageProtocol,
    view::View,
    widget::{
        commit_list::{CommitInfo, CommitListState},
        pending_overlay::PendingOverlay,
    },
};

#[derive(Debug)]
enum StatusLine {
    None,
    Input(String, Option<u16>, Option<String>),
    NotificationInfo(String),
    NotificationSuccess(String),
    NotificationWarn(String),
    NotificationError(String),
}

pub enum InitialSelection {
    Latest,
    Head,
}

#[derive(Debug)]
pub struct App<'a> {
    repository: Repository,
    view: View<'a>,
    status_line: StatusLine,
    pending_message: Option<String>,

    keybind: &'a KeyBind,
    core_config: &'a CoreConfig,
    ui_config: &'a UiConfig,
    color_theme: &'a ColorTheme,
    graph_color_set: &'a GraphColorSet,
    cell_width_type: CellWidthType,
    image_protocol: ImageProtocol,
    tx: Sender,

    numeric_prefix: String,
    view_area: Rect,
}

impl<'a> App<'a> {
    pub fn new(
        repository: Repository,
        graph_image_manager: GraphImageManager,
        graph: &Graph,
        keybind: &'a KeyBind,
        core_config: &'a CoreConfig,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        graph_color_set: &'a GraphColorSet,
        cell_width_type: CellWidthType,
        image_protocol: ImageProtocol,
        initial_selection: InitialSelection,
        tx: Sender,
    ) -> Self {
        let mut ref_name_to_commit_index_map = HashMap::new();
        let commits = graph
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let refs = repository.refs(&commit.commit_hash);
                for r in &refs {
                    ref_name_to_commit_index_map.insert(r.name().to_string(), i);
                }
                let (pos_x, _) = graph.commit_pos_map[&commit.commit_hash];
                let graph_color = graph_color_set.get(pos_x).to_ratatui_color();
                CommitInfo::new(commit.clone(), refs, graph_color)
            })
            .collect();
        let graph_cell_width = match cell_width_type {
            CellWidthType::Double => (graph.max_pos_x + 1) as u16 * 2,
            CellWidthType::Single => (graph.max_pos_x + 1) as u16,
        };
        let head = repository.head();
        let mut commit_list_state = CommitListState::new(
            commits,
            graph_image_manager,
            graph_cell_width,
            head,
            ref_name_to_commit_index_map,
            core_config.search.ignore_case,
            core_config.search.fuzzy,
        );
        if let InitialSelection::Head = initial_selection {
            match repository.head() {
                Head::Branch { name } => commit_list_state.select_ref(&name),
                Head::Detached { target } => commit_list_state.select_commit_hash(&target),
            }
        }
        let view = View::of_list(commit_list_state, ui_config, color_theme, tx.clone());

        Self {
            repository,
            status_line: StatusLine::None,
            pending_message: None,
            view,
            keybind,
            core_config,
            ui_config,
            color_theme,
            graph_color_set,
            cell_width_type,
            image_protocol,
            tx,
            numeric_prefix: String::new(),
            view_area: Rect::default(),
        }
    }
}

impl App<'_> {
    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        rx: Receiver,
    ) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            match rx.recv() {
                AppEvent::Key(key) => {
                    // Handle pending overlay - Esc hides it
                    if self.pending_message.is_some() {
                        if let Some(UserEvent::Cancel) = self.keybind.get(&key) {
                            self.pending_message = None;
                            self.tx.send(AppEvent::NotifyInfo(
                                "Operation continues in background".into(),
                            ));
                            continue;
                        }
                        // Block other keys while pending
                        continue;
                    }

                    match self.status_line {
                        StatusLine::None | StatusLine::Input(_, _, _) => {
                            // do nothing
                        }
                        StatusLine::NotificationInfo(_)
                        | StatusLine::NotificationSuccess(_)
                        | StatusLine::NotificationWarn(_) => {
                            // Clear message and pass key input as is
                            self.clear_status_line();
                        }
                        StatusLine::NotificationError(_) => {
                            // Clear message and cancel key input
                            self.clear_status_line();
                            continue;
                        }
                    }

                    let user_event = self.keybind.get(&key);

                    if let Some(UserEvent::Cancel) = user_event {
                        if !self.numeric_prefix.is_empty() {
                            // Clear numeric prefix and cancel the event
                            self.numeric_prefix.clear();
                            continue;
                        }
                    }

                    match user_event {
                        Some(UserEvent::ForceQuit) => {
                            self.tx.send(AppEvent::Quit);
                        }
                        Some(ue) => {
                            let event_with_count =
                                process_numeric_prefix(&self.numeric_prefix, *ue, key);
                            self.view.handle_event(event_with_count, key);
                            self.numeric_prefix.clear();
                        }
                        None => {
                            let is_input_mode =
                                matches!(self.status_line, StatusLine::Input(_, _, _))
                                    || matches!(self.view, View::CreateTag(_));
                            if is_input_mode {
                                // In input mode, pass all key events to the view
                                self.numeric_prefix.clear();
                                self.view.handle_event(
                                    UserEventWithCount::from_event(UserEvent::Unknown),
                                    key,
                                );
                            } else if let KeyCode::Char(c) = key.code {
                                // Accumulate numeric prefix
                                if c.is_ascii_digit()
                                    && (c != '0' || !self.numeric_prefix.is_empty())
                                {
                                    self.numeric_prefix.push(c);
                                }
                            }
                        }
                    }
                }
                AppEvent::Resize(w, h) => {
                    let _ = (w, h);
                }
                AppEvent::Quit => {
                    return Ok(());
                }
                AppEvent::OpenDetail => {
                    self.open_detail();
                }
                AppEvent::CloseDetail => {
                    self.close_detail();
                }
                AppEvent::ClearDetail => {
                    self.clear_detail();
                }
                AppEvent::OpenUserCommand(n) => {
                    self.open_user_command(n);
                }
                AppEvent::CloseUserCommand => {
                    self.close_user_command();
                }
                AppEvent::ClearUserCommand => {
                    self.clear_user_command();
                }
                AppEvent::OpenRefs => {
                    self.open_refs();
                }
                AppEvent::CloseRefs => {
                    self.close_refs();
                }
                AppEvent::OpenCreateTag => {
                    self.open_create_tag();
                }
                AppEvent::CloseCreateTag => {
                    self.close_create_tag();
                }
                AppEvent::AddTagToCommit {
                    commit_hash,
                    tag_name,
                } => {
                    self.add_tag_to_commit(&commit_hash, &tag_name);
                }
                AppEvent::OpenDeleteTag => {
                    self.open_delete_tag();
                }
                AppEvent::CloseDeleteTag => {
                    self.close_delete_tag();
                }
                AppEvent::RemoveTagFromCommit {
                    commit_hash,
                    tag_name,
                } => {
                    self.remove_tag_from_commit(&commit_hash, &tag_name);
                }
                AppEvent::OpenDeleteRef { ref_name, ref_type } => {
                    self.open_delete_ref(ref_name, ref_type);
                }
                AppEvent::CloseDeleteRef => {
                    self.close_delete_ref();
                }
                AppEvent::RemoveRefFromList { ref_name } => {
                    self.remove_ref_from_list(&ref_name);
                }
                AppEvent::OpenHelp => {
                    self.open_help();
                }
                AppEvent::CloseHelp => {
                    self.close_help();
                }
                AppEvent::ClearHelp => {
                    self.clear_help();
                }
                AppEvent::SelectOlderCommit => {
                    self.select_older_commit();
                }
                AppEvent::SelectNewerCommit => {
                    self.select_newer_commit();
                }
                AppEvent::SelectParentCommit => {
                    self.select_parent_commit();
                }
                AppEvent::CopyToClipboard { name, value } => {
                    self.copy_to_clipboard(name, value);
                }
                AppEvent::ClearStatusLine => {
                    self.clear_status_line();
                }
                AppEvent::UpdateStatusInput(msg, cursor_pos, msg_r) => {
                    self.update_status_input(msg, cursor_pos, msg_r);
                }
                AppEvent::NotifyInfo(msg) => {
                    self.info_notification(msg);
                }
                AppEvent::NotifySuccess(msg) => {
                    self.success_notification(msg);
                }
                AppEvent::NotifyWarn(msg) => {
                    self.warn_notification(msg);
                }
                AppEvent::NotifyError(msg) => {
                    self.error_notification(msg);
                }
                AppEvent::ShowPendingOverlay { message } => {
                    self.pending_message = Some(message);
                }
                AppEvent::HidePendingOverlay => {
                    self.pending_message = None;
                }
                AppEvent::Refresh => {
                    self.refresh();
                }
            }
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let base = Block::default()
            .fg(self.color_theme.fg)
            .bg(self.color_theme.bg);
        f.render_widget(base, f.area());

        let [view_area, status_line_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).areas(f.area());

        self.update_state(view_area);

        self.view.render(f, view_area);
        self.render_status_line(f, status_line_area);

        if let Some(message) = &self.pending_message {
            let overlay = PendingOverlay::new(message, self.color_theme);
            f.render_widget(overlay, f.area());
        }
    }
}

impl App<'_> {
    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let text: Line = match &self.status_line {
            StatusLine::None => {
                if self.numeric_prefix.is_empty() {
                    self.build_hotkey_hints()
                } else {
                    Line::raw(self.numeric_prefix.as_str())
                        .fg(self.color_theme.status_input_transient_fg)
                }
            }
            StatusLine::Input(msg, _, transient_msg) => {
                let msg_w = console::measure_text_width(msg.as_str());
                if let Some(t_msg) = transient_msg {
                    let t_msg_w = console::measure_text_width(t_msg.as_str());
                    let pad_w = area.width as usize - msg_w - t_msg_w - 2 /* pad */;
                    Line::from(vec![
                        msg.as_str().fg(self.color_theme.status_input_fg),
                        " ".repeat(pad_w).into(),
                        t_msg
                            .as_str()
                            .fg(self.color_theme.status_input_transient_fg),
                    ])
                } else {
                    Line::raw(msg).fg(self.color_theme.status_input_fg)
                }
            }
            StatusLine::NotificationInfo(msg) => Line::raw(msg).fg(self.color_theme.status_info_fg),
            StatusLine::NotificationSuccess(msg) => Line::raw(msg)
                .add_modifier(Modifier::BOLD)
                .fg(self.color_theme.status_success_fg),
            StatusLine::NotificationWarn(msg) => Line::raw(msg)
                .add_modifier(Modifier::BOLD)
                .fg(self.color_theme.status_warn_fg),
            StatusLine::NotificationError(msg) => Line::raw(format!("ERROR: {msg}"))
                .add_modifier(Modifier::BOLD)
                .fg(self.color_theme.status_error_fg),
        };
        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().fg(self.color_theme.divider_fg))
                .padding(Padding::horizontal(1)),
        );
        f.render_widget(paragraph, area);

        if let StatusLine::Input(_, Some(cursor_pos), _) = &self.status_line {
            let (x, y) = (area.x + cursor_pos + 1, area.y + 1);
            match &self.ui_config.common.cursor_type {
                CursorType::Native => {
                    f.set_cursor_position((x, y));
                }
                CursorType::Virtual(cursor) => {
                    let style = Style::default().fg(self.color_theme.virtual_cursor_fg);
                    f.buffer_mut().set_string(x, y, cursor, style);
                }
            }
        }
    }

    fn build_hotkey_hints(&self) -> Line<'static> {
        let hints: Vec<(UserEvent, &str)> = match &self.view {
            View::List(_) => vec![
                (UserEvent::Search, "search"),
                (UserEvent::Filter, "filter"),
                (UserEvent::IgnoreCaseToggle, "case"),
                (UserEvent::CreateTag, "tag"),
                (UserEvent::RefListToggle, "refs"),
                (UserEvent::Refresh, "refresh"),
                (UserEvent::HelpToggle, "help"),
            ],
            View::Detail(_) => vec![
                (UserEvent::ShortCopy, "copy"),
                (UserEvent::Close, "close"),
                (UserEvent::HelpToggle, "help"),
            ],
            View::Refs(_) => vec![
                (UserEvent::ShortCopy, "copy"),
                (UserEvent::UserCommandViewToggle(1), "delete"),
                (UserEvent::Close, "close"),
                (UserEvent::HelpToggle, "help"),
            ],
            View::CreateTag(_) | View::DeleteTag(_) | View::DeleteRef(_) => vec![
                (UserEvent::Confirm, "confirm"),
                (UserEvent::Cancel, "cancel"),
            ],
            View::Help(_) => vec![(UserEvent::Close, "close")],
            _ => vec![],
        };

        let key_fg = self.color_theme.help_key_fg;
        let desc_fg = self.color_theme.status_input_transient_fg;

        let mut spans: Vec<Span<'static>> = Vec::new();
        for (i, (event, desc)) in hints.iter().enumerate() {
            if let Some(key) = self.keybind.keys_for_event(*event).first() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                spans.push(Span::styled(key.clone(), Style::default().fg(key_fg)));
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    (*desc).to_string(),
                    Style::default().fg(desc_fg),
                ));
            }
        }
        Line::from(spans)
    }
}

impl App<'_> {
    fn update_state(&mut self, view_area: Rect) {
        self.view_area = view_area;
    }

    fn open_detail(&mut self) {
        if let View::List(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, changes) = self.repository.commit_detail(&selected);
            let refs = self.repository.refs(&selected);
            self.view = View::of_detail(
                commit_list_state,
                commit,
                changes,
                refs,
                self.ui_config,
                self.color_theme,
                self.image_protocol,
                self.tx.clone(),
            );
        }
    }

    fn close_detail(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            self.view = View::of_list(
                commit_list_state,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn clear_detail(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            view.clear();
        }
    }

    fn open_user_command(&mut self, user_command_number: usize) {
        if let View::List(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, _) = self.repository.commit_detail(&selected);
            self.view = View::of_user_command_from_list(
                commit_list_state,
                commit,
                user_command_number,
                self.view_area,
                self.core_config,
                self.ui_config,
                self.color_theme,
                self.image_protocol,
                self.tx.clone(),
            );
        } else if let View::Detail(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, _) = self.repository.commit_detail(&selected);
            self.view = View::of_user_command_from_detail(
                commit_list_state,
                commit,
                user_command_number,
                self.view_area,
                self.core_config,
                self.ui_config,
                self.color_theme,
                self.image_protocol,
                self.tx.clone(),
            );
        } else if let View::UserCommand(ref mut view) = self.view {
            let before_view_is_list = view.before_view_is_list();
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, _) = self.repository.commit_detail(&selected);
            if before_view_is_list {
                self.view = View::of_user_command_from_list(
                    commit_list_state,
                    commit,
                    user_command_number,
                    self.view_area,
                    self.core_config,
                    self.ui_config,
                    self.color_theme,
                    self.image_protocol,
                    self.tx.clone(),
                );
            } else {
                self.view = View::of_user_command_from_detail(
                    commit_list_state,
                    commit,
                    user_command_number,
                    self.view_area,
                    self.core_config,
                    self.ui_config,
                    self.color_theme,
                    self.image_protocol,
                    self.tx.clone(),
                );
            }
        }
    }

    fn close_user_command(&mut self) {
        if let View::UserCommand(ref mut view) = self.view {
            let before_view_is_list = view.before_view_is_list();
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, changes) = self.repository.commit_detail(&selected);
            let refs = self.repository.refs(&selected);
            if before_view_is_list {
                self.view = View::of_list(
                    commit_list_state,
                    self.ui_config,
                    self.color_theme,
                    self.tx.clone(),
                );
            } else {
                self.view = View::of_detail(
                    commit_list_state,
                    commit,
                    changes,
                    refs,
                    self.ui_config,
                    self.color_theme,
                    self.image_protocol,
                    self.tx.clone(),
                );
            }
        }
    }

    fn clear_user_command(&mut self) {
        if let View::UserCommand(ref mut view) = self.view {
            view.clear();
        }
    }

    fn open_refs(&mut self) {
        if let View::List(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let refs = self.get_current_refs();
            self.view = View::of_refs(
                commit_list_state,
                refs,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn get_current_refs(&self) -> Vec<Rc<Ref>> {
        self.repository.all_refs()
    }

    fn close_refs(&mut self) {
        if let View::Refs(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            self.view = View::of_list(
                commit_list_state,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn open_create_tag(&mut self) {
        if let View::List(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let commit_hash = commit_list_state.selected_commit_hash().clone();
            self.view = View::of_create_tag(
                commit_list_state,
                commit_hash,
                self.repository.path().to_path_buf(),
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn close_create_tag(&mut self) {
        if let View::CreateTag(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            self.view = View::of_list(
                commit_list_state,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn add_tag_to_commit(&mut self, commit_hash: &CommitHash, tag_name: &str) {
        let new_tag = Ref::Tag {
            name: tag_name.to_string(),
            target: commit_hash.clone(),
        };

        self.repository.add_ref(new_tag.clone());

        match &mut self.view {
            View::List(view) => {
                view.add_ref_to_commit(commit_hash, new_tag);
            }
            View::CreateTag(view) => {
                view.add_ref_to_commit(commit_hash, new_tag);
            }
            _ => {}
        }
    }

    fn open_delete_tag(&mut self) {
        if let View::List(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let commit_hash = commit_list_state.selected_commit_hash().clone();
            let tags = commit_list_state.selected_commit_refs();
            let has_tags = tags.iter().any(|r| matches!(r.as_ref(), Ref::Tag { .. }));
            if !has_tags {
                self.view = View::of_list(
                    commit_list_state,
                    self.ui_config,
                    self.color_theme,
                    self.tx.clone(),
                );
                self.tx
                    .send(AppEvent::NotifyWarn("No tags on this commit".into()));
                return;
            }
            self.view = View::of_delete_tag(
                commit_list_state,
                commit_hash,
                tags,
                self.repository.path().to_path_buf(),
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn close_delete_tag(&mut self) {
        if let View::DeleteTag(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            self.view = View::of_list(
                commit_list_state,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn remove_tag_from_commit(&mut self, commit_hash: &CommitHash, tag_name: &str) {
        self.repository.remove_ref(tag_name);

        match &mut self.view {
            View::List(view) => {
                view.remove_ref_from_commit(commit_hash, tag_name);
            }
            View::DeleteTag(view) => {
                view.remove_ref_from_commit(commit_hash, tag_name);
            }
            _ => {}
        }
    }

    fn open_delete_ref(&mut self, ref_name: String, ref_type: RefType) {
        if let View::Refs(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let ref_list_state = view.take_ref_list_state();
            let refs = view.take_refs();
            self.view = View::of_delete_ref(
                commit_list_state,
                ref_list_state,
                refs,
                self.repository.path().to_path_buf(),
                ref_name,
                ref_type,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn close_delete_ref(&mut self) {
        if let View::DeleteRef(ref mut view) = self.view {
            let Some(commit_list_state) = view.take_list_state() else {
                return;
            };
            let ref_list_state = view.take_ref_list_state();
            let refs = view.take_refs();
            self.view = View::of_refs_with_state(
                commit_list_state,
                ref_list_state,
                refs,
                self.ui_config,
                self.color_theme,
                self.tx.clone(),
            );
        }
    }

    fn remove_ref_from_list(&mut self, ref_name: &str) {
        self.repository.remove_ref(ref_name);

        match &mut self.view {
            View::Refs(view) => {
                view.remove_ref(ref_name);
            }
            View::DeleteRef(view) => {
                view.remove_ref(ref_name);
            }
            _ => {}
        }
    }

    fn open_help(&mut self) {
        let before_view = std::mem::take(&mut self.view);
        self.view = View::of_help(
            before_view,
            self.color_theme,
            self.image_protocol,
            self.tx.clone(),
            self.keybind,
            self.core_config,
        );
    }

    fn close_help(&mut self) {
        if let View::Help(ref mut view) = self.view {
            self.view = view.take_before_view();
        }
    }

    fn clear_help(&mut self) {
        if let View::Help(ref mut view) = self.view {
            view.clear();
        }
    }

    fn select_older_commit(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            view.select_older_commit(&self.repository);
        } else if let View::UserCommand(ref mut view) = self.view {
            view.select_older_commit(&self.repository, self.view_area);
        }
    }

    fn select_newer_commit(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            view.select_newer_commit(&self.repository);
        } else if let View::UserCommand(ref mut view) = self.view {
            view.select_newer_commit(&self.repository, self.view_area);
        }
    }

    fn select_parent_commit(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            view.select_parent_commit(&self.repository);
        } else if let View::UserCommand(ref mut view) = self.view {
            view.select_parent_commit(&self.repository, self.view_area);
        }
    }

    fn clear_status_line(&mut self) {
        self.status_line = StatusLine::None;
    }

    fn update_status_input(
        &mut self,
        msg: String,
        cursor_pos: Option<u16>,
        transient_msg: Option<String>,
    ) {
        self.status_line = StatusLine::Input(msg, cursor_pos, transient_msg);
    }

    fn info_notification(&mut self, msg: String) {
        self.status_line = StatusLine::NotificationInfo(msg);
    }

    fn success_notification(&mut self, msg: String) {
        self.status_line = StatusLine::NotificationSuccess(msg);
    }

    fn warn_notification(&mut self, msg: String) {
        self.status_line = StatusLine::NotificationWarn(msg);
    }

    fn error_notification(&mut self, msg: String) {
        self.status_line = StatusLine::NotificationError(msg);
    }

    fn copy_to_clipboard(&self, name: String, value: String) {
        match copy_to_clipboard(value) {
            Ok(_) => {
                let msg = format!("Copied {name} to clipboard successfully");
                self.tx.send(AppEvent::NotifySuccess(msg));
            }
            Err(msg) => {
                self.tx.send(AppEvent::NotifyError(msg));
            }
        }
    }

    fn refresh(&mut self) {
        // Reload repository from disk
        let sort = self.repository.sort_order();
        let path = self.repository.path().to_path_buf();

        let repository = match Repository::load(&path, sort) {
            Ok(repo) => repo,
            Err(e) => {
                self.tx
                    .send(AppEvent::NotifyError(format!("Refresh failed: {}", e)));
                return;
            }
        };

        // Recalculate graph
        let graph = Rc::new(calc_graph(&repository));

        // Create new graph image manager
        let graph_image_manager = GraphImageManager::new(
            Rc::clone(&graph),
            self.graph_color_set,
            self.cell_width_type,
            self.image_protocol,
            false, // don't preload
        );

        // Build new commit list state
        let mut ref_name_to_commit_index_map = HashMap::new();
        let commits = graph
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let refs = repository.refs(&commit.commit_hash);
                for r in &refs {
                    ref_name_to_commit_index_map.insert(r.name().to_string(), i);
                }
                let (pos_x, _) = graph.commit_pos_map[&commit.commit_hash];
                let graph_color = self.graph_color_set.get(pos_x).to_ratatui_color();
                CommitInfo::new(commit.clone(), refs, graph_color)
            })
            .collect();

        let graph_cell_width = match self.cell_width_type {
            CellWidthType::Double => (graph.max_pos_x + 1) as u16 * 2,
            CellWidthType::Single => (graph.max_pos_x + 1) as u16,
        };

        let head = repository.head();
        let commit_list_state = CommitListState::new(
            commits,
            graph_image_manager,
            graph_cell_width,
            head,
            ref_name_to_commit_index_map,
            self.core_config.search.ignore_case,
            self.core_config.search.fuzzy,
        );

        // Update app state
        self.repository = repository;
        self.view = View::of_list(
            commit_list_state,
            self.ui_config,
            self.color_theme,
            self.tx.clone(),
        );

        self.tx.send(AppEvent::NotifySuccess("Repository refreshed".into()));
    }
}

fn process_numeric_prefix(
    numeric_prefix: &str,
    user_event: UserEvent,
    _key_event: KeyEvent,
) -> UserEventWithCount {
    if user_event.is_countable() {
        let count = if numeric_prefix.is_empty() {
            1
        } else {
            numeric_prefix.parse::<usize>().unwrap_or(1)
        };
        UserEventWithCount::new(user_event, count)
    } else {
        UserEventWithCount::from_event(user_event)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rustfmt::skip]
    #[rstest]
    #[case("",    UserEvent::NavigateDown, UserEventWithCount::new(UserEvent::NavigateDown, 1))] // no prefix
    #[case("5",   UserEvent::NavigateUp,   UserEventWithCount::new(UserEvent::NavigateUp, 5))] // with prefix
    #[case("0",   UserEvent::PageDown,     UserEventWithCount::new(UserEvent::PageDown, 1))] // zero should be converted to 1
    #[case("42",  UserEvent::ScrollDown,   UserEventWithCount::new(UserEvent::ScrollDown, 42))] // multi-digit number
    #[case("999", UserEvent::PageDown,     UserEventWithCount::new(UserEvent::PageDown, 999))] // large number
    #[case("abc", UserEvent::ScrollUp,     UserEventWithCount::new(UserEvent::ScrollUp, 1))] // should fallback to 1
    #[case("5",   UserEvent::Quit,         UserEventWithCount::new(UserEvent::Quit, 1))] // non-countable event with prefix
    #[case("",    UserEvent::Confirm,      UserEventWithCount::new(UserEvent::Confirm, 1))] // non-countable event without prefix
    fn test_process_numeric_prefix(
        #[case] numeric_prefix: &str,
        #[case] user_event: UserEvent,
        #[case] expected: UserEventWithCount,
    ) {
        let dummy_key_event = KeyEvent::from(KeyCode::Enter); // KeyEvent is not used in the logic
        let actual = process_numeric_prefix(numeric_prefix, user_event, dummy_key_event);
        assert_eq!(actual, expected);
    }
}
