mod add_shift;
mod create;
mod schedule;

pub use add_shift::*;
use axum::{routing::get, Router};
pub use create::*;

use crate::app_state::AppState;

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/create", get(create_day_form).post(create_day))
        .route("/:id/schedule", get(schedule::schedule))
        .route("/:id/add_shift", get(add_shift_form).post(add_shift))
}
