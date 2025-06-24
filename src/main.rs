mod models;
mod database;
mod igdb_client;
mod handlers;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tracing_subscriber;

use crate::{
    database::Database,
    igdb_client::IgdbClient,
    handlers::{AppStateInner, AppState},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing - this fixes the E0425 error
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./games.db".to_string());
    let igdb_client_id = std::env::var("IGDB_CLIENT_ID")
        .expect("IGDB_CLIENT_ID must be set");
    let igdb_access_token = std::env::var("IGDB_ACCESS_TOKEN")
        .expect("IGDB_ACCESS_TOKEN must be set");
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    // Initialize database
    let db = Database::new(&database_url).await?;
    tracing::info!("Database connected successfully");

    // Initialize IGDB client
    let igdb_client = IgdbClient::new(igdb_client_id, igdb_access_token);
    tracing::info!("IGDB client initialized");

    // Create application state
    let state: AppState = Arc::new(AppStateInner {
        db,
        igdb_client,
    });

    // Build the application router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/games", get(handlers::get_games).post(handlers::create_game))
        .route("/api/games/:id", get(handlers::get_game).put(handlers::update_game).delete(handlers::delete_game))
        .route("/api/games/:id/metadata", post(handlers::fetch_game_metadata))
        .route("/api/search/igdb", get(handlers::search_igdb_games))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server starting on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
