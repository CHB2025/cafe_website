use askama::Template;
use axum::extract::{Query, State};
use cafe_website::{templates::Card, AppError, PaginatedQuery};
use chrono::NaiveDate;
use sqlx::{FromRow, Postgres};
use uuid::Uuid;

use crate::{app_state::AppState, models::Event};

use super::pagination::EventOrderBy;

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
    events: Vec<Event>,
    query: PaginatedQuery<EventOrderBy>,
    prev_disabled: bool,
    prev_query: PaginatedQuery<EventOrderBy>,
    current_page: i64,
    page_count: i64,
    next_disabled: bool,
    next_query: PaginatedQuery<EventOrderBy>,
}

pub async fn event_list(
    State(app_state): State<AppState>,
    Query(query): Query<PaginatedQuery<EventOrderBy>>,
) -> Result<Card<EventListTemplate>, AppError> {
    let pool = app_state.pool();

    let events = sqlx::query_as::<Postgres, Event>(&format!("SELECT * FROM event {}", query.sql()))
        .fetch_all(pool)
        .await?;

    let event_count = sqlx::query_scalar!("SELECT COUNT(*) FROM event")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    let current_page = query.page();
    let page_count = query.page_count(event_count);

    Ok(Card {
        class: Some("w-fit"),
        child: EventListTemplate {
            events,
            query,

            prev_disabled: current_page == 1,
            prev_query: query.previous(),
            current_page,
            page_count,
            next_disabled: current_page == page_count,
            next_query: query.next(),
        },
        title: "Events".to_owned(),
        show_x: false,
    })
}
