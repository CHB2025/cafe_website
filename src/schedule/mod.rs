mod add_shift;
mod block_view;

pub use add_shift::*;
use axum::{routing::get, Router};

use crate::app_state::AppState;

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/:id/add_shift", get(add_shift_form).post(add_shift))
}

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:id", get(block_view::schedule))
}
