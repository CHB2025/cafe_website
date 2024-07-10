use askama::Template;
use axum::{
    extract::{Query, RawQuery},
    http::StatusCode,
    response::IntoResponse,
    Form,
};
use cafe_website::{templates::Card, AppError, Redirect};
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use tracing::info;

use crate::{config, models::User, session::Session};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    from: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

fn login_err<E>(_: E) -> AppError {
    AppError::inline(StatusCode::BAD_REQUEST, "Invalid username or password")
}

#[derive(Template)]
#[template(path = "accounts/login.html")]
struct LoginTemplate {
    query: String,
}

pub async fn login_form(RawQuery(query): RawQuery) -> impl IntoResponse {
    Card {
        class: Some("w-fit"),
        title: "Log In".to_owned(),
        child: LoginTemplate {
            query: query.unwrap_or("".to_string()),
        },
        show_x: false,
    }
}

pub async fn login(
    session: Session,
    Query(params): Query<LoginParams>,
    Form(login): Form<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let con = config().pool();

    if session.is_authenticated() {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "Already authenticated",
        ));
    }

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", login.email)
        .fetch_one(con)
        .await
        .map_err(login_err)?;

    // Check password
    let pw_hash = user.password.clone();
    spawn_blocking(move || {
        let hash = PasswordHash::new(&pw_hash)?;
        Scrypt
            .verify_password(login.password.as_bytes(), &hash)
            .map_err(login_err)
    })
    .await??;

    let user_name = user.name.clone();

    session.set_auth_user(user).await?;

    info!("{} logged in", user_name);

    Ok((
        [("HX-Trigger", "auth-change".to_owned())],
        Redirect::to(params.from.unwrap_or("/".to_string())),
    ))
}

pub async fn logout(session: Session) -> Result<impl IntoResponse, AppError> {
    session.remove_auth_user().await?;
    Ok((
        [("HX-Trigger", "auth-change")],
        Redirect::to("/".to_string()),
    ))
}
