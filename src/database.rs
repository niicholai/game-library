use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use anyhow::Result;
use chrono::{Utc, DateTime};
use uuid::Uuid;
use std::str::FromStr;
use crate::models::{Game, CreateGameRequest, UpdateGameRequest};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Database { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    // Admin-only game management methods
    pub async fn create_game(&self, request: CreateGameRequest) -> Result<Game> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let game = sqlx::query_as::<_, Game>(
            r#"
            INSERT INTO games (
                id, igdb_id, name, file_path, file_size, is_available, added_by, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
            .bind(&id)
            .bind(request.igdb_id)
            .bind(&request.name)
            .bind(&request.file_path)
            .bind(None::<i64>) // file_size
            .bind(true) // is_available
            .bind(None::<String>) // added_by
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

        Ok(game)
    }

    pub async fn get_game_by_id(&self, id: &str) -> Result<Option<Game>> {
        let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(game)
    }

    // Get all available games (for store/catalog view)
    pub async fn get_available_games(&self, page: i64, per_page: i64) -> Result<(Vec<Game>, i64)> {
        let offset = (page - 1) * per_page;

        let games = sqlx::query_as::<_, Game>(
            "SELECT * FROM games WHERE is_available = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
            .bind(true)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total = sqlx::query("SELECT COUNT(*) as count FROM games WHERE is_available = ?")
            .bind(true)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");

        Ok((games, total))
    }

    // Admin-only: Get all games including unavailable ones
    pub async fn get_games(&self, page: i64, per_page: i64) -> Result<(Vec<Game>, i64)> {
        let offset = (page - 1) * per_page;

        let games = sqlx::query_as::<_, Game>(
            "SELECT * FROM games ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total = sqlx::query("SELECT COUNT(*) as count FROM games")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");

        Ok((games, total))
    }

    #[allow(dead_code)]
    pub async fn update_game(&self, id: &str, request: UpdateGameRequest) -> Result<Option<Game>> {
        let now = Utc::now();

        if let Some(name) = request.name {
            sqlx::query("UPDATE games SET name = ?, updated_at = ? WHERE id = ?")
                .bind(name)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(file_path) = request.file_path {
            sqlx::query("UPDATE games SET file_path = ?, updated_at = ? WHERE id = ?")
                .bind(file_path)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_game_by_id(id).await
    }

    #[allow(dead_code)]
    pub async fn delete_game(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM games WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_game_metadata(&self, id: &str, igdb_game: &crate::models::IgdbGame) -> Result<()> {
        let now = Utc::now();

        let release_date = igdb_game.first_release_date.and_then(|timestamp| {
            DateTime::from_timestamp(timestamp, 0)
        });

        let cover_url = igdb_game.cover.as_ref().map(|cover| {
            format!("https:{}", cover.url.replace("t_thumb", "t_cover_big"))
        });

        let screenshots = igdb_game.screenshots.as_ref().map(|screenshots| {
            serde_json::to_string(&screenshots.iter()
                .map(|s| format!("https:{}", s.url.replace("t_thumb", "t_screenshot_med")))
                .collect::<Vec<_>>())
                .unwrap_or_default()
        });

        let genres = igdb_game.genres.as_ref().map(|genres| {
            serde_json::to_string(genres).unwrap_or_default()
        });

        let platforms = igdb_game.platforms.as_ref().map(|platforms| {
            serde_json::to_string(platforms).unwrap_or_default()
        });

        let developer = igdb_game.involved_companies.as_ref().and_then(|companies| {
            companies.iter()
                .find(|c| c.developer)
                .map(|c| c.company.name.clone())
        });

        let publisher = igdb_game.involved_companies.as_ref().and_then(|companies| {
            companies.iter()
                .find(|c| c.publisher)
                .map(|c| c.company.name.clone())
        });

        sqlx::query(
            r#"
            UPDATE games SET
                summary = ?, storyline = ?, rating = ?, release_date = ?,
                cover_url = ?, screenshots = ?, genres = ?, platforms = ?,
                developer = ?, publisher = ?, updated_at = ?
            WHERE id = ?
            "#
        )
            .bind(&igdb_game.summary)
            .bind(&igdb_game.storyline)
            .bind(igdb_game.rating)
            .bind(release_date)
            .bind(cover_url)
            .bind(screenshots)
            .bind(genres)
            .bind(platforms)
            .bind(developer)
            .bind(publisher)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // User game library methods
    pub async fn install_game_for_user(&self, user_id: &str, game_id: &str, install_path: Option<String>) -> Result<bool> {
        let now = Utc::now();
        let user_game_id = Uuid::new_v4().to_string();

        // Check if game exists and is available
        let game_exists = sqlx::query("SELECT id FROM games WHERE id = ? AND is_available = ?")
            .bind(game_id)
            .bind(true)
            .fetch_optional(&self.pool)
            .await?
            .is_some();

        if !game_exists {
            return Ok(false);
        }

        // Insert or update user_games record
        let result = sqlx::query(
            r#"
            INSERT INTO user_games (id, user_id, game_id, is_installed, install_path, installed_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, game_id) DO UPDATE SET
                is_installed = excluded.is_installed,
                install_path = excluded.install_path,
                installed_at = excluded.installed_at
            "#
        )
            .bind(&user_game_id)
            .bind(user_id)
            .bind(game_id)
            .bind(true)
            .bind(install_path)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn uninstall_game_for_user(&self, user_id: &str, game_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE user_games SET is_installed = ?, install_path = ? WHERE user_id = ? AND game_id = ?"
        )
            .bind(false)
            .bind(None::<String>)
            .bind(user_id)
            .bind(game_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_library(&self, user_id: &str, page: i64, per_page: i64) -> Result<(Vec<UserGameWithDetails>, i64)> {
        let offset = (page - 1) * per_page;

        let user_games = sqlx::query_as::<_, UserGameWithDetails>(
            r#"
            SELECT
                ug.id as user_game_id,
                ug.is_installed,
                ug.install_path,
                ug.installed_at,
                ug.last_played,
                ug.play_time_minutes,
                g.id,
                g.igdb_id,
                g.name,
                g.summary,
                g.storyline,
                g.rating,
                g.release_date,
                g.cover_url,
                g.screenshots,
                g.genres,
                g.platforms,
                g.developer,
                g.publisher,
                g.file_path,
                g.file_size,
                g.is_available,
                g.added_by,
                g.created_at,
                g.updated_at
            FROM user_games ug
            JOIN games g ON ug.game_id = g.id
            WHERE ug.user_id = ?
            ORDER BY ug.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total = sqlx::query(
            "SELECT COUNT(*) as count FROM user_games WHERE user_id = ?"
        )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");

        Ok((user_games, total))
    }

    pub async fn get_user_game(&self, user_id: &str, game_id: &str) -> Result<Option<UserGameWithDetails>> {
        let user_game = sqlx::query_as::<_, UserGameWithDetails>(
            r#"
            SELECT
                ug.id as user_game_id,
                ug.is_installed,
                ug.install_path,
                ug.installed_at,
                ug.last_played,
                ug.play_time_minutes,
                g.id,
                g.igdb_id,
                g.name,
                g.summary,
                g.storyline,
                g.rating,
                g.release_date,
                g.cover_url,
                g.screenshots,
                g.genres,
                g.platforms,
                g.developer,
                g.publisher,
                g.file_path,
                g.file_size,
                g.is_available,
                g.added_by,
                g.created_at,
                g.updated_at
            FROM user_games ug
            JOIN games g ON ug.game_id = g.id
            WHERE ug.user_id = ? AND ug.game_id = ?
            "#
        )
            .bind(user_id)
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user_game)
    }

    #[allow(dead_code)]
    pub async fn update_user_game_playtime(&self, user_id: &str, game_id: &str, additional_minutes: i64) -> Result<bool> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE user_games
            SET play_time_minutes = play_time_minutes + ?, last_played = ?
            WHERE user_id = ? AND game_id = ?
            "#
        )
            .bind(additional_minutes)
            .bind(now)
            .bind(user_id)
            .bind(game_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

// New struct for user game details
#[derive(Debug, sqlx::FromRow)]
pub struct UserGameWithDetails {
    pub user_game_id: String,
    pub is_installed: bool,
    pub install_path: Option<String>,
    pub installed_at: Option<DateTime<Utc>>,
    pub last_played: Option<DateTime<Utc>>,
    pub play_time_minutes: i64,

    // Game details (flattened from games table)
    pub id: String,
    #[allow(dead_code)]
    pub igdb_id: Option<i32>,
    pub name: String,
    pub summary: Option<String>,
    #[allow(dead_code)]
    pub storyline: Option<String>,
    #[allow(dead_code)]
    pub rating: Option<f64>,
    #[allow(dead_code)]
    pub release_date: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub cover_url: Option<String>,
    #[allow(dead_code)]
    pub screenshots: Option<String>,
    #[allow(dead_code)]
    pub genres: Option<String>,
    #[allow(dead_code)]
    pub platforms: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    #[allow(dead_code)]
    pub file_path: Option<String>,
    #[allow(dead_code)]
    pub file_size: Option<i64>,
    #[allow(dead_code)]
    pub is_available: bool,
    #[allow(dead_code)]
    pub added_by: Option<String>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
}
