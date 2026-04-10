// Acceptance tests for specs/0001-create-task.md
//
// Each test name corresponds to one Given/When/Then bullet in that spec.
// If you change a test, update the spec; if you change the spec, update the test.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::Row;
use tempfile::TempDir;
use tower::ServiceExt;

use vibe_board::{build_router, db::init_pool};

struct TestApp {
    router: axum::Router,
    pool: sqlx::SqlitePool,
    _tmp: TempDir,
}

async fn setup() -> TestApp {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("test.db");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = init_pool(&url).await.expect("init pool");
    // Spec 0001 doesn't touch workspaces, but build_router now requires
    // workspace_root and checker_path (added by spec 0002 scaffolding).
    // Pass throwaway values that this spec's tests will never reach.
    let workspace_root = tmp.path().join("ws_unused");
    let router = build_router(pool.clone(), workspace_root, None);
    TestApp { router, pool, _tmp: tmp }
}

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// AC #1: GIVEN empty DB, WHEN GET /, THEN 200 + Todo column with zero cards.
#[tokio::test]
async fn ac1_get_root_on_empty_db_renders_empty_todo_column() {
    let app = setup().await;
    let resp = app
        .router
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains("Todo"), "expected Todo column heading");
    assert!(
        !body.contains("class=\"card\""),
        "expected no card elements on empty board, got: {body}"
    );
}

// AC #2: GIVEN empty DB, WHEN POST /tasks title=Write spec, THEN row inserted + re-rendered card.
#[tokio::test]
async fn ac2_post_task_inserts_row_and_renders_card() {
    let app = setup().await;
    let resp = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("title=Write spec"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "expected success, got {}",
        resp.status()
    );

    let row = sqlx::query("SELECT title, status FROM tasks")
        .fetch_one(&app.pool)
        .await
        .expect("row should exist");
    let title: String = row.get("title");
    let status: String = row.get("status");
    assert_eq!(title, "Write spec");
    assert_eq!(status, "todo");

    let body = body_string(resp).await;
    assert!(body.contains("Write spec"), "expected card text in body");
    assert!(body.contains("Todo"), "expected Todo column heading");
}

// AC #3: GIVEN a task exists, WHEN GET /, THEN body contains the task title in Todo column.
#[tokio::test]
async fn ac3_get_root_renders_existing_task() {
    let app = setup().await;
    sqlx::query("INSERT INTO tasks (title, status) VALUES (?, ?)")
        .bind("Write spec")
        .bind("todo")
        .execute(&app.pool)
        .await
        .unwrap();

    let resp = app
        .router
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;

    // Crude but sufficient: the title appears between the Todo heading and the form.
    let todo_idx = body.find("Todo").expect("Todo heading present");
    let form_idx = body.find("<form").expect("form present");
    let slice = &body[todo_idx..form_idx];
    assert!(
        slice.contains("Write spec"),
        "expected 'Write spec' between Todo heading and form, body was: {body}"
    );
}

// AC #4: GIVEN empty DB, WHEN POST /tasks with empty title, THEN 400 and no row inserted.
#[tokio::test]
async fn ac4_post_task_with_empty_title_returns_400_and_inserts_nothing() {
    let app = setup().await;
    let resp = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("title="))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "no rows should be inserted on bad request");
}
