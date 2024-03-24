use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AdminInvite {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateUser {
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub allow_signups: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Day {
    pub event_id: Uuid,
    pub date: NaiveDate,
    pub entertainment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Shift {
    pub id: Uuid,
    pub event_id: Uuid,
    pub date: NaiveDate,
    pub worker_id: Option<Uuid>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub title: String,
    pub description: Option<String>,
    pub public_signup: bool,
}
