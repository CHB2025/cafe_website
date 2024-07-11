use axum::{routing::get, Router};

use list::worker_list;
pub use model::Worker;

mod list;
mod model;
mod shift_list;
mod view;

pub fn public_router() -> Router {
    Router::new()
        .route("/:id", get(view::view).patch(view::save))
        .route("/:id/details", get(view::details))
        .route("/:id/edit", get(view::edit))
        .route("/:id/shifts", get(shift_list::shift_list))
        .route("/:id/cancel_shift", get(shift_list::cancel_shift))
}

pub fn protected_router() -> Router {
    Router::new().route("/list", get(worker_list))
}
