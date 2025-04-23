use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub base_rpc_url: String,
}

impl Config {
    const PATH: &str = "data/config.toml";

    async fn read_from_file(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let cfg_str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&cfg_str)?)
    }

    pub async fn read_default() -> Self {
        Self::read_from_file(Self::PATH)
            .await
            .expect("default config to be valid")
    }
}
