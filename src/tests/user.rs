use super::test_helpers::{spawn_test_app, TestDb};

use crate::{JsonBody, Token, UserData};

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
    assert_eq!(res.status(), 201);

    let body: JsonBody<Token> = res.body_json().await.unwrap();
    let token = body.data;
    dbg!(&token);
    assert!(!token.is_empty());

    let route = "/user/login";

    // Log in with legal user data.
    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&login_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<String> = res.body_json().await.unwrap();
    let token = body.data;
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

    let route = "/user/me";

    // Get user data with authentication header.
    let mut res = surf::get(format!("{}{}", app.address, &route))
        .header("Authentication", format!("{}", token))
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<UserData> = res.body_json().await.unwrap();
    let user = body.data;
    dbg!(&user);
    assert!(!user.id.is_nil());
    assert_eq!(user.username, username);

    // Receive 403: Forbidden with illegal authentication header.
    let res = surf::get(format!("{}{}", app.address, &route))
        .header("Authentication", format!("{}", "thisisnotatoken"))
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 403);

    // Fail to get user data without authentication header.
    let res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 401);
}
