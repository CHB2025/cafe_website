use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::{http::StatusCode, Form};
use cafe_website::templates::Card;
use cafe_website::{AppError, Redirect};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{self, PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::config;
use crate::models::User;
use crate::models::{AdminInvite, CreateUser};
use crate::session::Session;

#[derive(Template)]
#[template(path = "accounts/create.html")]
pub struct AccountCreateTemplate {
    invite_id: Uuid,
}

pub async fn account_creation_form(
    Path(invite_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query_as!(
        AdminInvite,
        "SELECT * FROM admin_invite WHERE accepted_at IS NULL AND id = $1",
        invite_id
    )
    .fetch_one(config().pool())
    .await?;

    Ok(Card {
        child: AccountCreateTemplate { invite_id },
        class: Some("w-fit"),
        title: "Create Account".to_owned(),
        show_x: false,
    })
}

pub async fn create_account(
    mut session: Session,
    Path(invite_id): Path<Uuid>,
    Form(mut user): Form<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let transaction = config().pool().begin().await?;

    let invite = sqlx::query_as!(
        AdminInvite,
        "UPDATE admin_invite SET accepted_at = now() WHERE accepted_at IS NULL AND id = $1 RETURNING *",
        invite_id
    )
    .fetch_one(config().pool())
    .await?;

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> password_hash::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let already_exists = sqlx::query!("SELECT id FROM users WHERE email = $1", invite.email)
        .fetch_optional(config().pool())
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
    .fetch_one(config().pool())
    .await?;

    session.set_auth_user(new_user);

    transaction.commit().await?;

    Ok((
        // create_session(cookie_jar, new_user.id),
        [("HX-Tritter", "auth-change")],
        session,
        Redirect::to("/".to_owned()),
    ))
}
