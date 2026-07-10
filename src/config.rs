use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone)]
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

    pub fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn load_credentials(&self) -> Credentials {
        // Try config.json first
        if let Ok(content) = std::fs::read_to_string(self.credentials_path()) {
            if let Ok(creds) = serde_json::from_str::<Credentials>(&content) {
                if creds.client_id.is_some() && creds.client_secret.is_some() {
                    return creds;
                }
            }
        }
        // Fall back to env vars (dev workflow)
        let cid = std::env::var("YOUTUBE_CLIENT_ID")
            .ok()
            .filter(|s| !s.is_empty() && s != "YOUR_CLIENT_ID");
        let csecret = std::env::var("YOUTUBE_CLIENT_SECRET")
            .ok()
            .filter(|s| !s.is_empty() && s != "YOUR_CLIENT_SECRET");
        Credentials {
            client_id: cid,
            client_secret: csecret,
        }
    }

    pub fn save_credentials(&self, creds: &Credentials) -> Result<()> {
        let json = serde_json::to_string_pretty(creds)?;
        std::fs::write(self.credentials_path(), json)?;
        Ok(())
    }

    pub fn has_valid_credentials(&self) -> bool {
        let c = self.load_credentials();
        c.client_id.is_some() && c.client_secret.is_some()
    }
}
