use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Form,
};
use cafe_website::{filters, AppError, Redirect};
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{AdminInvite, User},
};

#[derive(Template)]
#[template(path = "email/messages/invite.html")]
struct InviteMessage {
    user: String,
    url: String,
}

#[derive(Template)]
#[template(path = "accounts/invite_list.html")]
pub struct InviteListTempl {
    invites: Vec<AdminInvite>,
}

pub async fn invite_list(State(app_state): State<AppState>) -> Result<InviteListTempl, AppError> {
    let invites = sqlx::query_as!(
        AdminInvite,
        "SELECT * from admin_invite WHERE accepted_at IS NULL"
    )
    .fetch_all(app_state.pool())
    .await?;
    Ok(InviteListTempl { invites })
}

#[derive(Deserialize)]
pub struct InviteForm {
    email: String,
}

pub async fn invite_user(
    State(app_state): State<AppState>,
    user: Option<User>,
    Form(invite): Form<InviteForm>,
) -> Result<impl IntoResponse, AppError> {
    let Some(user) = user else { unreachable!() };

    let em_rx = Regex::new(r#"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"#).expect("Email regex should be valid");
    if !em_rx.is_match(&invite.email) {
        return Err(AppError::inline(StatusCode::BAD_REQUEST, "Invalid email"));
    }

    // Start transaction
    let tran = app_state.pool().begin().await?;

    let existing = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", invite.email)
        .fetch_optional(app_state.pool())
        .await?;
    if existing.is_some() {
        return Err(AppError::inline(
            StatusCode::BAD_REQUEST,
            "This user already exists",
        ));
    }

    let id = sqlx::query_scalar!(
        "INSERT INTO admin_invite (email) VALUES ($1) RETURNING id",
        invite.email
    )
    .fetch_one(app_state.pool())
    .await?;

    // Email
    let message = InviteMessage {
        user: user.name.clone(),
        url: format!("{}/account/create/{}", app_state.config().url(), id),
    };
    sqlx::query!(
        "INSERT INTO email (status, kind, address, subject, message)
        VALUES ('pending', 'html', $1, $2, $3)",
        invite.email,
        format!(
            "{} has invited you to join the Cornerstone Cafe!",
            user.name
        ),
        message.render()?,
    )
    .execute(app_state.pool())
    .await?;

    tran.commit().await?;

    Ok(Redirect::to("/account/manage".to_owned()))
}

pub async fn cancel_invite(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!("DELETE FROM admin_invite WHERE id = $1", id)
        .execute(app_state.pool())
        .await?;
    Ok(Redirect::to("/account/manage".to_owned()))
}
