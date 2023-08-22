mod create;
mod list;
mod list_row;
use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, patch},
    Router,
};

use create::*;
use list::*;

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

pub fn event_router() -> Router<AppState> {
    Router::new()
        .route("/create", get(create_event_form).post(create_event))
        .route("/update/:id", patch(patch_event))
        .route("/option_list", get(event_option_list))
        .route("/list", get(event_list))
        .route("/list/row/:id", get(event_table_row))
        .route("/list/row/:id/edit", get(edit_event_table_row))
}