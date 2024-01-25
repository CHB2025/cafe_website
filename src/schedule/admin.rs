use askama::Template;
use axum::extract::{Path, State};
use cafe_website::{filters, templates::Card, AppError};
use chrono::{Local, NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{Shift, Worker},
};

struct ShiftWorker {
    shift: Shift,
    worker: Option<Worker>,
}

#[derive(Template)]
#[template(path = "schedule/admin/worker_list.html")]
struct WorkerListTemplate {
    shift_workers: Vec<ShiftWorker>,
    future: bool,
}

#[derive(Template)]
#[template(path = "schedule/admin.html")]
pub struct ShiftAdminTemplate {
    current: Card<WorkerListTemplate>,
    up_next: Card<WorkerListTemplate>,
}

pub async fn schedule_admin(
    State(app_state): State<AppState>,
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
) -> Result<ShiftAdminTemplate, AppError> {
    // Pretty bad. Should get all current and next n (2 distinct times? or all?)
    // in two diff queries, ordering by end_time and start_time respectively.
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE event_id = $1 AND date = $2 ORDER BY start_time, end_time",
        event_id,
        date
    )
    .fetch_all(app_state.pool())
    .await?;
    let w_handles: Vec<tokio::task::JoinHandle<Result<Option<Worker>, sqlx::Error>>> = shifts
        .iter()
        .map(|s| {
            let state = app_state.clone();
            let w_id = s.worker_id;
            tokio::spawn(async move {
                match w_id {
                    Some(id) => {
                        let worker =
                            sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
                                .fetch_one(state.pool())
                                .await?;
                        Ok(Some(worker))
                    }
                    None => Ok(None),
                }
            })
        })
        .collect();
    let mut all = Vec::new();
    for (shift, handle) in shifts.into_iter().zip(w_handles.into_iter()) {
        let worker = handle.await??;
        all.push(ShiftWorker { shift, worker });
    }
    let mut current = Vec::new();
    let mut up_next = Vec::new();
    let now: NaiveTime = Local::now().naive_local().time();
    for ws in all {
        if ws.shift.start_time > now {
            up_next.push(ws);
            continue;
        }
        if ws.shift.end_time > now {
            current.push(ws);
        }
    }
    Ok(ShiftAdminTemplate {
        current: Card {
            child: WorkerListTemplate {
                shift_workers: current,
                future: false,
            },
            class: Some("min-w-72 flex-1"),
            title: "On Now".to_owned(),
            show_x: false,
        },
        up_next: Card {
            child: WorkerListTemplate {
                shift_workers: up_next,
                future: true,
            },
            class: Some("min-w-72 flex-1"),
            title: "Up Next".to_owned(),
            show_x: false,
        },
    })
}
