mod test_helpers;
use test_helpers::spawn_test_app;

#[async_std::test]
async fn it_works() {
    let app = spawn_test_app().await;

    let address = &format!("http://{}/health-check", app.address);
    dbg!(address.clone());

    let res = surf::get(address)
        .await
        .expect("Failed to execute GET request at /health-check.");

    assert_eq!(res.status(), 200);
}
