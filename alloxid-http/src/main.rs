use alloxid_http::settings::Settings;
use alloxid_http::{configure_app, Result};
use sqlx::postgres::PgPool;

use alloxid_http::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("alloxid".into(), "debug".into());
    init_subscriber(subscriber);

    let settings = Settings::new()?;

    let address = format!("{}:{}", settings.app.host, settings.app.port);
    let cors_url = format!("{}", settings.app.cors_url);

    let db_pool = PgPool::connect(&settings.database.full_url()).await?;
    let app = configure_app(db_pool, settings).await?;

    println!(
        "\nServer listening on {}, CORS allowed for {}",
        address, cors_url
    );
    axum::Server::bind(&address.parse().expect("Failed to parse app address."))
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
