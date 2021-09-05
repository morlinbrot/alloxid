mod helpers;
use helpers::spawn_test_app;

#[async_std::test]
async fn health_check() {
    let app = spawn_test_app().await;

    let route = "/health-check";

    let res = surf::get(format!("{}{}", app.address, route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);
}
