use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};
use tabled::Tabled;
use utoipa::ToSchema;

use crate::config::LeaderboardSortOrder;

const LEADERBOARD_SIZE: i32 = 10;

/// A participant's current leaderboard entry.
#[derive(Debug, FromRow, Serialize, Tabled, ToSchema)]
pub struct Score {
    /// The participant's unique identifier.
    #[tabled(order = 0)]
    pub uploader: String,
    /// Timestamp when this score was first recorded (UTC).
    #[tabled(order = 2)]
    pub created_at: time::PrimitiveDateTime,
    /// The numeric score value.
    #[tabled[order = 1]]
    pub value: f64,
}

/// Descriptive statistics computed across all current leaderboard scores.
#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct ScoreStats {
    /// Total number of scores on the leaderboard.
    pub count: Option<i64>,
    /// Arithmetic mean of all scores.
    pub mean: Option<f64>,
    /// Median (50th percentile) score.
    pub median: Option<f64>,
    /// Lowest score.
    pub min: Option<f64>,
    /// Highest score.
    pub max: Option<f64>,
    /// Difference between the highest and lowest score.
    pub range: Option<f64>,
    /// Sample standard deviation.
    pub stddev: Option<f64>,
    /// Population standard deviation.
    pub stddev_pop: Option<f64>,
    /// Sample variance.
    pub variance: Option<f64>,
    /// 25th percentile score.
    pub p25: Option<f64>,
    /// 75th percentile score.
    pub p75: Option<f64>,
    /// Interquartile range (p75 − p25).
    pub iqr: Option<f64>,
    /// Most frequently occurring score value.
    pub mode: Option<f64>,
    /// Timestamp of the earliest score entry.
    pub earliest_at: Option<time::PrimitiveDateTime>,
    /// Timestamp of the latest score entry.
    pub latest_at: Option<time::PrimitiveDateTime>,
}

impl Score {
    /// Fetch a single score by its uploader
    pub async fn from_uploader(
        pool: &PgPool,
        uploader: &String,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as("SELECT uploader, created_at, value FROM score WHERE uploader = $1")
            .bind(uploader)
            .fetch_optional(pool)
            .await
    }

    /// Insert a new score and return the created record
    pub async fn create(pool: &PgPool, uploader: &String, value: f64) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            r#"INSERT INTO score (uploader, value, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (uploader) DO UPDATE
              SET value = EXCLUDED.value,
                  created_at = EXCLUDED.created_at
            RETURNING *"#,
        )
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
    pub async fn delete_by_uploader(pool: &PgPool, uploader: &String) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM score WHERE uploader = $1")
            .bind(uploader)
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
    }

    pub async fn leaderboard(
        pool: &PgPool,
        sort: LeaderboardSortOrder,
    ) -> Result<Vec<Score>, sqlx::Error> {
        Self::leaderboard_num(pool, LEADERBOARD_SIZE, sort).await
    }

    pub async fn leaderboard_num(
        pool: &PgPool,
        num: i32,
        sort: LeaderboardSortOrder,
    ) -> Result<Vec<Score>, sqlx::Error> {
        match sort {
            LeaderboardSortOrder::Ascending => {
                sqlx::query_as(
                    "SELECT uploader, value, created_at FROM score ORDER BY value ASC LIMIT $1",
                )
                .bind(num)
                .fetch_all(pool)
                .await
            }
            LeaderboardSortOrder::Descending => {
                sqlx::query_as(
                    "SELECT uploader, value, created_at FROM score ORDER BY value DESC LIMIT $1",
                )
                .bind(num)
                .fetch_all(pool)
                .await
            }
        }
    }

    pub async fn get_score_stats(pool: &sqlx::PgPool) -> Result<ScoreStats, sqlx::Error> {
        sqlx::query_as(
            r#"
        SELECT
            COUNT(value)                                                AS "count",
            AVG(value)                                                  AS "mean",
            PERCENTILE_CONT(0.5)  WITHIN GROUP (ORDER BY value)        AS "median",
            MIN(value)                                                  AS "min",
            MAX(value)                                                  AS "max",
            MAX(value) - MIN(value)                                     AS "range",
            STDDEV(value)                                               AS "stddev",
            STDDEV_POP(value)                                           AS "stddev_pop",
            VARIANCE(value)                                             AS "variance",
            PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY value)        AS "p25",
            PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY value)        AS "p75",
            PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY value)
                - PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY value)  AS "iqr",
            MODE()                WITHIN GROUP (ORDER BY value)        AS "mode",
            MIN(created_at)                                                 AS "earliest_at",
            MAX(created_at)                                                 AS "latest_at"
        FROM score
        "#,
        )
        .fetch_one(pool)
        .await
    }
}
