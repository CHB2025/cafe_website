use std::sync::{Arc, Mutex};

use axum::{extract::FromRequestParts, http::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use cafe_website::AppError;
use uuid::Uuid;

use crate::config;

use super::{DbSession, Session};

/// Extracts the session from the request cookies (which creates one if it
/// doesn't exist), and updates the response cookie store with it
pub async fn session_provider<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, AppError> {
    // Need to get cookie jar using config instead of state. A bit messy
    let mut jar = {
        let (mut parts, body) = request.into_parts();
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(&mut parts, config())
            .await
            .expect("Infallible");
        request = Request::from_parts(parts, body);
        jar
    };

    let session_cookie = jar
        .get("session")
        .unwrap_or_else(|| Cookie::named("session"));
    let initial_id = session_cookie.value().parse::<Uuid>().ok();

    let db_session = if let Some(ref id) = initial_id {
        sqlx::query_as!(
            DbSession,
            "SELECT * FROM session 
                WHERE id = $1 AND (expires_at IS NULL OR expires_at > now())",
            id
        )
        .fetch_one(config().pool())
        .await
        .ok()
    } else {
        None
    };

    let db_session = match db_session {
        Some(s) => s,
        None => {
            sqlx::query_as!(DbSession, "INSERT INTO session DEFAULT VALUES RETURNING *")
                .fetch_one(config().pool())
                .await?
        }
    };

    let session = Session(Arc::new(Mutex::new(db_session)));
    request.extensions_mut().insert(session.clone());

    let response = next.run(request).await;
    if initial_id.is_some_and(|id| id == session.id()) {
        // No changes
        Ok(response)
    } else {
        jar = jar.remove(Cookie::named("session"));
        let session_id = session.id().to_string();
        let cookie = Cookie::build("session", session_id)
            .domain(config().domain())
            .path("/")
            .same_site(SameSite::Strict)
            .secure(true)
            .http_only(true)
            .permanent() // This is in place of expires bc expires uses time, I'm currently using chrono
            .finish();
        Ok((jar.add(cookie), response).into_response())
    }
}
