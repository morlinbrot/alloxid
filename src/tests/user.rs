use super::test_helpers::{spawn_test_app, TestDb};

use crate::UserCreationData;

#[async_std::test]
async fn create_user_and_login() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/user";

    let username = "synul";
    let login_data = serde_json::json!({ "username": username, "password": "my-pw" });

    // Create a user.
    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&login_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let user: UserCreationData = res.body_json().await.unwrap();
    dbg!(&user);
    assert!(!user.id.is_nil());
    assert!(!user.token.is_empty());
    assert_eq!(user.username, username);

    let route = "/user/login";

    // Log in with legal user data.
    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&login_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let token: String = res.body_json().await.unwrap();
    dbg!(&token);
    assert!(!token.is_empty());

    // Fail to log in with illegal user data.
    let wrong_data = serde_json::json!({ "username": username, "password": "wrong-pw"});
    let res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&wrong_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 401);
}
