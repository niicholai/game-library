use sqlx::{SqlitePool, Row};
use anyhow::Result;
use chrono::{DateTime, Utc}; // Added DateTime import
use uuid::Uuid;
use crate::models::{Game, CreateGameRequest, UpdateGameRequest};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn create_game(&self, request: CreateGameRequest) -> Result<Game> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let game = sqlx::query_as::<_, Game>(
            r#"
            INSERT INTO games (
                id, igdb_id, name, file_path, is_installed, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
            .bind(&id)
            .bind(request.igdb_id)
            .bind(&request.name)
            .bind(request.file_path)
            .bind(false)
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

    pub async fn update_game(&self, id: &str, request: UpdateGameRequest) -> Result<Option<Game>> {
        let now = Utc::now();

        // Simplified update approach - update each field individually if provided
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

        if let Some(is_installed) = request.is_installed {
            sqlx::query("UPDATE games SET is_installed = ?, updated_at = ? WHERE id = ?")
                .bind(is_installed)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_game_by_id(id).await
    }

    pub async fn delete_game(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM games WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_game_metadata(&self, id: &str, igdb_game: &crate::models::IgdbGame) -> Result<()> {
        let now = Utc::now();

        let release_date = igdb_game.first_release_date
            .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0));

        let cover_url = igdb_game.cover.as_ref()
            .map(|cover| format!("https:{}", cover.url.replace("t_thumb", "t_cover_big")));

        let screenshots = igdb_game.screenshots.as_ref()
            .map(|screenshots| {
                serde_json::to_string(&screenshots.iter()
                    .map(|s| format!("https:{}", s.url.replace("t_thumb", "t_screenshot_med")))
                    .collect::<Vec<_>>())
                    .unwrap_or_default()
            });

        let genres = igdb_game.genres.as_ref()
            .map(|genres| serde_json::to_string(genres).unwrap_or_default());

        let platforms = igdb_game.platforms.as_ref()
            .map(|platforms| serde_json::to_string(platforms).unwrap_or_default());

        // Fixed tuple destructuring
        let developer = igdb_game.involved_companies.as_ref()
            .and_then(|companies| companies.iter()
                .find(|c| c.developer)
                .map(|c| c.company.name.clone()));

        let publisher = igdb_game.involved_companies.as_ref()
            .and_then(|companies| companies.iter()
                .find(|c| c.publisher)
                .map(|c| c.company.name.clone()));

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
}
