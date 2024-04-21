use axum::{
    routing::{get, patch, put},
    Router,
};

mod crud;
mod signup;
mod view;

use crud::{delete_shift, update_shift};
use view::{edit_form, view};

use self::crud::remove_worker;

pub fn public_router() -> Router {
    Router::new().route("/:id", get(view)).route(
        "/:id/signup",
        get(signup::signup_form).patch(signup::signup),
    )
}

pub fn protected_router() -> Router {
    Router::new()
        .route("/:id/edit", get(edit_form))
        .route("/:id/remove_worker", patch(remove_worker))
        .route("/:id", put(update_shift).delete(delete_shift))
}
