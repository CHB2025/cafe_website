use askama::Template;

use askama_axum::IntoResponse;
use axum::{extract::Path, http::StatusCode};
use cafe_website::{AppError, Redirect};
use uuid::Uuid;

use crate::{config, models::User, session::Session};

#[derive(Template)]
#[template(path = "accounts/admin.html")]
pub struct UserAdminTempl {}

#[derive(Template)]
#[template(path = "accounts/user_list.html")]
pub struct UserListTempl {
    users: Vec<User>,
    current: Uuid,
}

pub async fn admin() -> UserAdminTempl {
    UserAdminTempl {}
}

pub async fn user_list(session: Session) -> Result<UserListTempl, AppError> {
    let Some(user) = session.user_id() else {
        unreachable!()
    };
    let users = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(config().pool())
        .await?;
    Ok(UserListTempl {
        users,
        current: user,
    })
}

pub async fn remove_user(
    Path(id): Path<Uuid>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let Some(user_id) = session.user_id() else {
        unreachable!()
    };
    if id == user_id {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "You may not remove yourself",
        ));
    }
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(config().pool())
        .await?;
    Ok(Redirect::to("/account/manage".to_owned()))
}
