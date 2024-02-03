use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Form};
use axum_extra::extract::PrivateCookieJar;
use cafe_website::templates::Card;
use cafe_website::{AppError, Redirect};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{self, PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::models::User;
use crate::session::create_session;
use crate::{
    app_state::AppState,
    models::{AdminInvite, CreateUser},
};

#[derive(Template)]
#[template(path = "accounts/create.html")]
pub struct AccountCreateTemplate {
    invite_id: Uuid,
}

pub async fn account_creation_form(
    State(app_state): State<AppState>,
    Path(invite_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query_as!(
        AdminInvite,
        "SELECT * FROM admin_invite WHERE accepted_at IS NULL AND id = $1",
        invite_id
    )
    .fetch_one(app_state.pool())
    .await?;

    Ok(Card {
        child: AccountCreateTemplate { invite_id },
        class: Some("w-fit"),
        title: "Create Account".to_owned(),
        show_x: false,
    })
}

pub async fn create_account(
    State(app_state): State<AppState>,
    cookie_jar: PrivateCookieJar,
    Path(invite_id): Path<Uuid>,
    Form(mut user): Form<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let transaction = app_state.pool().begin().await?;

    let invite = sqlx::query_as!(
        AdminInvite,
        "UPDATE admin_invite SET accepted_at = now() WHERE accepted_at IS NULL AND id = $1 RETURNING *",
        invite_id
    )
    .fetch_one(app_state.pool())
    .await?;

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> password_hash::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let already_exists = sqlx::query!("SELECT id FROM users WHERE email = $1", invite.email)
        .fetch_optional(app_state.pool())
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
        invite.email,
        user.password
    )
    .fetch_one(app_state.pool())
    .await?;

    transaction.commit().await?;

    Ok((
        create_session(cookie_jar, new_user.id),
        Redirect::to("/".to_owned()),
    ))
}
