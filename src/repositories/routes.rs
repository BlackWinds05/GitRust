use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::git_http;
use super::handlers;

pub fn repo_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}", get(handlers::overview))
        .route("/{owner}/{repo}/tree", get(handlers::tree_view))
        .route("/{owner}/{repo}/commits", get(handlers::commits))
        .route("/{owner}/{repo}/commit/{sha}", get(handlers::commit_detail))
        .route("/{owner}/{repo}/blob/{ref_name}", get(handlers::blob_view))
        .route("/{owner}/{repo}/branches", get(handlers::branches))
        .route("/{owner}/{repo}/-/graph", get(handlers::graph))
        .route("/{owner}/{repo}/-/graph/data", get(handlers::graph_data))
        .route("/{owner}/{repo}/-/stats", get(handlers::stats))
        // Git Smart HTTP
        .route("/{owner}/{repo}/git/info/refs", get(git_http::info_refs))
        .route("/{owner}/{repo}/git/git-upload-pack", post(git_http::upload_pack))
        .route("/{owner}/{repo}/git/git-receive-pack", post(git_http::receive_pack))
        // Repository settings
        .route("/{owner}/{repo}/-/settings", get(handlers::settings_page))
        .route("/{owner}/{repo}/-/settings", post(handlers::settings_save))
        .route("/{owner}/{repo}/-/settings/rename", post(handlers::settings_rename))
        .route("/{owner}/{repo}/-/settings/transfer", post(handlers::settings_transfer))
        .route("/{owner}/{repo}/-/settings/delete", post(handlers::delete_repo))
        // File editing
        .route("/{owner}/{repo}/-/edit/{ref_name}", get(handlers::edit_file_form))
        .route("/{owner}/{repo}/-/edit/{ref_name}", post(handlers::edit_file_save))
        .route("/{owner}/{repo}/-/new-file", get(handlers::new_file_form))
        .route("/{owner}/{repo}/-/new-file", post(handlers::new_file_save))
}

pub fn repo_settings_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/settings", get(handlers::settings_page))
        .route("/{owner}/{repo}/-/settings", post(handlers::settings_save))
}
