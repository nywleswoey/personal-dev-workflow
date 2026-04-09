use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::State,
    http::StatusCode,
    response::Response,
    Form,
};
use serde::Deserialize;

use crate::db::{insert_task, list_tasks_by_status};
use crate::models::{Task, TaskStatus};
use crate::AppState;

#[derive(Template)]
#[template(path = "board.html")]
pub struct BoardTemplate {
    pub todo_tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskForm {
    pub title: String,
}

pub async fn get_board(State(state): State<AppState>) -> Response {
    match list_tasks_by_status(&state.pool, TaskStatus::Todo).await {
        Ok(tasks) => BoardTemplate { todo_tasks: tasks }.into_response(),
        Err(e) => {
            tracing::error!("failed to load tasks: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}

pub async fn post_task(
    State(state): State<AppState>,
    Form(form): Form<CreateTaskForm>,
) -> Response {
    let title = form.title.trim();
    if title.is_empty() {
        return (StatusCode::BAD_REQUEST, "title is required").into_response();
    }
    if let Err(e) = insert_task(&state.pool, title).await {
        tracing::error!("failed to insert task: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }
    match list_tasks_by_status(&state.pool, TaskStatus::Todo).await {
        Ok(tasks) => BoardTemplate { todo_tasks: tasks }.into_response(),
        Err(e) => {
            tracing::error!("failed to reload tasks: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}
