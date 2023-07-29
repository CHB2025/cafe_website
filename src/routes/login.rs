use axum::{extract::State, http::StatusCode, response::Html, Form};
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::{app_state::AppState, models::User, utils};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

fn login_err<E>(_: E) -> (StatusCode, Html<&'static str>) {
    (
        StatusCode::FORBIDDEN,
        Html("<span class=\"error\">Invalid username or password.</span>"),
    )
}

pub async fn login(
    State(app_state): State<AppState>,
    Form(login): Form<LoginRequest>,
) -> Result<Html<&'static str>, (StatusCode, Html<&'static str>)> {
    // What do we do if user is already logged in?
    // get user
    let con = app_state.pool();
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", login.email)
        .fetch_one(con)
        .await
        .map_err(login_err)?;

    // Check password
    spawn_blocking(move || {
        let hash = PasswordHash::new(&user.password).map_err(utils::ise)?;
        Scrypt
            .verify_password(login.password.as_bytes(), &hash)
            .map_err(login_err)
    })
    .await
    .map_err(utils::ise)??;
    // TODO: create user session
    Ok(Html("Success"))
}
