use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_sessions::extractors::{ReadableSession, WritableSession};
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
    mut session: WritableSession,
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
    let pw_hash = user.password.clone();
    spawn_blocking(move || {
        let hash = PasswordHash::new(&pw_hash).map_err(utils::ise)?;
        Scrypt
            .verify_password(login.password.as_bytes(), &hash)
            .map_err(login_err)
    })
    .await
    .map_err(utils::ise)??;

    session.insert("user", user).expect("serializable");
    Ok(Html("<span class=\"success\" hx-get=\"/\" hx-trigger=\"load delay:1s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>"))
}

pub async fn logout(mut session: WritableSession) -> impl IntoResponse {
    session.destroy();
    Redirect::to("/login")
}

pub async fn login_button(session: ReadableSession) -> Html<String> {
    let (href, text) = match session.get::<User>("user") {
        Some(_) => ("/logout", "Log Out"),
        None => ("/login", "Log In"),
    };
    Html(format!(
        "<a class=\"nav-item button\" href=\"{}\" hx-boost=\"true\">{}</a>",
        href, text
    ))
}
