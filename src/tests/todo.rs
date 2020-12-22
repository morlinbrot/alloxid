use super::test_helpers::{spawn_test_app, TestDb};

use crate::Todo;

#[async_std::test]
async fn get_todos() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/todo/all";

    let mut res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let todos: Vec<Todo> = res.body_json().await.unwrap();
    assert_eq!(todos.len(), 1);
}

#[async_std::test]
async fn get_todo() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let id = 42;
    let route = format!("/todo/{}", id);

    let mut res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));
    dbg!(&res);
    assert_eq!(res.status(), 200);

    let todo: Todo = res.body_json().await.unwrap();
    dbg!(&todo);
    assert_eq!(todo.id, 42);
}

#[async_std::test]
async fn create_todo() {
    let test_db = TestDb::new().await;
    let app = spawn_test_app(test_db.pool()).await;

    let route = "/todo";

    let json = serde_json::json!({ "text": "Test creating a todo.", "completed": true });
    dbg!(&json);

    let mut res = surf::post(format!("{}{}", app.address, &route))
        .body(http_types::Body::from_json(&json).unwrap())
        .await
        .expect(&format!("Failed to execute POST request at {}", &route));
    dbg!(&res);

    assert_eq!(res.status(), 200);

    let todo: Todo = res.body_json().await.unwrap();
    dbg!(&todo);
}
