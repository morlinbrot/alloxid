use super::test_helpers::{spawn_test_app, TestApp, TestDb};
use serde::{Deserialize, Serialize};

use crate::{JsonBody, Token, UserData};

#[derive(Deserialize, Serialize)]
struct TestUser {
    username: &'static str,
    password: &'static str,
}

async fn create_user(app: &TestApp) -> (surf::Response, TestUser) {
    let route = "/user";

    let user_data = TestUser {
        username: "synul",
        password: "my-pw",
    };
    let json = serde_json::json!(&user_data);

    (
        surf::post(format!("{}{}", app.address, &route))
            .body(http_types::Body::from_json(&json).unwrap())
            .await
            .expect("Failed to create user."),
        user_data,
    )
}

#[async_std::test]
async fn create_user_and_login() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    // Create a user.
    let (mut res, user_data) = create_user(&app).await;

    dbg!(&res);
    assert_eq!(res.status(), 201);

    let body: JsonBody<Token> = res.body_json().await.unwrap();
    let token = body.data;
    dbg!(&token);
    assert!(!token.is_empty());

    let route = "/user/login";

    // Log in with legal user data.
    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&user_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<String> = res.body_json().await.unwrap();
    let token = body.data;
    dbg!(&token);
    assert!(!token.is_empty());

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
    assert_eq!(user.username, user_data.username);
}

#[async_std::test]
async fn login_with_wrong_pw_returns_401() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let _ = create_user(&app).await;

    let route = "/user/login";

    // Fail to log in with illegal user data.
    let wrong_data = serde_json::json!({ "username": "synul", "password": "wrong-pw"});
    let res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&wrong_data).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 401);
}

#[async_std::test]
async fn get_me_without_token_returns_401() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let _ = create_user(&app).await;

    let route = "/user/me";

    let res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 401);
}

#[async_std::test]
async fn get_me_with_illegal_token_returns_403() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let _ = create_user(&app).await;

    let route = "/user/me";

    let res = surf::get(format!("{}{}", app.address, &route))
        .header("Authentication", format!("{}", "thisisnotatoken"))
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 403);
}

fn _user_already_taken() {
    todo!()
}

fn _wrong_username() {
    todo!()
}
