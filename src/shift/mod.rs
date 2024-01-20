use axum::{
    routing::{get, patch, put},
    Router,
};

use crate::app_state::AppState;

mod crud;
mod signup;
mod view;

use crud::{delete_shift, update_shift};
use view::{edit_form, view};

use self::crud::remove_worker;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:id", get(view)).route(
        "/:id/signup",
        get(signup::signup_form).patch(signup::signup),
    )
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:id/edit", get(edit_form))
        .route("/:id/remove_shift", patch(remove_worker))
        .route("/:id", put(update_shift).delete(delete_shift))
}
