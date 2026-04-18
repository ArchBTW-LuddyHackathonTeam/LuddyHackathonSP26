use anyhow::Result;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Controls the direction in which leaderboard scores are ranked.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LeaderboardSortOrder {
    /// Rank from lowest score to highest (ascending order).
    Descending,
    /// Rank from highest score to lowest (descending order).
    Ascending,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, ToSchema)]
pub struct ServerConfig {
    /// TCP port the HTTP server listens on.
    pub port: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct LeaderboardConfig {
    /// Human-readable title displayed above the leaderboard.
    pub title: String,
    /// Whether scores are sorted ascending or descending.
    pub sort_order: LeaderboardSortOrder,
}

/// Top-level application configuration (mirrors `config.toml`).
#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct Config {
    pub server: ServerConfig,
    pub leaderboard: LeaderboardConfig,
}

impl Config {
    pub async fn save(&self) -> Result<()> {
        tokio::fs::write("config.toml", toml::to_string_pretty(&self)?).await?;
        Ok(())
    }
}
