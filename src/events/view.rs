use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Day, Event},
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EventParams {
    day: Option<Uuid>,
}

#[derive(Template)]
#[template(path = "events/view.html")]
pub struct EventViewTemplate {
    event: Event,
    days: Vec<Day>,
    selected_day_id: Uuid,
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

    let selected_day_id = query.day.unwrap_or(
        days.first()
            .ok_or(AppError::block(
                StatusCode::NOT_FOUND,
                "No schedules found for this event",
            ))?
            .id,
    );
    Ok(EventViewTemplate {
        event,
        days,
        selected_day_id,
    })
}