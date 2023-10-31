use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub allow_signups: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Day {
    pub event_id: Uuid,
    pub date: NaiveDate,
    pub entertainment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Worker {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
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
