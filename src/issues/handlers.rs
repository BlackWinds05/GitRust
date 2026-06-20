use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::helpers;
use crate::issues::{model::Label, service};
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IssueRepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct IssueListQuery {
    pub state: Option<String>, pub page: Option<u32>,
    pub milestone: Option<String>, pub label: Option<String>,
    pub assignee: Option<String>, pub author: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateIssueForm {
    pub title: String, pub description: String,
    pub milestone_id: Option<String>, pub due_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_csv_or_vec")]
    pub label_ids: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_csv_or_vec")]
    pub assignee_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateIssueForm {
    pub title: String, pub description: String,
    pub milestone_id: Option<String>, pub due_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_csv_or_vec")]
    pub label_ids: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_csv_or_vec")]
    pub assignee_ids: Vec<String>,
}

fn deserialize_csv_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where D: serde::Deserializer<'de> {
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;
    struct CsvOrVec;
    impl<'de> Visitor<'de> for CsvOrVec {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or sequence of strings")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            if v.is_empty() { Ok(vec![]) }
            else { Ok(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()) }
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut v = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                if !s.is_empty() { v.push(s); }
            }
            Ok(v)
        }
    }
    deserializer.deserialize_any(CsvOrVec)
}

#[derive(Deserialize)]
pub struct CommentForm { pub body: String }

fn parse_uuid_list(raw: &[String]) -> Vec<Uuid> {
    raw.iter().filter_map(|s| Uuid::parse_str(s).ok()).collect()
}
fn parse_optional_uuid(s: &Option<String>) -> Option<Uuid> {
    s.as_ref().and_then(|v| Uuid::parse_str(v).ok())
}
fn parse_optional_date(s: &Option<String>) -> Option<chrono::NaiveDate> {
    s.as_ref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
}
pub async fn list(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>, Query(query): Query<IssueListQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = query.page.unwrap_or(1);
    let state_filter = query.state.as_deref();
    let (issues, pagination) = service::list_issues(
        &state.pool, repo.id, state_filter, page, 20,
        query.milestone.as_deref(), query.label.as_deref(),
        query.assignee.as_deref(), query.author.as_deref(),
        query.search.as_deref(),
    ).await?;
    let all_milestones = crate::milestones::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let all_labels = crate::labels::service::list_labels(&state.pool, repo.id).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/issues/list.jinja", context! {
        current_user, repo => repo_info, issues, pagination,
        current_state => state_filter.unwrap_or("open"),
        sidebar_active => "issues", flash_message,
        all_milestones, all_labels,
    }).await?;
    Ok(Html(html))
}

pub async fn new_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let labels: Vec<Label> = sqlx::query_as("SELECT * FROM issue_labels WHERE repository_id = $1 ORDER BY name")
        .bind(repo.id).fetch_all(&state.pool).await.unwrap_or_default();
    let milestones = crate::milestones::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let members = crate::members::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/issues/new.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "issues",
        all_labels => labels, milestones, members,
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>, Form(form): Form<CreateIssueForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let milestone_id = parse_optional_uuid(&form.milestone_id);
    let due_date = parse_optional_date(&form.due_date);
    let label_ids = parse_uuid_list(&form.label_ids);
    let assignee_ids = parse_uuid_list(&form.assignee_ids);
    let issue = service::create_issue(&state.pool, repo.id, current_user.id, &form.title, &form.description, milestone_id, &label_ids, &assignee_ids, due_date).await?;
    helpers::flash::set_flash(&session, "success", &format!("Issue #{} created.", issue.number)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", params.owner, params.repo, issue.number)))
}

pub async fn detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let flash_message = helpers::flash::get_flash(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let issue = service::get_issue(&state.pool, repo.id, number).await?;
    let labels: Vec<Label> = sqlx::query_as("SELECT * FROM issue_labels WHERE repository_id = $1 ORDER BY name")
        .bind(repo.id).fetch_all(&state.pool).await.unwrap_or_default();
    let comments = service::list_comments(&state.pool, issue.id).await.unwrap_or_default();
    let milestones = crate::milestones::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let members = crate::members::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/issues/detail.jinja", context! {
        current_user, repo => repo_info, issue, all_labels => labels,
        sidebar_active => "issues", comments, milestones, members,
        flash_message,
    }).await?;
    Ok(Html(html))
}

pub async fn edit_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let issue = service::get_issue(&state.pool, repo.id, number).await?;
    let labels: Vec<Label> = sqlx::query_as("SELECT * FROM issue_labels WHERE repository_id = $1 ORDER BY name")
        .bind(repo.id).fetch_all(&state.pool).await.unwrap_or_default();
    let milestones = crate::milestones::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let members = crate::members::service::list(&state.pool, repo.id).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/issues/edit.jinja", context! {
        current_user, repo => repo_info, issue, sidebar_active => "issues",
        all_labels => labels, milestones, members,
    }).await?;
    Ok(Html(html))
}

pub async fn update(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Form(form): Form<UpdateIssueForm>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let milestone_id = parse_optional_uuid(&form.milestone_id);
    let due_date = parse_optional_date(&form.due_date);
    let label_ids = parse_uuid_list(&form.label_ids);
    let assignee_ids = parse_uuid_list(&form.assignee_ids);
    service::update_issue(&state.pool, repo.id, number, &form.title, &form.description, milestone_id, &assignee_ids, &label_ids, due_date).await?;
    helpers::flash::set_flash(&session, "success", &format!("Issue #{} updated.", number)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", owner, repo_name, number)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::delete_issue(&state.pool, repo.id, number).await?;
    helpers::flash::set_flash(&session, "success", "Issue deleted.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues", owner, repo_name)))
}

pub async fn close(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<impl IntoResponse> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::close_issue(&state.pool, repo.id, number).await?;
    helpers::flash::set_flash(&session, "success", &format!("Issue #{} closed.", number)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", owner, repo_name, number)))
}

pub async fn reopen(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    service::reopen_issue(&state.pool, repo.id, number).await?;
    helpers::flash::set_flash(&session, "success", &format!("Issue #{} reopened.", number)).await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", owner, repo_name, number)))
}

pub async fn comment_create(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Form(form): Form<CommentForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let issue = service::get_issue(&state.pool, repo.id, number).await?;
    service::create_comment(&state.pool, issue.id, current_user.id, &form.body).await?;
    helpers::flash::set_flash(&session, "success", "Comment added.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", owner, repo_name, number)))
}

pub async fn comment_delete(
    State(state): State<Arc<AppState>>, session: Session,
    Path((owner, repo_name, number, comment_id)): Path<(String, String, i32, Uuid)>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &owner, &repo_name).await?;
    let issue = service::get_issue(&state.pool, repo.id, number).await?;
    service::delete_comment(&state.pool, comment_id, issue.id).await?;
    helpers::flash::set_flash(&session, "success", "Comment deleted.").await.ok();
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", owner, repo_name, number)))
}
