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

fn error_html(status: u16, title: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{} - GitRust</title>
<link rel="stylesheet" href="/static/css/app.css?v=1">
</head>
<body style="background:var(--color-bg);color:var(--color-text-primary);font-family:var(--font-sans);">
<div class="auth-page" style="min-height:60vh;">
<div class="auth-card" style="text-align:center;max-width:500px;">
<h1 style="font-size:48px;color:var(--color-text-secondary);margin:0;">{}</h1>
<h1 style="font-size:24px;margin:8px 0;">{}</h1>
<p style="color:var(--color-text-secondary);margin-bottom:20px;">{}</p>
<a href="/" style="color:var(--color-text-link);">Back to Home</a>
</div>
</div>
</body>
</html>"#,
        title,
        match status {
            400 => "&#9888;",
            401 => "&#128274;",
            403 => "&#128683;",
            404 => "&#128269;",
            409 => "&#9888;",
            _ => "&#128165;",
        },
        title,
        message
    )
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status();
        let msg = self.message();
        let title = match status_code.as_u16() {
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            409 => "Conflict",
            _ => "Internal Server Error",
        };
        (
            status_code,
            axum::response::Html(error_html(status_code.as_u16(), title, &msg)),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
