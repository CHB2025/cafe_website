use std::fmt;

use askama::Template;
use axum::extract::{Query, State};
use cafe_website::{filters, AppError, PaginatedQuery};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use tracing::debug;
use uuid::Uuid;

use crate::app_state::AppState;

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
    event_id: Option<Uuid>,
}

impl fmt::Display for EmailQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).unwrap_or_default();
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
    let mut builder = QueryBuilder::new("SELECT * FROM email");
    builder.push(" WHERE recipient IS NOT NULL ");

    if let Some(recip) = query.recipient {
        builder.push(" AND recipient = ").push_bind(recip);
    }
    if let Some(status) = query.status {
        builder.push(" AND status = ").push_bind(status);
    }
    if let Some(eid) = query.event_id {
        builder.push(" AND event_id = ").push_bind(eid);
    }
    builder
        .push(format!(" ORDER BY {order_by} {order_dir} LIMIT "))
        .push_bind(take)
        .push(" OFFSET ")
        .push_bind(skip);

    debug!(query = builder.sql());

    let emails = builder.build_query_as().fetch_all(app_state.pool()).await?;
    Ok(EmailListTemplate {
        emails,
        pagination,
        query,
    })
}
