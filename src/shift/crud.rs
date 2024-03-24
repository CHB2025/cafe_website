use askama_axum::IntoResponse;
use axum::{extract::{Path, State, Query}, Form};
use cafe_website::{AppError, Redirect};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, models::Shift};
use crate::worker::Worker;

use super::view::ShiftTemplate;


#[derive(Serialize, Deserialize)]
pub struct ShiftUpdate {
    title: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    description: Option<String>,
    public_signup: Option<String>,
}

pub async fn update_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(ShiftUpdate { title, start_time, end_time, description, public_signup }): Form<ShiftUpdate>
) -> Result<impl IntoResponse, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "UPDATE shift SET title = $1, start_time = $2, end_time = $3, description = $4, public_signup = $5 WHERE id = $6 RETURNING *",
        title,
        start_time,
        end_time,
        description, 
        public_signup.is_some_and(|s| s == "on"),
        id
    ).fetch_one(app_state.pool()).await?;
    let worker = match shift.worker_id {
        Some(id) => Some(
            sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
                .fetch_one(app_state.pool())
                .await?,
        ),
        None => None,
    };

    Ok(([("HX-Retarget", "#modal")], ShiftTemplate { shift, worker, logged_in: true }))
}

pub async fn delete_shift(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "DELETE FROM shift WHERE id = $1 RETURNING *",
        id
    ).fetch_one(app_state.pool()).await?;

    Ok(Redirect::to(format!("/event/{}?date={}", shift.event_id, shift.date)))
}


#[derive(Deserialize)]
pub struct RmWorkerQuery {
    id: Uuid
}

pub async fn remove_worker(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(RmWorkerQuery { id: worker_id }): Query<RmWorkerQuery>,
) -> Result<impl IntoResponse, AppError> {
    let shift = sqlx::query_as!(
        Shift,
        "UPDATE shift SET worker_id = NULL WHERE id = $1 AND worker_id = $2 RETURNING *",
        id, worker_id
    ).fetch_one(app_state.pool()).await?;
    Ok(ShiftTemplate { shift, worker: None, logged_in: true })
}
