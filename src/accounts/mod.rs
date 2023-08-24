use axum::{routing::get, Router};

mod create;
mod login;

use create::{account_creation_form, create_account};
pub use login::{login, login_form, logout};

use crate::app_state::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/create", get(account_creation_form).post(create_account))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
}
