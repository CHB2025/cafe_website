use askama::Template;
use axum::{routing::get, Router};
use cafe_website::AppError;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    filters,
    models::{Shift, Worker},
};

mod list;
mod model;
mod sender;

pub use model::{Email, EmailKind, EmailStatus};
pub use sender::send_all;

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
    app_state: &AppState,
    worker: Worker,
    shift: Shift,
) -> Result<Uuid, AppError> {
    let (recipient, event_id) = (worker.id, shift.event_id);
    let subject = format!("Thanks {}!", worker.name_first);

    let config = app_state.config();
    let mut base_url = config.website.base_url.clone();
    if config.website.port != 443 {
        base_url += ":";
        base_url += &config.website.port.to_string();
    };

    let message = SignupEmail {
        worker,
        shift,
        domain: base_url,
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
    .fetch_one(app_state.pool())
    .await?;
    Ok(id)
}
