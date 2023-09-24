use askama::Template;
use axum::{
    extract::{Query, RawQuery, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use tracing::log::debug;

use crate::{app_state::AppState, error::AppError, models::User};

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

    let name = user.name.clone();
    //session.insert("user", user).expect("serializable");
    let session = Cookie::new("session", user.id.to_string());
    // Set secure, samesite, and expires
    debug!("{}", session.to_string());
    let cookie_jar = cookie_jar.add(session);

    Ok((
        cookie_jar,
        // ["HX-Trigger", "auth-change"],
        Html(format!(
            r##"<span class="success" hx-get="{path}" hx-trigger="load delay:1s" hx-target="#content" hx-push-url="true">Welcome {name}</span>"##,
            path = params.from.unwrap_or("/".to_string()),
        )),
    ))
}

pub async fn logout(mut cookie_jar: PrivateCookieJar) -> impl IntoResponse {
    // session.remove("user");
    // session.destroy();
    if let Some(session) = cookie_jar.get("session") {
        cookie_jar = cookie_jar.remove(session);
    }
    (
        cookie_jar,
        Html(
            r##"<span hx-get="/" hx-trigger="load" hx-target="#content" hx-push-url="true"></span>"##,
        ),
    )
}
