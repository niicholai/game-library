use reqwest::Client;
use anyhow::{Result, anyhow};
use crate::models::IgdbGame;

pub struct IgdbClient {
    client: Client,
    client_id: String,
    access_token: String,
}

impl IgdbClient {
    pub fn new(client_id: String, access_token: String) -> Self {
        Self {
            client: Client::new(),
            client_id,
            access_token,
        }
    }

    // Extract common request logic to avoid duplication
    async fn make_igdb_request(&self, body: String) -> Result<Vec<IgdbGame>> {
        let response = self.client
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("IGDB API request failed: {}", response.status()));
        }

        let games: Vec<IgdbGame> = response.json().await?;
        Ok(games)
    }

    pub async fn search_games(&self, query: &str, limit: u32) -> Result<Vec<IgdbGame>> {
        let body = format!(
            r#"
            search "{}";
            fields id,name,summary,storyline,rating,first_release_date,
                   cover.url,screenshots.url,genres.name,platforms.name,
                   involved_companies.company.name,involved_companies.developer,
                   involved_companies.publisher;
            limit {};
            "#,
            query, limit
        );

        self.make_igdb_request(body).await
    }

    pub async fn get_game_by_id(&self, igdb_id: i64) -> Result<Option<IgdbGame>> {
        let body = format!(
            r#"
            fields id,name,summary,storyline,rating,first_release_date,
                   cover.url,screenshots.url,genres.name,platforms.name,
                   involved_companies.company.name,involved_companies.developer,
                   involved_companies.publisher;
            where id = {};
            "#,
            igdb_id
        );

        let mut games = self.make_igdb_request(body).await?;
        Ok(games.pop())
    }
}
