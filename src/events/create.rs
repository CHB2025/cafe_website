use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html, Form};
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::models::Event;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
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
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    if event_input.start_date > event_input.end_date {
        return Err((
            StatusCode::BAD_REQUEST,
            Html(r##"<span class="error">Start date must be before end date</span>"##),
        ));
    }
    let conn = app_state.pool();
    let transaction = conn.begin().await.map_err(utils::ise)?;
    let event = sqlx::query_as!( Event,
        "INSERT INTO event (name, start_date, end_date, allow_signups) VALUES ($1, $2, $3, $4) RETURNING *",
        event_input.name,
        event_input.start_date,
        event_input.end_date,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;

    // Probably a better way to do this
    for offset in 0..=(event.end_date - event.start_date).num_days() as u64 {
        let date = event.start_date + Days::new(offset);
        sqlx::query!(
            "INSERT INTO day (event_id, date) VALUES ($1, $2)",
            event.id,
            date,
        )
        .execute(conn)
        .await
        .map_err(utils::ise)?;
    }
    transaction.commit().await.map_err(utils::ise)?;
    Ok(Html(format!("<span class=\"success\" hx-get=\"/event/{}\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>", event.id)))
}
