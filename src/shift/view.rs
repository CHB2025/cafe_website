use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::Path;
use cafe_website::{filters, AppError};
use uuid::Uuid;

use crate::config;
use crate::models::Shift;
use crate::session::Session;
use crate::worker::Worker;

#[derive(Debug, Template, Clone)]
#[template(path = "shift/view.html")]
pub struct ShiftTemplate {
    pub(super) shift: Shift,
    pub(super) worker: Option<Worker>,
    pub(super) logged_in: bool,
}

#[derive(Debug, Template, Clone)]
#[template(path = "shift/edit.html")]
pub struct ShiftEditTemplate {
    shift: Shift,
}

pub async fn view(Path(id): Path<Uuid>, session: Session) -> Result<impl IntoResponse, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "SELECT s.* 
        FROM shift as s
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(config().pool())
    .await?;
    let worker = match shift.worker_id {
        Some(id) => Some(
            sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
                .fetch_one(config().pool())
                .await?,
        ),
        None => None,
    };

    Ok((
        [("HX-Replace-Url", "false")],
        ShiftTemplate {
            shift,
            logged_in: session.is_authenticated(),
            worker,
        },
    ))
}

pub async fn edit_form(Path(id): Path<Uuid>) -> Result<impl IntoResponse, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "SELECT s.*
        FROM shift as s
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(config().pool())
    .await?;

    Ok(([("HX-Replace-Url", "false")], ShiftEditTemplate { shift }))
}
