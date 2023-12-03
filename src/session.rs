use crate::{app_state::AppState, models::User};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use cafe_website::AppError;
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

pub fn create_session(jar: PrivateCookieJar, user_id: Uuid) -> PrivateCookieJar {
    let mut cookie = Cookie::new("session", user_id.to_string());
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_expires(None);
    cookie.set_same_site(SameSite::Strict);
    jar.add(cookie)
}

pub fn destroy_session(jar: PrivateCookieJar) -> PrivateCookieJar {
    jar.remove(Cookie::named("session"))
}
