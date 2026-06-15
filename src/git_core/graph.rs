use chrono::TimeZone;
use git2::{Repository, Sort};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct GraphNode {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author_name: String,
    pub formatted_time: String,
    pub col: usize,
    pub row: usize,
    pub branches: Vec<String>,
    pub parent_shas: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub from_sha: String,
    pub to_sha: String,
    pub from_col: usize,
    pub to_col: usize,
    pub from_row: usize,
    pub to_row: usize,
}

#[derive(Debug, Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub branches: Vec<String>,
}

pub fn build_graph(repo: &Repository) -> Result<GraphData, git2::Error> {
    let branches = super::repo::branches(repo).unwrap_or_default();

    // Collect branch tip OIDs
    let mut tip_oids: Vec<git2::Oid> = Vec::new();
    for branch_name in &branches {
        if let Ok(reference) = repo.find_reference(&format!("refs/heads/{}", branch_name)) {
            if let Some(oid) = reference.target() {
                tip_oids.push(oid);
            }
        }
    }

    if tip_oids.is_empty() {
        return Ok(GraphData { nodes: vec![], edges: vec![], branches });
    }

    // Walk all commits from all branch tips, newest first
    let mut revwalk = repo.revwalk()?;
    for oid in &tip_oids {
        revwalk.push(*oid)?;
    }
    revwalk.set_sorting(Sort::TIME)?;

    let mut ordered_oids: Vec<git2::Oid> = Vec::new();
    for oid in &mut revwalk {
        if let Ok(oid) = oid {
            ordered_oids.push(oid);
        }
    }
    if ordered_oids.len() > 100 {
        ordered_oids.truncate(100);
    }

    // Map branch name -> oid
    let mut branch_map: HashMap<git2::Oid, Vec<String>> = HashMap::new();
    for branch_name in &branches {
        if let Ok(reference) = repo.find_reference(&format!("refs/heads/{}", branch_name)) {
            if let Some(oid) = reference.target() {
                branch_map.entry(oid).or_default().push(branch_name.clone());
            }
        }
    }

    // Read commit metadata: oid -> (GraphNode without position, parent oids)
    struct RawCommit { node: GraphNode, parent_oids: Vec<git2::Oid> }
    let mut raw_map: HashMap<git2::Oid, RawCommit> = HashMap::new();

    for oid in &ordered_oids {
        if let Ok(commit) = repo.find_commit(*oid) {
            let time = commit.time();
            let dt = chrono::Utc.timestamp_opt(time.seconds(), 0)
                .single().unwrap_or_else(|| chrono::Utc::now());
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            drop(author);
            let msg = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
            let parent_shas: Vec<String> = commit.parent_ids().map(|p| p.to_string()).collect();
            let parent_oids: Vec<git2::Oid> = commit.parent_ids().collect();

            raw_map.insert(*oid, RawCommit {
                node: GraphNode {
                    sha: oid.to_string(),
                    short_sha: oid.to_string().chars().take(8).collect(),
                    message: msg,
                    author_name,
                    formatted_time: super::commit::format_time(dt),
                    col: 0,
                    row: 0,
                    branches: branch_map.get(oid).cloned().unwrap_or_default(),
                    parent_shas,
                },
                parent_oids,
            });
        }
    }

    // Assign lanes (cols) — process newest first, reserve lane for first parent
    let mut lane_of: HashMap<git2::Oid, usize> = HashMap::new();
    let mut next_lane: usize = 0;

    for oid in &ordered_oids {
        // If this commit already has a lane (assigned by its child), use it; otherwise take next free
        let my_lane = *lane_of.entry(*oid).or_insert_with(|| {
            let l = next_lane;
            next_lane += 1;
            l
        });

        // Reserve the same lane for the first parent (keeps straight lines)
        if let Some(raw) = raw_map.get(oid) {
            if let Some(first_parent) = raw.parent_oids.first() {
                lane_of.entry(*first_parent).or_insert(my_lane);
            }
            // Other parents will get their own lanes when we reach them
        }
    }

    // Assign rows (reverse chronological: newest = row 0)
    let row_of: HashMap<git2::Oid, usize> = ordered_oids.iter()
        .enumerate()
        .map(|(i, oid)| (*oid, i))
        .collect();

    // Build final node list
    let mut nodes: Vec<GraphNode> = Vec::new();
    for oid in &ordered_oids {
        if let Some(raw) = raw_map.remove(oid) {
            let mut node = raw.node;
            node.col = lane_of.get(oid).copied().unwrap_or(0);
            node.row = row_of.get(oid).copied().unwrap_or(0);
            nodes.push(node);
        }
    }

    // Build edges: child -> parent
    let node_by_sha: HashMap<&str, &GraphNode> = nodes.iter()
        .map(|n| (n.sha.as_str(), n))
        .collect();

    let mut edges: Vec<GraphEdge> = Vec::new();
    for node in &nodes {
        for parent_sha in &node.parent_shas {
            if let Some(parent) = node_by_sha.get(parent_sha.as_str()) {
                edges.push(GraphEdge {
                    from_sha: node.sha.clone(),
                    to_sha: parent_sha.clone(),
                    from_col: node.col,
                    to_col: parent.col,
                    from_row: node.row,
                    to_row: parent.row,
                });
            }
        }
    }

    Ok(GraphData { nodes, edges, branches })
}
