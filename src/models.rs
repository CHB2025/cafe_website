use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
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

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub allow_signups: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Day {
    pub id: i32,
    pub event_id: i32,
    pub date: NaiveDate,
    pub entertainment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Worker {
    pub id: i32,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Shift {
    pub id: i32,
    pub day_id: i32,
    pub worker_id: Option<i32>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub title: String,
    pub description: Option<String>,
}
