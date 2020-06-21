use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct Configuration {
    pub db_url: String,
    pub host: String,
    pub port: usize,
}

pub fn get_config() -> Configuration {
    dotenv().expect("Couldn't load .env file.");

    #[cfg(not(test))]
    let db_url = env::var("DATABASE_URL").expect("Couldn't find DATABASE_URL.");

    #[cfg(test)]
    let db_url = env::var("DATABASE_URL_TEST").expect("Couldn't find DATABASE_URL_TEST.");

    let host = env::var("HOST").unwrap();
    let port: usize = env::var("PORT").unwrap().parse().unwrap();

    Configuration { db_url, port, host }
}
