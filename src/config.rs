use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_addr: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // In a real app, you would load from a file and/or environment variables.
        // For the MVP, we'll hardcode some defaults.
        Ok(Config {
            server_addr: "127.0.0.1:3000".to_string(),
            database_url: "sqlite:lcore.db".to_string(),
        })
    }
} 