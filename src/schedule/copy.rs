use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, models::Shift};

#[derive(Template, Debug, Clone)]
#[template(path = "schedule/copy.html")]
pub struct CopyTemplate {
    id: Uuid,
    event_id: Uuid,
}

pub async fn copy_form(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<CopyTemplate, AppError> {
    let event_id = sqlx::query_scalar!("SELECT event_id FROM day WHERE id = $1", id)
        .fetch_one(app_state.pool())
        .await?;
    Ok(CopyTemplate { id, event_id })
}

#[derive(Deserialize)]
pub struct CopyBody {
    day_id: Uuid,
    event_id: Uuid,
}

pub async fn copy(
    State(app_state): State<AppState>,
    Path(copy_from): Path<Uuid>,
    Form(CopyBody {
        event_id,
        day_id: copy_to,
    }): Form<CopyBody>,
) -> Result<Html<String>, AppError> {
    let tran = app_state.pool().begin().await?;
    let shifts = sqlx::query_as!(Shift, "SELECT * FROM shift WHERE day_id = $1", copy_from)
        .fetch_all(app_state.pool())
        .await?;
    sqlx::query!("DELETE FROM shift WHERE day_id = $1", copy_to)
        .execute(app_state.pool())
        .await?;
    for shift in shifts {
        sqlx::query!(
            "INSERT INTO shift (day_id, start_time, end_time, title, description, public_signup) VALUES ($1, $2, $3, $4, $5, $6)",
            copy_to,
            shift.start_time,
            shift.end_time,
            shift.title,
            shift.description,
            shift.public_signup,
        ).execute(app_state.pool()).await?;
    }
    tran.commit().await?;
    Ok(Html(format!(
        r##"<span class="success" hx-get="/event/{event_id}" hx-trigger="load delay:1s" hx-target="#content">Copied the shifts successfully!</span>"##
    )))
}
