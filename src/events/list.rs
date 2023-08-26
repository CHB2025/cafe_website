use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Form,
};
use sqlx::Postgres;

use crate::{app_state::AppState, models::Event};

use super::{
    create::EventInput,
    list_row::*,
    pagination::*,
};

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
    Query(query @ OrdinalPaginatedQuery {
        order_by,
        order_dir,
        take,
        skip,
    }): Query<OrdinalPaginatedQuery>,
) -> Result<EventListTemplate, StatusCode> {
    let pool = app_state.pool();

    let events = sqlx::query_as::<Postgres, Event>(&format!(
        "SELECT * FROM event ORDER BY {order_by} {order_dir} LIMIT $1 OFFSET $2"
    ))
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event_count = sqlx::query_scalar!("SELECT COUNT(*) FROM event")
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
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
        prev_query: serde_urlencoded::to_string(prev_params)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        current_page,
        page_count,
        next_disabled: current_page == page_count,
        next_query: serde_urlencoded::to_string(next_params)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    })
}

pub async fn event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<EventListRowTemplate, StatusCode> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(EventListRowTemplate { event })
}

pub async fn edit_event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<EditEventListRowTemplate, StatusCode> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(EditEventListRowTemplate { event })
}

pub async fn patch_event(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Form(event_input): Form<EventInput>,
) -> Result<EventListRowTemplate, (StatusCode, String)> {
    let conn = app_state.pool();
    let event = sqlx::query_as!(
        Event, 
        "UPDATE event SET name = $2, start_date = $3, end_date = $4, allow_signups = $5 WHERE id = $1 RETURNING *",
        id,
        event_input.name,
        event_input.start_date,
        event_input.end_date,
        event_input.allow_signups.is_some_and(|s| s == "on")
    )
    .fetch_one(conn)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update event".to_owned()))?;
    Ok(EventListRowTemplate { event })
}
