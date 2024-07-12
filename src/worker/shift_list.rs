use askama::Template;
use axum::extract::{Path, Query};
use cafe_website::{error, filters, AppError};
use serde::Deserialize;
use std::borrow::Borrow;
use tracing::info;
use uuid::Uuid;

use crate::{
    config,
    models::{Event, Shift},
    session::Session,
};

#[derive(Template)]
#[template(path = "worker/shift_list.html")]
pub enum ShiftList {
    Some {
        worker_id: Uuid,
        event_id: Uuid,
        events: Vec<Event>,
        shifts: Vec<Shift>,
        may_cancel: bool,
    },
    None,
}

#[derive(Deserialize)]
pub struct ShiftListQuery {
    event_id: Option<Uuid>,
}

pub async fn shift_list(
    session: Session,
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
    .fetch_all(config().pool())
    .await?;
    if events.is_empty() {
        return Ok(ShiftList::None);
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
    .fetch_all(config().pool())
    .await?;

    let in_future = shifts.first().expect("Must have 1+ shift to be here").date
        > chrono::Local::now().date_naive();

    Ok(ShiftList::Some {
        worker_id,
        event_id,
        may_cancel: in_future && (session.is_authenticated() || selected_event.allow_signups),
        events,
        shifts,
    })
}

#[derive(Deserialize)]
pub struct CancelShiftParams {
    shift_id: Uuid,
}

pub async fn cancel_shift(
    session: Session,
    Path(worker_id): Path<Uuid>,
    Query(CancelShiftParams { shift_id }): Query<CancelShiftParams>,
) -> Result<ShiftList, AppError> {
    let tran = config().pool().begin().await?;

    let shift = sqlx::query_as!(
        Shift,
        "UPDATE shift SET worker_id = NULL WHERE id = $1 AND worker_id = $2 RETURNING *",
        shift_id,
        worker_id
    )
    .fetch_one(config().pool())
    .await?;

    let list = shift_list(
        session,
        Path(worker_id),
        Query(ShiftListQuery {
            event_id: Some(shift.event_id),
        }),
    )
    .await?;

    tran.commit().await?;

    info!(
        "Worker {} has canceled their shift: {} {}-{} on {}",
        worker_id,
        shift.title,
        filters::time_short(&shift.start_time).expect("Infallible"),
        filters::time_short(&shift.end_time).expect("Infallible"),
        filters::date_short(&shift.date).expect("Infallible"),
    );
    Ok(list)
}
