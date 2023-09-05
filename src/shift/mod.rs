use axum::{
    routing::{get, patch},
    Router,
};

use crate::app_state::AppState;

mod crud;
mod view;

use crud::{delete_shift, update_shift};
use view::{edit_form, view};

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:id", get(view))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:id/edit", get(edit_form))
        .route("/:id", patch(update_shift).delete(delete_shift))
}
