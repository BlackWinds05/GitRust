use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error("Git error: {0}")]
    Git(String),
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        AppError::Git(format!("Git operation failed: {}", e))
    }
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Git(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::NotFound(msg) => msg.clone(),
            AppError::Forbidden(msg) => msg.clone(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Unauthorized => "Please log in to continue.".into(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::Internal(e) => format!("Internal server error: {}", e),
            AppError::Database(e) => format!("Database error: {}", e),
            AppError::Template(e) => format!("Template error: {}", e),
            AppError::Git(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = self.message();

        (status, format!("{} - {}", status.as_u16(), message)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
