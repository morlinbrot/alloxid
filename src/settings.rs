use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub app: App,
    pub database: Database,
}

#[derive(Deserialize)]
pub struct App {
    pub host: String,
    pub port: usize,
}

#[derive(Deserialize)]
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

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        config.merge(Environment::with_prefix("app"))?;

        config.try_into()
    }
}

impl Database {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    #[cfg(test)]
    pub fn url_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
