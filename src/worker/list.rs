use core::fmt;
use std::fmt::Display;

use askama::Template;
use axum::extract::{Query, State};
use cafe_website::{AppError, PaginatedQuery};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder};
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Hash, Deserialize, Serialize, Debug, FromRow)]
pub struct WorkerWithShiftAgg {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
    pub shifts: Option<i64>,
}

#[derive(Template, Hash)]
#[template(path = "worker/list.html")]
pub struct WorkerListTemplate {
    workers: Vec<WorkerWithShiftAgg>,
    pagination: PaginatedQuery<WorkerOrderBy>,
    page_count: i64,
    event_name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkerOrderBy {
    NameFirst,
    NameLast,
    Email,
    Phone,
    #[default]
    Shifts,
}

impl Display for WorkerOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NameFirst => "name_first",
            Self::NameLast => "name_last",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Shifts => "shifts",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub struct WorkerQuery {
    event_id: Option<Uuid>,
}

pub async fn worker_list(
    State(app_state): State<AppState>,
    Query(pagination): Query<PaginatedQuery<WorkerOrderBy>>,
    Query(query): Query<WorkerQuery>,
) -> Result<WorkerListTemplate, AppError> {
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
    if let Some(event_id) = query.event_id {
        worker_builder
            .push(" INNER JOIN day as d ON (d.event_id, d.date) = (s.event_id, s.date) WHERE d.event_id = ")
            .push_bind(event_id);
        count_builder
            .push(" INNER JOIN day as d ON (d.event_id, d.date) = (s.event_id, s.date) WHERE d.event_id = ")
            .push_bind(event_id);
    };
    // Add other where clauses here
    worker_builder.push(" GROUP BY w.id");
    count_builder.push(" GROUP BY w.id");
    worker_builder.push(" ").push(pagination.sql());

    let (workers, count): (Vec<WorkerWithShiftAgg>, i64) = tokio::try_join!(
        worker_builder.build_query_as().fetch_all(app_state.pool()),
        count_builder
            .build_query_scalar()
            .fetch_one(app_state.pool())
    )?;

    tracing::debug!("{} workers", count);

    let event_name = if let Some(event_id) = query.event_id {
        sqlx::query_scalar!("SELECT name FROM event where id = $1", event_id)
            .fetch_one(app_state.pool())
            .await?
    } else {
        "all events".to_owned()
    };

    Ok(WorkerListTemplate {
        workers,
        pagination,
        page_count: pagination.page_count(count),
        event_name,
    })
}
