use config::{Config, ConfigBuilder, Environment, File};
use log::info;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub(crate) struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub shutdown_timeout: u64,
}

// #[derive(Debug, Deserialize)]
// struct CorsConfig {
//     allowed_origin: String,
//     allowed_methods: Vec<String>,
//     allowed_headers: Vec<String>,
//     supports_credentials: bool,
//     max_age: u32,
// }

#[derive(Debug, Deserialize)]
pub(crate) struct AppConfig {
    pub server: ServerConfig,
    // cors: CorsConfig,
}

impl AppConfig {
    /// Load configuration from file and environment variables using `ConfigBuilder`.
    pub(crate) fn from_env() -> Result<Self, config::ConfigError> {
        // Get the config file path from the environment or default to "config.toml"
        let config_path = env::var("CONFIG_FILE_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());
        info!("Loading configuration from: {}", config_path);

        // Build the configuration
        let builder: ConfigBuilder<_> = Config::builder()
            // Load the configuration from the file
            .add_source(File::with_name(&config_path))
            // Load environment variables with prefix APP and separator '__'
            .add_source(Environment::with_prefix("APP").separator("__"));

        // Try to build and deserialize into the AppConfig struct
        builder.build()?.try_deserialize()
    }
}
