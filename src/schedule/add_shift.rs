use askama::Template;
use axum::{
    extract::{Path, State},
    Form,
};
use cafe_website::{templates::Card, AppError, Redirect};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Template)]
#[template(path = "schedule/add_shift.html")]
pub struct CreateShiftTemplate {
    event_id: Uuid,
    date: NaiveDate,
}

pub async fn add_shift_form(
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
) -> Card<CreateShiftTemplate> {
    Card::modal(
        "Add Shift".to_owned(),
        CreateShiftTemplate { event_id, date },
    )
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
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
    Form(shift_input): Form<CreateShiftInput>,
) -> Result<Redirect, AppError> {
    let CreateShiftInput {
        title,
        start_time,
        end_time,
        description,
        public_signup,
    } = shift_input;

    sqlx::query!(
        "INSERT INTO shift (date, event_id, title, start_time, end_time, description, public_signup) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        date,
        event_id,
        title,
        start_time,
        end_time,
        description,
        public_signup.is_some_and(|s| s == "on")
    ).fetch_one(app_state.pool()).await?;

    Ok(Redirect::to(format!("/event/{event_id}")))
}
