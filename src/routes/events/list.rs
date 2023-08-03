use std::fmt;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use crate::{app_state::AppState, models::Event};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdinalPaginatedQuery {
    pub order_by: Option<String>,
    pub order_dir: Option<OrderDirection>,
    pub take: Option<i64>,
    pub skip: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
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

/// ### Panics
/// if take=0
pub async fn event_list(
    State(app_state): State<AppState>,
    Query(op_params): Query<OrdinalPaginatedQuery>,
) -> Result<Html<String>, StatusCode> {
    let order_by = op_params.order_by.unwrap_or("name".to_string());
    let take = op_params.take.unwrap_or(10);
    let skip = op_params.skip.unwrap_or(0);
    let pool = app_state.pool();

    let events = sqlx::query_scalar::<Postgres, i32>(&format!(
        "SELECT id FROM event ORDER BY $1 {} LIMIT $2 OFFSET $3",
        op_params.order_dir.unwrap_or(OrderDirection::Asc)
    ))
    .bind(order_by)
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row_futs: Vec<_> = events
        .into_iter()
        .map(|id| tokio::spawn(event_table_row(State(app_state.clone()), Path(id))))
        .collect();

    let event_count = sqlx::query_scalar!("SELECT COUNT(*) FROM event")
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or(0);

    let page_options: String = (0..(event_count / take + (event_count % take > 0) as i64))
        .map(|page| format!(r#"<option value="{page}">{}</option>"#, page + 1))
        .collect();

    let mut table_rows = String::new();
    for row in row_futs {
        let Ok(Ok(Html(r))) = row.await else {
            continue;
        };
        table_rows += &r;
    }

    Ok(Html(format!(
        r##"
        <div class="card">
            <form class="list-controls">
                <div class="form-item">
                    <label>Page</label>
                    <select value="{}">
                        {page_options}
                    </select>
                </div>
            </form>
            <table class="list" cellspacing="0">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Starts</th>
                        <th>Ends</th>
                        <th>Allow Signups</th>
                        <th>Modify</th>
                        <th>Delete</th>
                    </tr>
                </thead>
                <tbody>
                    {table_rows}
                <tbody>
            </table>
        </div>
    "##,
        skip / take
    )))
}

pub async fn event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, StatusCode> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Html(format!(
        r##"
            <tr hx-target="this" hx-swap="outerHTML">
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td><button hx-get="/event/list/row/{id}/edit">Modify</button></td>
                <td><button hx-confirm="Are you sure you want to delete this event and all related schedules?">Delete</button></td>
            </tr>
        "##,
        event.name, event.start_date, event.end_date, event.allow_signups
    )))
}

pub async fn edit_event_table_row(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, StatusCode> {
    let pool = app_state.pool();

    let event = sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Html(format!(
        r##"
            <tr hx-target="this" hx-swap="outerHTML">
                <td><input type="text" value="{}"></input></td>
                <td><input type="date" value="{}"></input></td>
                <td><input type="date" value="{}"></input></td>
                <td><input type="checkbox" checked="{}"></input></td>
                <td><button>Save</button></td>
                <td><button hx-confirm="Are you sure you want to discard your changes?" hx-get="/event/list/row/{id}">Discard</button></td>
            </tr>
        "##,
        event.name, event.start_date, event.end_date, event.allow_signups
    )))
}
