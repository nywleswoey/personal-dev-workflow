// Spec 0002: HTTP routes for workspaces.
//
// Red-baseline scaffold. Handlers compile but `todo!()` at runtime; they
// will be filled in across issues #2-#7. The router is wired into
// `build_router` in lib.rs so that compilation succeeds end-to-end.

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

use crate::workspace::{list_active_workspaces, Workspace};
use crate::AppState;

#[derive(Template)]
#[template(path = "workspaces.html")]
pub struct WorkspacesTemplate {
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceForm {
    pub name: String,
    #[serde(default)]
    pub repos: Vec<String>,
}

pub async fn get_workspaces(State(state): State<AppState>) -> Response {
    match list_active_workspaces(&state.pool).await {
        Ok(workspaces) => WorkspacesTemplate { workspaces }.into_response(),
        Err(e) => {
            tracing::error!("failed to list workspaces: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}

pub async fn post_workspace(
    State(_state): State<AppState>,
    Form(_form): Form<CreateWorkspaceForm>,
) -> Response {
    todo!("issue #3-#6: implement post_workspace handler")
}

pub async fn post_workspace_archive(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> Response {
    todo!("issue #7: implement post_workspace_archive handler")
}

pub fn workspace_router() -> Router<AppState> {
    Router::new()
        .route("/workspaces", get(get_workspaces).post(post_workspace))
        .route("/workspaces/:name/archive", post(post_workspace_archive))
}
