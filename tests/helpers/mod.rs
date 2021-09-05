use sqlx::{Connection, Executor, PgConnection, PgPool};

use fullstack::configure_app;
use fullstack::settings::Settings;

pub struct TestApp {
    pub address: String,
    // We want to keep this alive until the end of the test.
    #[allow(dead_code)]
    test_db: TestDb,
}

#[allow(dead_code)]
pub struct TestDb {
    db_name: String,
    db_pool: PgPool,
    conn_string: String,
}

impl TestDb {
    pub async fn new(settings: &Settings) -> Self {
        let Settings { database, .. } = settings;

        let conn_string = database.conn_string();
        let full_url = database.full_url();

        // create_db(&conn_string, &db_name).await;
        let mut pg_conn = PgConnection::connect(&conn_string).await.expect("Failed to connect to Postgres.");
        pg_conn.execute(&*format!(r#"CREATE DATABASE "{}";"#, database.name)).await.expect("Failed to create database.");

        let db_pool = PgPool::connect(&full_url).await.expect("Failed to connect to database.");
        migrate_db(&db_pool).await;

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
    fn drop(&mut self) {
        async_std::task::block_on(self.db_pool.close());
        let _ = self.db_pool;
        async_std::task::block_on(drop_db(&self.conn_string, &self.db_name));
    }
}


pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);

    let settings = Settings::new_for_test().expect("Failed to load configuration.");

    let test_db = TestDb::new(&settings).await;

    let address = format!("http://{}:{}", settings.app.host, settings.app.port);

    let app = configure_app(test_db.pool(), settings).await.unwrap();

    let _ = async_std::task::spawn(app.listen(address.clone()));

    TestApp { address, test_db }
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
