use askama::Template;
use axum::extract::{Path, Query, State};
use cafe_website::{error, filters, AppError};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{Event, Shift, User},
};

#[derive(Template)]
#[template(path = "worker/shift_list.html")]
pub struct ShiftList {
    worker_id: Uuid,
    event_id: Uuid,
    events: Vec<Event>,
    shifts: Vec<Shift>,
    may_cancel: bool,
}

#[derive(Deserialize)]
pub struct ShiftListQuery {
    event_id: Option<Uuid>,
}

pub async fn shift_list(
    State(app_state): State<AppState>,
    user: Option<User>,
    Path(worker_id): Path<Uuid>,
    Query(query): Query<ShiftListQuery>,
) -> Result<ShiftList, AppError> {
    let events = sqlx::query_as!(
        Event,
        "SELECT e.* FROM event e JOIN shift s ON e.id = s.event_id
        WHERE s.worker_id = $1
        GROUP BY e.id
        ORDER BY MIN(s.date) ASC",
        worker_id
    )
    .fetch_all(app_state.pool())
    .await?;
    if events.is_empty() {
        return Err(error::NOT_FOUND);
    }

    let selected_event = match query.event_id {
        Some(id) => events.iter().find(|e| e.id == id).ok_or(error::NOT_FOUND)?,
        None => events.last().expect("Checked above"),
    };
    let event_id = selected_event.id;

    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE event_id = $1 AND worker_id = $2 ORDER BY date, start_time",
        event_id,
        worker_id
    )
    .fetch_all(app_state.pool())
    .await?;

    let in_future = shifts.first().expect("Must have 1+ shift to be here").date
        > chrono::Local::now().date_naive();

    Ok(ShiftList {
        worker_id,
        event_id,
        may_cancel: in_future && (user.is_some() || selected_event.allow_signups),
        events,
        shifts,
    })
}
