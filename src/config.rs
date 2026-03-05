use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cookie_secure: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cookie_secure: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub log_file_path: String,
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_file_path: "app.log".to_string(),
            level: "info".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub log: LogConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let server = ServerConfig {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true")
                .unwrap_or(false),
        };

        let log = LogConfig {
            log_file_path: env::var("LOG_FILE").unwrap_or_else(|_| "app.log".to_string()),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        };

        Ok(AppConfig { server, log })
    }
}
