use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};
use tabled::Tabled;

use crate::config::LeaderboardSortOrder;

const LEADERBOARD_SIZE: i32 = 10;

#[derive(Debug, FromRow, Serialize, Tabled)]
pub struct Score {
    #[tabled(order = 0)]
    pub uploader: String,
    #[tabled(order = 2)]
    pub created_at: time::PrimitiveDateTime,
    #[tabled[order = 1]]
    pub value: f64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ScoreStats {
    pub count: Option<i64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub range: Option<f64>,
    pub stddev: Option<f64>,
    pub stddev_pop: Option<f64>,
    pub variance: Option<f64>,
    pub p25: Option<f64>,
    pub p75: Option<f64>,
    pub iqr: Option<f64>,
    pub mode: Option<f64>,
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
            MODE()                WITHIN GROUP (ORDER BY value)        AS "mode"
        FROM score
        "#,
        )
        .fetch_one(pool)
        .await
    }
}
