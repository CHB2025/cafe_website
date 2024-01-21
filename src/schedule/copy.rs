use askama::Template;
use axum::{
    extract::{Path, State},
    Form,
};
use cafe_website::{AppError, Redirect};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, models::Shift};

#[derive(Template, Debug, Clone)]
#[template(path = "schedule/copy.html")]
pub struct CopyTemplate {
    event_id: Uuid,
    date: NaiveDate,
}

pub async fn copy_form(
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
) -> Result<CopyTemplate, AppError> {
    Ok(CopyTemplate { event_id, date })
}

#[derive(Deserialize)]
pub struct CopyBody {
    event_id: Uuid,
    date: NaiveDate,
}

pub async fn copy(
    State(app_state): State<AppState>,
    Path((event_from, date_from)): Path<(Uuid, NaiveDate)>,
    Form(CopyBody {
        event_id: event_to,
        date: date_to,
    }): Form<CopyBody>,
) -> Result<Redirect, AppError> {
    let tran = app_state.pool().begin().await?;
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE event_id = $1 AND date = $2",
        event_from,
        date_from
    )
    .fetch_all(app_state.pool())
    .await?;
    sqlx::query!(
        "DELETE FROM shift WHERE event_id = $1 AND date = $2",
        event_to,
        date_to
    )
    .execute(app_state.pool())
    .await?;
    for shift in shifts {
        sqlx::query!(
            "INSERT INTO shift (event_id, date, start_time, end_time, title, description, public_signup) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            event_to,
            date_to,
            shift.start_time,
            shift.end_time,
            shift.title,
            shift.description,
            shift.public_signup,
        ).execute(app_state.pool()).await?;
    }
    tran.commit().await?;
    Ok(Redirect::to(format!("/event/{event_to}")))
}
