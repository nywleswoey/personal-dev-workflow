// Spec 0002: HTTP routes for workspaces.
//
// Red-baseline scaffold. Handlers compile but `todo!()` at runtime; they
// will be filled in across issues #2-#7. The router is wired into
// `build_router` in lib.rs so that compilation succeeds end-to-end.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

use crate::workspace::Workspace;
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

pub async fn get_workspaces(State(_state): State<AppState>) -> Response {
    todo!("issue #2: implement get_workspaces handler")
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
