use axum::{extract::State, http::StatusCode, response::Html, Form};
use diesel::dsl::exists;
use diesel::{insert_into, prelude::*, select};
use scrypt::password_hash::rand_core::OsRng;
use scrypt::password_hash::{PasswordHasher, SaltString};
use scrypt::Scrypt;
use tokio::task::spawn_blocking;

use crate::schema::users::dsl::*;
use crate::utils;
use crate::{app_state::AppState, models::CreateUser};

pub async fn signup(
    State(app_state): State<AppState>,
    Form(mut user): Form<CreateUser>,
) -> Result<Html<&'static str>, (StatusCode, Html<&'static str>)> {
    // TODO: add path wildcard and Hashmap/database table for invitations

    let pswd = user.password.clone();
    let pwd_fut = spawn_blocking(move || -> anyhow::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Scrypt.hash_password(pswd.as_bytes(), &salt)?.to_string())
    });

    let mut conn = app_state.db_connection().map_err(utils::ise)?;
    let already_exists = select(exists(users.filter(email.eq(&user.email))))
        .get_result::<bool>(&mut conn)
        .map_err(utils::ise)?;
    if already_exists {
        pwd_fut.abort();
        return Err((
            StatusCode::CONFLICT,
            Html("<span class=\"error\">Email already taken</span>"),
        ));
    }

    user.password = pwd_fut.await.map_err(utils::ise)?.map_err(utils::ise)?;

    insert_into(users)
        .values(&user)
        .execute(&mut conn)
        .map_err(utils::ise)?;

    // TODO: create session for the new user

    Ok(Html("Created User!"))
}
