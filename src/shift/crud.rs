use axum::{extract::{Path, State}, response::Html, Form};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError};

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
) -> Result<Html<String>, AppError> {
    sqlx::query!(
        "UPDATE shift SET title = $1, start_time = $2, end_time = $3, description = $4, public_signup = $5 WHERE id = $6",
        title,
        start_time,
        end_time,
        description, 
        public_signup.is_some_and(|s| s == "on"),
        id
    ).execute(app_state.pool()).await?;
    Ok(Html(format!(
        r##"<span class="success" hx-get="/shift/{id}" hx-target="closest form" hx-swap="outerHTML" hx-trigger="load delay:2s">Success</span>"##
    )))
}

pub async fn delete_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    let event_id = sqlx::query_scalar!(
        "SELECT e.id 
        FROM shift as s 
        INNER JOIN day as d ON s.day_id = d.id
        INNER JOIN event as e ON d.event_id = e.id
        WHERE s.id = $1",
        id
    ).fetch_one(app_state.pool()).await?;

    sqlx::query!(
        "DELETE FROM shift WHERE id = $1",
        id
    ).fetch_one(app_state.pool()).await?;

    Ok(Html(format!(
        r##"<span class="success" hx-get="/event/{event_id}" hx-target="#content" hx-swap="innerHTML" hx-trigger="load delay:2s">Success</span>"##
    )))
}
