use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, patch},
    Router,
};

mod crud;
mod list;
mod list_row;
mod pagination;
mod view;

use crud::*;
use list::*;
use list_row::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, models::Event, utils};

use self::view::view;

#[derive(Serialize, Deserialize)]
pub struct EventOptionQuery {
    selected: Option<Uuid>,
}

pub async fn event_option_list(
    State(app_state): State<AppState>,
    Query(query): Query<EventOptionQuery>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    let conn = app_state.pool();
    let events = sqlx::query_as!(Event, "SELECT * from event ORDER BY id ASC")
        .fetch_all(conn)
        .await
        .map_err(utils::ise)?;
    let result: String = events
        .iter()
        .map(|e| {
            let sel = if query.selected.is_some_and(|s_id| s_id == e.id) {
                "selected"
            } else {
                ""
            };
            format!("<option value=\"{}\" {}>{}</option>", e.id, sel, e.name)
        })
        .collect();
    Ok(Html(result))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:id", patch(patch_event).delete(delete_event))
        .route("/create", get(create_event_form).post(create_event))
        .route("/option_list", get(event_option_list))
        .route("/list", get(event_list))
        .route("/list/row/:id", get(event_table_row))
        .route("/list/row/:id/edit", get(edit_event_table_row))
}

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:id", get(view))
}
