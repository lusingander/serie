use rustc_hash::{FxHashMap, FxHashSet};

use crate::git::{Commit, CommitHash, Ref, Repository};

type CommitPosMap<'a> = FxHashMap<&'a CommitHash, (usize, usize)>;

#[derive(Debug)]
pub struct Graph<'a> {
    pub commits: Vec<&'a Commit>,
    pub commit_pos_map: CommitPosMap<'a>,
    pub edges: Vec<Vec<Edge>>,
    pub max_pos_x: usize,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub edge_type: EdgeType,
    pub pos_x: usize,
    pub associated_line_pos_x: usize,
}

impl Edge {
    pub fn new(edge_type: EdgeType, pos_x: usize, line_pos_x: usize) -> Self {
        Self {
            edge_type,
            pos_x,
            associated_line_pos_x: line_pos_x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeType {
    Vertical,    // │
    Horizontal,  // ─
    Up,          // ╵
    Down,        // ╷
    Left,        // ╴
    Right,       // ╶
    RightTop,    // ╮
    RightBottom, // ╯
    LeftTop,     // ╭
    LeftBottom,  // ╰
}

impl EdgeType {
    pub fn is_vertically_related(&self) -> bool {
        matches!(self, EdgeType::Vertical | EdgeType::Up | EdgeType::Down)
    }
}

pub fn calc_graph<'a>(repository: &'a Repository, primary_branch: Option<&str>) -> Graph<'a> {
    let commits = repository.all_commits();

    let (commit_pos_map, warning) = calc_commit_positions(&commits, repository, primary_branch);
    let (graph_edges, max_pos_x) = calc_edges(&commit_pos_map, &commits, repository);

    Graph {
        commits,
        commit_pos_map,
        edges: graph_edges,
        max_pos_x,
        warning,
    }
}

fn find_primary_branch_tip<'a>(
    repository: &'a Repository,
    primary_branch: &str,
) -> Option<&'a CommitHash> {
    if primary_branch.is_empty() {
        return None;
    }

    for r in repository.all_refs() {
        if let Ref::Branch { name, target } = r {
            if name == primary_branch {
                return Some(target);
            }
        }
    }

    None
}

fn primary_branch_spine<'a>(
    commits: &[&'a Commit],
    repository: &'a Repository,
    primary_branch: Option<&str>,
) -> (FxHashSet<&'a CommitHash>, Option<String>) {
    let Some(branch) = primary_branch else {
        return (FxHashSet::default(), None);
    };

    let Some(tip) = find_primary_branch_tip(repository, branch) else {
        let msg = format!("Primary branch '{branch}' could not be resolved");
        return (FxHashSet::default(), Some(msg));
    };

    if !commits.iter().any(|c| &c.commit_hash == tip) {
        let msg = format!("Primary branch '{branch}' tip is not included in the loaded commits");
        return (FxHashSet::default(), Some(msg));
    }

    let mut spine = FxHashSet::default();
    let mut current = Some(tip);

    while let Some(hash) = current {
        spine.insert(hash);
        let parents = repository.parents_hash(hash);
        current = parents.first().copied();
    }

    (spine, None)
}

fn calc_commit_positions<'a>(
    commits: &[&'a Commit],
    repository: &'a Repository,
    primary_branch: Option<&str>,
) -> (CommitPosMap<'a>, Option<String>) {
    let mut commit_pos_map: CommitPosMap = FxHashMap::default();
    let mut commit_line_state: Vec<Option<&CommitHash>> = Vec::new();
    let mut line_last_occupied_pos_y: Vec<usize> = Vec::new();
    let (primary_spine, warning) = primary_branch_spine(commits, repository, primary_branch);
    let has_visible_primary = !primary_spine.is_empty();
    let min_col = if has_visible_primary { 1 } else { 0 };

    for (pos_y, commit) in commits.iter().enumerate() {
        let is_primary = primary_spine.contains(&commit.commit_hash);
        let filtered_children_hash = filtered_children_hash(commit, repository);
        if filtered_children_hash.is_empty() {
            let merge_child_pos_y = repository
                .children_hash(&commit.commit_hash)
                .into_iter()
                .map(|hash| commit_pos_map[hash].1)
                .min();

            let pos_x = if is_primary {
                // For a primary spine commit with no first-parent children (i.e. tip of primary branch),
                // verify that column 0 is vacant and doesn't overlap an active merge line before assigning it.
                let col_0_vacant = commit_line_state.first().is_none_or(|c| c.is_none());
                let col_0_no_overlap = merge_child_pos_y.is_none_or(|child_y| {
                    line_last_occupied_pos_y.first().copied().unwrap_or(0) <= child_y
                });
                if col_0_vacant && col_0_no_overlap {
                    0
                } else {
                    get_first_vacant_line(
                        &commit_line_state,
                        &line_last_occupied_pos_y,
                        merge_child_pos_y,
                        min_col,
                    )
                }
            } else {
                get_first_vacant_line(
                    &commit_line_state,
                    &line_last_occupied_pos_y,
                    merge_child_pos_y,
                    min_col,
                )
            };
            add_commit_line(commit, &mut commit_line_state, pos_x);
            commit_pos_map.insert(&commit.commit_hash, (pos_x, pos_y));
            occupy_line(&mut line_last_occupied_pos_y, pos_x, pos_y);
        } else {
            let mut pos_x =
                update_commit_line(commit, &mut commit_line_state, &filtered_children_hash);

            // If this is a primary spine commit and update_commit_line placed it in a column > 0
            // (e.g. because first-parent children were on non-zero columns), properly relocate
            // the state line to column 0 if vacant to keep commit_line_state and commit_pos_map synchronized.
            if is_primary && pos_x != 0 && commit_line_state.first().is_none_or(|c| c.is_none()) {
                commit_line_state[pos_x] = None;
                if commit_line_state.is_empty() {
                    commit_line_state.push(Some(&commit.commit_hash));
                } else {
                    commit_line_state[0] = Some(&commit.commit_hash);
                }
                pos_x = 0;
            }

            commit_pos_map.insert(&commit.commit_hash, (pos_x, pos_y));
            for child_hash in filtered_children_hash {
                let child_pos_x = commit_pos_map[child_hash].0;
                occupy_line(&mut line_last_occupied_pos_y, child_pos_x, pos_y);
            }
        }
    }

    (commit_pos_map, warning)
}

fn filtered_children_hash<'a>(
    commit: &'a Commit,
    repository: &'a Repository,
) -> Vec<&'a CommitHash> {
    repository
        .children_hash(&commit.commit_hash)
        .into_iter()
        .filter(|child_hash| {
            let child_parents_hash = repository.parents_hash(child_hash);
            !child_parents_hash.is_empty() && *child_parents_hash[0] == commit.commit_hash
        })
        .collect()
}

fn get_first_vacant_line(
    commit_line_state: &[Option<&CommitHash>],
    line_last_occupied_pos_y: &[usize],
    merge_child_pos_y: Option<usize>,
    min_col: usize,
) -> usize {
    commit_line_state
        .iter()
        .enumerate()
        .skip(min_col)
        .position(|(pos_x, commit)| {
            // A line can be vacant in the current state while still containing an edge above
            // this row. Reusing it for a merge edge that reaches above that edge would make two
            // unrelated branches appear to be connected.
            commit.is_none()
                && merge_child_pos_y.is_none_or(|merge_child_pos_y| {
                    line_last_occupied_pos_y.get(pos_x).copied().unwrap_or(0) <= merge_child_pos_y
                })
        })
        .map(|pos_x| pos_x + min_col)
        .unwrap_or_else(|| commit_line_state.len().max(min_col))
}

fn occupy_line(line_last_occupied_pos_y: &mut Vec<usize>, pos_x: usize, pos_y: usize) {
    if line_last_occupied_pos_y.len() <= pos_x {
        line_last_occupied_pos_y.resize(pos_x + 1, 0);
    }
    line_last_occupied_pos_y[pos_x] = pos_y;
}

fn add_commit_line<'a>(
    commit: &'a Commit,
    commit_line_state: &mut Vec<Option<&'a CommitHash>>,
    pos_x: usize,
) {
    if commit_line_state.len() <= pos_x {
        commit_line_state.resize(pos_x + 1, None);
    }
    commit_line_state[pos_x] = Some(&commit.commit_hash);
}

fn update_commit_line<'a>(
    commit: &'a Commit,
    commit_line_state: &mut [Option<&'a CommitHash>],
    target_commit_hashes: &[&CommitHash],
) -> usize {
    if commit_line_state.is_empty() {
        return 0;
    }
    let mut min_pos_x = commit_line_state.len().saturating_sub(1);
    for target_hash in target_commit_hashes {
        for (pos_x, commit_hash) in commit_line_state.iter().enumerate() {
            if let Some(hash) = commit_hash {
                if hash == target_hash {
                    commit_line_state[pos_x] = None;
                    if min_pos_x > pos_x {
                        min_pos_x = pos_x;
                    }
                    break;
                }
            }
        }
    }
    commit_line_state[min_pos_x] = Some(&commit.commit_hash);
    min_pos_x
}

#[derive(Debug, Clone)]
struct WrappedEdge<'a> {
    edge: Edge,
    edge_parent_hash: &'a CommitHash,
}

impl<'a> WrappedEdge<'a> {
    fn new(
        edge_type: EdgeType,
        pos_x: usize,
        line_pos_x: usize,
        edge_parent_hash: &'a CommitHash,
    ) -> Self {
        Self {
            edge: Edge::new(edge_type, pos_x, line_pos_x),
            edge_parent_hash,
        }
    }
}

fn calc_edges(
    commit_pos_map: &CommitPosMap,
    commits: &[&Commit],
    repository: &Repository,
) -> (Vec<Vec<Edge>>, usize) {
    let mut max_pos_x = 0;
    let mut edges: Vec<Vec<WrappedEdge>> = vec![vec![]; commits.len()];

    for commit in commits {
        let (pos_x, pos_y) = commit_pos_map[&commit.commit_hash];
        let hash = &commit.commit_hash;

        for child_hash in repository.children_hash(hash) {
            let (child_pos_x, child_pos_y) = commit_pos_map[child_hash];

            if pos_x == child_pos_x {
                // commit
                edges[pos_y].push(WrappedEdge::new(EdgeType::Up, pos_x, pos_x, hash));
                for y in ((child_pos_y + 1)..pos_y).rev() {
                    edges[y].push(WrappedEdge::new(EdgeType::Vertical, pos_x, pos_x, hash));
                }
                edges[child_pos_y].push(WrappedEdge::new(EdgeType::Down, pos_x, pos_x, hash));
            } else {
                let child_first_parent_hash = &commits[child_pos_y].parent_commit_hashes[0];
                if *child_first_parent_hash == *hash {
                    // branch
                    if pos_x < child_pos_x {
                        edges[pos_y].push(WrappedEdge::new(
                            EdgeType::Right,
                            pos_x,
                            child_pos_x,
                            hash,
                        ));
                        for x in (pos_x + 1)..child_pos_x {
                            edges[pos_y].push(WrappedEdge::new(
                                EdgeType::Horizontal,
                                x,
                                child_pos_x,
                                hash,
                            ));
                        }
                        edges[pos_y].push(WrappedEdge::new(
                            EdgeType::RightBottom,
                            child_pos_x,
                            child_pos_x,
                            hash,
                        ));
                    } else {
                        edges[pos_y].push(WrappedEdge::new(
                            EdgeType::Left,
                            pos_x,
                            child_pos_x,
                            hash,
                        ));
                        for x in (child_pos_x + 1)..pos_x {
                            edges[pos_y].push(WrappedEdge::new(
                                EdgeType::Horizontal,
                                x,
                                child_pos_x,
                                hash,
                            ));
                        }
                        edges[pos_y].push(WrappedEdge::new(
                            EdgeType::LeftBottom,
                            child_pos_x,
                            child_pos_x,
                            hash,
                        ));
                    }
                    for y in ((child_pos_y + 1)..pos_y).rev() {
                        edges[y].push(WrappedEdge::new(
                            EdgeType::Vertical,
                            child_pos_x,
                            child_pos_x,
                            hash,
                        ));
                    }
                    edges[child_pos_y].push(WrappedEdge::new(
                        EdgeType::Down,
                        child_pos_x,
                        child_pos_x,
                        hash,
                    ));
                } else {
                    // merge
                    // skip
                }
            }
        }

        if max_pos_x < pos_x {
            max_pos_x = pos_x;
        }

        // draw down edge if has parent but parent not in the graph (when max_count is set)
        if !commit.parent_commit_hashes.is_empty()
            && repository.commit(&commit.parent_commit_hashes[0]).is_none()
        {
            edges[pos_y].push(WrappedEdge::new(EdgeType::Down, pos_x, pos_x, hash));
            ((pos_y + 1)..commits.len()).for_each(|y| {
                edges[y].push(WrappedEdge::new(EdgeType::Vertical, pos_x, pos_x, hash));
            });
        }
    }

    for commit in commits {
        let (pos_x, pos_y) = commit_pos_map[&commit.commit_hash];
        let hash = &commit.commit_hash;

        for child_hash in repository.children_hash(hash) {
            let (child_pos_x, child_pos_y) = commit_pos_map[child_hash];

            if pos_x == child_pos_x {
                // commit
                // skip
            } else {
                let child_first_parent_hash = &commits[child_pos_y].parent_commit_hashes[0];
                if *child_first_parent_hash == *hash {
                    // branch
                    // skip
                } else {
                    // merge
                    let mut overlap = false;
                    let mut new_pos_x = pos_x;

                    let mut skip_judge_overlap = true;
                    for y in (child_pos_y + 1)..pos_y {
                        let processing_commit_pos_x =
                            commit_pos_map.get(&commits[y].commit_hash).unwrap().0;
                        if processing_commit_pos_x == new_pos_x {
                            skip_judge_overlap = false;
                            break;
                        }
                        if edges[y]
                            .iter()
                            .filter(|e| e.edge.pos_x == pos_x)
                            .filter(|e| matches!(e.edge.edge_type, EdgeType::Vertical))
                            .any(|e| e.edge_parent_hash != hash)
                        {
                            skip_judge_overlap = false;
                            break;
                        }
                    }

                    if !skip_judge_overlap {
                        for y in (child_pos_y + 1)..pos_y {
                            let processing_commit_pos_x =
                                commit_pos_map.get(&commits[y].commit_hash).unwrap().0;
                            if processing_commit_pos_x == new_pos_x {
                                overlap = true;
                                if new_pos_x < processing_commit_pos_x + 1 {
                                    new_pos_x = processing_commit_pos_x + 1;
                                }
                            }
                            for edge in &edges[y] {
                                if edge.edge.pos_x >= new_pos_x
                                    && edge.edge_parent_hash != hash
                                    && matches!(edge.edge.edge_type, EdgeType::Vertical)
                                {
                                    overlap = true;
                                    if new_pos_x < edge.edge.pos_x + 1 {
                                        new_pos_x = edge.edge.pos_x + 1;
                                    }
                                }
                            }
                        }
                    }

                    if overlap {
                        // detour
                        edges[pos_y].push(WrappedEdge::new(EdgeType::Right, pos_x, pos_x, hash));
                        for x in (pos_x + 1)..new_pos_x {
                            edges[pos_y].push(WrappedEdge::new(
                                EdgeType::Horizontal,
                                x,
                                pos_x,
                                hash,
                            ));
                        }
                        edges[pos_y].push(WrappedEdge::new(
                            EdgeType::RightBottom,
                            new_pos_x,
                            pos_x,
                            hash,
                        ));
                        for y in ((child_pos_y + 1)..pos_y).rev() {
                            edges[y].push(WrappedEdge::new(
                                EdgeType::Vertical,
                                new_pos_x,
                                pos_x,
                                hash,
                            ));
                        }
                        edges[child_pos_y].push(WrappedEdge::new(
                            EdgeType::RightTop,
                            new_pos_x,
                            pos_x,
                            hash,
                        ));
                        for x in (child_pos_x + 1)..new_pos_x {
                            edges[child_pos_y].push(WrappedEdge::new(
                                EdgeType::Horizontal,
                                x,
                                pos_x,
                                hash,
                            ));
                        }
                        edges[child_pos_y].push(WrappedEdge::new(
                            EdgeType::Right,
                            child_pos_x,
                            pos_x,
                            hash,
                        ));

                        if max_pos_x < new_pos_x {
                            max_pos_x = new_pos_x;
                        }
                    } else {
                        edges[pos_y].push(WrappedEdge::new(EdgeType::Up, pos_x, pos_x, hash));
                        for y in ((child_pos_y + 1)..pos_y).rev() {
                            edges[y].push(WrappedEdge::new(EdgeType::Vertical, pos_x, pos_x, hash));
                        }
                        if pos_x < child_pos_x {
                            edges[child_pos_y].push(WrappedEdge::new(
                                EdgeType::LeftTop,
                                pos_x,
                                pos_x,
                                hash,
                            ));
                            for x in (pos_x + 1)..child_pos_x {
                                edges[child_pos_y].push(WrappedEdge::new(
                                    EdgeType::Horizontal,
                                    x,
                                    pos_x,
                                    hash,
                                ));
                            }
                            edges[child_pos_y].push(WrappedEdge::new(
                                EdgeType::Left,
                                child_pos_x,
                                pos_x,
                                hash,
                            ));
                        } else {
                            edges[child_pos_y].push(WrappedEdge::new(
                                EdgeType::RightTop,
                                pos_x,
                                pos_x,
                                hash,
                            ));
                            for x in (child_pos_x + 1)..pos_x {
                                edges[child_pos_y].push(WrappedEdge::new(
                                    EdgeType::Horizontal,
                                    x,
                                    pos_x,
                                    hash,
                                ));
                            }
                            edges[child_pos_y].push(WrappedEdge::new(
                                EdgeType::Right,
                                child_pos_x,
                                pos_x,
                                hash,
                            ));
                        }
                    }
                }
            }
        }

        if max_pos_x < pos_x {
            max_pos_x = pos_x;
        }
    }

    let edges: Vec<Vec<Edge>> = edges
        .into_iter()
        .map(|es| {
            let mut es: Vec<Edge> = es.into_iter().map(|e| e.edge).collect();
            es.sort_by_key(|e| (e.associated_line_pos_x, e.pos_x, e.edge_type));
            es.dedup();
            es
        })
        .collect();

    (edges, max_pos_x)
}
