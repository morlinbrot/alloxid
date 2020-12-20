mod test_helpers;
use test_helpers::{spawn_test_app, TestDb};

use super::Todo;

mod create_user;

#[async_std::test]
async fn health_check() {
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
