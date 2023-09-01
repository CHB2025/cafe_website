use askama::Template;
use axum::extract::{Query, State};
use sqlx::Postgres;

use crate::{app_state::AppState, error::AppError, models::Event};

use super::pagination::*;

#[derive(Template)]
#[template(path = "events/list.html")]
pub struct EventListTemplate {
    events: Vec<Event>,
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

    let events = sqlx::query_as::<Postgres, Event>(&format!(
        "SELECT * FROM event ORDER BY {order_by} {order_dir} LIMIT $1 OFFSET $2"
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
