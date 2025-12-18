use std::{path::PathBuf, rc::Rc};

use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    color::ColorTheme,
    config::{CoreConfig, UiConfig},
    event::{Sender, UserEventWithCount},
    git::{Commit, CommitHash, FileChange, Ref, RefType},
    keybind::KeyBind,
    protocol::ImageProtocol,
    view::{
        create_tag::CreateTagView,
        delete_ref::DeleteRefView,
        delete_tag::DeleteTagView,
        detail::DetailView,
        help::HelpView,
        list::ListView,
        refs::RefsView,
        user_command::{UserCommandView, UserCommandViewBeforeView},
    },
    widget::{commit_list::CommitListState, ref_list::RefListState},
};

#[derive(Debug, Default)]
pub enum View<'a> {
    #[default]
    Default, // dummy variant to make #[default] work
    List(Box<ListView<'a>>),
    Detail(Box<DetailView<'a>>),
    UserCommand(Box<UserCommandView<'a>>),
    Refs(Box<RefsView<'a>>),
    CreateTag(Box<CreateTagView<'a>>),
    DeleteTag(Box<DeleteTagView<'a>>),
    DeleteRef(Box<DeleteRefView<'a>>),
    Help(Box<HelpView<'a>>),
}

impl<'a> View<'a> {
    pub fn handle_event(&mut self, event_with_count: UserEventWithCount, key_event: KeyEvent) {
        match self {
            View::Default => {}
            View::List(view) => view.handle_event(event_with_count, key_event),
            View::Detail(view) => view.handle_event(event_with_count, key_event),
            View::UserCommand(view) => view.handle_event(event_with_count, key_event),
            View::Refs(view) => view.handle_event(event_with_count, key_event),
            View::CreateTag(view) => view.handle_event(event_with_count, key_event),
            View::DeleteTag(view) => view.handle_event(event_with_count, key_event),
            View::DeleteRef(view) => view.handle_event(event_with_count, key_event),
            View::Help(view) => view.handle_event(event_with_count, key_event),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            View::Default => {}
            View::List(view) => view.render(f, area),
            View::Detail(view) => view.render(f, area),
            View::UserCommand(view) => view.render(f, area),
            View::Refs(view) => view.render(f, area),
            View::CreateTag(view) => view.render(f, area),
            View::DeleteTag(view) => view.render(f, area),
            View::DeleteRef(view) => view.render(f, area),
            View::Help(view) => view.render(f, area),
        }
    }

    pub fn of_list(
        commit_list_state: CommitListState,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::List(Box::new(ListView::new(
            commit_list_state,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_detail(
        commit_list_state: CommitListState,
        commit: Rc<Commit>,
        changes: Vec<FileChange>,
        refs: Vec<Rc<Ref>>,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> Self {
        View::Detail(Box::new(DetailView::new(
            commit_list_state,
            commit,
            changes,
            refs,
            ui_config,
            color_theme,
            image_protocol,
            tx,
        )))
    }

    pub fn of_user_command_from_list(
        commit_list_state: CommitListState,
        commit: Rc<Commit>,
        user_command_number: usize,
        view_area: Rect,
        core_config: &'a CoreConfig,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> Self {
        View::UserCommand(Box::new(UserCommandView::new(
            commit_list_state,
            commit,
            user_command_number,
            view_area,
            core_config,
            ui_config,
            color_theme,
            image_protocol,
            tx,
            UserCommandViewBeforeView::List,
        )))
    }

    pub fn of_user_command_from_detail(
        commit_list_state: CommitListState,
        commit: Rc<Commit>,
        user_command_number: usize,
        view_area: Rect,
        core_config: &'a CoreConfig,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
    ) -> Self {
        View::UserCommand(Box::new(UserCommandView::new(
            commit_list_state,
            commit,
            user_command_number,
            view_area,
            core_config,
            ui_config,
            color_theme,
            image_protocol,
            tx,
            UserCommandViewBeforeView::Detail,
        )))
    }

    pub fn of_refs(
        commit_list_state: CommitListState,
        refs: Vec<Rc<Ref>>,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::Refs(Box::new(RefsView::new(
            commit_list_state,
            refs,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_refs_with_state(
        commit_list_state: CommitListState,
        ref_list_state: RefListState,
        refs: Vec<Rc<Ref>>,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::Refs(Box::new(RefsView::with_state(
            commit_list_state,
            ref_list_state,
            refs,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_create_tag(
        commit_list_state: CommitListState,
        commit_hash: CommitHash,
        repo_path: PathBuf,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::CreateTag(Box::new(CreateTagView::new(
            commit_list_state,
            commit_hash,
            repo_path,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_delete_tag(
        commit_list_state: CommitListState,
        commit_hash: CommitHash,
        tags: Vec<Rc<Ref>>,
        repo_path: PathBuf,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::DeleteTag(Box::new(DeleteTagView::new(
            commit_list_state,
            commit_hash,
            tags,
            repo_path,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_delete_ref(
        commit_list_state: CommitListState,
        ref_list_state: RefListState,
        refs: Vec<Rc<Ref>>,
        repo_path: PathBuf,
        ref_name: String,
        ref_type: RefType,
        ui_config: &'a UiConfig,
        color_theme: &'a ColorTheme,
        tx: Sender,
    ) -> Self {
        View::DeleteRef(Box::new(DeleteRefView::new(
            commit_list_state,
            ref_list_state,
            refs,
            repo_path,
            ref_name,
            ref_type,
            ui_config,
            color_theme,
            tx,
        )))
    }

    pub fn of_help(
        before: View<'a>,
        color_theme: &'a ColorTheme,
        image_protocol: ImageProtocol,
        tx: Sender,
        keybind: &'a KeyBind,
        core_config: &'a CoreConfig,
    ) -> Self {
        View::Help(Box::new(HelpView::new(
            before,
            color_theme,
            image_protocol,
            tx,
            keybind,
            core_config,
        )))
    }
}
