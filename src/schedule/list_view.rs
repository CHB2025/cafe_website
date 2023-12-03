use askama::Template;
use axum::extract::{Path, State};
use cafe_website::AppError;
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    filters,
    models::{Shift, User},
};

#[derive(Template)]
#[template(path = "schedule/list_view.html")]
pub struct ListViewTemplate {
    grouped_shifts: Vec<ShiftGroup>,
}

#[derive(Debug)]
struct ShiftGroup {
    start_time: NaiveTime,
    shifts: Vec<Shift>,
}

pub async fn list_view(
    State(app_state): State<AppState>,
    user: Option<User>,
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
) -> Result<ListViewTemplate, AppError> {
    let logged_in = user.is_some();

    let shifts = if logged_in {
        sqlx::query_as!(
            Shift,
            "SELECT * FROM shift WHERE event_id = $1 AND date = $2 ORDER BY start_time ASC",
            event_id,
            date
        )
        .fetch_all(app_state.pool())
        .await?
    } else {
        sqlx::query_as!(
            Shift,
            "SELECT * FROM shift WHERE event_id = $1 AND date = $2 AND public_signup = TRUE AND worker_id IS NULL ORDER BY start_time, title ASC",
            event_id,
            date
        ).fetch_all(app_state.pool())
        .await?
    };

    let start_time = shifts
        .first()
        .map(|s| s.start_time)
        .unwrap_or(NaiveTime::from_hms_opt(8, 00, 00).expect("Valid time"));

    let mut current = ShiftGroup {
        shifts: Vec::new(),
        start_time,
    };
    let mut grouped_shifts = vec![];
    for shift in shifts {
        if shift.start_time != current.start_time {
            grouped_shifts.push(current);
            current = ShiftGroup {
                shifts: Vec::new(),
                start_time: shift.start_time,
            }
        }
        current.shifts.push(shift);
    }
    if !current.shifts.is_empty() {
        grouped_shifts.push(current);
    }

    Ok(ListViewTemplate { grouped_shifts })
}
