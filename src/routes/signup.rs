use axum::{extract::State, http::StatusCode, response::Html, Form};
use axum_sessions::extractors::WritableSession;
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;

use crate::models::User;
use crate::utils;
use crate::{app_state::AppState, models::CreateUser};

pub async fn signup_form() -> Html<String> {
    Html(r##"
        <form class="card form" action="/signup" method="post" hx-boost="true" hx-params="not password_repeat" hx-target="#signup_results" hx-indicator="#signup-submit">
          <div class="form-item">
            <label>Name:</label>
            <input type="text" name="name" required="true"></input>
          </div>
          <div class="form-item">
            <label>Email:</label>
            <input name="email" type="email" required="true"></input>
          </div>
          <div class="form-item">
            <label>Password:</label>
            <input 
              _="
                on htmx:validation:validate
                  if my.value != the value of the next <input/>
                    call me.setCustomValidity('Passwords must match')
                  else 
                    call me.setCustomValidity('')
                  end
                end
              " 
              name="password" 
              type="password" 
              required="true">
            </input>
          </div>
          <div class="form-item">
            <label>Password:</label>
            <input name="password_repeat" type="password" required="true"></input>
          </div>
          <div class="form-item">
            <button id="signup-submit" type="submit">Submit</button>
          </div>
          <div id="signup_results" class="form-item"></div>
        </form>
    "##.to_string())
}

pub async fn signup(
    mut session: WritableSession,
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

    let new_user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *",
        user.name,
        user.email,
        user.password
    )
    .fetch_one(conn)
    .await
    .map_err(utils::ise)?;

    session.insert("user", new_user).expect("serializable");

    Ok(Html("<span class=\"success\" hx-get=\"/\" hx-trigger=\"load delay:2s\" hx-target=\"#content\" hx-push-url=\"true\">Success</span>"))
}
