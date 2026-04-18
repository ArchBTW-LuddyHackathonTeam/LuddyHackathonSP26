use std::collections::HashMap;

use sqlx::PgPool;
use time::PrimitiveDateTime;

use crate::router::AppState;

#[derive(Clone)]
pub struct ScoreUpdate {
    pub uploader: String,
    pub value: f64,
    pub timestamp: PrimitiveDateTime,
}

pub async fn run_coalescer(state: AppState) {
    let flush_interval = tokio::time::Duration::from_millis(50);
    loop {
        tokio::time::sleep(flush_interval).await;

        if state.pending.is_empty() {
            continue;
        }

        // Drain the map atomically
        let batch: Vec<(String, f64)> = state
            .pending
            .iter()
            .map(|e| (e.key().clone(), *e.value()))
            .collect();
        state.pending.clear();

        if let Err(e) = flush_batch(&state.db, &batch).await {
            eprintln!("Flush error: {e}");
        }
    }
}

async fn flush_batch(db: &PgPool, batch: &[(String, f64)]) -> Result<(), sqlx::Error> {
    // Coalesce update for the same user
    let mut coalesced: HashMap<String, f64> = HashMap::new();

    for (uploader, value) in batch {
        coalesced.entry(uploader.clone()).or_insert(*value);
    }

    let uploaders: Vec<String> = coalesced.keys().cloned().collect();
    let values: Vec<f64> = coalesced.values().copied().collect();

    sqlx::query(
        r#"
        WITH upsert as (
            INSERT INTO score (uploader, value)
            SELECT * FROM UNNEST($1::text[], $2::float8[])
            ON CONFLICT (uploader)
            DO UPDATE SET value = EXCLUDED.value, 
                          created_at = EXCLUDED.created_at
        ),
        hist AS (
            INSERT INTO score_history (uploader, value)
            SELECT * FROM UNNEST($1::text[], $2::float8[])
        )
        SELECT 1
        "#,
    )
    .bind(uploaders)
    .bind(values)
    .execute(db)
    .await?;

    Ok(())
}
