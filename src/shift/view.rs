use askama::Template;
use axum::extract::{Path, State};
use axum_sessions::extractors::ReadableSession;
use chrono::NaiveTime;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, models::User};

#[derive(Debug, Template, Clone)]
#[template(path = "shift/view.html")]
pub struct ShiftTemplate {
    shift: ShiftWithEvent,
    logged_in: bool,
}

#[derive(Debug, Template, Clone)]
#[template(path = "shift/edit.html")]
pub struct ShiftEditTemplate {
    shift: ShiftWithEvent,
}

#[derive(Debug, Clone)]
struct ShiftWithEvent {
    id: Uuid,
    day_id: Uuid,
    event_id: Uuid,
    worker_id: Option<Uuid>,
    start_time: NaiveTime,
    end_time: NaiveTime,
    title: String,
    description: Option<String>,
    public_signup: bool,
}

pub async fn view(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    session: ReadableSession,
) -> Result<ShiftTemplate, AppError> {
    let logged_in =
        !session.is_destroyed() && !session.is_expired() && session.get::<User>("user").is_some();
    let shift = sqlx::query_as!(
        ShiftWithEvent,
        "SELECT s.*, e.id as event_id 
        FROM shift as s
        INNER JOIN day as d ON s.day_id = d.id
        INNER JOIN event as e ON d.event_id = e.id
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(app_state.pool())
    .await?;

    Ok(ShiftTemplate { shift, logged_in })
}

pub async fn edit_form(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ShiftEditTemplate, AppError> {
    let shift = sqlx::query_as!(
        ShiftWithEvent,
        "SELECT s.*, e.id as event_id 
        FROM shift as s
        INNER JOIN day as d ON s.day_id = d.id
        INNER JOIN event as e ON d.event_id = e.id
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(app_state.pool())
    .await?;

    Ok(ShiftEditTemplate { shift })
}
