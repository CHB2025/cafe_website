use axum::{extract::{Path, State, Query}, response::Html, Form};
use cafe_website::{AppError, Redirect};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, models::Shift};

use super::view::ShiftTemplate;

#[derive(Serialize, Deserialize)]
pub struct ShiftUpdate {
    title: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    description: Option<String>,
    public_signup: Option<String>,
}

pub async fn update_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(ShiftUpdate { title, start_time, end_time, description, public_signup }): Form<ShiftUpdate>
) -> Result<Redirect, AppError> {
    sqlx::query!(
        "UPDATE shift SET title = $1, start_time = $2, end_time = $3, description = $4, public_signup = $5 WHERE id = $6",
        title,
        start_time,
        end_time,
        description, 
        public_signup.is_some_and(|s| s == "on"),
        id
    ).execute(app_state.pool()).await?;

    // Ok(ShiftTemplate { shift, worker, logged_in: true })
    Ok(Redirect::targeted(format!("/shift/{}", id), "#modal".to_owned()))
}

pub async fn delete_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    let event_id = sqlx::query_scalar!(
        "DELETE FROM shift WHERE id = $1 RETURNING event_id",
        id
    ).fetch_one(app_state.pool()).await?;

    Ok(Html(format!(
        r##"<span class="success" hx-get="/event/{event_id}" hx-target="#content" hx-swap="innerHTML" hx-trigger="load delay:2s">Success</span>"##
    )))
}


#[derive(Deserialize)]
pub struct RmWorkerQuery {
    id: Uuid
}

pub async fn remove_worker(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(RmWorkerQuery { id: worker_id }): Query<RmWorkerQuery>,
) -> Result<Redirect, AppError> {
    sqlx::query!(
        "UPDATE shift SET worker_id = NULL WHERE id = $1 AND worker_id = $2",
        id, worker_id
    ).execute(app_state.pool()).await?;
    Ok(Redirect::targeted(format!("/shift/{}", id), "#modal".to_owned()))
}
