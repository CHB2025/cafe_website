use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Form,
};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::models::Shift;

#[derive(Template)]
#[template(path = "schedule/add_shift.html")]
pub struct CreateShiftTemplate {
    day_id: i32,
}

pub async fn add_shift_form(Path(day_id): Path<i32>) -> CreateShiftTemplate {
    CreateShiftTemplate { day_id }
}

// Cannot deserialize the time values? Works when they are strings
#[derive(Serialize, Deserialize)]
pub struct CreateShiftInput {
    title: String,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    start_time: NaiveTime,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    end_time: NaiveTime,
    description: String,
}

pub async fn add_shift(
    State(app_state): State<AppState>,
    Path(day_id): Path<i32>,
    Form(shift_input): Form<CreateShiftInput>,
) -> Result<Html<String>, StatusCode> {
    let CreateShiftInput {
        title,
        start_time,
        end_time,
        description,
    } = shift_input;
    let _shift = sqlx::query_as!(
        Shift,
        "INSERT INTO shift (day_id, title, start_time, end_time, description) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        day_id,
        title,
        start_time,
        end_time,
        description
    ).fetch_one(app_state.pool()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(format!(
        r##"<span hx-target="#content" hx-push-url="true" hx-get="/day/{day_id}/schedule" hx-trigger="load">Success</span>"##
    )))
}
