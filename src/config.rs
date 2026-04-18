use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Config {
    pub secret: Uuid,
}

impl Config {
    pub async fn save(&self) -> Result<()> {
        tokio::fs::write("Config.toml", toml::to_string_pretty(&self)?).await?;
        Ok(())
    }
}
