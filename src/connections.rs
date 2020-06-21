use sqlx::postgres::PgPool;
use sqlx::prelude::*;

#[derive(sqlx::FromRow, Debug)]
struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

/// Overview of different kinds of connections possible in sqlx.
#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPool::new("postgres://dbuser:dbuser@localhost/fullstack").await?;

    let mut cursor = sqlx::query("SELECT * FROM todos").fetch(&pool);
    while let Some(row) = cursor.next().await? {
        let id: i32 = row.get("id");
        println!("{}", id);
    }

    let cursor = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&pool)
        .await?;
    for todo in cursor {
        println!("{} {} {}", todo.id, todo.text, todo.completed);
    }

    let todos = sqlx::query!("SELECT * FROM todos").fetch_all(&pool).await?;
    for todo in todos {
        println!("{:?}", todo);
    }

    let todos = sqlx::query_as!(Todo, "SELECT * from todos")
        .fetch_all(&pool)
        .await?;
    for todo in todos {
        println!("{:?}", todo);
    }

    Ok(())
}
