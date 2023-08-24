use askama::Template;
use axum::{extract::State, http::StatusCode};

use crate::{app_state::AppState, models::Day};

#[derive(Template)]
#[template(path = "events/view.html")]
pub struct EventViewTemplate {
    day_ids: Vec<i32>,
    active_day: Day,
}

pub async fn view(State(app_state): State<AppState>) -> Result<EventViewTemplate, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
