use config::{Config, ConfigError, File};
use dotenv;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub app: App,
    pub database: Database,
}

#[derive(Clone, Debug, Deserialize)]
pub struct App {
    pub host: String,
    pub port: usize,
    pub secret: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub name: String,
    pub password: String,
    pub port: usize,
    pub username: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        config.merge(File::with_name("config/default"))?;

        #[cfg(test)]
        config.merge(File::with_name("config/test"))?;

        // Load .env file into environment, if present.
        dotenv::dotenv().expect("Failed to load .env file");

        // Set the app secret from environment. (Unfortunately `config` doesn't support
        // setting vars into nested parts of the config, e.g. `Settings.app`.
        let secret = std::env::var("APP_SECRET").expect("APP_SECRET must be set.");
        config.set("app.secret", secret)?;

        config.try_into()
    }

    pub fn new_for_test() -> Result<Self, ConfigError> {
        let mut settings = Settings::new()?;

        let db_name = format!("{}-{}", settings.database.name, Uuid::new_v4().to_string());
        settings.database.name = db_name;

        Ok(settings)
    }
}

impl Database {
    pub fn conn_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }

    pub fn full_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
