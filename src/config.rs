use anyhow::Result;
use std::path::PathBuf;

pub struct AppConfig {
    pub config_dir: PathBuf,
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tubesub");

        std::fs::create_dir_all(&config_dir)?;

        Ok(Self { config_dir })
    }
}
