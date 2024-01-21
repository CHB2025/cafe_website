use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::State, http::StatusCode, Form};
use axum_extra::extract::PrivateCookieJar;
use cafe_website::{AppError, Redirect};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{self, PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;

use crate::models::User;
use crate::session::create_session;
use crate::{app_state::AppState, models::CreateUser};

#[derive(Template)]
#[template(path = "accounts/create.html")]
pub struct AccountCreateTemplate {}

pub async fn account_creation_form() -> AccountCreateTemplate {
    AccountCreateTemplate {}
}

pub async fn create_account(
    State(app_state): State<AppState>,
    cookie_jar: PrivateCookieJar,
    Form(mut user): Form<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: add path wildcard and Hashmap/database table for invitations

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> password_hash::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let conn = app_state.pool();
    let already_exists = sqlx::query!("SELECT id FROM users WHERE email = $1", user.email)
        .fetch_optional(conn)
        .await?;
    if already_exists.is_some() {
        pwd_fut.abort();
        return Err(AppError::inline(
            StatusCode::CONFLICT,
            "Email already taken",
        ));
    }

    user.password = pwd_fut.await??;

    let new_user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *",
        user.name,
        user.email,
        user.password
    )
    .fetch_one(conn)
    .await?;

    Ok((
        create_session(cookie_jar, new_user.id),
        Redirect::to("/".to_owned()),
    ))
}
