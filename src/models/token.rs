use sqlx::{PgPool, prelude::FromRow};

#[derive(Debug, FromRow)]
pub struct Token {
    pub id: String,
}

impl Token {
    /// TODO
    pub async fn create(pool: &PgPool, token: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as("INSERT INTO token (id) VALUES ($1) RETURNING id")
            .bind(token)
            .fetch_one(pool)
            .await
    }

    /// TODO
    pub async fn get(pool: &PgPool, token: String) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM token WHERE id = $1")
            .bind(token)
            .fetch_optional(pool)
            .await
    }

    /// TODO
    pub async fn exists(pool: &PgPool, token: String) -> Result<bool, sqlx::Error> {
        sqlx::query("SELECT * FROM token WHERE id = $1")
            .bind(token)
            .fetch_optional(pool)
            .await
            .map(|r| r.is_some())
    }

    /// TODO
    pub async fn any_exists(pool: &PgPool) -> Result<bool, sqlx::Error> {
        sqlx::query("SELECT * FROM token")
            .fetch_optional(pool)
            .await
            .map(|r| r.is_some())
    }

    /// TODO
    pub async fn clear(pool: &PgPool) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM token")
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
    }
}

