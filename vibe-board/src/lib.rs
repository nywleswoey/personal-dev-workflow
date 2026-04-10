pub mod db;
pub mod models;
pub mod routes;
pub mod workspace;
pub mod workspace_routes;

use std::path::PathBuf;

use axum::{routing::get, Router};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    /// Root directory under which workspace worktrees are created. Resolved
    /// from `VIBE_BOARD_WORKSPACE_ROOT` (or `dirs::data_dir()/vibe-board/workspaces`)
    /// once at process startup; tests pass an explicit `TempDir` path.
    pub workspace_root: PathBuf,
    /// Absolute path to the `vibe-board-commit-check` binary, if resolvable.
    /// `None` means the binary was not found on `PATH` at startup, in which
    /// case workspace creation returns 400 with `data-error="commit-check-missing"`.
    pub checker_path: Option<PathBuf>,
}

pub fn build_router(
    pool: SqlitePool,
    workspace_root: PathBuf,
    checker_path: Option<PathBuf>,
) -> Router {
    let state = AppState {
        pool,
        workspace_root,
        checker_path,
    };
    Router::new()
        .route("/", get(routes::get_board))
        .route("/tasks", axum::routing::post(routes::post_task))
        .merge(workspace_routes::workspace_router())
        .with_state(state)
}
