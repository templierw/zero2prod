use config;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Deserialize)]
pub struct Settings {
    pub database: DataBaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DataBaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DataBaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(if self.require_ssl {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            })
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let env: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT");

    let config_path = std::env::current_dir()
        .expect("failed to determine cwd")
        .join("configuration");

    config::Config::builder()
        .add_source(config::File::from(config_path.join("base")).required(true))
        .add_source(config::File::from(config_path.join(env.as_str())).required(true))
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?
        .try_deserialize()
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

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment; use `local` or `production`",
                other
            )),
        }
    }
}
