use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    app_state::AppState,
    models::{Day, Event},
};

#[derive(Template)]
#[template(path = "events/view.html")]
pub struct EventViewTemplate {
    event: Event,
    days: Vec<Day>,
}

pub async fn view(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<EventViewTemplate, StatusCode> {
    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(app_state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let days = sqlx::query_as!(
        Day,
        "SELECT * FROM day WHERE event_id = $1 ORDER BY date ASC",
        id
    )
    .fetch_all(app_state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(EventViewTemplate { event, days })
}
