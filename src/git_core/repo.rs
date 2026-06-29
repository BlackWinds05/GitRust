use git2::Repository;
use std::path::{Path, PathBuf};
use std::fs;

pub fn repo_path(data_dir: &str, owner_id: &str, repo_name: &str) -> PathBuf {
    PathBuf::from(data_dir)
        .join("repositories")
        .join(owner_id)
        .join(format!("{}.git", repo_name))
}

pub fn init_bare(path: &Path) -> Result<Repository, git2::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    Repository::init_bare(path)
}

pub fn open_bare(path: &Path) -> Result<Repository, git2::Error> {
    Repository::open_bare(path)
}

pub fn default_branch(repo: &Repository) -> String {
    match repo.head() {
        Ok(head) => head.shorthand().unwrap_or("main").to_string(),
        Err(_) => "main".to_string(),
    }
}

pub fn branches(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    let mut branch_names = Vec::new();
    for branch in repo.branches(Some(git2::BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branch_names.push(name.to_string());
        }
    }
    Ok(branch_names)
}

pub fn commit_file(
    repo: &Repository,
    branch: &str,
    file_path: &str,
    content: &[u8],
    message: &str,
    author_name: &str,
    author_email: &str,
) -> Result<git2::Oid, git2::Error> {
    let refname = format!("refs/heads/{}", branch);
    let obj = repo.revparse_single(branch)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;

    let blob_oid = repo.blob(content)?;
    let mut tb = repo.treebuilder(Some(&tree))?;
    tb.insert(file_path, blob_oid, 0o100644)?;
    let new_tree_oid = tb.write()?;
    let new_tree = repo.find_tree(new_tree_oid)?;

    let sig = git2::Signature::now(author_name, author_email)?;
    let commit_oid = repo.commit(Some(&refname), &sig, &sig, message, &new_tree, &[&commit])?;
    repo.reference(&refname, commit_oid, true, "commit via web")?;
    Ok(commit_oid)
}
