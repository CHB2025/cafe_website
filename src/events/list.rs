use std::fmt;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Form,
};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use crate::{app_state::AppState, list::List, models::Event};

use super::{
    create::EventInput,
    list_row::{EditEventListRowTemplate, EventListRowTemplate},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrdinalPaginatedQuery {
    pub order_by: Option<String>,
    #[serde(default)]
    pub order_dir: OrderDirection,
    #[serde(default = "default_take")]
    pub take: i64,
    #[serde(default)]
    pub skip: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum OrderDirection {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

fn default_take() -> i64 {
    10
}

impl fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            OrderDirection::Asc => "asc",
            OrderDirection::Desc => "desc",
        };
        write!(f, "{}", str)
    }
}

pub async fn event_list(
    State(app_state): State<AppState>,
    Query(op_params): Query<OrdinalPaginatedQuery>,
) -> Result<List<6, EventListRowTemplate>, StatusCode> {
    let header = [
        "Name".to_string(),
        "Start Date".to_string(),
        "End Date".to_string(),
        "Allow Signups".to_string(),
        "Modify".to_string(),
        "Delete".to_string(),
    ];
    let order_by = op_params.order_by.clone().unwrap_or("name".to_string());
    let order_dir = op_params.order_dir.clone().to_string();
    let take = op_params.take;
    let skip = op_params.skip;
    let pool = app_state.pool();

    let events = sqlx::query_as::<Postgres, Event>(&format!(
        "SELECT * FROM event ORDER BY $1 {} LIMIT $2 OFFSET $3",
        op_params.order_dir
    ))
    .bind(order_by.clone())
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

    let current_page = op_params.skip / op_params.take + 1;
    let page_count = event_count / op_params.take + (event_count % op_params.take > 0) as i64;

    let mut prev_params = op_params.clone();
    prev_params.skip = 0.max(skip - take);
    let mut next_params = op_params.clone();
    next_params.skip = skip + take;

    Ok(List {
        container_class: "card".to_string(),
        header,
        rows: events
            .into_iter()
            .map(|event| EventListRowTemplate { event })
            .collect(),

        order_by,
        order_dir,

        prev_disabled: current_page == 1,
        prev_url: format!(
            "/event/list?{}",
            serde_urlencoded::to_string(prev_params)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        ),
        current_page,
        page_count,
        next_disabled: current_page == page_count,
        next_url: format!(
            "/event/list?{}",
            serde_urlencoded::to_string(next_params)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        ),
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
    let event = sqlx::query_as!(Event, "UPDATE event SET name = $2, start_date = $3, end_date = $4, allow_signups = $5 WHERE id = $1 RETURNING *", id, event_input.name, event_input.start_date, event_input.end_date, event_input.allow_signups.is_some_and(|s| s == "on")).fetch_one(conn).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update event".to_owned()))?;
    Ok(EventListRowTemplate { event })
}
