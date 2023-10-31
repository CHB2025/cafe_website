use askama::Template;
use axum::extract::{Query, State};
use chrono::NaiveDate;
use sqlx::{FromRow, Postgres};
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError};

use super::pagination::*;

#[derive(FromRow)]
pub struct EventWithDates {
    pub id: Uuid,
    pub name: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub allow_signups: bool,
}

#[derive(Template)]
#[template(path = "events/list.html")]
pub struct EventListTemplate {
    events: Vec<EventWithDates>,
    query: OrdinalPaginatedQuery,
    prev_disabled: bool,
    prev_query: String,
    current_page: i64,
    page_count: i64,
    next_disabled: bool,
    next_query: String,
}

pub async fn event_list(
    State(app_state): State<AppState>,
    Query(
        query @ OrdinalPaginatedQuery {
            order_by,
            order_dir,
            take,
            skip,
        },
    ): Query<OrdinalPaginatedQuery>,
) -> Result<EventListTemplate, AppError> {
    let pool = app_state.pool();

    let events = sqlx::query_as::<Postgres, EventWithDates>(&format!(
        "SELECT e.*, min(date) as start_date, max(date) as end_date FROM event AS e JOIN day ON e.id = day.event_id
        GROUP BY e.id
        ORDER BY {order_by} {order_dir} LIMIT $1 OFFSET $2"
    ))
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await?;

    let event_count = sqlx::query_scalar!("SELECT COUNT(*) FROM event")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    let current_page = skip / take + 1;
    let page_count = event_count / take + (event_count % take > 0) as i64;

    let prev_params = OrdinalPaginatedQuery {
        order_by,
        order_dir,
        take,
        skip: 0.max(skip - take),
    };
    let next_params = OrdinalPaginatedQuery {
        order_by,
        order_dir,
        take,
        skip: skip + take,
    };

    Ok(EventListTemplate {
        events,
        query,

        prev_disabled: current_page == 1,
        prev_query: serde_urlencoded::to_string(prev_params)?,
        current_page,
        page_count,
        next_disabled: current_page == page_count,
        next_query: serde_urlencoded::to_string(next_params)?,
    })
}
