use std::collections::HashMap;

use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};

use crate::{
    color::ColorSet,
    config::Config,
    event::{AppEvent, Receiver, Sender, UserEvent},
    external::copy_to_clipboard,
    git::Repository,
    graph::{Graph, GraphImage},
    keybind::KeyBind,
    protocol::ImageProtocol,
    view::View,
    widget::commit_list::{CommitInfo, CommitListState},
};

const MESSAGE_STATUTS_COLOR: Color = Color::Reset;
const INFO_STATUS_COLOR: Color = Color::Cyan;
const SUCCESS_STATUS_COLOR: Color = Color::Green;
const WARN_STATUS_COLOR: Color = Color::Yellow;
const ERROR_STATUS_COLOR: Color = Color::Red;

const DIVIDER_COLOR: Color = Color::DarkGray;

#[derive(Debug)]
enum StatusLine {
    None,
    Input(String, Option<u16>),
    NotificationInfo(String),
    NotificationSuccess(String),
    NotificationWarn(String),
    NotificationError(String),
}

#[derive(Debug)]
pub struct App<'a> {
    repository: &'a Repository,
    view: View<'a>,
    status_line: StatusLine,

    keybind: &'a KeyBind,
    config: &'a Config,
    image_protocol: ImageProtocol,
    tx: Sender,
}

impl<'a> App<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repository: &'a Repository,
        graph: &'a Graph,
        graph_image: &'a GraphImage,
        keybind: &'a KeyBind,
        config: &'a Config,
        color_set: &'a ColorSet,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> Self {
        let mut ref_name_to_commit_index_map = HashMap::new();
        let commits = graph
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let edges = &graph.edges[i];
                let graph_row_image = &graph_image.images[edges];
                let image_cell_width = graph_row_image.cell_count * 2;
                let image = image_protocol.encode(&graph_row_image.bytes, image_cell_width);
                let refs = repository.refs(&commit.commit_hash);
                for r in &refs {
                    ref_name_to_commit_index_map.insert(r.name(), i);
                }
                let (pos_x, _) = graph.commit_pos_map[&commit.commit_hash];
                let graph_color = color_set.get(pos_x).to_ratatui_color();
                CommitInfo::new(commit, image, refs, graph_color)
            })
            .collect();
        let graph_cell_width = (graph.max_pos_x + 1) as u16 * 2;
        let head = repository.head();
        let commit_list_state = CommitListState::new(
            commits,
            graph_cell_width,
            head,
            ref_name_to_commit_index_map,
        );
        let view = View::of_list(commit_list_state, config, tx.clone());

        Self {
            repository,
            status_line: StatusLine::None,
            view,
            keybind,
            config,
            image_protocol,
            tx,
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
                    match self.keybind.get(&key) {
                        Some(UserEvent::ForceQuit) => {
                            self.tx.send(AppEvent::Quit);
                        }
                        Some(ue) => {
                            match self.status_line {
                                StatusLine::None | StatusLine::Input(_, _) => {
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
                            self.view.handle_event(ue, key);
                        }
                        None => {
                            self.view.handle_event(&UserEvent::Unknown, key);
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
                AppEvent::OpenRefs => {
                    self.open_refs();
                }
                AppEvent::CloseRefs => {
                    self.close_refs();
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
                AppEvent::CopyToClipboard { name, value } => {
                    self.copy_to_clipboard(name, value);
                }
                AppEvent::ClearStatusLine => {
                    self.clear_status_line();
                }
                AppEvent::UpdateStatusInput(msg, cursor_pos) => {
                    self.update_status_input(msg, cursor_pos);
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
            }
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let [view_area, status_line_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).areas(f.size());

        self.view.render(f, view_area);
        self.render_status_line(f, status_line_area);
    }
}

impl App<'_> {
    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let text: Span = match &self.status_line {
            StatusLine::None => "".into(),
            StatusLine::Input(msg, _) => msg.as_str().fg(MESSAGE_STATUTS_COLOR),
            StatusLine::NotificationInfo(msg) => msg.as_str().fg(INFO_STATUS_COLOR),
            StatusLine::NotificationSuccess(msg) => msg
                .as_str()
                .add_modifier(Modifier::BOLD)
                .fg(SUCCESS_STATUS_COLOR),
            StatusLine::NotificationWarn(msg) => msg
                .as_str()
                .add_modifier(Modifier::BOLD)
                .fg(WARN_STATUS_COLOR),
            StatusLine::NotificationError(msg) => format!("ERROR: {}", msg)
                .add_modifier(Modifier::BOLD)
                .fg(ERROR_STATUS_COLOR),
        };
        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().fg(DIVIDER_COLOR))
                .padding(Padding::horizontal(1)),
        );
        f.render_widget(paragraph, area);

        if let StatusLine::Input(_, Some(cursor_pos)) = &self.status_line {
            f.set_cursor(area.x + cursor_pos + 1, area.y + 1);
        }
    }
}

impl App<'_> {
    fn open_detail(&mut self) {
        if let View::List(ref mut view) = self.view {
            let commit_list_state = view.take_list_state();
            let selected = commit_list_state.selected_commit_hash().clone();
            let (commit, changes) = self.repository.commit_detail(&selected);
            let refs = self
                .repository
                .refs(&selected)
                .into_iter()
                .cloned()
                .collect();
            self.view = View::of_detail(
                commit_list_state,
                commit,
                changes,
                refs,
                self.config,
                self.image_protocol,
                self.tx.clone(),
            );
        }
    }

    fn close_detail(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            let commit_list_state = view.take_list_state();
            self.view = View::of_list(commit_list_state, self.config, self.tx.clone());
        }
    }

    fn clear_detail(&mut self) {
        if let View::Detail(ref mut view) = self.view {
            view.clear();
        }
    }

    fn open_refs(&mut self) {
        if let View::List(ref mut view) = self.view {
            let commit_list_state = view.take_list_state();
            let refs = self.repository.all_refs().into_iter().cloned().collect();
            self.view = View::of_refs(commit_list_state, refs, self.config, self.tx.clone());
        }
    }

    fn close_refs(&mut self) {
        if let View::Refs(ref mut view) = self.view {
            let commit_list_state = view.take_list_state();
            self.view = View::of_list(commit_list_state, self.config, self.tx.clone());
        }
    }

    fn open_help(&mut self) {
        let before_view = std::mem::take(&mut self.view);
        self.view = View::of_help(
            before_view,
            self.image_protocol,
            self.tx.clone(),
            self.keybind,
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

    fn clear_status_line(&mut self) {
        self.status_line = StatusLine::None;
    }

    fn update_status_input(&mut self, msg: String, cursor_pos: Option<u16>) {
        self.status_line = StatusLine::Input(msg, cursor_pos);
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
                let msg = format!("Copied {} to clipboard successfully", name);
                self.tx.send(AppEvent::NotifySuccess(msg));
            }
            Err(msg) => {
                self.tx.send(AppEvent::NotifyError(msg));
            }
        }
    }
}
