use axum::{extract::State, http::StatusCode, response::Html, Form};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;

use crate::utils;
use crate::{app_state::AppState, models::CreateUser};

pub async fn signup(
    State(app_state): State<AppState>,
    Form(mut user): Form<CreateUser>,
) -> Result<Html<&'static str>, (StatusCode, Html<&'static str>)> {
    // TODO: add path wildcard and Hashmap/database table for invitations

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> anyhow::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let conn = app_state.pool();
    let already_exists = sqlx::query!("SELECT * FROM users WHERE email = $1", user.email)
        .fetch_optional(conn)
        .await
        .map_err(utils::ise)?;
    if already_exists.is_some() {
        pwd_fut.abort();
        return Err((
            StatusCode::CONFLICT,
            Html("<span class=\"error\">Email already taken</span>"),
        ));
    }

    user.password = pwd_fut.await.map_err(utils::ise)?.map_err(utils::ise)?;

    let new_user = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id",
        user.name,
        user.email,
        user.password
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;
    println!("Created new user: {:?}", new_user);

    // TODO: create session for the new user

    Ok(Html("Created User!"))
}
