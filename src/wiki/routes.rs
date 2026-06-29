use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;

pub fn wiki_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/wiki", get(handlers::home))
        .route("/{owner}/{repo}/-/wiki/search", get(handlers::search))
        .route("/{owner}/{repo}/-/wiki/{slug}", get(handlers::show))
        .route("/{owner}/{repo}/-/wiki/{slug}/edit", get(handlers::edit_form))
        .route("/{owner}/{repo}/-/wiki/{slug}/edit", post(handlers::save))
        .route("/{owner}/{repo}/-/wiki/{slug}/delete", post(handlers::delete))
        .route("/{owner}/{repo}/-/wiki/{slug}/history", get(handlers::history))
        .route("/{owner}/{repo}/-/wiki/{slug}/rollback/{revision}", post(handlers::rollback))
}
