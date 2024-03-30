use askama::Template;
use axum::extract::{Query, State};
use cafe_website::{
    pagination::{OrderDirection, PaginationControls},
    templates::Card,
    AppError, PaginatedQuery,
};
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
    controls: PaginationControls,
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

    Ok(Card {
        class: Some("w-fit"),
        child: EventListTemplate {
            events,
            query,
            controls: query.controls(event_count, "/event/list?".to_owned()),
        },
        title: "Events".to_owned(),
        show_x: false,
    })
}
