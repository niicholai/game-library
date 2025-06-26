use axum::{
    extract::{Extension, State, Path, Query},
    http::StatusCode,
    response::Json,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use crate::{
    auth::User,
    handlers::{AppState, ApiResponse, PaginationQuery},
    database::UserGameWithDetails,
};

#[derive(Deserialize)]
pub struct InstallGameRequest {
    pub install_path: Option<String>,
}

#[derive(Serialize)]
pub struct UserLibraryResponse {
    pub games: Vec<UserGameResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Serialize)]
pub struct UserGameResponse {
    pub user_game_id: String,
    pub is_installed: bool,
    pub install_path: Option<String>,
    pub installed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub play_time_minutes: i64,
    pub game: GameSummary,
}

#[derive(Serialize)]
pub struct GameSummary {
    pub id: String,
    pub name: String,
    pub summary: Option<String>,
    pub rating: Option<f64>,
    pub cover_url: Option<String>,
    pub genres: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
}

impl From<UserGameWithDetails> for UserGameResponse {
    fn from(user_game: UserGameWithDetails) -> Self {
        Self {
            user_game_id: user_game.user_game_id,
            is_installed: user_game.is_installed,
            install_path: user_game.install_path,
            installed_at: user_game.installed_at,
            last_played: user_game.last_played,
            play_time_minutes: user_game.play_time_minutes,
            game: GameSummary {
                id: user_game.id,
                name: user_game.name,
                summary: user_game.summary,
                rating: user_game.rating,
                cover_url: user_game.cover_url,
                genres: user_game.genres,
                developer: user_game.developer,
                publisher: user_game.publisher,
            },
        }
    }
}

// Get available games (store catalog)
#[debug_handler]
#[allow(unused_variables)]
pub async fn get_store_games(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<crate::models::GameListResponse>>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match state.db.get_available_games(page, per_page).await {
        Ok((games, total)) => {
            let response = crate::models::GameListResponse {
                games,
                total,
                page,
                per_page,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!("Failed to get store games: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Install game to user's library
#[debug_handler]
#[allow(unused_variables)]
pub async fn install_game(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(game_id): Path<String>,
    Json(request): Json<InstallGameRequest>,
) -> Result<StatusCode, StatusCode> {
    match state.db.install_game_for_user(&user.id, &game_id, request.install_path).await {
        Ok(true) => Ok(StatusCode::CREATED),
        Ok(false) => Err(StatusCode::NOT_FOUND), // Game doesn't exist or isn't available
        Err(e) => {
            tracing::error!("Failed to install game: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Uninstall game from user's library
#[debug_handler]
#[allow(unused_variables)]
pub async fn uninstall_game(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(game_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match state.db.uninstall_game_for_user(&user.id, &game_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to uninstall game: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get user's personal library
#[debug_handler]
#[allow(unused_variables)]
pub async fn get_user_library(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<UserLibraryResponse>>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match state.db.get_user_library(&user.id, page, per_page).await {
        Ok((user_games, total)) => {
            let games: Vec<UserGameResponse> = user_games.into_iter().map(|ug| ug.into()).collect();
            let response = UserLibraryResponse {
                games,
                total,
                page,
                per_page,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!("Failed to get user library: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get specific game in user's library
#[debug_handler]
#[allow(unused_variables)]
pub async fn get_user_game(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(game_id): Path<String>,
) -> Result<Json<ApiResponse<UserGameResponse>>, StatusCode> {
    match state.db.get_user_game(&user.id, &game_id).await {
        Ok(Some(user_game)) => Ok(Json(ApiResponse::success(user_game.into()))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get user game: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
