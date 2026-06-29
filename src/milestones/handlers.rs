use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::helpers;
use crate::milestones::service;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)] pub struct MRepoParams { pub owner: String, pub repo: String }
#[derive(Deserialize)] pub struct MilestoneForm { pub title: String, pub description: Option<String>, pub due_date: Option<String> }

fn parse_due(d: &Option<String>) -> Option<chrono::NaiveDate> {
    d.as_ref().and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
}

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<MRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let milestones = service::list(&state.pool, repo.id).await?;
    let html = state.templates.render("pages/repo/milestones/list.jinja", context! {
        current_user, repo => repo_info, milestones, sidebar_active => "milestones",
        flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<MRepoParams>,
    Form(form): Form<MilestoneForm>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let due = parse_due(&form.due_date);
    service::create(&state.pool, repo.id, &form.title, form.description.as_deref(), due).await?;
    helpers::flash::set_flash(&session, "success", "Milestone created.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones", params.owner, params.repo)))
}

pub async fn edit_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let milestone = service::get(&state.pool, repo.id, id).await?;
    let html = state.templates.render("pages/repo/milestones/edit.jinja", context! {
        current_user, repo => repo_info, milestone, sidebar_active => "milestones",
    }).await?;
    Ok(Html(html))
}

pub async fn update(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>, Form(form): Form<MilestoneForm>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let due = parse_due(&form.due_date);
    service::update_milestone(&state.pool, id, repo.id, &form.title, form.description.as_deref(), due).await?;
    helpers::flash::set_flash(&session, "success", "Milestone updated.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones", owner, repo_name)))
}

pub async fn toggle(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::toggle_milestone(&state.pool, id, repo.id).await?;
    helpers::flash::set_flash(&session, "success", "Milestone state toggled.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones", owner, repo_name)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::delete_milestone(&state.pool, id, repo.id).await?;
    helpers::flash::set_flash(&session, "success", "Milestone deleted.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones", owner, repo_name)))
}

pub async fn detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let milestone = service::get(&state.pool, repo.id, id).await?;
    let issues = service::get_issues_for_milestone(&state.pool, id).await.unwrap_or_default();
    let unbound_issues: Vec<crate::issues::model::IssueListItem> = sqlx::query_as::<_, crate::issues::model::IssueListItem>(
        r#"SELECT i.id, i.number, i.title, i.state,
                  u.username as author_username, u.display_name as author_display_name,
                  i.created_at,
                  (SELECT COUNT(*) FROM issue_label_assignments WHERE issue_id = i.id) as label_count
           FROM issues i JOIN users u ON i.author_id = u.id
           WHERE i.repository_id = $1 AND i.state = 'open' AND i.milestone_id IS NULL
           ORDER BY i.created_at DESC LIMIT 100"#,
    ).bind(repo.id).fetch_all(&state.pool).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/milestones/detail.jinja", context! {
        current_user, repo => repo_info, milestone, issues, unbound_issues,
        sidebar_active => "milestones", flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn bind_issue(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id)): Path<(String, String, Uuid)>,
    Form(form): Form<BindIssueForm>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let milestone = service::get(&state.pool, repo.id, id).await?;
    sqlx::query("UPDATE issues SET milestone_id=$1 WHERE id=$2 AND repository_id=$3")
        .bind(milestone.id).bind(form.issue_id).bind(repo.id).execute(&state.pool).await?;
    helpers::flash::set_flash(&session, "success", "Issue bound to milestone.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones/{}", owner, repo_name, id)))
}

pub async fn unbind_issue(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, id, issue_id)): Path<(String, String, Uuid, Uuid)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    sqlx::query("UPDATE issues SET milestone_id=NULL WHERE id=$1 AND repository_id=$2")
        .bind(issue_id).bind(repo.id).execute(&state.pool).await?;
    helpers::flash::set_flash(&session, "success", "Issue unbound from milestone.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/milestones/{}", owner, repo_name, id)))
}

#[derive(Deserialize)]
pub struct BindIssueForm { pub issue_id: Uuid }
