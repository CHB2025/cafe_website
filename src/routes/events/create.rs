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
    allow_signups: Option<String>, // "on" or "off"
}

pub async fn create_event_form() -> Html<String> {
    Html(r##"
        
        <form class="form card" action="/event/create" method="post" hx-boost="true" hx-target="#create_event_results" hx-indicator="#create_event_submit" hx-disinherit="true">
          <div class="form-item">
            <label>Name</label>
            <input name="name" type="text" required="true"></input>
          </div>
          <div class="form-item">
            <label>Start Date</label>
            <input name="start_date" type="date" required="true"></input>
          </div>
          <div class="form-item">
            <label>End Date</label>
            <input name="end_date" type="date" required="true"></input>
          </div>
          <div class="form-item">
            <label>Allow Signups</label>
            <input name="allow_signups" type="checkbox"></input>
            <div class="spacer"></div>
          </div>
          <div class="form-item">
            <button id="create_event_submit" type="submit">Submit</button>
          </div>
          <div id="create_event_results" class="form-item"></div>
        </form>
    "##.to_string(),
    )
}

pub async fn create_event(
    State(app_state): State<AppState>,
    Form(event_input): Form<EventInput>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    println!("Creating event: {:?}", event_input);
    let conn = app_state.pool();
    let event = sqlx::query_as!(
        Event,
        "INSERT INTO event (name, start_date, end_date, allow_signups) VALUES ($1, $2, $3, $4) RETURNING *",
        event_input.name,
        event_input.start_date,
        event_input.end_date,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;
    Ok(Html(format!("<span class=\"success\" hx-get=\"/event/{}\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>", event.id)))
}
