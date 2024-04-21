use askama::Template;
use axum::{extract::Path, Form};
use axum_extra::extract::Cached;
use cafe_website::AppError;
use regex::Regex;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use crate::{config, session::Session, worker::Worker};

#[derive(Template)]
#[template(path = "worker/view.html")]
pub struct WorkerView {
    id: Uuid,
}

#[derive(Template)]
#[template(path = "worker/details.html")]
pub struct WorkerDetails {
    id: Uuid,
    name_first: String,
    name_last: String,
    email: String,
    phone: Option<String>,
    error: Option<&'static str>,
    edit: bool,
    is_admin: bool,
}

pub async fn view(Path(id): Path<Uuid>) -> Result<WorkerView, AppError> {
    let _ = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;

    Ok(WorkerView { id })
}

pub async fn details(
    session: Cached<Session>,
    Path(id): Path<Uuid>,
) -> Result<WorkerDetails, AppError> {
    let Worker {
        id,
        email,
        phone,
        name_first,
        name_last,
    } = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;

    Ok(WorkerDetails {
        id,
        name_first,
        name_last,
        email,
        phone,
        error: None,
        is_admin: session.is_authenticated(),
        edit: false,
    })
}

pub async fn edit(
    session: Cached<Session>,
    Path(id): Path<Uuid>,
) -> Result<WorkerDetails, AppError> {
    let Worker {
        id,
        email,
        phone,
        name_first,
        name_last,
    } = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;

    Ok(WorkerDetails {
        id,
        name_first,
        name_last,
        email,
        phone,
        is_admin: session.is_authenticated(),
        error: None,
        edit: true,
    })
}

#[derive(Deserialize)]
pub struct WorkerEdit {
    name_first: String,
    name_last: String,
    email: String,
    phone: Option<String>,
}

pub async fn save(
    session: Cached<Session>,
    Path(id): Path<Uuid>,
    Form(req): Form<WorkerEdit>,
) -> Result<WorkerDetails, AppError> {
    let Worker {
        id: _,
        email,
        phone,
        name_first,
        name_last,
    } = sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
        .fetch_one(config().pool())
        .await?;
    if !session.is_authenticated() && email != req.email {
        return Ok(WorkerDetails {
            id,
            name_first,
            name_last,
            email,
            phone,
            error: Some("Unable to change email address. Contact an admin to change."),
            is_admin: session.is_authenticated(),
            edit: false,
        });
    }

    const PHONE_REGEX: &str = r#"^[2-9][0-9]{2}-[2-9][0-9]{2}-[0-9]{4}$"#;
    const EMAIL_REGEX: &str = r#"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"#;
    let em_rx = Regex::new(EMAIL_REGEX).expect("Email regex should be valid");
    let email_match = em_rx.is_match(&req.email);
    if !email_match {
        return Ok(WorkerDetails {
            id,
            name_first,
            name_last,
            email,
            phone,
            error: Some("Invalid email"),
            edit: true,
            is_admin: session.is_authenticated(),
        });
    }
    let phone_match = match req.phone.as_deref() {
        None | Some("") => true,
        Some(ph) => {
            debug!(?ph);
            let ph_rx = Regex::new(PHONE_REGEX).expect("Phone regex should be valid");
            ph_rx.is_match(ph)
        }
    };
    if !phone_match {
        return Ok(WorkerDetails {
            id,
            name_first,
            name_last,
            email,
            phone,
            error: Some("Invalid phone number"),
            edit: true,
            is_admin: session.is_authenticated(),
        });
    }

    let Worker {
        id,
        email,
        phone,
        name_first,
        name_last,
    } = sqlx::query_as!(
        Worker,
        "UPDATE worker 
        SET name_first = $1, name_last = $2, email = $3, phone = $4
        WHERE id = $5
        returning *",
        req.name_first,
        req.name_last,
        req.email,
        req.phone.filter(|ph| !ph.is_empty()),
        id
    )
    .fetch_one(config().pool())
    .await?;

    Ok(WorkerDetails {
        id,
        name_first,
        name_last,
        email,
        phone,
        is_admin: session.is_authenticated(),
        error: None,
        edit: false,
    })
}
