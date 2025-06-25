use crate::auth::{User, Session, CreateUserRequest, LoginRequest, AuthError};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::Result;

pub struct AuthService {
    pool: SqlitePool,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AuthError> {
        // Check if username already exists
        let existing_user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
            .bind(&request.username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        if existing_user.is_some() {
            return Err(AuthError::UsernameExists);
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|_| AuthError::InternalError)?;

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Insert new user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
            .bind(&user_id)
            .bind(&request.username)
            .bind(&password_hash)
            .bind(&request.email)
            .bind(request.is_admin)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<(User, Session), AuthError> {
        // Find user by username
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
            .bind(&request.username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !verify(&request.password, &user.password_hash)
            .map_err(|_| AuthError::InternalError)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Create session
        let session = self.create_session(&user.id).await?;

        Ok((user, session))
    }

    pub async fn create_session(&self, user_id: &str) -> Result<Session, AuthError> {
        let session_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // 24 hour sessions

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, user_id, token, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
            .bind(&session_id)
            .bind(user_id)
            .bind(&token)
            .bind(expires_at)
            .bind(now)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(session)
    }

    pub async fn validate_session(&self, token: &str) -> Result<User, AuthError> {
        let now = Utc::now();

        // Find valid session
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE token = ? AND expires_at > ?"
        )
            .bind(token)
            .bind(now)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?
            .ok_or(AuthError::SessionExpired)?;

        // Get user
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
            .bind(&session.user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
    }

    #[allow(dead_code)]
    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn cleanup_expired_sessions(&self) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(())
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AuthError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(users)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::InternalError)?;

        Ok(())
    }
}
