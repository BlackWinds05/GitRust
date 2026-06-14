use axum::{routing::get, Router};
use gitrust::config::Config;
use gitrust::state::AppState;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::EnvFilter;

async fn home() -> &'static str {
    "GitRust — Code Hosting Platform"
}

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;
    tracing::info!("Database connected.");

    let session_store = MemoryStore::default();

    let state = Arc::new(AppState::new(pool, config.clone()).await?);

    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .layer(SessionManagerLayer::new(session_store))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(gitrust::middleware::logging::trace_layer())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("GitRust listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
