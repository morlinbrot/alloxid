use sqlx::{Connection, Executor, PgConnection, PgPool};
use once_cell::sync::Lazy;
use tracing::{debug, instrument, trace};

use fullstack::configure_app;
use fullstack::settings::Settings;
use fullstack::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("fullstack-test".into(), "info,sqlx=warn".into());
    init_subscriber(subscriber);
});

pub struct TestApp {
    pub address: String,
    // We want to keep this alive until the end of the test.
    #[allow(dead_code)]
    pub test_db: TestDb,
    pub port: usize,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TestDb {
    pub db_name: String,
    db_pool: PgPool,
    conn_string: String,
}

impl TestDb {
    #[instrument(level = "debug", skip(settings))]
    pub async fn new(settings: &Settings) -> Self {
        let Settings { database, .. } = settings;

        let conn_string = database.conn_string();
        let full_url = database.full_url();

        // create_db(&conn_string, &db_name).await;
        let mut pg_conn = PgConnection::connect(&conn_string).await.expect("Failed to connect to Postgres.");
        pg_conn.execute(&*format!(r#"CREATE DATABASE "{}";"#, database.name)).await.expect("Failed to create database.");

        let db_pool = PgPool::connect(&full_url).await.expect("Failed to connect to database.");
        migrate_db(&db_pool).await;

        debug!("Created & migrated new TestDb: {}", &settings.database.name());
        Self {
            db_name: database.name(),
            db_pool,
            conn_string,
        }
    }

    pub fn pool(&self) -> PgPool {
        self.db_pool.clone()
    }
}

impl Drop for TestDb {
    #[instrument(level = "debug", skip(self))]
    fn drop(&mut self) {
        trace!("Blocking to close pool for TestDb: {}", &self.db_name);
        async_std::task::block_on(self.db_pool.close());
        trace!("Closed pool for TestDb: {}", &self.db_name);
        let _ = self.db_pool;
        trace!("Blocking to drop TestDb: {}", &self.db_name);
        async_std::task::block_on(drop_db(&self.conn_string, &self.db_name));
        debug!("Closed pool & dropped TestDb: {}", &self.db_name);
    }
}


#[instrument(level = "debug")]
pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);

    let settings = Settings::new_for_test().expect("Failed to load configuration.");

    let test_db = TestDb::new(&settings).await;

    let port = settings.app.port;
    let address = format!("http://{}:{}", settings.app.host, port);

    let app = configure_app(test_db.pool(), settings).await.unwrap();

    let _ = async_std::task::spawn(app.listen(address.clone()));

    debug!("TestApp listening on {} with DB {}", &address, &test_db.db_name);
    TestApp { address, test_db, port }
}

// async fn create_db(pg_conn: &str, db_name: &str) {
//     let mut conn = PgConnection::connect(pg_conn)
//         .await
//         .expect("Failed to connect to Postgres.");

//     conn.execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
//         .await
//         .expect("Failed to create database.");
// }

async fn migrate_db(db_pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(db_pool)
        .await
        .expect("Failed to migrate the database");
}

#[allow(dead_code)]
async fn drop_db(conn_string: &str, db_name: &str) {
    let mut conn = PgConnection::connect(conn_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Disconnect any existing connections to the DB
    conn.execute(&*format!(
        r#"
        SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{}'
        AND pid <> pg_backend_pid();
        "#,
        db_name
    ))
    .await
    .expect("Failed to drop existing connections to database.");

    conn.execute(&*format!(r#"DROP DATABASE "{}";"#, db_name))
        .await
        .expect("Failed to drop database.");
}
