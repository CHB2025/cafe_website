use axum::{extract::State, http::StatusCode, response::Html, Form};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::app_state::AppState;
use crate::models::Event;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    name: String,
    start_date: Date,
    end_date: Date,
}

pub async fn create_event(
    State(app_state): State<AppState>,
    Form(event_input): Form<EventInput>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    let conn = app_state.pool();
    let event = sqlx::query_as!(
        Event,
        "INSERT INTO event (name, start_date, end_date) VALUES ($1, $2, $3) RETURNING *",
        event_input.name,
        event_input.start_date,
        event_input.end_date
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;
    Ok(Html(format!("<span class=\"success\" hx-get=\"/event/{}\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>", event.id)))
}
