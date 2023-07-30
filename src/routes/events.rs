mod create;
use axum::{extract::State, http::StatusCode, response::Html};
pub use create::*;

use crate::{app_state::AppState, models::Event, utils};

pub async fn event_option_list(
    State(app_state): State<AppState>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    let conn = app_state.pool();
    let events = sqlx::query_as!(Event, "SELECT * from event WHERE start_date > CURRENT_DATE")
        .fetch_all(conn)
        .await
        .map_err(utils::ise)?;
    let result: String = events
        .iter()
        .map(|e| format!("<option value=\"{}\">{}</option>", e.id, e.name))
        .collect();
    Ok(Html(result))
}
