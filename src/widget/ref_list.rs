use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, StatefulWidget},
};
use semver::Version;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{app::AppContext, color::ColorTheme, git::Ref};

const TREE_BRANCH_ROOT_IDENT: &str = "__branches__";
const TREE_REMOTE_ROOT_IDENT: &str = "__remotes__";
const TREE_TAG_ROOT_IDENT: &str = "__tags__";
const TREE_STASH_ROOT_IDENT: &str = "__stashes__";

const TREE_BRANCH_ROOT_TEXT: &str = "Branches";
const TREE_REMOTE_ROOT_TEXT: &str = "Remotes";
const TREE_TAG_ROOT_TEXT: &str = "Tags";
const TREE_STASH_ROOT_TEXT: &str = "Stashes";

#[derive(Debug, Default)]
pub struct RefListState {
    tree_state: TreeState<String>,
}

impl RefListState {
    pub fn new() -> Self {
        let mut tree_state = TreeState::default();
        tree_state.select(vec![TREE_BRANCH_ROOT_IDENT.into()]);
        tree_state.open(vec![TREE_BRANCH_ROOT_IDENT.into()]);
        Self { tree_state }
    }
}

impl RefListState {
    pub fn select_next(&mut self) {
        self.tree_state.key_down();
    }

    pub fn select_prev(&mut self) {
        self.tree_state.key_up();
    }

    pub fn select_first(&mut self) {
        self.tree_state.select_first();
    }

    pub fn select_last(&mut self) {
        self.tree_state.select_last();
    }

    pub fn open_node(&mut self) {
        self.tree_state.key_right();
    }

    pub fn close_node(&mut self) {
        self.tree_state.key_left();
    }

    pub fn selected_ref_name(&self) -> Option<String> {
        self.tree_state.selected().last().cloned()
    }

    pub fn selected_branch(&self) -> Option<String> {
        let selected = self.tree_state.selected();
        if selected.len() > 1
            && (selected[0] == TREE_BRANCH_ROOT_IDENT || selected[0] == TREE_REMOTE_ROOT_IDENT)
        {
            selected.last().cloned()
        } else {
            None
        }
    }

    pub fn selected_tag(&self) -> Option<String> {
        let selected = self.tree_state.selected();
        if selected.len() > 1 && selected[0] == TREE_TAG_ROOT_IDENT {
            selected.last().cloned()
        } else {
            None
        }
    }

    pub fn current_tree_status(&self) -> (Vec<String>, Vec<Vec<String>>) {
        let selected = self.tree_state.selected().into();
        let opened = self.tree_state.opened().iter().cloned().collect();
        (selected, opened)
    }

    pub fn reset_tree_status(&mut self, selected: Vec<String>, opened: Vec<Vec<String>>) {
        self.tree_state.select(selected);
        for node in opened {
            self.tree_state.open(node);
        }
    }
}

pub struct RefList {
    items: Vec<TreeItem<'static, String>>,
    ctx: Rc<AppContext>,
}

impl RefList {
    pub fn new(refs: &[Ref], ctx: Rc<AppContext>) -> RefList {
        let items = build_ref_tree_nodes(refs)
            .into_iter()
            .map(|node| node.into_tree_item(&ctx.color_theme))
            .collect();
        RefList { items, ctx }
    }
}

impl StatefulWidget for RefList {
    type State = RefListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let tree = Tree::new(&self.items)
            .unwrap()
            .node_closed_symbol("\u{25b8} ") // ▸
            .node_open_symbol("\u{25be} ") // ▾
            .node_no_children_symbol("  ")
            .highlight_style(
                Style::default()
                    .bg(self.ctx.color_theme.ref_selected_bg)
                    .fg(self.ctx.color_theme.ref_selected_fg),
            )
            .block(
                Block::default()
                    .borders(Borders::LEFT)
                    .style(Style::default().fg(self.ctx.color_theme.divider_fg))
                    .padding(Padding::horizontal(1)),
            );
        tree.render(area, buf, &mut state.tree_state);
    }
}

fn build_ref_tree_nodes(refs: &[Ref]) -> Vec<RefTreeNode> {
    let mut branch_refs = Vec::new();
    let mut remote_refs = Vec::new();
    let mut tag_refs = Vec::new();
    let mut stash_refs = Vec::new();

    for r in refs {
        match r {
            Ref::Tag { name, .. } => tag_refs.push(name.into()),
            Ref::Branch { name, .. } => branch_refs.push(name.into()),
            Ref::RemoteBranch { name, .. } => remote_refs.push(name.into()),
            Ref::Stash { name, message, .. } => stash_refs.push((name.into(), message.into())),
        }
    }

    let mut branch_nodes = refs_to_ref_tree_nodes(branch_refs);
    let mut remote_nodes = refs_to_ref_tree_nodes(remote_refs);
    let mut tag_nodes = refs_to_ref_tree_nodes(tag_refs);
    let mut stash_nodes = refs_to_stash_ref_tree_nodes(stash_refs);

    sort_branch_tree_nodes(&mut branch_nodes);
    sort_branch_tree_nodes(&mut remote_nodes);
    sort_tag_tree_nodes(&mut tag_nodes);
    sort_stash_tree_nodes(&mut stash_nodes);

    vec![
        RefTreeNode::new(
            TREE_BRANCH_ROOT_IDENT.into(),
            TREE_BRANCH_ROOT_TEXT.into(),
            branch_nodes,
        ),
        RefTreeNode::new(
            TREE_REMOTE_ROOT_IDENT.into(),
            TREE_REMOTE_ROOT_TEXT.into(),
            remote_nodes,
        ),
        RefTreeNode::new(
            TREE_TAG_ROOT_IDENT.into(),
            TREE_TAG_ROOT_TEXT.into(),
            tag_nodes,
        ),
        RefTreeNode::new(
            TREE_STASH_ROOT_IDENT.into(),
            TREE_STASH_ROOT_TEXT.into(),
            stash_nodes,
        ),
    ]
}

struct RefTreeNode {
    identifier: String,
    name: String,
    children: Vec<RefTreeNode>,
}

impl RefTreeNode {
    fn new(identifier: String, name: String, children: Vec<Self>) -> Self {
        Self {
            identifier,
            name,
            children,
        }
    }

    fn into_tree_item(self, color_theme: &ColorTheme) -> TreeItem<'static, String> {
        let children = self
            .children
            .into_iter()
            .map(|child| child.into_tree_item(color_theme))
            .collect();
        TreeItem::new(self.identifier, self.name.fg(color_theme.fg), children).unwrap()
    }
}

fn refs_to_stash_ref_tree_nodes(ref_name_messages: Vec<(String, String)>) -> Vec<RefTreeNode> {
    let mut nodes: Vec<RefTreeNode> = Vec::new();
    for (name, message) in ref_name_messages {
        let node = RefTreeNode {
            identifier: name.clone(),
            name: message.to_string(),
            children: Vec::new(),
        };
        nodes.push(node);
    }
    nodes
}

fn refs_to_ref_tree_nodes(ref_names: Vec<String>) -> Vec<RefTreeNode> {
    let mut nodes: Vec<RefTreeNode> = Vec::new();

    for ref_name in ref_names {
        let mut parts = ref_name.split('/').collect::<Vec<_>>();
        let mut current_nodes = &mut nodes;
        let mut parent_identifier = String::new();

        while !parts.is_empty() {
            let part = parts.remove(0);
            if let Some(index) = current_nodes.iter().position(|n| n.name == part) {
                let node = &mut current_nodes[index];
                current_nodes = &mut node.children;
                parent_identifier.clone_from(&node.identifier);
            } else {
                let identifier = if parent_identifier.is_empty() {
                    part.to_string()
                } else {
                    format!("{parent_identifier}/{part}")
                };
                let node = RefTreeNode {
                    identifier: identifier.clone(),
                    name: part.to_string(),
                    children: Vec::new(),
                };
                current_nodes.push(node);
                current_nodes = current_nodes.last_mut().unwrap().children.as_mut();
                parent_identifier = identifier;
            }
        }
    }

    nodes
}

fn sort_branch_tree_nodes(nodes: &mut [RefTreeNode]) {
    nodes.sort_by(|a, b| {
        b.children
            .len()
            .cmp(&a.children.len())
            .then(a.name.cmp(&b.name))
    });
    for node in nodes {
        sort_branch_tree_nodes(&mut node.children);
    }
}

fn sort_tag_tree_nodes(nodes: &mut [RefTreeNode]) {
    nodes.sort_by(|a, b| {
        let a_version = parse_semantic_version_tag(&a.name);
        let b_version = parse_semantic_version_tag(&b.name);
        if a_version.is_none() && b_version.is_none() {
            // if both are not semantic versions, sort by name asc
            a.name.cmp(&b.name)
        } else {
            // if both are semantic versions, sort by version desc
            // if only one is a semantic version, it will be sorted first
            b_version.cmp(&a_version)
        }
    });
}

fn sort_stash_tree_nodes(nodes: &mut [RefTreeNode]) {
    nodes.sort_by(|a, b| a.identifier.cmp(&b.identifier));
}

fn parse_semantic_version_tag(tag: &str) -> Option<Version> {
    let tag = tag.trim_start_matches('v');
    Version::parse(tag).ok()
}
