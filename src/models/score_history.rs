use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Debug, FromRow, ToSchema)]
pub struct ScoreHistory {
    /// Unique identifier for this history entry.
    pub id: Uuid,
    /// The participant's identifier.
    pub uploader: String,
    /// Timestamp when this score was submitted (UTC).
    pub created_at: time::PrimitiveDateTime,
    /// The score value that was submitted.
    pub value: f64,
}

impl ScoreHistory {
    pub async fn query(
        pool: &PgPool,
        title: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
        count: i64,
        page: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, uploader, created_at, value FROM score_history \
             WHERE ($1::text IS NULL OR uploader = $1) \
             AND ($2::timestamp IS NULL OR created_at >= $2::timestamp) \
             AND ($3::timestamp IS NULL OR created_at <= $3::timestamp) \
             ORDER BY created_at DESC LIMIT $4 OFFSET $5",
        )
        .bind(title)
        .bind(start)
        .bind(end)
        .bind(count)
        .bind(count * (page - 1))
        .fetch_all(pool)
        .await
    }

    pub async fn all(pool: &PgPool, count: i64, page: i64) -> Result<Vec<Self>, sqlx::Error> {
        Self::query(pool, None, None, None, count, page).await
    }

    pub async fn from_user(
        pool: &PgPool,
        user: String,
        count: i64,
        page: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        Self::query(pool, Some(&user), None, None, count, page).await
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
            "SELECT id, uploader, created_at, value FROM score_history WHERE uploader = $1",
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
