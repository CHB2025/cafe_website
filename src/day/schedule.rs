use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use time::macros::time;

use crate::{app_state::AppState, models::Shift};

#[derive(Template)]
#[template(path = "schedule.html")]
pub struct ScheduleTemplate {
    editable: bool,
    day_id: i32,
    shift_columns: Vec<Vec<ScheduleItemTemplate>>,
}

#[derive(Template)]
#[template(path = "schedule-item.html")]
pub struct ScheduleItemTemplate {
    shift: Option<Shift>,
    time: i64,
}

pub async fn schedule(
    State(app_state): State<AppState>,
    Path(day_id): Path<i32>,
) -> Result<ScheduleTemplate, StatusCode> {
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE day_id = $1 ORDER BY start_time, title ASC",
        day_id
    )
    .fetch_all(app_state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let start_time = shifts
        .first()
        .map(|sh| sh.start_time)
        .unwrap_or(time!(8:00));
    let end_time = shifts
        .last()
        .map(|sh| sh.end_time)
        .unwrap_or(time!(10:30 pm));

    // Split the shifts into columns first, then turn each column into ScheduleItems with the missing time
    let mut columns: Vec<Vec<Shift>> = vec![];
    for shift in shifts {
        let mut col_ind = 0;
        while columns
            .get(col_ind)
            .and_then(|col| col.last())
            .is_some_and(|sh| sh.end_time > shift.start_time)
        {
            col_ind += 1;
        }
        if col_ind >= columns.len() {
            columns.push(vec![]);
        }
        columns[col_ind].push(shift);
    }

    let mut shift_columns: Vec<Vec<ScheduleItemTemplate>> = vec![];
    for column in columns {
        shift_columns.push(vec![]);
        let col_ref = shift_columns.last_mut().unwrap();
        let mut prev_end = start_time;

        for shift in column {
            if shift.end_time > prev_end {
                col_ref.push(ScheduleItemTemplate {
                    shift: None,
                    time: (shift.end_time - prev_end).whole_minutes(),
                });
            }
            prev_end = shift.end_time;
            col_ref.push(ScheduleItemTemplate {
                time: (shift.end_time - shift.start_time).whole_minutes(),
                shift: Some(shift),
            })
        }

        col_ref.push(ScheduleItemTemplate {
            shift: None,
            time: (end_time - prev_end).whole_minutes(),
        })
    }

    Ok(ScheduleTemplate {
        day_id,
        editable: true,
        shift_columns,
    })
}
