mod helpers;
use helpers::{spawn_test_app, TestDb};

#[async_std::test]
async fn health_check() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/health-check";

    let res = surf::get(format!("{}{}", app.address, route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);
}
