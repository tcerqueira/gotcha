use std::path::PathBuf;

use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub application: ApplicationConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub host: String,
    #[serde(with = "crate::serde::as_string")]
    pub port: u16,
    pub serve_dir: PathBuf,
    pub auth_origin: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: Secret<String>,
    #[serde(with = "crate::serde::as_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn server_dir() -> PathBuf {
    std::env::var("SERVER_DIR").map_or(std::env::current_dir().unwrap(), |p| {
        std::env::current_dir().unwrap().join(p)
    })
}

pub fn get_configuration() -> Result<Config, config::ConfigError> {
    let configuration_directory = server_dir().join("config");
    tracing::debug!("Loading config from: {configuration_directory:?}");
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Config>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` or `production`.",
            )),
        }
    }
}
