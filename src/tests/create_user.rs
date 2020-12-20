use crate::tests::test_helpers::{spawn_test_app, TestDb};

use crate::User;

#[async_std::test]
async fn create_user() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/user";

    let username = "synul";
    let json = serde_json::json!({ "username": username, "password": "my-pw" });

    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&json).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);

    let user: User = res.body_json().await.unwrap();
    dbg!(&user);

    assert_eq!(res.status(), 200);
    assert!(!user.id.is_nil());
    assert_eq!(user.username, username);
}
