use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LeaderboardSortOrder {
    Descending,
    Ascending,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LeaderboardConfig {
    pub title: String,
    pub sort_order: LeaderboardSortOrder,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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
