use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{
    database::Database,
    igdb_client::IgdbClient,
    models::{CreateGameRequest, UpdateGameRequest, GameListResponse, Game},
};

pub type AppState = std::sync::Arc<AppStateInner>;

pub struct AppStateInner {
    pub db: Database,
    pub igdb_client: IgdbClient,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub async fn health_check() -> Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    Json(response)
}

pub async fn create_game(
    State(state): State<AppState>,
    Json(request): Json<CreateGameRequest>,
) -> Result<Json<ApiResponse<Game>>, StatusCode> {
    match state.db.create_game(request).await {
        Ok(game) => Ok(Json(ApiResponse::success(game))),
        Err(e) => {
            tracing::error!("Failed to create game: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_games(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<GameListResponse>>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match state.db.get_games(page, per_page).await {
        Ok((games, total)) => {
            let response = GameListResponse {
                games,
                total,
                page,
                per_page,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!("Failed to get games: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_game(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Game>>, StatusCode> {
    match state.db.get_game_by_id(&id).await {
        Ok(Some(game)) => Ok(Json(ApiResponse::success(game))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get game: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn search_igdb_games(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<crate::models::IgdbGame>>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);

    match state.igdb_client.search_games(&params.q, limit).await {
        Ok(games) => Ok(Json(ApiResponse::success(games))),
        Err(e) => {
            tracing::error!("Failed to search IGDB: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn fetch_game_metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Game>>, StatusCode> {
    // First, get the game from our database
    let game = match state.db.get_game_by_id(&id).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get game: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // If the game has an IGDB ID, fetch metadata
    if let Some(igdb_id) = game.igdb_id {
        match state.igdb_client.get_game_by_id(igdb_id).await {
            Ok(Some(igdb_game)) => {
                // Update the game with metadata
                if let Err(e) = state.db.update_game_metadata(&id, &igdb_game).await {
                    tracing::error!("Failed to update game metadata: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }

                // Return the updated game
                match state.db.get_game_by_id(&id).await {
                    Ok(Some(updated_game)) => Ok(Json(ApiResponse::success(updated_game))),
                    Ok(None) => Err(StatusCode::NOT_FOUND),
                    Err(e) => {
                        tracing::error!("Failed to get updated game: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("Game not found in IGDB: {}", igdb_id);
                Err(StatusCode::NOT_FOUND)
            }
            Err(e) => {
                tracing::error!("Failed to fetch from IGDB: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // No IGDB ID, return the game as-is
        Ok(Json(ApiResponse::success(game)))
    }
}
