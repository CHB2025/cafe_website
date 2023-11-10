use askama::Template;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    filters,
    models::{Shift, User},
};

#[derive(Debug, Template, Clone)]
#[template(path = "shift/view.html")]
pub struct ShiftTemplate {
    shift: Shift,
    logged_in: bool,
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
) -> Result<ShiftTemplate, AppError> {
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

    Ok(ShiftTemplate { shift, logged_in })
}

pub async fn edit_form(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ShiftEditTemplate, AppError> {
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

    Ok(ShiftEditTemplate { shift })
}
