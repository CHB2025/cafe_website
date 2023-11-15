use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    email,
    error::AppError,
    models::{Shift, User, Worker},
};

#[derive(Template)]
#[template(path = "shift/signup.html")]
pub struct SignupFormTemplate {
    worker: Option<Worker>,
}

#[derive(Deserialize)]
pub struct SignupFormParams {
    email: String,
}

#[derive(Deserialize)]
pub struct SignupBody {
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
}

pub async fn signup_form(
    State(app_state): State<AppState>,
    Query(SignupFormParams { email }): Query<SignupFormParams>,
) -> Result<SignupFormTemplate, AppError> {
    let worker = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE email = $1", email)
        .fetch_optional(app_state.pool())
        .await?;

    Ok(SignupFormTemplate { worker })
}

pub async fn signup(
    State(app_state): State<AppState>,
    user: Option<User>,
    Path(id): Path<Uuid>,
    Form(body): Form<SignupBody>,
) -> Result<Html<String>, AppError> {
    // TODO: validate email and phone

    let logged_in = user.is_some();
    let tran = app_state.pool().begin().await?;
    let shift = sqlx::query_as!(Shift, "SELECT * FROM shift WHERE id = $1", id)
        .fetch_one(app_state.pool())
        .await?;

    // Prevent races (except when logged in. Hard to prevent without etags when logged in)
    // Probably should create a separate method for replacing a worker on a shift which takes the old workers id and
    // verifies that it is still the same before switching.
    if !logged_in && shift.worker_id.is_some() {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "This shift is already filled",
        ));
    }

    let worker = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE email = $1", body.email)
        .fetch_optional(app_state.pool())
        .await?;
    let worker = match worker {
        Some(w) => {
            // Check any overlapping shifts
            let overlaps = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM shift 
                WHERE event_id = $1
                AND date = $2 
                AND worker_id = $3
                AND (
                    start_time < $4 AND end_time > $5
                )
                ",
                shift.event_id,
                shift.date,
                w.id,
                shift.end_time,
                shift.start_time,
            )
            .fetch_one(app_state.pool())
            .await?;
            if overlaps.is_some_and(|c| c != 0) {
                return Err(AppError::inline(
                    StatusCode::BAD_REQUEST,
                    "You are already signed up for a shift during this one",
                ));
            }
            w
        }
        None => {
            // Create a new worker
            sqlx::query_as!(
                Worker,
                "INSERT INTO worker (email, name_first, name_last, phone) VALUES ($1, $2, $3, $4) RETURNING *",
                body.email,
                body.first_name.ok_or(AppError::inline(StatusCode::BAD_REQUEST, "Enter a first name"))?, 
                body.last_name.ok_or(AppError::inline(StatusCode::BAD_REQUEST, "Enter a last name"))?, 
                body.phone
            ).fetch_one(app_state.pool()).await?
        }
    };

    let (worker_name, worker_id) = (worker.name_first.clone(), worker.id);

    // Send email
    let _ = email::send_signup(app_state.pool(), worker, shift).await?;

    sqlx::query!(
        "UPDATE shift SET worker_id = $1 WHERE id = $2",
        worker_id,
        id
    )
    .execute(app_state.pool())
    .await?;

    tran.commit().await?;

    Ok(Html(format!(
        r##"<span class="Success" hx-get="/" hx-trigger="load:delay 2s" hx-target="#content">Thanks for signing up {}!</span>"##,
        worker_name
    )))
}
