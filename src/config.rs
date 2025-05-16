use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

pub fn load_config() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("config"))
        .build()?;

    settings.try_deserialize()
}

pub fn get_config() -> Settings {
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("Using default configuration");
            Settings {
                server: ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                },
                database: DatabaseConfig {
                    path: "data/constants.csv".to_string(),
                },
            }
        }
    };
    return config;
}