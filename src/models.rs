use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Game {
    pub id: String,
    pub igdb_id: Option<i64>,
    pub name: String,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub rating: Option<f64>,
    pub release_date: Option<DateTime<Utc>>,
    pub cover_url: Option<String>,
    pub screenshots: Option<String>, // JSON array as string
    pub genres: Option<String>, // JSON array as string
    pub platforms: Option<String>, // JSON array as string
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub is_installed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub igdb_id: Option<i64>,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGameRequest {
    pub name: Option<String>,
    pub file_path: Option<String>,
    pub is_installed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameListResponse {
    pub games: Vec<Game>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

// IGDB API Response structures - Added Serialize trait
#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub rating: Option<f64>,
    pub first_release_date: Option<i64>,
    pub cover: Option<IgdbCover>,
    pub screenshots: Option<Vec<IgdbScreenshot>>,
    pub genres: Option<Vec<IgdbGenre>>,
    pub platforms: Option<Vec<IgdbPlatform>>,
    pub involved_companies: Option<Vec<IgdbInvolvedCompany>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbCover {
    pub id: i64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbScreenshot {
    pub id: i64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbGenre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbPlatform {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbInvolvedCompany {
    pub company: IgdbCompany,
    pub developer: bool,
    pub publisher: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbCompany {
    pub id: i64,
    pub name: String,
}
