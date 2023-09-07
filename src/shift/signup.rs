use askama::Template;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, models::Shift};

#[derive(Template)]
#[template(path = "shift/signup.html")]
pub struct SignupFormTemplate {
    shift: Shift,
    show_all: bool,
    email: Option<String>,
}

#[derive(Deserialize)]
pub struct SignupFormParams {
    email: Option<String>,
}

pub async fn signup_form(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(SignupFormParams { email }): Query<SignupFormParams>,
) -> Result<SignupFormTemplate, AppError> {
    let shift = sqlx::query_as!(Shift, "SELECT * FROM shift WHERE id = $1", id)
        .fetch_one(app_state.pool())
        .await?;

    let show_all = if let Some(email) = email.clone() {
        sqlx::query_scalar!("SELECT Count(*) FROM worker WHERE email = $1", email)
            .fetch_one(app_state.pool())
            .await?
            .unwrap_or(0)
            == 0
    } else {
        false
    };

    Ok(SignupFormTemplate {
        shift,
        show_all,
        email,
    })
}
