use axum::{extract::{Path, Query, State}, response::{Html, Redirect}, Form};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::helpers::{self, slug::slugify};
use crate::markdown::render::render_markdown;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;
use crate::wiki::service;

#[derive(Deserialize)] pub struct WRepoParams { pub owner: String, pub repo: String }
#[derive(Deserialize)] pub struct WikiForm { pub title: String, pub content: String }
#[derive(Deserialize)] pub struct SearchQuery { pub q: String }

pub async fn home(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<WRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let pages = service::list_pages(&state.pool, repo.id).await?;
    let home_page = service::get_page(&state.pool, repo.id, "home").await?;
    let html = state.templates.render("pages/repo/wiki/home.jinja", context! {
        current_user, repo => repo_info, pages, home_page, sidebar_active => "wiki",
        flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn show(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, slug)): Path<(String, String, String)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = service::get_page(&state.pool, repo.id, &slug).await?;
    let content_html = page.as_ref().map(|p| render_markdown(&p.content));
    let html = state.templates.render("pages/repo/wiki/page.jinja", context! {
        current_user, repo => repo_info, page, content_html, slug, sidebar_active => "wiki",
        flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn edit_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, slug)): Path<(String, String, String)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = service::get_page(&state.pool, repo.id, &slug).await?;
    let html = state.templates.render("pages/repo/wiki/edit.jinja", context! {
        current_user, repo => repo_info, page, slug, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}

pub async fn save(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, _slug)): Path<(String, String, String)>, Form(form): Form<WikiForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let new_slug = slugify(&form.title);
    service::save_page(&state.pool, repo.id, current_user.id, &form.title, &new_slug, &form.content).await?;
    helpers::flash::set_flash(&session, "success", "Page saved.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/wiki/{}", owner, repo_name, new_slug)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, slug)): Path<(String, String, String)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::delete_page(&state.pool, repo.id, &slug).await?;
    helpers::flash::set_flash(&session, "success", "Page deleted.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/wiki", owner, repo_name)))
}

pub async fn history(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, slug)): Path<(String, String, String)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let revisions = service::get_revisions(&state.pool, repo.id, &slug).await?;
    let html = state.templates.render("pages/repo/wiki/history.jinja", context! {
        current_user, repo => repo_info, revisions, slug, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}

pub async fn rollback(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, slug, revision)): Path<(String, String, String, i32)>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let revs = service::get_revisions(&state.pool, repo.id, &slug).await?;
    let old = revs.into_iter().find(|r| r.revision == revision)
        .ok_or(AppError::NotFound("Revision not found.".into()))?;
    service::save_page(&state.pool, repo.id, current_user.id, &old.title, &slug, &old.content).await?;
    helpers::flash::set_flash(&session, "success", &format!("Rolled back to revision {}.", revision)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/wiki/{}", owner, repo_name, slug)))
}

pub async fn search(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<WRepoParams>, Query(q): Query<SearchQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let results = service::search_pages(&state.pool, repo.id, &q.q).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/wiki/search_results.jinja", context! {
        current_user, repo => repo_info, results, query => q.q, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}
