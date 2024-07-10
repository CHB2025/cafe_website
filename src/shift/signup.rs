use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Form,
};
use cafe_website::{filters, AppError};
use regex::Regex;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::{config, worker::Worker};
use crate::{email, models::Shift};

const PHONE_REGEX: &str = r#"^[2-9][0-9]{2}-[2-9][0-9]{2}-[0-9]{4}$"#;

#[derive(Template)]
#[template(path = "shift/signup.html")]
pub enum SignupForm {
    Empty(Shift),
    Known(Shift, Worker, Option<&'static str>),
    Unknown {
        shift: Shift,
        email: String,
        first_name: Option<String>,
        last_name: Option<String>,
        phone: Option<String>,
        error: Option<&'static str>,
    },
    Message(Shift, String),
}

#[derive(Deserialize)]
pub struct SignupFormParams {
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
}

#[derive(Deserialize)]
pub struct SignupBody {
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
}

pub async fn signup_form(
    Path(id): Path<Uuid>,
    Query(params): Query<SignupFormParams>,
) -> Result<SignupForm, AppError> {
    let shift = sqlx::query_as!(Shift, "SELECT * FROM shift WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;
    let worker = sqlx::query_as!(
        Worker,
        "SELECT * FROM worker WHERE email = $1",
        params.email
    )
    .fetch_optional(config().pool())
    .await?;

    Ok(match (params.email, worker) {
        (_, Some(worker)) => SignupForm::Known(shift, worker, None),
        (None, None) => SignupForm::Empty(shift),
        (Some(email), None) => SignupForm::Unknown {
            shift,
            email,
            first_name: params.first_name,
            last_name: params.last_name,
            phone: params.phone,
            error: None,
        },
    })
}

pub async fn signup(
    Path(id): Path<Uuid>,
    Form(body): Form<SignupBody>,
) -> Result<SignupForm, AppError> {
    let tran = config().pool().begin().await?;
    let shift = sqlx::query_as!(Shift, "SELECT * FROM shift WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;

    // Prevent races
    if shift.worker_id.is_some() {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "This shift is already filled",
        ));
    }

    let worker = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE email = $1", body.email)
        .fetch_optional(config().pool())
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
            .fetch_one(config().pool())
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
            let em_rx = Regex::new(r#"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"#).expect("Email regex should be valid");
            let email_match = em_rx.is_match(&body.email);
            if !email_match {
                return Ok(SignupForm::Unknown {
                    shift,
                    email: body.email,
                    first_name: body.first_name,
                    last_name: body.last_name,
                    phone: body.phone,
                    error: Some("Invalid email"),
                });
            }
            let phone_match = match body.phone.as_deref() {
                None | Some("") => true,
                Some(ph) => {
                    let ph_rx = Regex::new(PHONE_REGEX).expect("Phone regex should be valid");
                    ph_rx.is_match(ph)
                }
            };
            if !phone_match {
                return Ok(SignupForm::Unknown {
                    shift,
                    email: body.email,
                    first_name: body.first_name,
                    last_name: body.last_name,
                    phone: body.phone,
                    error: Some("Invalid phone number"),
                });
            }

            sqlx::query_as!(
                Worker,
                "INSERT INTO worker (email, name_first, name_last, phone) VALUES ($1, $2, $3, $4) RETURNING *",
                body.email,
                body.first_name.ok_or(AppError::inline(StatusCode::BAD_REQUEST, "Enter a first name"))?, 
                body.last_name.ok_or(AppError::inline(StatusCode::BAD_REQUEST, "Enter a last name"))?, 
                body.phone.filter(|s| !s.is_empty())
            ).fetch_one(config().pool()).await?
        }
    };

    let (worker_name, worker_last, worker_id) = (
        worker.name_first.clone(),
        worker.name_last.clone(),
        worker.id,
    );

    // Send email
    let _ = email::send_signup(worker, shift.clone()).await?;

    sqlx::query!(
        "UPDATE shift SET worker_id = $1 WHERE id = $2",
        worker_id,
        id
    )
    .execute(config().pool())
    .await?;

    tran.commit().await?;

    info!(
        "{} {} signed up: {} {}-{} on {}",
        worker_name,
        worker_last,
        shift.title,
        filters::time_short(&shift.start_time).expect("Infallible"),
        filters::time_short(&shift.end_time).expect("Infallible"),
        filters::date_short(&shift.date).expect("Infallible"),
    );

    Ok(SignupForm::Message(
        shift,
        format!("Thanks for signing up, {}", worker_name),
    ))
}
