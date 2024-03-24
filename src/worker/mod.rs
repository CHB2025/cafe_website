use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use cafe_website::AppError;
use uuid::Uuid;

use crate::app_state::AppState;

use list::worker_list;
pub use model::Worker;

mod list;
mod model;
mod shift_list;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(view))
        .route("/:id/shifts", get(shift_list::shift_list))
}

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/list", get(worker_list))
}

async fn view(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> Result<Worker, AppError> {
    Ok(
        sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", id)
            .fetch_one(app_state.pool())
            .await?,
    )
}
