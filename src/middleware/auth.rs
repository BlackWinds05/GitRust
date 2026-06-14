use axum::{
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;
use tower_sessions::Session;

use crate::state::AppState;

pub async fn require_auth(
    session: Session,
    State(_state): State<Arc<AppState>>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    match session.get_value("user_id").await {
        Ok(Some(_user_id)) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => {
            let response = Redirect::to("/auth/login").into_response();
            Ok(response)
        }
    }
}
