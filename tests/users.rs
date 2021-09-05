use serde::{Deserialize, Serialize};

use fullstack::{JsonBody, UserCreationData, UserData};

mod helpers;
use helpers::{spawn_test_app, TestApp};

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
    let app = spawn_test_app().await;

    // Create a user.
    let (mut res, user_data) = create_user(&app).await;

    dbg!(&res.status());
    assert_eq!(res.status(), 201);
    dbg!(&res.header("Location"));
    assert!(res.header("Location").is_some());

    let body: JsonBody<UserCreationData> = res.body_json().await.unwrap();
    let user = body.data;
    dbg!(&user);
    assert!(!user.id.is_nil());
    assert!(!user.token.is_empty());

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

    let route = format!("/user/{}", user.id);

    // Get user data with authentication header.
    let mut res = surf::get(format!("{}{}", app.address, &route))
        .header("Authorization", format!("Bearer {}", token))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<UserData> = res.body_json().await.unwrap();
    let user = body.data;
    dbg!(&user);
    assert!(!user.id.is_nil());
    assert_eq!(user.username, user_data.username);
}

// #[async_std::test]
// async fn login_with_illegal_data_returns_401() {
//     let app = spawn_test_app().await;

//     let _ = create_user(&app).await;

//     let route = "/user/login";

//     // Wrong user and wrong pw should throw the same error to not give existing usernames away.
//     let wrong_data = serde_json::json!({ "username": "synul", "password": "wrong-pw"});
//     let res = surf::post(format!("{}{}", app.address, &route))
//         .body(http_types::Body::from_json(&wrong_data).unwrap())
//         .await
//         .expect(&format!("Failed to execute POST request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);

//     let wrong_data = serde_json::json!({ "username": "wrong-user", "password": "wrong-pw"});
//     let res = surf::post(format!("{}{}", app.address, &route))
//         .body(http_types::Body::from_json(&wrong_data).unwrap())
//         .await
//         .expect(&format!("Failed to execute POST request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }

// #[async_std::test]
// async fn get_user_without_token_returns_401() {
//     let app = spawn_test_app().await;

//     let (mut res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.body_json().await.unwrap();
//     let user = body.data;

//     let route = format!("/user/{}", user.id);

//     let res = surf::get(format!("{}{}", app.address, &route))
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }

// #[async_std::test]
// async fn get_user_with_malformed_token_returns_401() {
//     let app = spawn_test_app().await;

//     let (mut res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.body_json().await.unwrap();
//     let user = body.data;

//     let route = format!("/user/{}", user.id);

//     let res = surf::get(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("{}", "thisisnotatoken"))
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }

// // #[allow(dead_code)]
// // #[async_std::test]
// // async fn get_user_with_illegal_token_returns_403() {
// //     todo!()
// // }

#[async_std::test]
async fn put_user_data_returns_200() {
    let app = spawn_test_app().await;

    let (mut res, _) = create_user(&app).await;
    let body: JsonBody<UserCreationData> = res.body_json().await.unwrap();
    let user = body.data;
    let token = user.token;

    let route = format!("/user/{}", user.id);

    let new_username = "my-new-username";
    let json = serde_json::json!({ "username": new_username });

    let mut res = surf::put(format!("{}{}", app.address, &route))
        .header("Authorization", format!("Bearer {}", token))
        .body(http_types::Body::from_json(&json).unwrap())
        .await
        .expect(&format!("Failed to execute PUT request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<UserData> = res.body_json().await.unwrap();
    let user = body.data;
    dbg!(&user);
    assert_eq!(user.username, new_username);
}

// #[async_std::test]
// async fn delete_user_returns_200_then_403() {
//     let app = spawn_test_app().await;

//     let (mut res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.body_json().await.unwrap();
//     let user = body.data;
//     let token = user.token;

//     let route = format!("/user/{}", user.id);

//     let res = surf::delete(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("Bearer {}", token))
//         .await
//         .expect(&format!("Failed to execute DELETE request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 200);

//     // Trying to retrieve the user after deletion should return 403.
//     let res = surf::get(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("Bearer {}", token))
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 403);
// }
