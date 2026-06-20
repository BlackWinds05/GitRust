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
use crate::git_core::{diff, repo as git_repo};
use crate::merge_requests::service;
use crate::middleware::auth::current_user_from_session;
use crate::helpers;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct MRRepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct MRListQuery { pub state: Option<String>, pub page: Option<u32> }

#[derive(Deserialize)]
pub struct MRNumber { pub number: i32 }

#[derive(Deserialize)]
pub struct CreateMRForm {
    pub title: String,
    pub description: String,
    pub source_branch: String,
    pub target_branch: String,
}

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<MRRepoParams>, Query(query): Query<MRListQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = query.page.unwrap_or(1);
    let state_filter = query.state.as_deref();
    let (mrs, pagination) = service::list_mrs(&state.pool, repo.id, state_filter, page, 20).await?;
    let html = state.templates.render("pages/repo/merge_requests/list.jinja", context! {
        current_user, repo => repo_info, merge_requests => mrs, pagination,
        current_state => state_filter.unwrap_or("open"), sidebar_active => "merge_requests",
    }).await?;
    Ok(Html(html))
}

pub async fn new_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<MRRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/merge_requests/new.jinja", context! {
        current_user, repo => repo_info, branches, sidebar_active => "merge_requests",
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<MRRepoParams>, Form(form): Form<CreateMRForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let mr = service::create_mr(&state.pool, repo.id, current_user.id,
        &form.title, &form.description, &form.source_branch, &form.target_branch).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/merge_requests/{}", params.owner, params.repo, mr.number)))
}

pub async fn detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let mr = service::get_mr(&state.pool, repo.id, number).await?;

    let has_conflicts = service::check_conflict(
        &state.config.data_dir,
        &repo.owner_id.to_string(),
        &repo.name,
        &mr.source_branch,
        &mr.target_branch,
    ).unwrap_or(false);

    if has_conflicts {
        service::update_merge_status(&state.pool, mr.id, "conflict").await?;
    }

    let flash_message = helpers::flash::get_flash(&session).await;

    let repo_path = git_repo::repo_path(&state.config.data_dir, &repo.owner_id.to_string(), &repo.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let diff_files = diff::compare_branches(&git_repo_obj, &mr.target_branch, &mr.source_branch).unwrap_or_default();
    let html = state.templates.render("pages/repo/merge_requests/detail.jinja", context! {
        current_user, repo => repo_info, mr, diff_files, sidebar_active => "merge_requests",
        has_conflicts, flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn close(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::close_mr(&state.pool, repo.id, number).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/merge_requests/{}", owner, repo_name, number)))
}

pub async fn merge(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::merge_mr(&state.pool, repo.id, number, current_user.id).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/merge_requests/{}", owner, repo_name, number)))
}
