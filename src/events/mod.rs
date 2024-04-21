use axum::{
    extract::Query,
    response::Html,
    routing::{get, patch},
    Router,
};

mod crud;
mod list;
mod list_row;
mod pagination;
mod view;

use cafe_website::AppError;
use crud::*;
use list::*;
use list_row::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config, models::Event, schedule};

use self::view::view;

#[derive(Serialize, Deserialize)]
pub struct EventOptionQuery {
    selected: Option<Uuid>,
}

pub async fn event_option_list(
    Query(query): Query<EventOptionQuery>,
) -> Result<Html<String>, AppError> {
    use std::fmt::Write;

    let events = sqlx::query_as!(Event, "SELECT * from event ORDER BY id ASC")
        .fetch_all(config().pool())
        .await?;

    let result: String = events.iter().fold(String::new(), |mut output, e| {
        let sel = if query.selected.is_some_and(|s_id| s_id == e.id) {
            "selected"
        } else {
            ""
        };
        let _ = write!(
            output,
            "<option value=\"{}\" {}>{}</option>",
            e.id, sel, e.name
        );
        output
    });

    Ok(Html(result))
}

pub fn protected_router() -> Router {
    Router::new()
        .route("/:id", patch(patch_event).delete(delete_event))
        .route("/create", get(create_event_form).post(create_event))
        .route("/option_list", get(event_option_list))
        .route("/day/option_list", get(schedule::option_list))
        .route("/list", get(event_list))
        .route("/list/row/:id", get(event_table_row))
        .route("/list/row/:id/edit", get(edit_event_table_row))
        .nest("/:id", schedule::protected_router())
}

pub fn public_router() -> Router {
    Router::new()
        .route("/:id", get(view))
        .nest("/:id", schedule::public_router())
}
