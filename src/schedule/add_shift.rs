use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    Form,
};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::models::Shift;

#[derive(Template)]
#[template(path = "schedule/add_shift.html")]
pub struct CreateShiftTemplate {
    day_id: Uuid,
}

pub async fn add_shift_form(Path(day_id): Path<Uuid>) -> CreateShiftTemplate {
    CreateShiftTemplate { day_id }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateShiftInput {
    title: String,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    start_time: NaiveTime,
    #[serde(deserialize_with = "crate::time_ext::deserialize_time")]
    end_time: NaiveTime,
    description: Option<String>,
    public_signup: Option<String>,
}

pub async fn add_shift(
    State(app_state): State<AppState>,
    Path(day_id): Path<Uuid>,
    Form(shift_input): Form<CreateShiftInput>,
) -> Result<Redirect, StatusCode> {
    let CreateShiftInput {
        title,
        start_time,
        end_time,
        description,
        public_signup,
    } = shift_input;
    let tran = app_state
        .pool()
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let shift = sqlx::query_as!(
        Shift,
        "INSERT INTO shift (day_id, title, start_time, end_time, description, public_signup) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        day_id,
        title,
        start_time,
        end_time,
        description,
        public_signup.is_some_and(|s| s == "on")
    ).fetch_one(app_state.pool()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = sqlx::query_scalar!(
        "SELECT e.id FROM event AS e INNER JOIN day AS d ON d.event_id = e.id WHERE d.id = $1",
        shift.day_id
    )
    .fetch_one(app_state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tran.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/event/{event}")))
}
