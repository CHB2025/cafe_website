use axum::{routing::get, Router};

use crate::app_state::AppState;

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
