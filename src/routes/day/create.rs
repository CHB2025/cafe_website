use axum::{extract::State, http::StatusCode, response::Html, Form};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::{app_state::AppState, models::Day, utils};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayInput {
    event_id: i32,
    date: Date,
    entertainment: String,
}

pub async fn create_day_form() -> Html<String> {
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
    "##.to_string())
}

pub async fn create_day(
    State(app_state): State<AppState>,
    Form(day_input): Form<DayInput>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    let conn = app_state.pool();
    let day = sqlx::query_as!(
        Day,
        "INSERT INTO day (event_id, date, entertainment) VALUES ($1, $2, $3) RETURNING *",
        day_input.event_id,
        day_input.date,
        day_input.entertainment
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;

    Ok(Html(format!("<span class=\"success\" hx-get=\"/day/{}\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>", day.id)))
}
