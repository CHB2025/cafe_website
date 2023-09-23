use core::fmt;
use std::fmt::Display;

use askama::Template;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use tracing::log::debug;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, pagination::PaginatedQuery};

#[derive(Hash, Deserialize, Serialize, Debug)]
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
    query: PaginatedQuery<WorkerOrderBy>,
    current_page: i64,
    page_count: i64,
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

pub async fn worker_list(
    State(app_state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Query(query): Query<PaginatedQuery<WorkerOrderBy>>,
) -> Result<WorkerListTemplate, AppError> {
    let workers = sqlx::query_as!(
        WorkerWithShiftAgg,
        "SELECT w.*, COUNT(*) as shifts 
        FROM worker as w 
        INNER JOIN shift as s ON w.id = s.worker_id
        INNER JOIN day as d ON d.id = s.day_id
        WHERE d.event_id = $1
        GROUP BY w.id",
        event_id
    )
    .fetch_all(app_state.pool())
    .await?;
    debug!("Worker list showing {} workers", workers.len());

    Ok(WorkerListTemplate {
        workers,
        query,
        current_page: 0,
        page_count: 0,
    })
}
