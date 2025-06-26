mod models;
mod database;
mod igdb_client;
mod handlers;
mod auth;
mod auth_service;
mod auth_handlers;
mod middleware;
mod user_handlers;

use axum::{
    routing::{get, post, delete},
    Router,
    middleware::from_fn_with_state,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use std::sync::Arc;
use tracing_subscriber;

use crate::{
    database::Database,
    igdb_client::IgdbClient,
    auth_service::AuthService,
    handlers::{AppStateInner, AppState},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./games.db".to_string());
    let igdb_client_id = std::env::var("IGDB_CLIENT_ID")
        .unwrap_or_else(|_| "your_client_id".to_string());
    let igdb_access_token = std::env::var("IGDB_ACCESS_TOKEN")
        .unwrap_or_else(|_| "your_access_token".to_string());
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

    // Initialize auth service
    let auth_service = AuthService::new(db.get_pool().clone());
    tracing::info!("Auth service initialized");

    // Create application state
    let state: AppState = Arc::new(AppStateInner {
        db,
        igdb_client,
        auth_service,
    });

    // Build the application router with multi-user game management
    let app = Router::new()
        // Public routes (no auth required)
        .route("/api/auth/login", post(auth_handlers::login))

        // User routes (auth required)
        .route("/api/auth/me", get(auth_handlers::me))
        .route("/api/auth/logout", post(auth_handlers::logout))
        .route("/api/store/games", get(user_handlers::get_store_games))
        .route("/api/user/library", get(user_handlers::get_user_library))
        .route("/api/user/library/{id}", get(user_handlers::get_user_game))
        .route("/api/user/games/{id}/install", post(user_handlers::install_game))
        .route("/api/user/games/{id}/uninstall", delete(user_handlers::uninstall_game))
        .route_layer(from_fn_with_state(state.clone(), middleware::auth_middleware))

        // Admin-only routes
        .route("/api/admin/users", get(auth_handlers::list_users).post(auth_handlers::create_user))
        .route("/api/admin/users/{id}", delete(auth_handlers::delete_user))
        .route("/api/admin/games", get(handlers::get_games).post(handlers::create_game))
        .route("/api/admin/games/{id}", get(handlers::get_game)) // Removed the .put(handlers::update_game)
        .route("/api/admin/games/{id}/metadata", post(handlers::fetch_game_metadata))
        .route("/api/admin/search/igdb", get(handlers::search_igdb_games))
        .route_layer(from_fn_with_state(state.clone(), middleware::admin_middleware))

        // Health check (no auth)
        .route("/health", get(handlers::health_check))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("static"));

    // Start the server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server starting on port {}", port);
    tracing::info!("Admin Web interface available at http://localhost:{}", port);
    tracing::info!("API Endpoints:");
    tracing::info!("  - POST /api/auth/login - User login");
    tracing::info!("  - GET /api/store/games - Browse available games");
    tracing::info!("  - GET /api/user/library - User's personal library");
    tracing::info!("  - POST /api/user/games/{{id}}/install - Install game");
    tracing::info!("  - Admin routes under /api/admin/*");

    axum::serve(listener, app).await?;

    Ok(())
}
