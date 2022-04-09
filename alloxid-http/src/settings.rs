use config::{Config, ConfigError, File};
use dotenv;
use names::Generator;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub app: App,
    pub database: Database,
}

#[derive(Clone, Debug, Deserialize)]
pub struct App {
    pub cors_url: String,
    pub host: String,
    pub port: usize,
    pub(crate) secret: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub name: String,
    password: String,
    pub port: usize,
    username: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        let mut cfg_path = std::env::current_dir().expect("Failed to read cwd");

        // We don't know if we're being run from the workspace or the crate root.
        let crate_root = Path::new("alloxid-http");
        if !cfg_path.ends_with(&crate_root) {
            cfg_path = Path::new(&cfg_path).join(crate_root);
        }

        config.merge(File::from(cfg_path.join("config/prod")))?;

        #[cfg(debug_assertions)]
        config.merge(File::from(cfg_path.join("config/dev")))?;

        #[cfg(test)]
        config.merge(File::from(cfg_path.join("config/test")))?;

        // Load .env file into environment, if present.
        dotenv::dotenv().expect("Failed to load .env file");

        // Set the app secret from environment. (Unfortunately `config` doesn't support
        // setting vars into nested parts of the config, e.g. `Settings.app`.
        let secret = std::env::var("APP_SECRET").expect("Failed to read APP_SECRET");
        config.set("app.secret", secret)?;

        let db_pw = std::env::var("DATABASE_PASSWORD").expect("Failed to read DATABASE_PASSWORD");
        config.set("database.password", db_pw)?;

        config.try_into()
    }

    pub fn new_for_test() -> Result<Self, ConfigError> {
        let mut settings = Settings::new()?;

        let mut generator = Generator::default();
        let name = generator
            .next()
            .expect("Failed to generate random db name.");
        let db_name = format!("{}-{}", settings.database.name, name);
        settings.database.name = db_name;

        let mut rng = thread_rng();
        let port = rng.gen_range(8080..9000);
        settings.app.port = port;

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
