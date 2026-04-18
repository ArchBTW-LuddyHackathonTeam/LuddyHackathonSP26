use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Serialize, Debug, FromRow)]
pub struct ScoreHistory {
    pub id: Uuid,
    pub uploader: String,
    pub created_at: time::PrimitiveDateTime,
    pub value: f64,
}

impl ScoreHistory {
    /// TODO
    pub async fn all(pool: &PgPool, count: i64, page: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as("SELECT id, uploader, created_at, value FROM score_history ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(count)
            .bind(count*(page - 1))
            .fetch_all(pool)
            .await
    }

    /// TODO
    pub async fn from_user(
        pool: &PgPool,
        user: String,
        count: i64,
        page: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as("SELECT id, uploader, created_at, value FROM score_history WHERE uploader = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(user)
            .bind(count)
            .bind(count*(page - 1))
            .fetch_all(pool)
            .await
    }

    /// Fetch a single score history entry by its id
    pub async fn from_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as("SELECT id, uploader, created_at, value FROM score_history WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    // TODO: Add paging
    /// Fetch all score histories by an uploader
    pub async fn by_uploader(pool: &PgPool, uploader: String) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, uploader, created_at, value, FROM score_history where uploader = $1",
        )
        .bind(uploader)
        .fetch_all(pool)
        .await
    }

    /// Insert a new score history entry and return the created record.
    pub async fn create(pool: &PgPool, uploader: String, value: f64) -> Result<Self, sqlx::Error> {
        sqlx::query_as("INSERT INTO score_history (uploader, value) VALUES ($1, $2) RETURNING id, uploader, value, created_at")
            .bind(uploader)
            .bind(value)
            .fetch_one(pool)
            .await
    }

    /// Removes all entries from the table
    pub async fn delete_all(pool: &PgPool) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM score_history")
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
    }
}
