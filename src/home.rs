use askama::Template;
use axum::http::StatusCode;
use uuid::Uuid;

use crate::config;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    event: Option<Uuid>,
}

pub async fn view() -> Result<HomeTemplate, StatusCode> {
    let event: Option<Uuid> = sqlx::query_scalar!(
        "SELECT event.id FROM event 
        JOIN day ON event.id = event_id
        GROUP BY event.id
        HAVING min(date) > now() AND allow_signups = true
        ORDER BY min(date) ASC"
    )
    .fetch_optional(config().pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(HomeTemplate { event })
}
