use core::fmt;
use std::fmt::Display;

use askama::Template;
use axum::extract::{Query, State};
use cafe_website::{AppError, PaginatedQuery};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;

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
    pagination: PaginatedQuery<WorkerOrderBy>,
    current_page: i64,
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
    let (workers, event_name) = match query.event_id {
        Some(event_id) => (
            sqlx::query_as!(
                WorkerWithShiftAgg,
                "SELECT w.*, COUNT(*) as shifts 
                FROM worker as w 
                INNER JOIN shift as s ON w.id = s.worker_id
                INNER JOIN day as d ON (d.event_id, d.date) = (s.event_id, s.date)
                WHERE d.event_id = $1
                GROUP BY w.id",
                event_id
            )
            .fetch_all(app_state.pool())
            .await?,
            sqlx::query_scalar!("SELECT name FROM event WHERE id = $1", event_id)
                .fetch_one(app_state.pool())
                .await?,
        ),
        None => (
            sqlx::query_as!(
                WorkerWithShiftAgg,
                "SELECT w.*, COUNT(*) as shifts
                FROM worker as w INNER JOIN shift as s on w.id = s.worker_id
                GROUP BY w.id",
            )
            .fetch_all(app_state.pool())
            .await?,
            "all events".to_owned(),
        ),
    };

    Ok(WorkerListTemplate {
        workers,
        pagination,
        current_page: 0,
        page_count: 0,
        event_name,
    })
}
