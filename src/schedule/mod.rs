mod add_shift;
mod admin;
mod copy;
mod view;

pub use add_shift::*;
use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use cafe_website::AppError;
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, models::Day};

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
    let result: String = days.iter().fold(String::new(), |mut output, d| {
        use std::fmt::Write;
        let _ = write!(
            output,
            r##"<option value="{}">{}</option>"##,
            d.date, d.date
        );
        output
    });
    Ok(Html(result))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/:date/add_shift", get(add_shift_form).post(add_shift))
        .route("/:date/copy", get(copy::copy_form).post(copy::copy))
        .route("/:date/admin", get(admin::schedule_admin))
}

pub fn public_router() -> Router<AppState> {
    Router::new().route("/:date", get(view::schedule))
}
