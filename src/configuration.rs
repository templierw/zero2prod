use config;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DataBaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String
}

#[derive(Deserialize)]
pub struct DataBaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DataBaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn connection_string_no_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let env : Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT");

    let config_path = std::env::current_dir()
        .expect("failed to determine cwd")
        .join("configuration");

    config::Config::builder()
        .add_source(config::File::from(config_path.join("base")).required(true))
        .add_source(config::File::from(config_path.join(env.as_str())).required(true))
        .build()?
        .try_deserialize()
}

pub enum Environment {
    Local,
    Production
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment; use `local` or `production`", other))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string() {
        let config = get_configuration().unwrap();
        assert_eq!(
            config.database.connection_string(),
            "postgres://postgres:password@127.0.0.1:5432/newsletter"
        )
    }
}
