use diesel::prelude::*;
use time::{Date, Time};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::day)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Day {
    pub id: i32,
    pub date: Date,
    pub entertainment: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::worker)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Worker {
    pub id: i32,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::shift)]
#[diesel(belongs_to(Worker))]
#[diesel(belongs_to(Day))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Shift {
    pub id: i32,
    pub day_id: i32,
    pub worker_id: Option<i32>,
    pub start_time: Time,
    pub end_time: Time,
    pub title: String,
    pub description: Option<String>,
}
