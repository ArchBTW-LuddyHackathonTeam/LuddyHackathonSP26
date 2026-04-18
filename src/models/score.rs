use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};

#[derive(Debug, FromRow, Serialize)]
pub struct Score {
    pub uploader: String,
    pub created_at: time::PrimitiveDateTime,
    pub value: f64,
}

impl Score {
    /// Fetch a single score by its uploader
    pub async fn from_uploader(
        pool: &PgPool,
        uploader: String,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as("SELECT uploader, created_at, value FROM score WHERE uploader = $1")
            .bind(uploader)
            .fetch_optional(pool)
            .await
    }

    /// Insert a new score and return the created record
    pub async fn create(pool: &PgPool, uploader: String, value: f64) -> Result<Self, sqlx::Error> {
        sqlx::query_as("INSERT INTO score (uploader, value) VALUES ($1, $2) RETURNING uploader, created_at, value")
                .bind(uploader)
                .bind(value)
                .fetch_one(pool)
                .await
    }

    /// Delete this score. Returns the number of rows affected.
    pub async fn delete(&self, pool: &PgPool) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM score WHERE uploader = $1")
            .bind(&self.uploader)
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
    }

    /// Directly delete a record by its uploader. Returns the number of rows affected.
    pub async fn delete_by_uploader(pool: &PgPool, uploader: String) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM score WHERE uploader = $1")
            .bind(uploader)
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
    }
}
