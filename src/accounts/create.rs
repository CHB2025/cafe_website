use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html, Form};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{self, PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;

use crate::models::User;
use crate::utils;
use crate::{app_state::AppState, models::CreateUser};

#[derive(Template)]
#[template(path = "accounts/create.html")]
pub struct AccountCreateTemplate {}

pub async fn account_creation_form() -> AccountCreateTemplate {
    AccountCreateTemplate {}
}

pub async fn create_account(
    State(app_state): State<AppState>,
    Form(mut user): Form<CreateUser>,
) -> Result<Html<&'static str>, (StatusCode, Html<&'static str>)> {
    // TODO: add path wildcard and Hashmap/database table for invitations

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> password_hash::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let conn = app_state.pool();
    let already_exists = sqlx::query!("SELECT id FROM users WHERE email = $1", user.email)
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

    let _new_user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *",
        user.name,
        user.email,
        user.password
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;

    // TODO: Create session for new user

    Ok(Html("<span class=\"success\" hx-get=\"/\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>"))
}
