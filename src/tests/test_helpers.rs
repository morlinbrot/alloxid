use crate::configure_app;
use crate::settings::Settings;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
}

#[allow(dead_code)]
pub struct TestDb {
    db_name: String,
    pg_conn: String,
    pool: PgPool,
}

pub async fn spawn_test_app(db_pool: PgPool) -> TestApp {
    let Settings { app, .. } = Settings::new().expect("Failed to load configuration.");

    let address = format!("http://{}:{}", app.host, app.port);

    let app = configure_app(db_pool).await.unwrap();

    let _ = async_std::task::spawn(app.listen(address.clone()));
    TestApp { address }
}

impl TestDb {
    pub async fn new() -> Self {
        let Settings { database, .. } = Settings::new().expect("Failed to load configuration.");

        let pg_conn = database.url_without_db();
        let db_name = format!("{}-{}", database.name, Uuid::new_v4().to_string());
        let full_url = format!("{}/{}", pg_conn, db_name);

        create_db(&pg_conn, &db_name).await;

        let pool = PgPool::connect(&full_url)
            .await
            .expect("Failed to connect to database.");

        migrate_db(&pool).await;

        Self {
            db_name,
            pg_conn,
            pool,
        }
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}

//impl Drop for TestDb {
//    fn drop(&mut self) {
//        futures::executor::block_on(self.pool.close());
//        //let _ = self.pool;
//        futures::executor::block_on(drop_db(&self.pg_conn, &self.db_name));
//    }
//}

async fn create_db(pg_conn: &str, db_name: &str) {
    let mut conn = PgConnection::connect(pg_conn)
        .await
        .expect("Failed to connect to Postgres.");

    conn.execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
        .await
        .expect("Failed to create database.");
}

async fn migrate_db(db_pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(db_pool)
        .await
        .expect("Failed to migrate the database");
}

#[allow(dead_code)]
async fn drop_db(pg_conn: &str, db_name: &str) {
    let mut conn = PgConnection::connect(pg_conn)
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
