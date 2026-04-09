pub mod db;
pub mod models;
pub mod routes;

use axum::{routing::get, Router};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn build_router(pool: SqlitePool) -> Router {
    let state = AppState { pool };
    Router::new()
        .route("/", get(routes::get_board))
        .route("/tasks", axum::routing::post(routes::post_task))
        .with_state(state)
}
