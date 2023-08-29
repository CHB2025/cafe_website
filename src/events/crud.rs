use askama::Template;
use axum::{extract::{State, Path}, http::StatusCode, Form, response::Html};
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, models::Event, utils};

use super::list_row::EventListRowTemplate;

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

pub async fn patch_event(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(event_input): Form<EventInput>,
) -> Result<EventListRowTemplate, (StatusCode, String)> {
    let conn = app_state.pool();
    let event = sqlx::query_as!(
        Event, 
        "UPDATE event SET name = $2, start_date = $3, end_date = $4, allow_signups = $5 WHERE id = $1 RETURNING *",
        id,
        event_input.name,
        event_input.start_date,
        event_input.end_date,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(conn)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update event".to_owned()))?;
    Ok(EventListRowTemplate { event })
}

pub async fn delete_event(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>
) -> StatusCode {
    match sqlx::query!("DELETE FROM event WHERE id = $1", id).execute(app_state.pool()).await {
        Ok(x) if x.rows_affected() == 1 => StatusCode::OK,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
