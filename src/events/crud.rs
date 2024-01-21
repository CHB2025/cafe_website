use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Form,
};
use cafe_website::{AppError, Redirect};
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, models::Event};

use super::list_row::EventListRowTemplate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub allow_signups: Option<String>, // "on" or "off"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditEventInput {
    pub name: String,
    pub allow_signups: Option<String>, // "on" or "off"
}

#[derive(Template)]
#[template(path = "events/create.html")]
pub struct EventCreateTemplate {}

pub async fn create_event_form() -> EventCreateTemplate {
    EventCreateTemplate {}
}

pub async fn create_event(
    State(app_state): State<AppState>,
    Form(event_input): Form<EventInput>,
) -> Result<Redirect, AppError> {
    if event_input.start_date > event_input.end_date {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "Start date must be before end date",
        ));
    }
    let conn = app_state.pool();
    let transaction = conn.begin().await?;
    let event = sqlx::query_as!(
        Event,
        "INSERT INTO event (name, allow_signups) VALUES ($1, $2) RETURNING *",
        event_input.name,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(conn)
    .await?;

    // Probably a better way to do this
    for offset in 0..=(event_input.end_date - event_input.start_date).num_days() as u64 {
        let date = event_input.start_date + Days::new(offset);
        sqlx::query!(
            "INSERT INTO day (event_id, date) VALUES ($1, $2)",
            event.id,
            date,
        )
        .execute(conn)
        .await?;
    }
    transaction.commit().await?;
    Ok(Redirect::to(format!("/event/{}", event.id)))
}

pub async fn patch_event(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(event_input): Form<EditEventInput>,
) -> Result<EventListRowTemplate, AppError> {
    let event = sqlx::query_as!(
        Event,
        "UPDATE event SET name = $2, allow_signups = $3 WHERE id = $1 RETURNING *",
        id,
        event_input.name,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(app_state.pool())
    .await?;

    Ok(EventListRowTemplate { event })
}

pub async fn delete_event(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
    match sqlx::query!("DELETE FROM event WHERE id = $1", id)
        .execute(app_state.pool())
        .await
    {
        Ok(x) if x.rows_affected() == 1 => StatusCode::OK,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
