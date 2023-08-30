use askama::Template;
use axum::{
    extract::{Query, RawQuery, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use axum_sessions::extractors::WritableSession;
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use crate::{app_state::AppState, models::User, utils};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    from: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

fn login_err<E>(_: E) -> (StatusCode, Html<&'static str>) {
    (
        StatusCode::BAD_REQUEST,
        Html("<span class=\"error\">Invalid username or password.</span>"),
    )
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
    mut session: WritableSession,
    State(app_state): State<AppState>,
    Query(params): Query<LoginParams>,
    Form(login): Form<LoginRequest>,
) -> Result<Html<String>, (StatusCode, Html<&'static str>)> {
    let con = app_state.pool();
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", login.email)
        .fetch_one(con)
        .await
        .map_err(login_err)?;

    // Check password
    let pw_hash = user.password.clone();
    spawn_blocking(move || {
        let hash = PasswordHash::new(&pw_hash).map_err(utils::ise)?;
        Scrypt
            .verify_password(login.password.as_bytes(), &hash)
            .map_err(login_err)
    })
    .await
    .map_err(utils::ise)??;

    let name = user.name.clone();
    session.insert("user", user).expect("serializable");

    Ok(Html(format!(
        r##"<span class="success" hx-get="{path}" hx-trigger="load delay:1s" hx-target="#content" hx-push-url="true">Welcome {name}</span>"##,
        path = params.from.unwrap_or("/".to_string()),
    )))
}

pub async fn logout(mut session: WritableSession) -> impl IntoResponse {
    session.remove("user");
    session.destroy();
    Html(r##"<span hx-get="/" hx-trigger="load" hx-target="#content" hx-push-url="true"></span>"##)
}
