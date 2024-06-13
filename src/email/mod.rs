use askama::Template;
use axum::{routing::get, Router};
use cafe_website::{filters, AppError};
use uuid::Uuid;

use crate::config::Admin;
use crate::worker::Worker;
use crate::{config, models::Shift};

mod list;
mod model;
mod sender;

pub use model::{Email, EmailKind, EmailStatus};
pub use sender::send_all;

// Verify emails? anything else?
pub fn public_router() -> Router {
    Router::new()
}

pub fn protected_router() -> Router {
    Router::new().route("/list", get(list::email_list))
}

#[derive(Template)]
#[template(path = "email/messages/signup.html")]
pub struct SignupEmail {
    worker: Worker,
    shift: Shift,
    domain: String,
    admin: &'static Admin,
}

pub async fn send_signup(worker: Worker, shift: Shift) -> Result<Uuid, AppError> {
    let (recipient, event_id, address) = (worker.id, shift.event_id, worker.email.clone());
    let subject = format!("Thanks {}!", worker.name_first); // injection?

    let message = SignupEmail {
        worker,
        shift,
        domain: config().url(),
        admin: &config().admin,
    }
    .render()?;

    let id = sqlx::query_scalar!(
        "INSERT INTO email (status, kind, recipient, address, subject, message, event_id)
        VALUES ('pending', 'html', $1, $2, $3, $4, $5) RETURNING id",
        recipient,
        address,
        subject,
        message,
        event_id
    )
    .fetch_one(config().pool())
    .await?;
    Ok(id)
}
