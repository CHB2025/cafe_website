use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::models::{Event, Worker};

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Default,
    sqlx::Type,
    Debug,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "email_status")]
#[sqlx(rename_all = "lowercase")]
pub enum EmailStatus {
    #[default]
    Draft,
    Pending,
    Sent,
    Failed,
}

impl fmt::Display for EmailStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Sent => "sent",
            Self::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Default,
    sqlx::Type,
    Debug,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "email_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum EmailKind {
    #[default]
    Html,
    Text,
}

// Add event_id?
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow, Debug)]
pub struct Email {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub status: EmailStatus,
    pub kind: EmailKind,
    pub recipient: Uuid,
    pub subject: String,
    pub message: String,
    pub event_id: Uuid,
}

impl Email {
    pub async fn get_recipient(self, pool: &Pool<Postgres>) -> sqlx::Result<Worker> {
        sqlx::query_as!(Worker, "SELECT * FROM worker WHERE id = $1", self.recipient)
            .fetch_one(pool)
            .await
    }

    pub async fn get_event(self, pool: &Pool<Postgres>) -> sqlx::Result<Event> {
        sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", self.event_id)
            .fetch_one(pool)
            .await
    }
}
