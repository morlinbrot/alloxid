use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use fullstack::model::user::{UserCreationData, UserData};
use fullstack::JsonBody;

mod helpers;
use helpers::{spawn_test_app, TestApp};

#[derive(Deserialize, Serialize)]
struct TestUser {
    username: &'static str,
    password: &'static str,
}

async fn create_user(app: &TestApp) -> (reqwest::Response, TestUser) {
    let route = "/user";

    let user_data = TestUser {
        username: "synul",
        password: "my-pw",
    };
    let json = serde_json::json!(user_data);

    let client = reqwest::Client::new();
    let req = client
        .post(format!("{}{}", app.address, &route))
        .json(&json);

    (
        req.send()
            .await
            .expect("Failed to send create user request."),
        user_data,
    )
}

#[instrument]
#[tokio::test]
async fn create_user_and_login() {
    let app = spawn_test_app().await;
    info!(
        "create_user_and_login: app_port={} db_name={}",
        &app.port, &app.test_db.db_name
    );

    // Create a user.
    let (res, user_data) = create_user(&app).await;

    // dbg!(&res.status());
    assert_eq!(res.status(), 201);
    // dbg!(&res.headers().get("Location"));
    assert!(res.headers().get("Location").is_some());

    let body: JsonBody<UserCreationData> = res.json().await.unwrap();
    let user = body.data;
    dbg!(&user);
    assert!(!user.id.is_nil());
    assert!(!user.token.is_empty());

    let route = "/user/login";

    let client = reqwest::Client::new();
    // Log in with legal user data.
    let res = client
        .post(format!("{}{}", app.address, &route))
        .json(&user_data)
        .send()
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let body: JsonBody<String> = res.json().await.unwrap();
    let token = body.data;
    dbg!(&token);
    assert!(!token.is_empty());

    // let route = format!("/user/{}", user.id);
    //
    // // Get user data with authentication header.
    // let res = client
    //     .get(format!("{}{}", app.address, &route))
    //     .header("Authorization", format!("Bearer {}", token))
    //     .send()
    //     .await
    //     .expect(&format!("Failed to execute GET request at {}", &route));
    // dbg!(&res);
    // assert_eq!(res.status(), 200);
    //
    // let body: JsonBody<UserData> = res.json().await.unwrap();
    // let user = body.data;
    // dbg!(&user);
    // assert!(!user.id.is_nil());
    // assert_eq!(user.username, user_data.username);
}

#[instrument]
#[tokio::test]
async fn create_user_with_malformatted_data_returns_400() {
    let app = spawn_test_app().await;
    info!(
        "create_user_and_login: app_port={} db_name={}",
        &app.port, &app.test_db.db_name
    );

    let route = "/user";

    let json = serde_json::json!({
        "foo": "bar",
    });

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}{}", app.address, &route))
        .json(&json)
        .send()
        .await
        .expect("Failed to send create user request.");

    // dbg!(&res.status());
    assert_eq!(res.status(), 400);
}

// #[instrument]
// // #[ignore]
// #[tokio::test]
// async fn login_with_illegal_data_returns_401() {
//     let app = spawn_test_app().await;
//     info!(
//         "login_with_illegal_data_returns_401: app_port={} db_name={}",
//         &app.port, &app.test_db.db_name
//     );
//
//     let client = reqwest::Client::new();
//     let _ = create_user(&app).await;
//
//     let route = "/user/login";
//
//     // Wrong user and wrong pw should throw the same error to not give existing usernames away.
//     let wrong_data = serde_json::json!({ "username": "synul", "password": "wrong-pw"});
//     let res = client
//         .post(format!("{}{}", app.address, &route))
//         .json(&wrong_data)
//         .send()
//         .await
//         .expect(&format!("Failed to execute POST request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
//
//     let wrong_data = serde_json::json!({ "username": "wrong-user", "password": "wrong-pw"});
//     let res = client
//         .post(format!("{}{}", app.address, &route))
//         .json(&wrong_data)
//         .send()
//         .await
//         .expect(&format!("Failed to execute POST request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }
//
// #[instrument]
// // #[ignore]
// #[tokio::test]
// async fn get_user_without_token_returns_401() {
//     let app = spawn_test_app().await;
//     info!(
//         "get_user_without_token_returns_401: app_port={} db_name={}",
//         &app.port, &app.test_db.db_name
//     );
//
//     let (res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.json().await.unwrap();
//     let user = body.data;
//
//     let route = format!("/user/{}", user.id);
//
//     let res = reqwest::get(format!("{}{}", app.address, &route))
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }
//
// #[instrument]
// // #[ignore]
// #[tokio::test]
// async fn get_user_with_malformed_token_returns_401() {
//     let app = spawn_test_app().await;
//     info!(
//         "get_user_with_malformed_token_returns_401: app_port={} db_name={}",
//         &app.port, &app.test_db.db_name
//     );
//
//     let (res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.json().await.unwrap();
//     let user = body.data;
//
//     let route = format!("/user/{}", user.id);
//
//     let client = reqwest::Client::new();
//     let res = client
//         .get(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("{}", "thisisnotatoken"))
//         .send()
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 401);
// }
//
// // #[allow(dead_code)]
// // #[tokio::test]
// // async fn get_user_with_illegal_token_returns_403() {
// //     todo!()
// // }
//
// #[instrument]
// // #[ignore]
// #[tokio::test]
// async fn put_user_data_returns_200() {
//     let app = spawn_test_app().await;
//     info!(
//         "put_user_data_returns_200: app_port={} db_name={}",
//         &app.port, &app.test_db.db_name
//     );
//
//     let (res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.json().await.unwrap();
//     let user = body.data;
//     let token = user.token;
//
//     let route = format!("/user/{}", user.id);
//
//     let new_username = "my-new-username";
//     let json = serde_json::json!({ "username": new_username });
//
//     let client = reqwest::Client::new();
//     let res = client
//         .put(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("Bearer {}", token))
//         .json(&json)
//         .send()
//         .await
//         .expect(&format!("Failed to execute PUT request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 200);
//
//     let body: JsonBody<UserData> = res.json().await.unwrap();
//     let user = body.data;
//     dbg!(&user);
//     assert_eq!(user.username, new_username);
// }
//
// #[instrument]
// // #[ignore]
// #[tokio::test]
// async fn delete_user_returns_200_then_403() {
//     let app = spawn_test_app().await;
//     info!(
//         "delete_user_returns_200_then_403: app_port={} db_name={}",
//         &app.port, &app.test_db.db_name
//     );
//
//     let (res, _) = create_user(&app).await;
//     let body: JsonBody<UserCreationData> = res.json().await.unwrap();
//     let user = body.data;
//     let token = user.token;
//
//     let route = format!("/user/{}", user.id);
//
//     let client = reqwest::Client::new();
//     let res = client
//         .delete(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await
//         .expect(&format!("Failed to execute DELETE request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 200);
//
//     // Trying to retrieve the user after deletion should return 403.
//     let res = client
//         .get(format!("{}{}", app.address, &route))
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await
//         .expect(&format!("Failed to execute GET request at {}", &route));
//     dbg!(&res);
//     assert_eq!(res.status(), 403);
// }
