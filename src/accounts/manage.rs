use askama::Template;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use cafe_website::{AppError, Redirect};
use uuid::Uuid;

use crate::{app_state::AppState, models::User};

#[derive(Template)]
#[template(path = "accounts/admin.html")]
pub struct UserAdminTempl {}

#[derive(Template)]
#[template(path = "accounts/user_list.html")]
pub struct UserListTempl {
    users: Vec<User>,
    current: User,
}

pub async fn admin() -> UserAdminTempl {
    UserAdminTempl {}
}

pub async fn user_list(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UserListTempl, AppError> {
    let Some(user) = user else { unreachable!() };
    let users = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(app_state.pool())
        .await?;
    Ok(UserListTempl {
        users,
        current: user,
    })
}

pub async fn remove_user(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    user: Option<User>,
) -> Result<impl IntoResponse, AppError> {
    let Some(user) = user else { unreachable!() };
    if id == user.id {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "You may not remove yourself",
        ));
    }
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(app_state.pool())
        .await?;
    Ok(Redirect::to("/account/manage".to_owned()))
}
