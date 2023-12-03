use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use cafe_website::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    filters,
    models::{Day, Event},
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EventParams {
    date: Option<NaiveDate>,
}

#[derive(Template)]
#[template(path = "events/view.html")]
pub struct EventViewTemplate {
    event: Event,
    days: Vec<Day>,
    selected_date: NaiveDate,
}

pub async fn view(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<EventParams>,
) -> Result<EventViewTemplate, AppError> {
    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(app_state.pool())
        .await?;
    let days = sqlx::query_as!(
        Day,
        "SELECT * FROM day WHERE event_id = $1 ORDER BY date ASC",
        id
    )
    .fetch_all(app_state.pool())
    .await?;

    let selected_date = query.date.unwrap_or(
        days.first()
            .ok_or(AppError::block(
                StatusCode::NOT_FOUND,
                "No schedules found for this event",
            ))?
            .date,
    );
    Ok(EventViewTemplate {
        event,
        days,
        selected_date,
    })
}
