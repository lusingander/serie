use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};
use semver::Version;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{color::ColorTheme, git::Ref};

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

    pub fn selected_local_branch(&self) -> Option<String> {
        let selected = self.tree_state.selected();
        if selected.len() > 1 && selected[0] == TREE_BRANCH_ROOT_IDENT {
            selected.last().cloned()
        } else {
            None
        }
    }

    pub fn selected_remote_branch(&self) -> Option<String> {
        let selected = self.tree_state.selected();
        if selected.len() > 1 && selected[0] == TREE_REMOTE_ROOT_IDENT {
            selected.last().cloned()
        } else {
            None
        }
    }

    pub fn adjust_selection_after_delete(&mut self) {
        // After item deletion, tree_state may point to non-existent item
        // The safest approach: just move selection, the tree widget will adjust
        // Try down first (next item takes deleted item's place), then up as fallback
        let before: Vec<String> = self.tree_state.selected().to_vec();
        self.tree_state.key_down();
        if self.tree_state.selected() == before {
            // key_down didn't move (we were at the end), try key_up
            self.tree_state.key_up();
        }
    }
}

pub struct RefList<'a> {
    items: Vec<TreeItem<'a, String>>,
    color_theme: &'a ColorTheme,
}

impl<'a> RefList<'a> {
    pub fn new(refs: &'a [Rc<Ref>], color_theme: &'a ColorTheme) -> RefList<'a> {
        let items = build_ref_tree_items(refs, color_theme);
        RefList { items, color_theme }
    }
}

impl StatefulWidget for RefList<'_> {
    type State = RefListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let make_block = || {
            Block::default()
                .borders(Borders::LEFT)
                .style(Style::default().fg(self.color_theme.divider_fg))
                .padding(Padding::horizontal(1))
        };

        let Ok(tree) = Tree::new(&self.items) else {
            Paragraph::new("Error: failed to build ref tree")
                .fg(self.color_theme.status_error_fg)
                .block(make_block())
                .render(area, buf);
            return;
        };
        let tree = tree
            .node_closed_symbol("\u{25b8} ") // ▸
            .node_open_symbol("\u{25be} ") // ▾
            .node_no_children_symbol("  ")
            .highlight_style(
                Style::default()
                    .bg(self.color_theme.ref_selected_bg)
                    .fg(self.color_theme.ref_selected_fg),
            )
            .block(make_block());
        StatefulWidget::render(tree, area, buf, &mut state.tree_state);
    }
}

fn build_ref_tree_items<'a>(
    refs: &'a [Rc<Ref>],
    color_theme: &'a ColorTheme,
) -> Vec<TreeItem<'a, String>> {
    let mut branch_refs = Vec::new();
    let mut remote_refs = Vec::new();
    let mut tag_refs = Vec::new();
    let mut stash_refs = Vec::new();

    for r in refs {
        match r.as_ref() {
            Ref::Tag { name, .. } => tag_refs.push(name.clone()),
            Ref::Branch { name, .. } => branch_refs.push(name.clone()),
            Ref::RemoteBranch { name, .. } => remote_refs.push(name.clone()),
            Ref::Stash { name, message, .. } => stash_refs.push((name.clone(), message.clone())),
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

    let branch_items = ref_tree_nodes_to_tree_items(branch_nodes, color_theme);
    let remote_items = ref_tree_nodes_to_tree_items(remote_nodes, color_theme);
    let tag_items = ref_tree_nodes_to_tree_items(tag_nodes, color_theme);
    let stash_items = ref_tree_nodes_to_tree_items(stash_nodes, color_theme);

    [
        tree_item(
            TREE_BRANCH_ROOT_IDENT.into(),
            TREE_BRANCH_ROOT_TEXT.into(),
            branch_items,
            color_theme,
        ),
        tree_item(
            TREE_REMOTE_ROOT_IDENT.into(),
            TREE_REMOTE_ROOT_TEXT.into(),
            remote_items,
            color_theme,
        ),
        tree_item(
            TREE_TAG_ROOT_IDENT.into(),
            TREE_TAG_ROOT_TEXT.into(),
            tag_items,
            color_theme,
        ),
        tree_item(
            TREE_STASH_ROOT_IDENT.into(),
            TREE_STASH_ROOT_TEXT.into(),
            stash_items,
            color_theme,
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

struct RefTreeNode {
    identifier: String,
    name: String,
    children: Vec<RefTreeNode>,
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
        let mut current_nodes = &mut nodes;
        let mut parent_identifier = String::new();

        for part in ref_name.split('/') {
            if let Some(index) = current_nodes.iter().position(|n| n.name == part) {
                let node = &mut current_nodes[index];
                parent_identifier.clone_from(&node.identifier);
                current_nodes = &mut node.children;
            } else {
                let identifier = if parent_identifier.is_empty() {
                    part.to_string()
                } else {
                    format!("{parent_identifier}/{part}")
                };
                current_nodes.push(RefTreeNode {
                    identifier: identifier.clone(),
                    name: part.to_string(),
                    children: Vec::new(),
                });
                let Some(last) = current_nodes.last_mut() else {
                    break;
                };
                current_nodes = &mut last.children;
                parent_identifier = identifier;
            }
        }
    }

    nodes
}

fn ref_tree_nodes_to_tree_items(
    nodes: Vec<RefTreeNode>,
    color_theme: &ColorTheme,
) -> Vec<TreeItem<'_, String>> {
    nodes
        .into_iter()
        .filter_map(|node| {
            if node.children.is_empty() {
                tree_item(node.identifier, node.name, Vec::new(), color_theme)
            } else {
                let children = ref_tree_nodes_to_tree_items(node.children, color_theme);
                tree_item(node.identifier, node.name, children, color_theme)
            }
        })
        .collect()
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

fn tree_item<'a>(
    identifier: String,
    name: String,
    children: Vec<TreeItem<'a, String>>,
    color_theme: &'a ColorTheme,
) -> Option<TreeItem<'a, String>> {
    TreeItem::new(identifier, name.fg(color_theme.fg), children).ok()
}
