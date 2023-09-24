use crate::{app_state::AppState, error::AppError, models::User};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{cookie::Key, PrivateCookieJar};
use uuid::Uuid;

#[async_trait]
impl FromRequestParts<AppState> for Option<User> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        let Some(session) = cookie_jar.get("session") else {
            return Ok(None);
        };

        let Some(user_id): Option<Uuid> = session.value().parse().ok() else {
            return Ok(None);
        };

        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(state.pool())
            .await?;

        Ok(Some(user))
    }
}
