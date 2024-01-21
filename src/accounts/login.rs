use askama::Template;
use axum::{
    extract::{Query, RawQuery, State},
    http::StatusCode,
    response::IntoResponse,
    Form,
};
use axum_extra::extract::PrivateCookieJar;
use cafe_website::{AppError, Redirect};
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::{
    app_state::AppState,
    models::User,
    session::{create_session, destroy_session},
};

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
    LoginTemplate {
        query: query.unwrap_or("".to_string()),
    }
}

pub async fn login(
    State(app_state): State<AppState>,
    cookie_jar: PrivateCookieJar,
    Query(params): Query<LoginParams>,
    Form(login): Form<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let con = app_state.pool();
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

    Ok((
        [("HX-Trigger", "auth-change".to_owned())],
        create_session(cookie_jar, user.id),
        Redirect::to(params.from.unwrap_or("/".to_string())),
    ))
}

pub async fn logout(cookie_jar: PrivateCookieJar) -> impl IntoResponse {
    (
        [("HX-Trigger", "auth-change")],
        destroy_session(cookie_jar),
        Redirect::to("/".to_string()),
    )
}
