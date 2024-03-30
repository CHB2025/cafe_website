use std::fmt;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Query, State};
use cafe_website::{
    filters,
    pagination::{OrderDirection, PaginationControls},
    templates::Card,
    AppError, PaginatedQuery,
};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::app_state::AppState;

use super::{Email, EmailKind, EmailStatus};

const DEFAULT_TAKE: i64 = 6;

#[derive(Template, Clone)]
#[template(path = "email/list.html")]
pub struct EmailListTemplate {
    emails: Vec<Email>,
    pagination: PaginatedQuery<EmailOrderBy, DEFAULT_TAKE, false>,
    query: EmailQuery,
    controls: PaginationControls,
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
    status: Option<EmailStatus>,
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
    Query(pagination): Query<PaginatedQuery<EmailOrderBy, DEFAULT_TAKE, false>>,
    Query(query): Query<EmailQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mut builder = QueryBuilder::new("SELECT * FROM email WHERE recipient IS NOT NULL");
    let mut count_builder =
        QueryBuilder::new("SELECT Count(*) FROM email WHERE recipient IS NOT NULL");

    if let Some(recip) = query.recipient {
        builder.push(" AND recipient = ").push_bind(recip);
        count_builder.push(" AND recipient = ").push_bind(recip);
    }
    if let Some(status) = query.status {
        builder.push(" AND status = ").push_bind(status);
        count_builder.push(" AND status = ").push_bind(status);
    }
    if let Some(eid) = query.event_id {
        builder.push(" AND event_id = ").push_bind(eid);
        count_builder.push(" AND event_id = ").push_bind(eid);
    }
    builder.push(" ").push(pagination.sql());

    let (emails, count) = tokio::try_join!(
        builder.build_query_as().fetch_all(app_state.pool()),
        count_builder
            .build_query_scalar()
            .fetch_one(app_state.pool())
    )?;
    Ok(Card {
        class: None,
        title: "Emails".to_owned(),
        child: EmailListTemplate {
            emails,
            pagination,
            query: query.clone(),
            controls: pagination.controls(count, format!("/email/list?{query}&")),
        },
        show_x: false,
    })
}
