mod test_helpers;
use test_helpers::spawn_test_app;

use super::Todo;

#[async_std::test]
async fn it_works() {
    let app = spawn_test_app().await;

    let route = "/health-check";

    let res = surf::get(format!("{}{}", app.address, route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));

    assert_eq!(res.status(), 200);
}

#[async_std::test]
async fn returns_all_todos() {
    let app = spawn_test_app().await;

    let route = "/api/all";

    let mut res = surf::get(format!("{}{}", app.address, &route))
        .await
        .expect(&format!("Failed to execute GET request at {}", &route));

    let todos: Vec<Todo> = res.body_json().await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(todos.len(), 1);
}
