use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_connection_string: String,
    pub default_driver: String,
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_connection_string: "Server=127.0.0.1;User ID=sa;Password=Xl9api20@Strong!;TrustServerCertificate=True;".to_string(),
            default_driver: "mssql".to_string(),
            port: 8080,
        }
    }
}

pub fn load_config() -> AppConfig {
    let mut config = AppConfig::default();

    // 1. Try reading from Web.config (backward compatibility with original ASP.NET project)
    if Path::new("Web.config").exists() {
        if let Ok(content) = fs::read_to_string("Web.config") {
            if let Ok(doc) = roxmltree::Document::parse(&content) {
                for node in doc.descendants() {
                    if node.has_tag_name("add") {
                        if let Some(conn_str) = node.attribute("connectionString") {
                            config.default_connection_string = conn_str.to_string();
                        }
                    }
                }
            }
        }
    }

    // 2. Override with environment variables if present
    if let Ok(env_conn) = std::env::var("CONNECTION_STRING").or_else(|_| std::env::var("DATABASE_URL")) {
        config.default_connection_string = env_conn;
    }

    if let Ok(env_driver) = std::env::var("DATABASE_DRIVER") {
        config.default_driver = env_driver;
    }

    if let Ok(env_port) = std::env::var("PORT") {
        if let Ok(port) = env_port.parse::<u16>() {
            config.port = port;
        }
    }

    config
}
