mod add_shift;
mod block_view;
mod copy;
mod list_view;

pub use add_shift::*;
use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, models::Day};

#[derive(Deserialize)]
pub struct OptionListParams {
    event_id: Uuid,
}

pub async fn option_list(
    State(app_state): State<AppState>,
    Query(params): Query<OptionListParams>,
) -> Result<Html<String>, AppError> {
    let days = sqlx::query_as!(
        Day,
        "SELECT * FROM day WHERE event_id = $1 ORDER BY date ASC",
        params.event_id
    )
    .fetch_all(app_state.pool())
    .await?;
    let result: String = days
        .iter()
        .map(|d| format!(r##"<option value="{}">{}</option>"##, d.date, d.date))
        .collect();
    Ok(Html(result))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:id/add_shift", get(add_shift_form).post(add_shift))
        .route("/:id/copy", get(copy::copy_form).post(copy::copy))
        .route("/option_list", get(option_list))
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(block_view::schedule))
        .route("/:id/list", get(list_view::list_view))
}
