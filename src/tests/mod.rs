mod test_helpers;
use test_helpers::{spawn_test_app, TestDb};

use super::{Todo, User};

#[async_std::test]
async fn health_check() {
    tide::log::start();
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/health-check";

    let res = surf::get(format!("{}{}", app.address, route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));

    assert_eq!(res.status(), 200);
}

#[async_std::test]
async fn get_todos() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/api/all";

    let mut res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));

    let todos: Vec<Todo> = res.body_json().await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(todos.len(), 1);
}

#[async_std::test]
async fn create_user() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/user";

    let username = "synul";
    let json = serde_json::json!({ "username": username });

    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&json).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));

    let user: User = res.body_json().await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(!user.id.is_nil());
    assert_eq!(user.username, username);
}
