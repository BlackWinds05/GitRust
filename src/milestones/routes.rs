use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;

pub fn milestone_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/milestones", get(handlers::list))
        .route("/{owner}/{repo}/-/milestones/new", post(handlers::create))
        .route("/{owner}/{repo}/-/milestones/{id}", get(handlers::detail))
        .route("/{owner}/{repo}/-/milestones/{id}/edit", get(handlers::edit_form))
        .route("/{owner}/{repo}/-/milestones/{id}/edit", post(handlers::update))
        .route("/{owner}/{repo}/-/milestones/{id}/toggle", post(handlers::toggle))
        .route("/{owner}/{repo}/-/milestones/{id}/delete", post(handlers::delete))
        .route("/{owner}/{repo}/-/milestones/{id}/bind", post(handlers::bind_issue))
        .route("/{owner}/{repo}/-/milestones/{id}/unbind/{issue_id}", post(handlers::unbind_issue))
}
