use askama::Template;
use axum::{extract::State, http::StatusCode};
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    event: Option<Uuid>,
}

pub async fn view(State(app_state): State<AppState>) -> Result<HomeTemplate, StatusCode> {
    let event: Option<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM event WHERE allow_signups = true AND start_date > now() ORDER BY start_date ASC"
    )
    .fetch_optional(app_state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(HomeTemplate { event })
}
