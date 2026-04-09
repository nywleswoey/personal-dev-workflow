use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{SqlitePool, Row};
use std::str::FromStr;

use crate::models::{Task, TaskStatus};

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn insert_task(pool: &SqlitePool, title: &str) -> Result<Task, sqlx::Error> {
    let status = TaskStatus::Todo.as_str();
    let row = sqlx::query("INSERT INTO tasks (title, status) VALUES (?, ?) RETURNING id, title, status")
        .bind(title)
        .bind(status)
        .fetch_one(pool)
        .await?;
    Ok(Task {
        id: row.get("id"),
        title: row.get("title"),
        status: row.get("status"),
    })
}

pub async fn list_tasks_by_status(
    pool: &SqlitePool,
    status: TaskStatus,
) -> Result<Vec<Task>, sqlx::Error> {
    let status_str = status.as_str();
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT id, title, status FROM tasks WHERE status = ? ORDER BY id ASC",
    )
    .bind(status_str)
    .fetch_all(pool)
    .await?;
    Ok(tasks)
}
