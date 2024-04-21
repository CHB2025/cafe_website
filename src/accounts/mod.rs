use axum::{
    routing::{delete, get},
    Router,
};

mod create;
mod invite;
mod login;
mod manage;

use create::{account_creation_form, create_account};
pub use login::{login, login_form, logout};

pub fn public_router() -> Router {
    Router::new().route(
        "/create/:id",
        get(account_creation_form).post(create_account),
    )
}

pub fn protected_router() -> Router {
    Router::new()
        .route("/manage", get(manage::admin))
        .route("/users", get(manage::user_list))
        .route(
            "/invites",
            get(invite::invite_list).post(invite::invite_user),
        )
        .route("/invites/:id", delete(invite::cancel_invite))
        .route("/:id", delete(manage::remove_user))
}
