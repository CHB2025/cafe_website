use askama::Template;
use axum::extract::{Path, State};
use cafe_website::AppError;
use uuid::Uuid;

use crate::{app_state::AppState, models::Event};

#[derive(Template)]
#[template(path = "events/list_row.html")]
pub struct EventListRowTemplate {
    pub event: Event,
}

#[derive(Template)]
#[template(path = "events/edit_list_row.html")]
pub struct EditEventListRowTemplate {
    pub event: Event,
}

pub async fn event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<EventListRowTemplate, AppError> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await?;

    Ok(EventListRowTemplate { event })
}

pub async fn edit_event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<EditEventListRowTemplate, AppError> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await?;

    Ok(EditEventListRowTemplate { event })
}
