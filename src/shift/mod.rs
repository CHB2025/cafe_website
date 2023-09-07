use axum::{
    routing::{get, put},
    Router,
};

use crate::app_state::AppState;

mod crud;
mod signup;
mod view;

use crud::{delete_shift, update_shift};
use view::{edit_form, view};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(view))
        .route("/:id/signup", get(signup::signup_form))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:id/edit", get(edit_form))
        .route("/:id", put(update_shift).delete(delete_shift))
}
