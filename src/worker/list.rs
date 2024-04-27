use core::fmt;
use std::fmt::Display;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::Query;
use cafe_website::{
    pagination::{OrderDirection, PaginationControls},
    AppError, PaginatedQuery,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder};
use tracing::debug;
use uuid::Uuid;

use crate::{config, models::Event};

#[derive(Hash, Deserialize, Serialize, Debug, FromRow)]
pub struct WorkerWithShiftAgg {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
    pub shifts: Option<i64>,
}

#[derive(Template)]
#[template(path = "worker/list.html")]
pub struct WorkerListTemplate {
    event_id: Option<Uuid>,
    events: Vec<Event>,
    workers: Vec<WorkerWithShiftAgg>,
    pagination: PaginatedQuery<WorkerOrderBy, 10, false>,
    query: WorkerQuery,
    controls: PaginationControls,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkerOrderBy {
    Name,
    Email,
    Phone,
    #[default]
    Shifts,
}

impl Display for WorkerOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Name => "name_last",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Shifts => "shifts",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub struct WorkerQuery {
    event_id: Option<String>,
}

impl Display for WorkerQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).unwrap_or_default();
        write!(f, "{}", s)
    }
}

pub async fn worker_list(
    Query(pagination): Query<PaginatedQuery<WorkerOrderBy, 10, false>>,
    Query(query): Query<WorkerQuery>,
) -> Result<impl IntoResponse, AppError> {
    let event_id = match query.event_id.as_deref() {
        None | Some("") => None,
        Some(s) => Some(Uuid::try_from(s)?),
    };

    let mut worker_builder = QueryBuilder::new(
        "SELECT w.*, COUNT(*) as shifts 
        FROM worker as w 
        INNER JOIN shift as s ON w.id = s.worker_id",
    );
    let mut count_builder = QueryBuilder::new(
        "SELECT COUNT(DISTINCT w.id) 
        FROM worker as w 
        INNER JOIN shift as s ON w.id = s.worker_id",
    );
    if let Some(event_id) = event_id {
        worker_builder
            .push(" WHERE s.event_id = ")
            .push_bind(event_id);
        count_builder
            .push(" WHERE s.event_id = ")
            .push_bind(event_id);
    };
    // Add other where clauses here
    worker_builder
        .push(" GROUP BY w.id")
        .push(" ")
        .push(pagination.sql());

    let (workers, count, events): (Vec<WorkerWithShiftAgg>, i64, Vec<Event>) = tokio::try_join!(
        worker_builder.build_query_as().fetch_all(config().pool()),
        count_builder
            .build_query_scalar()
            .fetch_one(config().pool()),
        sqlx::query_as!(Event, "SELECT * from event ORDER BY name").fetch_all(config().pool())
    )?;

    debug!(?count, "workers:");

    let list = WorkerListTemplate {
        event_id,
        events,
        workers,
        pagination,
        controls: pagination.controls(count, format!("/worker/list?{query}&")),
        query,
    };
    Ok(list)
}
