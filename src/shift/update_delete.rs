use axum::{extract::Path, response::Html};
use chrono::NaiveTime;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError};

pub struct ShiftUpdate {
    title: String,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    start_time: NaiveTime,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    end_time: NaiveTime,
    description: Option<String>,
    public_signup: Option<String>,
}

pub async fn update_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    sqlx::query!(
        "UPDATE "
    )
    Ok(Html(format!(
        r##"<span class="success" hx-get="/shift/{id}" hx-target="first form" hx-swap="outerHTML" hx-trigger="load:delay 2s">Success</span>"##
    )))
}

pub async fn delete_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    Ok(Html(format!(
        r##"<span class="success" hx-get="/shift/{id}" hx-target="first form" hx-swap="outerHTML" hx-trigger="load:delay 2s">Success</span>"##
    )))
}
