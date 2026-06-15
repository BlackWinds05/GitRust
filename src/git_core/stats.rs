use chrono::{Datelike, TimeZone, Utc};
use git2::{Repository, Sort};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, Default)]
pub struct CommitWeek {
    pub week: String,
    pub count: usize,
    pub pct: usize, // percentage of max for bar height
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ContributorStat {
    pub author_name: String,
    pub commits: usize,
    pub pct: usize, // percentage of max for bar width
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct LanguageStat {
    pub language: String,
    pub files: usize,
    pub pct: usize, // percentage of total files
}

#[derive(Debug, Serialize, Default)]
pub struct RepoStats {
    pub total_commits: u64,
    pub total_branches: usize,
    pub total_files: usize,
    pub commit_activity: Vec<CommitWeek>,
    pub contributors: Vec<ContributorStat>,
    pub languages: Vec<LanguageStat>,
}

pub fn compute_stats(repo: &Repository) -> Result<RepoStats, git2::Error> {
    let branches = super::repo::branches(repo).unwrap_or_default();

    // --- Commit activity by week ---
    let mut week_counts: HashMap<String, usize> = HashMap::new();
    let mut author_counts: HashMap<String, usize> = HashMap::new();

    // Walk all commits
    let mut revwalk = repo.revwalk()?;
    // Push all branch tips
    for branch_name in &branches {
        if let Ok(reference) = repo.find_reference(&format!("refs/heads/{}", branch_name)) {
            if let Some(oid) = reference.target() {
                revwalk.push(oid).ok();
            }
        }
    }
    // If no branches, push HEAD
    if branches.is_empty() {
        if let Ok(head) = repo.head() {
            if let Some(oid) = head.target() {
                revwalk.push(oid).ok();
            }
        }
    }
    revwalk.set_sorting(Sort::TIME)?;

    let mut total_commits: u64 = 0;

    for oid_result in &mut revwalk {
        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        total_commits += 1;

        if let Ok(commit) = repo.find_commit(oid) {
            let time = commit.time();
            let dt = Utc.timestamp_opt(time.seconds(), 0)
                .single()
                .unwrap_or_else(Utc::now);
            let iso_week = dt.iso_week();
            let week_key = format!("{}-W{:02}", iso_week.year(), iso_week.week());

            *week_counts.entry(week_key).or_insert(0) += 1;

            let author = commit.author();
            let name = author.name().unwrap_or("Unknown").to_string();
            drop(author);
            *author_counts.entry(name).or_insert(0) += 1;
        }
    }

    // Sort weeks chronologically, take last 26 (6 months)
    let mut weeks: Vec<(String, usize)> = week_counts.into_iter().collect();
    weeks.sort();
    if weeks.len() > 26 {
        weeks = weeks.split_off(weeks.len() - 26);
    }
    let max_week = weeks.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let commit_activity: Vec<CommitWeek> = weeks
        .into_iter()
        .map(|(w, c)| CommitWeek {
            week: w, count: c,
            pct: if max_week > 0 { (c * 100) / max_week } else { 0 },
        })
        .collect();

    // --- Top contributors (top 10) ---
    let mut authors: Vec<(String, usize)> = author_counts.into_iter().collect();
    authors.sort_by(|a, b| b.1.cmp(&a.1));
    authors.truncate(10);
    let max_author = authors.first().map(|a| a.1).unwrap_or(1);
    let contributors: Vec<ContributorStat> = authors
        .into_iter()
        .map(|(name, commits)| ContributorStat {
            author_name: name,
            commits,
            pct: if max_author > 0 { (commits * 100) / max_author } else { 0 },
        })
        .collect();

    // --- Language / file type breakdown ---
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    let mut total_files: usize = 0;

    // Walk the default branch tree
    let default_branch = super::repo::default_branch(repo);
    if let Ok(obj) = repo.revparse_single(&default_branch) {
        if let Ok(commit) = obj.peel_to_commit() {
            let tree = commit.tree()?;
            let mut tree_walker = Vec::new();
            tree_walker.push(("".to_string(), tree));

            while let Some((prefix, tree)) = tree_walker.pop() {
                for entry in tree.iter() {
                    let name = entry.name().unwrap_or("").to_string();
                    match entry.kind() {
                        Some(git2::ObjectType::Blob) => {
                            total_files += 1;
                            let ext = std::path::Path::new(&name)
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e.to_lowercase())
                                .unwrap_or_else(|| {
                                    if name.starts_with('.') || name.contains("Makefile") || name.contains("Dockerfile") {
                                        name.to_lowercase()
                                    } else {
                                        "other".to_string()
                                    }
                                });
                            *ext_counts.entry(ext).or_insert(0) += 1;
                        }
                        Some(git2::ObjectType::Tree) => {
                            let full_path = if prefix.is_empty() { name.clone() } else { format!("{}/{}", prefix, name) };
                            if let Ok(subtree) = repo.find_tree(entry.id()) {
                                tree_walker.push((full_path, subtree));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let mut exts: Vec<(String, usize)> = ext_counts.into_iter().collect();
    exts.sort_by(|a, b| b.1.cmp(&a.1));
    if exts.len() > 10 {
        exts.truncate(10);
    }
    let total_counted: usize = exts.iter().map(|(_, c)| c).sum();
    let languages: Vec<LanguageStat> = exts
        .into_iter()
        .map(|(ext, files)| LanguageStat {
            language: ext,
            files,
            pct: if total_counted > 0 { (files * 100) / total_counted } else { 0 },
        })
        .collect();

    Ok(RepoStats {
        total_commits,
        total_branches: branches.len(),
        total_files,
        commit_activity,
        contributors,
        languages,
    })
}
