use askama::Template;
use axum::extract::{Path, State};
use chrono::NaiveTime;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError, models::Shift};

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
    Path(id): Path<Uuid>,
) -> Result<ListViewTemplate, AppError> {
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE day_id = $1 ORDER BY start_time ASC",
        id
    )
    .fetch_all(app_state.pool())
    .await?;

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
