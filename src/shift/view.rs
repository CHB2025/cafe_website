use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use cafe_website::{filters, AppError};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{Shift, User, Worker},
};

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

pub async fn view(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    user: Option<User>,
) -> Result<impl IntoResponse, AppError> {
    let logged_in = user.is_some();
    let shift = sqlx::query_as!(
        Shift,
        "SELECT s.* 
        FROM shift as s
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(app_state.pool())
    .await?;
    let worker = match shift.worker_id {
        Some(id) => Some(
            sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
                .fetch_one(app_state.pool())
                .await?,
        ),
        None => None,
    };

    Ok((
        [("HX-Replace-Url", "false")],
        ShiftTemplate {
            shift,
            logged_in,
            worker,
        },
    ))
}

pub async fn edit_form(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "SELECT s.*
        FROM shift as s
        WHERE s.id = $1
        ",
        id
    )
    .fetch_one(app_state.pool())
    .await?;

    Ok(([("HX-Replace-Url", "false")], ShiftEditTemplate { shift }))
}
