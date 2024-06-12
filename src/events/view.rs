use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
};
use cafe_website::{filters, AppError};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config,
    models::{Day, Event},
    session::Session,
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
    authenticated: bool,
}

pub async fn view(
    session: Session,
    Path(id): Path<Uuid>,
    Query(query): Query<EventParams>,
) -> Result<EventViewTemplate, AppError> {
    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;
    let days = sqlx::query_as!(
        Day,
        "SELECT * FROM day WHERE event_id = $1 ORDER BY date ASC",
        id
    )
    .fetch_all(config().pool())
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
        authenticated: session.is_authenticated(),
    })
}
