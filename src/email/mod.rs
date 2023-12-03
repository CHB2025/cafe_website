use askama::Template;
use axum::{routing::get, Router};
use cafe_website::AppError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    filters,
    models::{Shift, Worker},
};

mod list;
mod model;
pub use model::{Email, EmailKind, EmailStatus};

// Verify emails? anything else?
pub fn public_router() -> Router<AppState> {
    Router::new()
}

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/list", get(list::email_list))
}

#[derive(Template)]
#[template(path = "email/messages/signup.html")]
pub struct SignupEmail {
    worker: Worker,
    shift: Shift,
    domain: String,
}

pub async fn send_signup(
    pool: &Pool<Postgres>,
    worker: Worker,
    shift: Shift,
) -> Result<Uuid, AppError> {
    let (recipient, event_id) = (worker.id, shift.event_id);
    let subject = format!("Thanks {}!", worker.name_first);
    let message = SignupEmail {
        worker,
        shift,
        domain: "https://chbarch.local:3000".to_string(), // Need to set this via config somehow
    }
    .render()?;

    let id = sqlx::query_scalar!(
        "INSERT INTO email (status, kind, recipient, subject, message, event_id) 
        VALUES ('pending', 'html', $1, $2, $3, $4) RETURNING id",
        recipient,
        subject,
        message,
        event_id
    )
    .fetch_one(pool)
    .await?;
    Ok(id)
}
