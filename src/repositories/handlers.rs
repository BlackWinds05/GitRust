use axum::{
    extract::{Path, Query, State},
    response::{Html, Redirect},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::git_core::{commit, diff, repo as git_repo, tree};
use crate::helpers;
use crate::markdown::render::render_markdown;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct TreeQuery { pub ref_name: Option<String>, pub path: Option<String> }

#[derive(Deserialize)]
pub struct CommitsQuery { pub ref_name: Option<String>, pub page: Option<u32> }

#[derive(Deserialize)]
pub struct CommitParams { pub owner: String, pub repo: String, pub sha: String }

#[derive(Deserialize)]
pub struct BlobParams { pub owner: String, pub repo: String, pub ref_name: String }

pub async fn overview(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _owner_name) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let entries = tree::list_tree(&git_repo_obj, &default_branch, "").unwrap_or_default();
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let readme_html = tree::find_readme(&git_repo_obj, &default_branch)
        .and_then(|(name, content)| {
            if name.to_lowercase().ends_with(".md") || name.to_lowercase().ends_with(".markdown") {
                let text = String::from_utf8_lossy(&content).to_string();
                Some(render_markdown(&text))
            } else {
                Some(format!("<pre>{}</pre>", String::from_utf8_lossy(&content)))
            }
        });
    let commit_count = commit::get_commit_count(&git_repo_obj, &default_branch).unwrap_or(0);
    let clone_url = format!("{}/{}/{}/git/", state.config.base_url, repo_info.owner_name, repo_info.name);
    let html = state.templates.render("pages/repo/overview.jinja", context! {
        current_user, repo => repo_info, default_branch, entries,
        branches, readme_html, commit_count, clone_url, sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}

pub async fn tree_view(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<TreeQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let ref_name = query.ref_name.unwrap_or(default_branch);
    let current_path = query.path.unwrap_or_default();
    let entries = tree::list_tree(&git_repo_obj, &ref_name, &current_path).unwrap_or_default();
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();

    let mut breadcrumbs: Vec<(String, String)> = Vec::new();
    if !current_path.is_empty() {
        let mut accum = String::new();
        for part in current_path.split('/') {
            if !part.is_empty() {
                if !accum.is_empty() { accum.push('/'); }
                accum.push_str(part);
                breadcrumbs.push((part.to_string(), accum.clone()));
            }
        }
    }

    let html = state.templates.render("pages/repo/tree.jinja", context! {
        current_user, repo => repo_info, current_ref => ref_name,
        entries, branches, path => current_path, breadcrumbs,
        sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}

pub async fn blob_view(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<BlobParams>, Query(query): Query<TreeQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let file_path = query.path.unwrap_or(params.ref_name.clone());
    let content_bytes = crate::git_core::blob::read_blob(&git_repo_obj, &params.ref_name, &file_path).unwrap_or_default();
    let is_bin = crate::git_core::blob::is_binary(&content_bytes);
    let content = if is_bin { "[Binary file]".to_string() } else { String::from_utf8_lossy(&content_bytes).to_string() };
    let language = crate::git_core::blob::detect_language(&file_path);
    let file_size = content_bytes.len() as i64;

    let mut breadcrumbs: Vec<(String, String)> = Vec::new();
    if !file_path.is_empty() {
        let mut accum = String::new();
        for part in file_path.split('/') {
            if !part.is_empty() {
                if !accum.is_empty() { accum.push('/'); }
                accum.push_str(part);
                breadcrumbs.push((part.to_string(), accum.clone()));
            }
        }
    }

    let html = state.templates.render("pages/repo/blob.jinja", context! {
        current_user, repo => repo_info, current_ref => params.ref_name,
        file_path, content, is_binary => is_bin,
        language, file_size, breadcrumbs, sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}

pub async fn graph(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/graph.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "graph", branches,
    }).await?;
    Ok(Html(html))
}

pub async fn graph_data(
    State(state): State<Arc<AppState>>,
    Path(params): Path<RepoParams>,
) -> AppResult<axum::response::Json<serde_json::Value>> {
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let data = crate::git_core::graph::build_graph(&git_repo_obj)?;
    Ok(axum::response::Json(serde_json::to_value(&data).unwrap_or_default()))
}

pub async fn stats(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let stats = crate::git_core::stats::compute_stats(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/stats.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "stats",
        stats,
    }).await?;
    Ok(Html(html))
}

pub async fn commits(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<CommitsQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let ref_name = query.ref_name.unwrap_or(default_branch);
    let page = query.page.unwrap_or(1);
    let (commits_list, total) = commit::list_commits(&git_repo_obj, &ref_name, page, 20).unwrap_or_default();
    let pagination = crate::helpers::pagination::Pagination::new(page, 20, total);
    let html = state.templates.render("pages/repo/commits.jinja", context! {
        current_user, repo => repo_info, current_ref => ref_name,
        commits => commits_list, pagination, sidebar_active => "commits",
    }).await?;
    Ok(Html(html))
}

pub async fn commit_detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<CommitParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let detail = commit::get_commit_detail(&git_repo_obj, &params.sha)?;
    let diffs = diff::commit_diff(&git_repo_obj, &params.sha).unwrap_or_default();
    let html = state.templates.render("pages/repo/commit.jinja", context! {
        current_user, repo => repo_info, commit => detail,
        diff_files => diffs, sidebar_active => "commits",
    }).await?;
    Ok(Html(html))
}

pub async fn branches(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let branch_list = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/branches.jinja", context! {
        current_user, repo => repo_info, branches_list => branch_list,
        sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}


#[derive(Deserialize)]
pub struct SettingsForm {
    pub description: String,
    pub is_private: Option<String>,
    pub default_branch: Option<String>,
    pub max_file_size_mb: Option<i32>,
    pub enable_notifications: Option<String>,
}

pub async fn settings_page(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let branches = git_repo::open_bare(&git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name))
        .ok().and_then(|r| git_repo::branches(&r).ok()).unwrap_or_default();
    let html = state.templates.render("pages/repo/settings.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "settings", branches,
    }).await?;
    Ok(Html(html))
}

pub async fn settings_save(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Form(form): Form<SettingsForm>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;

    let is_private = form.is_private.as_deref() == Some("on");
    let enable_notifications = form.enable_notifications.as_deref() != Some("off");
    let max_file_size_mb = form.max_file_size_mb.unwrap_or(100);

    sqlx::query(
        "UPDATE repositories SET description=$1, is_private=$2, default_branch=$3, max_file_size_mb=$4, enable_notifications=$5, updated_at=now() WHERE id=$6"
    )
    .bind(&form.description)
    .bind(is_private)
    .bind(form.default_branch.unwrap_or_else(|| "main".to_string()))
    .bind(max_file_size_mb)
    .bind(enable_notifications)
    .bind(repository.id)
    .execute(&state.pool)
    .await?;

    helpers::flash::set_flash(&session, "success", "Settings saved.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/settings", params.owner, params.repo)))
}

#[derive(Deserialize)]
pub struct RenameForm { pub new_name: String }

pub async fn settings_rename(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Form(form): Form<RenameForm>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let new_name = form.new_name.trim().to_string();
    if new_name.is_empty() || new_name.len() > 128 {
        return Err(AppError::BadRequest("Invalid repository name.".into()));
    }
    let old_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    sqlx::query("UPDATE repositories SET name=$1, updated_at=now() WHERE id=$2")
        .bind(&new_name).bind(repository.id).execute(&state.pool).await?;
    let new_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &new_name);
    std::fs::rename(&old_path, &new_path).ok();
    helpers::flash::set_flash(&session, "success", &format!("Renamed to {}.", new_name)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}", params.owner, new_name)))
}

#[derive(Deserialize)]
pub struct TransferForm { pub new_owner: String }

pub async fn settings_transfer(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Form(form): Form<TransferForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let target = sqlx::query_as::<_, (uuid::Uuid,)>("SELECT id FROM users WHERE username=$1")
        .bind(&form.new_owner).fetch_optional(&state.pool).await?
        .ok_or_else(|| AppError::NotFound("User not found.".into()))?;
    sqlx::query("INSERT INTO repository_transfers (repository_id, from_owner_type, from_owner_id, to_owner_type, to_owner_id, transferred_by) VALUES ($1,$2,$3,'user',$4,$5)")
        .bind(repository.id).bind(&repository.owner_type).bind(&repository.owner_id.to_string()).bind(target.0).bind(current_user.id)
        .execute(&state.pool).await?;
    let old_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    sqlx::query("UPDATE repositories SET owner_type='user', owner_id=$1, updated_at=now() WHERE id=$2")
        .bind(target.0).bind(repository.id).execute(&state.pool).await?;
    let new_path = git_repo::repo_path(&state.config.data_dir, &target.0.to_string(), &repository.name);
    std::fs::rename(&old_path, &new_path).ok();
    helpers::flash::set_flash(&session, "success", "Repository transferred.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}", form.new_owner, repository.name)))
}

// ── File Editing ──

#[derive(Deserialize)]
pub struct BlobQueryExtra { pub path: Option<String> }

pub async fn edit_file_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, ref_name)): Path<(String, String, String)>,
    Query(query): Query<BlobQueryExtra>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = service::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = service::get_repo_info(&state.pool, &repo).await?;
    let file_path = query.path.as_deref().unwrap_or("");
    let git_repo_obj = git_repo::open_bare(&git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name))?;
    let content = crate::git_core::blob::read_blob(&git_repo_obj, &ref_name, file_path)
        .map(|data| String::from_utf8_lossy(&data).to_string()).unwrap_or_default();
    let html = state.templates.render("pages/repo/edit_file.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "files",
        file_path, content, current_ref => ref_name,
    }).await?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct EditFileForm { pub content: String, pub commit_message: String, pub branch: String }

pub async fn edit_file_save(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, _ref_name)): Path<(String, String, String)>,
    Query(query): Query<BlobQueryExtra>, Form(form): Form<EditFileForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = service::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let file_path = query.path.as_deref().unwrap_or("");
    let git_repo_obj = git_repo::open_bare(&git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name))?;
    git_repo::commit_file(&git_repo_obj, &form.branch, file_path, form.content.as_bytes(), &form.commit_message, &current_user.username, &format!("{}@gitrust.local", current_user.username))?;
    helpers::flash::set_flash(&session, "success", "File committed.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/blob/{}/{}?path={}", owner, repo_name, form.branch, file_path, file_path)))
}

pub async fn new_file_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<BlobQueryExtra>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repo).await?;
    let dir_path = query.path.as_deref().unwrap_or("");
    let git_repo_obj = git_repo::open_bare(&git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name))?;
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let html = state.templates.render("pages/repo/new_file.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "files",
        dir_path, branches, default_branch,
    }).await?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct NewFileForm { pub file_name: String, pub content: String, pub commit_message: String, pub branch: String }

pub async fn new_file_save(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<BlobQueryExtra>, Form(form): Form<NewFileForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let dir_path = query.path.as_deref().unwrap_or("");
    let file_path = if dir_path.is_empty() { form.file_name.clone() } else { format!("{}/{}", dir_path, form.file_name) };
    let git_repo_obj = git_repo::open_bare(&git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name))?;
    git_repo::commit_file(&git_repo_obj, &form.branch, &file_path, form.content.as_bytes(), &form.commit_message, &current_user.username, &format!("{}@gitrust.local", current_user.username))?;
    helpers::flash::set_flash(&session, "success", "File created.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/blob/{}/{}", params.owner, params.repo, form.branch, file_path)))
}

pub async fn delete_repo(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;

    // Only allow owner to delete
    if repository.owner_type == "user" {
        let owner: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
            .bind(&params.owner).fetch_one(&state.pool).await?;
        if owner.0 != current_user.id {
            return Err(AppError::Forbidden("Only the owner can delete this repository.".into()));
        }
    }

    // Remove bare repo from filesystem
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    std::fs::remove_dir_all(&repo_path).ok();

    // Delete from database
    sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(repository.id).execute(&state.pool).await?;

    Ok(Redirect::to("/projects"))
}
