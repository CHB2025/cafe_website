use std::fmt;

use askama::Template;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, pagination::PaginatedQuery};

use super::{Email, EmailKind, EmailStatus};

#[derive(Template, Clone, PartialEq, Eq, Hash)]
#[template(path = "email/list.html")]
pub struct EmailListTemplate {
    emails: Vec<Email>,
    pagination: PaginatedQuery<EmailOrderBy>,
    query: EmailQuery,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailOrderBy {
    #[default]
    CreatedAt,
    SentAt,
    Status,
}

impl fmt::Display for EmailOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CreatedAt => "created_at",
            Self::SentAt => "sent_at",
            Self::Status => "status",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default, Debug)]
pub struct EmailQuery {
    recipient: Option<Uuid>,
    status: Option<EmailStatus>, // Will this work?
}

impl fmt::Display for EmailQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).unwrap_or(String::new());
        write!(f, "{}", s)
    }
}

pub async fn email_list(
    State(app_state): State<AppState>,
    Query(
        pagination @ PaginatedQuery {
            order_by,
            order_dir,
            take,
            skip,
        },
    ): Query<PaginatedQuery<EmailOrderBy>>,
    Query(query): Query<EmailQuery>,
) -> Result<EmailListTemplate, AppError> {
    let emails: Vec<Email> = sqlx::query_as(&format!(
        "SELECT * FROM email
        ORDER BY {order_by} {order_dir} LIMIT $1 OFFSET $2"
    ))
    .bind(take)
    .bind(skip)
    .fetch_all(app_state.pool())
    .await?;
    Ok(EmailListTemplate {
        emails,
        pagination,
        query,
    })
}
