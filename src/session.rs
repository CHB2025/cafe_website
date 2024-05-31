use std::sync::{Arc, Mutex};

use crate::{config::config, models::User};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
use cafe_website::AppError;
use chrono::NaiveDateTime;
use tracing::error;
use uuid::Uuid;

mod provider;
pub use provider::session_provider;

/// Struct representing the current user session When used as an extractor, the
/// session is cached in the request extensions so other extractors can run with
/// little cost
#[derive(Clone, Debug)]
pub struct Session(Arc<Mutex<DbSession>>);

// what if another call simultaneously changes the session?
#[derive(PartialEq, Eq, Debug)]
struct DbSession {
    id: Uuid,
    created_at: NaiveDateTime,
    expires_at: Option<NaiveDateTime>,
    user_id: Option<Uuid>,
}

impl Session {
    /// Returns the id of the session
    pub fn id(&self) -> Uuid {
        self.0.lock().unwrap().id
    }

    /// Returns true if a user is authenticated on the current session
    ///
    /// Shorthand for Session.user_id().is_some()
    pub fn is_authenticated(&self) -> bool {
        self.user_id().is_some()
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.0.lock().unwrap().created_at
    }

    pub fn expires_at(&self) -> Option<NaiveDateTime> {
        self.0.lock().unwrap().expires_at
    }

    /// Returns the id of the currently authenticated user if there is one
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.lock().unwrap().user_id
    }

    /// Set the currently authenticated user
    pub async fn set_auth_user(&self, user: User) -> Result<(), sqlx::Error> {
        let session_id = self.0.lock().unwrap().id;

        let session = sqlx::query_as!(
            DbSession,
            "UPDATE session SET user_id = $1 
            WHERE id = $2
            RETURNING *",
            user.id,
            session_id,
        )
        .fetch_one(config().pool())
        .await?;

        *self.0.lock().unwrap() = session;
        Ok(())
    }

    /// Creates a new session and invalidates the existing one
    pub async fn remove_auth_user(&self) -> Result<(), sqlx::Error> {
        let session_id = self.0.lock().unwrap().id;
        let tran = config().pool().begin().await?;
        sqlx::query_as!(
            DbSession,
            "UPDATE session SET expires_at = now() WHERE id = $1",
            session_id
        )
        .execute(config().pool())
        .await?;
        let new_session =
            sqlx::query_as!(DbSession, "INSERT INTO session DEFAULT VALUES RETURNING *")
                .fetch_one(config().pool())
                .await?;
        tran.commit().await?;
        *self.0.lock().unwrap() = new_session;
        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session) =
            Extension::from_request_parts(parts, state)
                .await
                .map_err(|_| {
                    error!(
                    "Unable to find session in request extensions. Is the session provider set up?"
                );
                    cafe_website::error::ISE
                })?;
        Ok(session)
    }
}
