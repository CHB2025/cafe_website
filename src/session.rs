use std::convert::Infallible;

use crate::{config::config, models::User};
use axum::{
    async_trait, extract::FromRequestParts, http::request::Parts, response::IntoResponseParts,
};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use cafe_website::AppError;
use uuid::Uuid;

#[derive(Clone)]
pub struct Session {
    jar: PrivateCookieJar,
    user: Option<User>,
}

impl Session {
    /// Returns true if a user is authenticated on the current session
    ///
    /// Shorthand for Session.user().is_some()
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// Returns the currently authenticated user if there is one
    pub fn user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    /// Set the currently authenticated user
    pub fn set_auth_user(&mut self, user: User) {
        let mut cookie = Cookie::new("session", user.id.to_string());
        cookie.set_secure(true);
        cookie.set_http_only(true);
        cookie.set_expires(None);
        cookie.set_same_site(SameSite::Strict);
        // This clone is annoying, but not sure how else to do it
        self.jar = self.jar.clone().add(cookie);

        self.user = Some(user);
    }

    /// remove the currently authenticated user
    pub fn remove_auth_user(&mut self) {
        // This clone is annoying, but not sure how else to do it
        self.jar = self.jar.clone().remove(Cookie::named("session"));
        self.user = None;
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, config())
            .await
            .expect("Infallible");
        let Some(session) = jar.get("session") else {
            return Ok(Session { jar, user: None });
        };

        let Some(user_id): Option<Uuid> = session.value().parse().ok() else {
            return Ok(Session { jar, user: None });
        };

        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(config().pool())
            .await?;

        Ok(Session {
            jar,
            user: Some(user),
        })
    }
}

impl IntoResponseParts for Session {
    type Error = Infallible;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        self.jar.into_response_parts(res)
    }
}
