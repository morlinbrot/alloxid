use fullstack::settings::Settings;
use fullstack::{configure_app, Result};
use sqlx::postgres::PgPool;

use fullstack::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> Result<()> {

    let subscriber = get_subscriber("fullstack".into(), "debug".into());
    init_subscriber(subscriber);

    let settings = Settings::new()?;
    let address = format!("{}:{}", settings.app.host, settings.app.port);

    let db_pool = PgPool::connect(&settings.database.full_url()).await?;

    let app = configure_app(db_pool, settings).await?;

    println!("Server listening on {}", address);
    app.listen(address).await?;

    Ok(())
}
